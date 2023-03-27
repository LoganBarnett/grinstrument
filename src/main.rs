mod action;
mod akai_apc_mini_mk2;
mod device;
mod error;
mod midi;
mod reducer;
mod state;
mod utils;

use crate::akai_apc_mini_mk2::AkaiApcMiniMk2;
use crate::{
    action::Action, device::Color, device::ColorStyle, error::AppError,
    midi::diagnose_midi_devices, state::initial_state,
};
use coremidi::{
    Client, Destination, Destinations, EventList, OutputPort, Protocol, Source,
};
use device::Device;
use futures::executor::block_on;
use midi::{connect_to_controller, get_destination, get_source};
use redux_rs::Store;
use state::{GlobalState, Layer, Note, PlayMode, Section};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{result::Result, time::Duration};

include!(concat!(env!("OUT_DIR"), "/constants.rs"));

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
    let (_client, mut input_port, output_port) =
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
        // Set the grid to be the initial state.
        state_to_device(&device, &output_port, &dest, &initial_state())?;
        println!("Subscribing...");
        store
            .subscribe(move |state: &GlobalState| {
                state_to_device(&device, &output_port, &dest, state)
                    .unwrap_or_else(|err| {
                        println!("Error sending state to device: {:#?}", err);
                        ()
                    })
            })
            .await;
    }
    println!("Setting up timer...");
    let _scheduler = thread::spawn(move || {
        let duration = Duration::from_millis(1000);
        loop {
            println!("Time interval: Seeing if we can grab the mutex for the store...");
            if let Ok(store) = store_mutex.lock() {
                println!("Pumping the interval...");
                block_on(store.dispatch(Action::TimeInterval));
            }
            thread::sleep(duration);
        }
    });
    println!("Everything started up, waiting for input!");
    thread::park();
    Ok(())
}

fn note_to_device(
    device: &dyn Device,
    output_port: &OutputPort,
    dest: &Destination,
    interval: usize,
    section_index: usize,
    layer_index: usize,
    note_interval: usize,
    note: &Note,
) -> Result<(), AppError> {
    (0..8)
        .map(|note_octave| {
            (note_interval..8)
                .map(|note_interval_by_length| {
                    device.set_grid_button(
                        &output_port,
                        &dest,
                        note_interval,
                        note_octave,
                        note_color(
                            layer_index,
                            section_index,
                            interval,
                            note_interval,
                            &note,
                            note_octave,
                            note_interval_by_length,
                        ),
                    )
                })
                .collect::<Result<(), AppError>>()
        })
        .collect::<Result<(), AppError>>()
}

fn layer_to_device(
    device: &dyn Device,
    output_port: &OutputPort,
    dest: &Destination,
    interval: usize,
    section_index: usize,
    active_layer_index: usize,
    layer_index: usize,
    layer: &Layer,
) -> Result<(), AppError> {
    device
        .set_layer_button(
            &output_port,
            &dest,
            layer_index,
            Color {
                style: ColorStyle::Steady100,
                rgb: active_color(layer_index, active_layer_index),
            },
        )
        .and_then(|()| {
            if layer_index == active_layer_index {
                layer
                    .notes
                    .iter()
                    .enumerate()
                    .map(|(note_index, note)| {
                        note_to_device(
                            device,
                            output_port,
                            dest,
                            interval,
                            section_index,
                            layer_index,
                            note_index,
                            &note,
                        )
                    })
                    .collect::<Result<(), AppError>>()
            } else {
                Ok(())
            }
        })
}

fn section_to_device(
    device: &dyn Device,
    output_port: &OutputPort,
    dest: &Destination,
    interval: usize,
    active_section_index: usize,
    active_layer_index: usize,
    section_index: usize,
    section: &Section,
) -> Result<(), AppError> {
    device
        .set_section_button(
            &output_port,
            &dest,
            section_index,
            Color {
                style: ColorStyle::Steady100,
                rgb: active_color(section_index, active_section_index),
            },
        )
        .and_then(|()| {
            if section_index == active_section_index {
                section
                    .layers
                    .iter()
                    .enumerate()
                    .map(|(layer_index, layer)| {
                        layer_to_device(
                            device,
                            output_port,
                            dest,
                            interval,
                            section_index,
                            active_layer_index,
                            layer_index,
                            &layer,
                        )
                    })
                    .collect::<Result<(), AppError>>()
            } else {
                Ok(())
            }
        })
}

fn state_to_device(
    device: &dyn Device,
    output_port: &OutputPort,
    dest: &Destination,
    state: &GlobalState,
) -> Result<(), AppError> {
    println!("State has changed...");
    device
        .set_play_button(
            &output_port,
            &dest,
            play_mode_color(state.player.play_mode.clone()),
        )
        .and_then(|()| {
            state
                .sections
                .iter()
                .enumerate()
                .map(|(section_index, section)| {
                    section_to_device(
                        device,
                        output_port,
                        dest,
                        state.player.interval,
                        state.player.active_section_index,
                        state.player.active_layer_index,
                        section_index,
                        section,
                    )
                })
                .collect::<Result<(), AppError>>()
        })
}

// Order dictates the layer.
const LAYER_COLORS: &[u32] = &[
    0x0000ff, 0x00ffff, 0x00ff00, 0xffff00, 0xff0000, 0xff00ff, 0xffaa00,
    0xffffff,
];

fn active_color(current: usize, active: usize) -> u32 {
    if current == active {
        1
    } else {
        0
    }
}

// TODO: Consider not using Color here since the color style is discarded in the
// device.
fn note_color(
    layer: usize,
    section: usize,
    interval: usize,
    note_index: usize,
    note: &Note,
    octave: usize,
    length_pos: usize,
) -> Color {
    // TODO: Do not hardcode section sizes.
    if interval == note_index + (section * 8) {
        // Active note and interval.
        if note.length > 0 && note.octaves.contains(&octave) {
            Color {
                rgb: LAYER_COLORS[layer],
                style: ColorStyle::Steady95,
            }
            // Interval active here.
        } else {
            Color {
                rgb: LAYER_COLORS[layer],
                style: ColorStyle::Steady50,
            }
        }
        // Active note with nothing else.
    } else if note.length > 0 && note.octaves.contains(&octave) {
        if length_pos == 0 {
            // Where the note begins.
            Color {
                rgb: LAYER_COLORS[layer],
                style: ColorStyle::Steady75,
            }
        } else {
            // Any part of a longer note.
            Color {
                rgb: LAYER_COLORS[layer],
                style: ColorStyle::Steady65,
            }
        }
        // Vacant.
    } else {
        Color {
            rgb: 0,
            style: ColorStyle::Steady100,
        }
    }
}

fn play_mode_color(play_mode: PlayMode) -> Color {
    match play_mode {
        PlayMode::Playing => Color {
            rgb: 0x1,
            style: ColorStyle::Steady100,
        },
        PlayMode::Paused => Color {
            rgb: 0x1,
            style: ColorStyle::Blink2,
        },
        PlayMode::Stopped => Color {
            rgb: 0x0,
            style: ColorStyle::Steady100,
        },
    }
}

fn _show_destinations() -> Result<(), AppError> {
    for (i, destination) in Destinations.into_iter().enumerate() {
        let display_name = destination
            .display_name()
            .ok_or(AppError::DisplayNameError)?;
        println!("[{}] {}", i, display_name);
    }
    Ok(())
}

fn _receive_source(source: &Source) {
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
