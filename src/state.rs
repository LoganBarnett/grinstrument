const NOTE_COUNT: usize = 8;

#[derive(Clone, Default)]
pub enum PlayMode {
    Paused,
    Playing,
    #[default]
    Stopped,
}

#[derive(Clone, Debug)]
pub struct Note {
    pub octaves: Vec<usize>,
    pub length: usize,
}

/**
 * A Layer represents a collection of notes for an instrument, which can overlap
 * with other layers or be sequenced against other layers.
 */
#[derive(Clone)]
pub struct Layer {
    pub notes: [Note; NOTE_COUNT],
    pub instrument: String,
}

/**
 * A Player represents the play state. What are we playing? Are we playing at
 * all? Are we looping?
 */
#[derive(Clone, Default)]
pub struct Player {
    pub play_mode: PlayMode,
    pub active_layer_index: usize,
    pub active_section_index: usize,
}

/**
 * Sections contain one or more layers. All of the layers in a section are
 * played in parallel. Sections can be sequenced together.
 */
#[derive(Clone)]
pub struct Section {
    pub layers: Vec<Layer>,
}

#[derive(Default, Clone)]
pub struct GlobalState {
    pub sections: Vec<Section>,
    pub player: Player,
}

pub fn initial_state() -> GlobalState {
    GlobalState {
        sections: (0..8)
            .map(|_| {
                Section {
                    layers: (0..8)
                        .map(|_| {
                            Layer {
                                instrument: "Beep Boops".to_string(),
                                notes: (0..8)
                                    .map(|_| {
                                        Note {
                                            octaves: vec![],
                                            length: 0,
                                        }
                                    })
                                    .collect::<Vec<Note>>()
                                    .try_into()
                                    .unwrap()
                            }
                        })
                        .collect::<Vec<Layer>>(),
                }
            })
            .collect::<Vec<Section>>(),
        player: Player {
            play_mode: PlayMode::Paused,
            active_layer_index: 0,
            active_section_index: 0,
        },
    }
}
