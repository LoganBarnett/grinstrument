use coremidi::{Destination, EventBuffer, OutputPort, Protocol};
use itertools::Itertools;
use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::{
    action::Action,
    akai_apc_mini_mk2_constants::AKAI_APC_MINI_MK_2_COLORS_SQUARED,
    device::{Color, ColorStyle, Device},
    error::AppError,
    state::PlayMode,
};

// Leftovers. I need to go through to see if these are still useful.
const _GRID_OFFSET: u32 = 56;
const _GRID_GREEN: u32 = 1;
const _NOTE_ON: u32 = 0x2090407f;
const _NOTE_OFF: u32 = 0x2080407f;
const _NOTE_ON_C: u32 = 0x40903c00;
const _MASK_HALF: u32 = 0xffff0000;
const _MASK: u32 = 0xffff0000;
const _LIGHT_DIM: u32 = 0x01;

// Weird because scene launch offset should be 0x7000 but it makes the math
// weird since it's just an offset and we need to shift the whole number.
pub const SCENE_LAUNCH_OFFSET: u32 = 0x00000070;
pub const TRACK_OFFSET: u32 = 0x00000064;
pub const SHIFT_BUTTON: u32 = 0x0000007a;
pub const NOTE_ON_STATUS: u32 = 0x20900000;
pub const _COLOR_INTENSITY: u32 = 0x20960000;
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
pub const _BAR_BLINK: u32 = 2;
pub const GRID_MASK: u32 = 0x0000ff00;

lazy_static! {
    static ref COLORS_BY_VELOCITY: HashMap<u32, u32> = HashMap::from([
        (0, 0),
        (0x1E1E1E, 1),
        (0x7F7F7F, 2),
        (0xFFFFFF, 3),
        (0xFF4C4C, 4),
        (0xFF0000, 5),
        (0x590000, 6),
        (0x190000, 7),
        (0xFFBD6C, 8),
        (0xFF5400, 9),
        (0x591D00, 10),
        (0x271B00, 11),
        (0xFFFF4C, 12),
        (0xFFFF00, 13),
        (0x595900, 14),
        (0x191900, 15),
        (0x88FF4C, 16),
        (0x54FF00, 17),
        (0x1D5900, 18),
        (0x142B00, 19),
        (0x4CFF4C, 20),
        (0x00FF00, 21),
        (0x005900, 22),
        (0x001900, 23),
        (0x4CFF5E, 24),
        (0x00FF19, 25),
        (0x00590D, 26),
        (0x001902, 27),
        (0x4CFF88, 28),
        (0x00FF55, 28),
        (0x00591D, 29),
        (0x001F12, 30),
        (0x4CFFB7, 31),
        (0x00FF99, 32),
        (0x005935, 33),
        (0x001912, 34),
        (0x4CC3FF, 35),
        (0x00A9FF, 36),
        (0x004152, 37),
        (0x001019, 38),
        (0x4C88FF, 40),
        (0x0055FF, 41),
        (0x001D59, 42),
        (0x000819, 43),
        (0x4C4CFF, 44),
        (0x0000FF, 45),
        (0x000059, 46),
        (0x000019, 47),
        (0x874CFF, 48),
        (0x5400FF, 49),
        (0x190064, 50),
        (0x0F0030, 51),
        (0xFF4CFF, 52),
        (0xFF00FF, 53),
        (0x590059, 54),
        (0x190019, 55),
        (0xFF4C87, 56),
        (0xFF0054, 57),
        (0x59001D, 58),
        (0x220013, 59),
        (0xFF1500, 60),
        (0x993500, 61),
        (0x795100, 62),
        (0x795100, 62),
        (0x436400, 63),
        (0x033900, 64),
        (0x005735, 65),
        (0x00547F, 66),
        (0x0000FF, 67),
        (0x00454F, 68),
        (0x2500CC, 69),
        (0x7F7F7F, 70),
        (0x202020, 71),
        (0xFF0000, 72),
        (0xBDFF2D, 73),
        (0xAFED06, 74),
        (0x64FF09, 75),
        (0x108B00, 76),
        (0x00FF87, 77),
        (0x00A9FF, 78),
        (0x002AFF, 79),
        (0x3F00FF, 80),
        (0x7A00FF, 81),
        (0xB21A7D, 82),
        (0x402100, 83),
        (0xFF4A00, 84),
        (0x88E106, 85),
        (0x72FF15, 86),
        (0x00FF00, 87),
        (0x3BFF26, 88),
        (0x59FF71, 89),
        (0x38FFCC, 90),
        (0x5B8AFF, 91),
        (0x3151C6, 92),
        (0x877FE9, 93),
        (0xD31DFF, 94),
        (0xFF005D, 95),
        (0xFF7F00, 96),
        (0xB9B000, 97),
        (0x90FF00, 98),
        (0x835D07, 99),
        (0x392b00, 100),
        (0x144C10, 101),
        (0x0D5038, 102),
        (0x15152A, 103),
        (0x16205A, 104),
        (0x693C1C, 105),
        (0xA8000A, 106),
        (0xDE513D, 107),
        (0xD86A1C, 108),
        (0xFFE126, 109),
        (0x9EE12F, 110),
        (0x67B50F, 111),
        (0x1E1E30, 112),
        (0xDCFF6B, 113),
        (0x80FFBD, 114),
        (0x9A99FF, 115),
        (0x8E66FF, 116),
        (0x404040, 117),
        (0x757575, 118),
        (0xE0FFFF, 119),
        (0xA00000, 120),
        (0x350000, 121),
        (0x1AD000, 122),
        (0x074200, 123),
        (0xB9B000, 124),
        (0x3F3100, 125),
        (0xB35F00, 126),
        (0x4B1502, 127),
    ]);
}

