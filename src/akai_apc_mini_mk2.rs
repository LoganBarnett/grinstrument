use crate::{device::Device, action::Action};

pub const NOTE_ON_STATUS: u32 = 0x20900000;
pub const COLOR_INTENSITY: u32 = 0x20960000;
pub const LED_10_BRIGHT: u32 = 0x00000000;
pub const LED_25_BRIGHT: u32 = 0x00010000;
pub const LED_50_BRIGHT: u32 = 0x00020000;
pub const LED_65_BRIGHT: u32 = 0x00030000;
pub const LED_75_BRIGHT: u32 = 0x00040000;
pub const LED_95_BRIGHT: u32 = 0x00050000;
pub const LED_100_BRIGHT: u32 = 0x00060000;
pub const PULSING_1_16: u32 = 0x00070000;
pub const PULSING_1_8: u32 = 0x00080000;
pub const PULSING_1_4: u32 = 0x00090000;
pub const PULSING_1_2: u32 = 0x000a0000;
pub const BLINKING_1_24: u32 = 0x000b0000;
pub const BLINKING_1_16: u32 = 0x000c0000;
pub const BLINKING_1_8: u32 = 0x000d0000;
pub const BLINKING_1_4: u32 = 0x000e0000;
pub const BLINKING_1_2: u32 = 0x000f0000;
pub const NOTE_OFF_STATUS: u32 = 0x20800000;
pub const BAR_BLINK: u32 = 2;
pub const GRID_MASK: u32 = 0x0000ff00;

#[derive(Clone)]
pub struct AkaiApcMiniMk2 {}

impl Device for AkaiApcMiniMk2 {
    fn midi_to_action(&self, context: u32, data: u32) -> Action {
        let command = data >> 20;
        if command == (NOTE_ON_STATUS >> 20) {
            println!("Note on {:08x}", command);
            let grid = (GRID_MASK & data) >> 8;
            println!("Grid: {:08x}", grid);
            if grid < 64 {
                let x = grid % 8;
                let y = grid / 8;
                println!("Coords: {} {}", x, y);
                Action::GridToggle { x, y }
            } else if grid >= 64 && grid <= 0x6b {
                let bottom_button = (grid - 4) % 8;
                println!("Bottom button: {}", bottom_button);
                Action::BottomToggle { pos: bottom_button }
            } else {
                println!("Unsupported message {:08x}", command);
                Action::Noop
            }
        } else if command == (NOTE_OFF_STATUS >> 20) {
            println!("Note off {:08x}", command);
            Action::Noop
        } else {
            println!("Unsupported message {:08x}", command);
            Action::Noop
        }
    }
}
