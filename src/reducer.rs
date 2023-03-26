use crate::action::Action;
use crate::state::{GlobalState, Note};

pub fn reducer(state: GlobalState, action: Action) -> GlobalState {
    match action {
        Action::Noop => state,
        Action::LayerSelect { pos } => {
            let mut new_state = state.clone();
            new_state.player.active_layer_index = pos as usize;
            new_state
        }
        Action::GridToggle { x, y } => {
            let mut new_state = state.clone();
            let layer_opt = new_state
                .sections
                .get_mut(state.player.active_section_index)
                .and_then(|section| {
                    section.layers.get_mut(state.player.active_layer_index)
                });
            match layer_opt {
                Some(layer) => {
                    if let Some(note) = layer.notes.get_mut(x as usize) {
                        let new_octaves =
                            if note.octaves.contains(&(y as usize)) {
                                note.octaves
                                    .iter()
                                    .filter(|a| **a as u32 != y)
                                    // TODO: Kill myself for doing this.
                                    .map(|a| a.clone()) // Ugh. Whhhhhyy?
                                    .collect::<Vec<usize>>()
                            } else {
                                let mut octaves: Vec<usize> =
                                    note.octaves.to_vec();
                                octaves.push(y as usize);
                                octaves
                            };
                        *note = Note {
                            length: 1,
                            octaves: new_octaves,
                        };
                        new_state
                    } else {
                        state
                    }
                }
                None => state,
            }
        }
        Action::PlayModeChange(play_mode) => {
            let mut new_state = state.clone();
            new_state.player = state.player.clone();
            new_state.player.play_mode = play_mode;
            new_state
        }
        Action::SectionSelect { pos } => {
            let mut new_state = state.clone();
            new_state.player.active_section_index = pos as usize;
            new_state
        }
        Action::TimeInterval => {
            let mut new_state = state.clone();
            new_state.player.interval = state.player.interval + 1;
            new_state
        }
    }
}