#[derive(Clone, Copy)]
pub struct AkaiApcMiniMk2 {}

fn color_square(rgb: u32) -> (u32, u32, u32) {
    let r = (0xff0000 & rgb) >> 16;
    let g = (0x00ff00 & rgb) >> 8;
    let b = 0x0000ff & rgb;
    (r.pow(2), g.pow(2), b.pow(2))
}

fn _color_square_tuple(rgb: u32) -> (u32, (u32, u32, u32)) {
    (rgb, color_square(rgb))
}

fn color_distance(
    (ra, ga, ba): (u32, u32, u32),
    (rb, gb, bb): (u32, u32, u32),
) -> u32 {
    ((ra as i64 - rb as i64).abs()
        + (ga as i64 - gb as i64).abs()
        + (ba as i64 - bb as i64).abs()) as u32
}

fn nearest_color(rgb: u32) -> u32 {
    // A cheap shortcut. May not need it.
    if rgb == 0 {
        return 0;
    }
    let squared = color_square(rgb);
    AKAI_APC_MINI_MK_2_COLORS_SQUARED
        .to_vec()
        .into_iter()
        .map(|(x, fixed)| (x, color_distance(squared, fixed)))
        .sorted_by_key(|(_, fixed)| fixed.clone())
        .collect::<Vec<(u32, u32)>>()
        .get(0)
        .map(|(original, _square)| *original)
        // We should always get one because we know the list is populated. Therefore
        // this unwrap should always be safe.
        .unwrap_or(0)
}

fn color_style_to_u32(style: ColorStyle) -> u32 {
    match style {
        ColorStyle::Steady100 => LED_100_BRIGHT,
        ColorStyle::Steady95 => LED_95_BRIGHT,
        ColorStyle::Steady75 => LED_75_BRIGHT,
        ColorStyle::Steady65 => LED_65_BRIGHT,
        ColorStyle::Steady50 => LED_50_BRIGHT,
        ColorStyle::Steady25 => LED_25_BRIGHT,
        ColorStyle::Steady10 => LED_10_BRIGHT,
        ColorStyle::Pulse2 => PULSING_1_2,
        ColorStyle::Pulse4 => PULSING_1_4,
        ColorStyle::Pulse8 => PULSING_1_8,
        ColorStyle::Pulse16 => PULSING_1_16,
        ColorStyle::Blink2 => BLINKING_1_2,
        ColorStyle::Blink4 => BLINKING_1_4,
        ColorStyle::Blink8 => BLINKING_1_8,
        ColorStyle::Blink16 => BLINKING_1_16,
        ColorStyle::Blink24 => BLINKING_1_24,
    }
}

