use crate::state::PlayMode;

pub enum Action {
    Noop,
    BottomToggle { pos: u32 },
    GridToggle { x: u32, y: u32 },
    LayerSelect { pos: u32 },
    PlayModeChange(PlayMode),
}
