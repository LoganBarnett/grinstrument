use crate::state::PlayMode;

pub enum Action {
    Noop,
    GridToggle { x: u32, y: u32 },
    LayerSelect { pos: u32 },
    PlayModeChange(PlayMode),
    SectionSelect { pos: u32 },
}
