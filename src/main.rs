use coremidi::{OutputPort, Destination, Destinations, Source, EventList, Endpoint, Sources, EventBuffer, Protocol, Client};
use std::sync::Arc;
use std::result::Result;
use std::env;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const NOTE_ON_STATUS: u32 = 0x20900000;
const COLOR_INTENSITY: u32 = 0x20960000;
const LED_10_BRIGHT: u32 = 0x00000000;
const LED_25_BRIGHT: u32 = 0x00010000;
const LED_50_BRIGHT: u32 = 0x00020000;
const LED_65_BRIGHT: u32 = 0x00030000;
const LED_75_BRIGHT: u32 = 0x00040000;
const LED_95_BRIGHT: u32 = 0x00050000;
const LED_100_BRIGHT: u32 = 0x00060000;
const PULSING_1_16: u32 = 0x00070000;
const PULSING_1_8: u32 = 0x00080000;
const PULSING_1_4: u32 = 0x00090000;
const PULSING_1_2: u32 = 0x000a0000;
const BLINKING_1_24: u32 = 0x000b0000;
const BLINKING_1_16: u32 = 0x000c0000;
const BLINKING_1_8: u32 = 0x000d0000;
const BLINKING_1_4: u32 = 0x000e0000;
const BLINKING_1_2: u32 = 0x000f0000;
const NOTE_OFF_STATUS: u32 = 0x20800000;
const BAR_BLINK: u32 = 2;

const GRID_OFFSET: u32 = 56;
const GRID_GREEN: u32 = 1;

const NOTE_ON: u32  = 0x2090407f;
const NOTE_OFF: u32 = 0x2080407f;
const NOTE_ON_C: u32 = 0x40903c00;
const MASK_HALF: u32 = 0xffff0000;
const MASK: u32 = 0xffff0000;
const GRID_MASK: u32 = 0x0000ff00;

const LIGHT_DIM: u32 = 0x01;

#[derive(Debug)]
pub enum AppError {
    DestinationDisplayNameError,
    SourceDisplayNameError,
    SourceNotFoundError,
    SourceListenError(i32),
    SourceUniqueIdError,
}

#[derive(Clone)]
struct GlobalState {
    grid_state: GridState,
}

// Shameless theft from https://stackoverflow.com/a/57557329
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

#[derive(Clone)]
struct GridState {
    color_picker: u32,
}

static COLOR_PICKER: u32 = 0;

fn main() -> Result<(), AppError> {

    fibble_destinations()?;
    // println!();
    // println!("System sources:");

    // for (i, source) in Sources.into_iter().enumerate() {
    //     let display_name = get_display_name(&source);
    //     println!("[{}] {}", i, display_name);
    //     receive_source(&source);
    // }
    Ok(())
}

// fn get_display_name(endpoint: &Endpoint) -> String {
//     endpoint
//         .display_name()
//         .unwrap_or_else(|| "[Unknown Display Name]".to_string())
// }

fn fibble_destinations() -> Result<(), AppError> {
    println!("System destinations:");
    let client = Client::new("Example Client").unwrap();
    let output_port = client.output_port("Example Port").unwrap();
    for (i, destination) in Destinations.into_iter().enumerate() {
        let display_name = destination.display_name().ok_or(AppError::DestinationDisplayNameError)?;
        println!("[{}] {}", i, display_name);
        if display_name != "APC mini mk2 Control" {
            continue;
        }
        for j in 0..127 {
            let payload = NOTE_ON_STATUS | (j << 8) | 0;
            // println!("[{}] Sending as byte {}...", i, j);
            println!("Example payload: {:08x}", payload);
            let note_on = EventBuffer::new(Protocol::Midi10).with_packet(0, &[payload]);

            // let note_off = EventBuffer::new(Protocol::Midi10).with_packet(0, &[NOTE_OFF]);
            output_port.send(&destination, &note_on).unwrap();
        };

            // thread::sleep(Duration::from_millis(100));

            // output_port.send(&destination, &note_off).unwrap();
            // thread::sleep(Duration::from_millis(100));
    };
    let source = Sources.into_iter().find(|s| {
        s.display_name().map_or(false, |name| name == "APC mini mk2 Control")
    }).ok_or(AppError::SourceNotFoundError)?;
    listen_to_color_selection(&client, &source)
}

fn get_destination(name: &str) -> Option<Destination> {
    Destinations.into_iter().find(|dest| {
        match dest.display_name() {
            Some(display_name) => display_name == name,
            None => false,
        }
    })
}