fn set_grid_button_internal(
    output_port: &OutputPort,
    dest: &Destination,
    x: usize,
    y: usize,
    color: Color,
) -> Result<(), AppError> {
    let nearest = nearest_color(color.rgb);
    // println!("Color nearest to {:08x}: {:08x}", color.rgb, nearest);
    let payload = NOTE_ON_STATUS
        | color_style_to_u32(color.style)
        | (x as u32 + (y as u32 * 8)) << 8
        | COLORS_BY_VELOCITY.get(&nearest).unwrap_or(&0);
    let note_on = EventBuffer::new(Protocol::Midi10).with_packet(0, &[payload]);
    output_port
        .send(&dest, &note_on)
        .map_err(AppError::OutputSendError)
}

impl Device for AkaiApcMiniMk2 {
    fn midi_to_action(&self, _context: u32, data: u32) -> Action {
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
            } else if grid >= TRACK_OFFSET && grid <= 0x6b {
                let bottom_button = grid - TRACK_OFFSET;
                println!("Bottom button: {}", bottom_button);
                Action::SectionSelect { pos: bottom_button }
            } else if grid >= SCENE_LAUNCH_OFFSET && grid < SHIFT_BUTTON {
                let scene_launch_button = grid - SCENE_LAUNCH_OFFSET;
                // println!("Scene launch button: {}", scene_launch_button);
                Action::LayerSelect {
                    pos: scene_launch_button,
                }
            } else if grid == SHIFT_BUTTON {
                println!("Changing play mode to playing");
                Action::PlayModeChange(PlayMode::Playing)
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

    fn set_grid_button(
        &self,
        output_port: &OutputPort,
        dest: &Destination,
        x: usize,
        y: usize,
        color: Color,
    ) -> Result<(), AppError> {
        set_grid_button_internal(
            &output_port,
            &dest,
            x,
            y,
            Color {
                rgb: color.rgb,
                style: ColorStyle::Steady75,
            },
        )
    }

    fn set_interval(
        &self,
        output_port: &OutputPort,
        dest: &Destination,
        x: usize,
        y: usize,
        color: Color,
    ) -> Result<(), AppError> {
        set_grid_button_internal(
            &output_port,
            &dest,
            x,
            y,
            Color {
                rgb: color.rgb,
                style: ColorStyle::Steady50,
            },
        )
    }

    fn set_layer_button(
        &self,
        output_port: &OutputPort,
        dest: &Destination,
        layer_index: usize,
        color: Color,
    ) -> Result<(), AppError> {
        // Always use NoteOn even though we turn off buttons this way.
        let payload = NOTE_ON_STATUS
            | (SCENE_LAUNCH_OFFSET + layer_index as u32) << 8
            | color.rgb;
        // println!("Setting Layer button {} to color {:08x} as payload {:08x}", layer_index, color.rgb, payload);
        let event =
            EventBuffer::new(Protocol::Midi10).with_packet(0, &[payload]);
        output_port
            .send(&dest, &event)
            .map_err(AppError::OutputSendError)
    }

    fn set_play_button(
        &self,
        output_port: &OutputPort,
        dest: &Destination,
        color: Color,
    ) -> Result<(), AppError> {
        // Always use NoteOn even though we turn off buttons this way.
        let payload = NOTE_ON_STATUS | SHIFT_BUTTON << 8 | color.rgb;
        println!(
            "Setting play button to color {:08x} as payload {:08x}",
            color.rgb, payload,
        );
        let event =
            EventBuffer::new(Protocol::Midi10).with_packet(0, &[payload]);
        output_port
            .send(&dest, &event)
            .map_err(AppError::OutputSendError)
    }

    fn set_section_button(
        &self,
        output_port: &OutputPort,
        dest: &Destination,
        section_index: usize,
        color: Color,
    ) -> Result<(), AppError> {
        // Always use NoteOn even though we turn off buttons this way.
        let payload = NOTE_ON_STATUS
            | (TRACK_OFFSET + section_index as u32) << 8
            | color.rgb;
        // println!("Setting section button {} to color {:08x} as payload {:08x}", section_index, color.rgb, payload);
        let event =
            EventBuffer::new(Protocol::Midi10).with_packet(0, &[payload]);
        output_port
            .send(&dest, &event)
            .map_err(AppError::OutputSendError)
    }
}
