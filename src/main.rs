mod action;
mod akai_apc_mini_mk2;
mod device;
mod error;
mod midi;
mod reducer;
mod state;
mod utils;

use crate::{
    akai_apc_mini_mk2::{
        AkaiApcMiniMk2,
        LED_100_BRIGHT,
        NOTE_ON_STATUS,
        PULSING_1_2,
    },
    device::Color,
    device::ColorStyle,
    error::AppError,
    midi::diagnose_midi_devices,
    state::initial_state,
};
use coremidi::{
    Client, Destination, Destinations, EventBuffer, EventList, OutputPort,
    Protocol, Source,
};
use device::Device;
use futures::executor::block_on;
use midi::{connect_to_controller, get_destination, get_source};
use redux_rs::Store;
use state::{GlobalState, Note};
use std::result::Result;
use std::sync::{Arc, Mutex};
use std::thread;

include!(concat!(env!("OUT_DIR"), "/constants.rs"));

const GRID_OFFSET: u32 = 56;
const GRID_GREEN: u32 = 1;

const NOTE_ON: u32 = 0x2090407f;
const NOTE_OFF: u32 = 0x2080407f;
const NOTE_ON_C: u32 = 0x40903c00;
const MASK_HALF: u32 = 0xffff0000;
const MASK: u32 = 0xffff0000;

const LIGHT_DIM: u32 = 0x01;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    diagnose_midi_devices();
    let store_mutex = Arc::new(Mutex::new(Store::new_with_state(
        reducer::reducer,
        initial_state(),
    )));
    let device = AkaiApcMiniMk2 {};
    let callback = enclose!(
    (store_mutex) move |event_list: &EventList, mut_context: &mut u32| {
        println!("Got midi event");
        let context = mut_context.clone();
        if let Ok(store) = store_mutex.lock() {
            for (_size, event_packet) in event_list.iter().enumerate() {
                for data in event_packet.data() {
                    block_on(store.dispatch(
                        device.midi_to_action(context, *data)
                    ))
                }
            }
        }
    });
    let (client, mut input_port, output_port) =
        connect_to_controller(callback)?;
    let dest = get_destination("APC mini mk2 Control")
        .ok_or(AppError::DestinationNotFoundError)?;
    let source = get_source("APC mini mk2 Control")
        .ok_or(AppError::SourceNotFoundError)?;
    let source_id = source.unique_id().ok_or(AppError::SourceUniqueIdError)?;
    if let Some(name) = source.display_name() {
        println!("Listening to {}.", name);
    }
    input_port
        .connect_source(&source, source_id)
        .map_err(AppError::SourceListenError)?;
    if let Ok(store) = store_mutex.lock() {
        println!("Subscribing...");
        store
            .subscribe(move |state: &GlobalState| {
                println!("State has changed...");
                for (i, note) in
                    state.sections[0].layers[0].notes.iter().enumerate()
                {
                    for j in 0..8 {
                        device.set_grid_button(
                            &output_port,
                            &dest,
                            i,
                            j,
                            Color {
                                rgb: note_color(&note, j),
                                style: ColorStyle::Steady,
                            },
                        );
                        // grid_button_color_to_midi(
                        //     &output_port,
                        //     &dest,
                        //     i as u32,
                        //     j as u32,
                        //     note_color(&note, j),
                        // );
                    }
                }
            })
            .await;
    }
    println!("Everything started up, waiting for input!");
    thread::park();
    Ok(())
}

fn note_color(note: &Note, index: usize) -> u32 {
    if note.length > 0 && note.octaves.contains(&index) {
        0xff0000
    } else {
        0
    }
}

fn show_destinations() -> Result<(), AppError> {
    for (i, destination) in Destinations.into_iter().enumerate() {
        let display_name = destination
            .display_name()
            .ok_or(AppError::DisplayNameError)?;
        println!("[{}] {}", i, display_name);
    }
    Ok(())
}

fn bottom_row_to_midi(button_bits: u32) {
    // Inefficient, but whatever.
    let client = Client::new("Example Client").unwrap();
    let port = client.output_port("Example Port").unwrap();
    match get_destination("APC mini mk2 Control") {
        Some(dest) => {
            for shift in 0..8 {
                let enabled = if button_bits & (1 << shift) == 0 {
                    0
                } else {
                    1
                };
                let button_id = (64 + 36 + shift) << 8;
                let payload = NOTE_ON_STATUS + button_id + enabled;
                println!("sending {:08x}", payload);
                let note_on = EventBuffer::new(Protocol::Midi10)
                    .with_packet(0, &[payload]);
                port.send(&dest, &note_on).unwrap();
            }
        }
        None => (),
    }
}

fn grid_button_color_to_midi(
    port: &OutputPort,
    dest: &Destination,
    x: u32,
    y: u32,
    color: u32,
) {
    let payload = NOTE_ON_STATUS | LED_100_BRIGHT | (x + (y * 8)) << 8 | color;
    println!("Sending color to {}, {}: {:08x}", x, y, payload);
    let note_on = EventBuffer::new(Protocol::Midi10).with_packet(0, &[payload]);
    port.send(&dest, &note_on).unwrap();
}

fn receive_source(source: &Source) {
    let client = Client::new("Example Client").unwrap();
    let source_id = source.unique_id().unwrap_or(0);

    let callback = |event_list: &EventList, context: &mut u32| {
        print!("{:08x}: {:?}", *context, event_list);
    };
    let mut input_port = client
        .input_port_with_protocol("Example Port", Protocol::Midi10, callback)
        .unwrap();
    input_port.connect_source(&source, source_id).unwrap();
    let mut input_line = String::new();
    println!("Press Enter to Finish");
    std::io::stdin()
        .read_line(&mut input_line)
        .expect("Failed to read line");

    input_port.disconnect_source(&source).unwrap();
}