fn show_destinations() -> Result<(), AppError> {
    let client = Client::new("Example Client").unwrap();
    let output_port = client.output_port("Example Port").unwrap();
    for (i, destination) in Destinations.into_iter().enumerate() {
        let display_name = destination.display_name().ok_or(AppError::DestinationDisplayNameError)?;
        println!("[{}] {}", i, display_name);
    }
    Ok(())
}

fn handle_packet(grid_state: &GridState, packet: u32) -> GridState {
    let command = packet >> 20;
    if command == (NOTE_ON_STATUS >> 20) {
        println!("Note on {:08x}", command);
        let grid = (GRID_MASK & packet) >> 8;
        println!("Grid: {:08x}", grid);
        if grid < 64 {
            let x = grid % 8;
            let y = grid / 8;
            println!("Coords: {} {}", x, y);
        } else if grid >= 64 && grid <= 0x6b {
            let bottom_button = (grid - 4) % 8;
            println!("Bottom button: {}", bottom_button);
            return toggle_bottom(&grid_state, bottom_button);
        }
    } else {
        println!("Note off {:08x}", command);
    }
    grid_state.clone()
}

fn toggle_bottom(grid_state: &GridState, button: u32) -> GridState {
    let new_color = grid_state.color_picker ^ (1 << button);
    println!("New color picker: {} {:32b}", new_color, new_color);
    return GridState {
        color_picker: new_color,
    };
}

fn listen_to_color_selection(client: &Client, source: &Source) -> Result<(), AppError> {
    let source_id = source.unique_id().ok_or(AppError::SourceUniqueIdError)?;
    let global_state = Arc::new(Mutex::new(GlobalState {
        grid_state: GridState { color_picker: 0 },
    }));
    let callback = enclose!((global_state) move |event_list: &EventList, context: &mut u32| {
        for (_size, event) in event_list.iter().enumerate() {
            for packet in event.data() {
                // println!("herp {:08x}", packet);
                let mut gs = global_state.lock().unwrap();
                let new_grid_state = handle_packet(&gs.grid_state, *packet);
                if new_grid_state.color_picker != gs.grid_state.color_picker {
                    gs.grid_state = new_grid_state;
                    bottom_row_to_midi(gs.grid_state.color_picker);
                    color_test_cell(gs.grid_state.color_picker);
                }
            }
        }
       // print!("{:08x}: {:?}", *context, event_list);
    });
    let mut input_port = client
        .input_port_with_protocol("Example Port", Protocol::Midi10, callback)
        .map_err(AppError::SourceListenError)?;
    input_port.connect_source(&source, source_id)
        .map_err(AppError::SourceListenError)?;
    thread::park();
    Ok(())
}

fn color_test_cell(color: u32) {
    // Inefficient, but whatever.
    let client = Client::new("Example Client").unwrap();
    let port = client.output_port("Example Port").unwrap();
    match get_destination("APC mini mk2 Control") {
        Some(dest) => {
            grid_button_color_to_midi(port, dest, 0, 2, color);
        },
        None => (),
    }
}

fn bottom_row_to_midi(button_bits: u32) {
    // Inefficient, but whatever.
    let client = Client::new("Example Client").unwrap();
    let port = client.output_port("Example Port").unwrap();
    match get_destination("APC mini mk2 Control") {
        Some(dest) => {
            for shift in 0..8 {
                let enabled = if button_bits & (1 << shift) == 0 { 0 } else { 1 };
                let button_id = (64 + 36 + shift) << 8;
                let payload = NOTE_ON_STATUS + button_id + enabled;
                println!("sending {:08x}", payload);
                let note_on = EventBuffer::new(Protocol::Midi10).with_packet(0, &[payload]);
                port.send(&dest, &note_on).unwrap();
            }
        },
        None => (),
    }
}

fn grid_button_color_to_midi(port: OutputPort, dest: Destination, x: u32, y: u32, color: u32) {
    let payload = NOTE_ON_STATUS | PULSING_1_2 | (x + (y * 8)) << 8 | color;
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
    let mut input_port = client.input_port_with_protocol("Example Port", Protocol::Midi10, callback).unwrap();
    input_port.connect_source(&source, source_id).unwrap();
    let mut input_line = String::new();
    println!("Press Enter to Finish");
    std::io::stdin()
        .read_line(&mut input_line)
        .expect("Failed to read line");

    input_port.disconnect_source(&source).unwrap();
}