use crate::action::Action;

pub trait Device {
    fn midi_to_action(&self, context: u32, packet: u32) -> Action;
}
