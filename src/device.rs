use coremidi::{Destination, OutputPort};

use crate::{action::Action, error::AppError};

// TODO: This should be part of the concrete device.
pub enum ColorStyle {
    Steady100,
    Steady95,
    Steady75,
    Steady65,
    Steady50,
    Steady25,
    Steady10,
    Pulse2,
    Pulse4,
    Pulse8,
    Pulse16,
    Blink2,
    Blink4,
    Blink8,
    Blink16,
    Blink24,
}

pub struct Color {
    pub rgb: u32,
    pub style: ColorStyle,
}

pub trait Device {
    fn midi_to_action(&self, context: u32, packet: u32) -> Action;

    fn set_grid_button(
        &self,
        output_port: &OutputPort,
        dest: &Destination,
        x: usize,
        y: usize,
        color: Color,
    ) -> Result<(), AppError>;

    fn set_interval(
        &self,
        output_port: &OutputPort,
        dest: &Destination,
        x: usize,
        y: usize,
        color: Color,
    ) -> Result<(), AppError>;

    fn set_layer_button(
        &self,
        output_port: &OutputPort,
        dest: &Destination,
        layer: usize,
        color: Color,
    ) -> Result<(), AppError>;

    fn set_play_button(
        &self,
        output_port: &OutputPort,
        dest: &Destination,
        color: Color,
    ) -> Result<(), AppError>;

    fn set_section_button(
        &self,
        output_port: &OutputPort,
        dest: &Destination,
        section_index: usize,
        color: Color,
    ) -> Result<(), AppError>;

}
