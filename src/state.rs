const NOTE_COUNT: usize = 8;

#[derive(Clone, Default)]
pub enum PlayMode {
    Paused,
    Playing,
    #[default]
    Stopped,
}

#[derive(Clone)]
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
    pub active_section: usize,
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
        sections: vec![Section {
            layers: vec![Layer {
                notes: [
                    Note {
                        octaves: vec![],
                        length: 0,
                    },
                    Note {
                        octaves: vec![],
                        length: 0,
                    },
                    Note {
                        octaves: vec![],
                        length: 0,
                    },
                    Note {
                        octaves: vec![],
                        length: 0,
                    },
                    Note {
                        octaves: vec![],
                        length: 0,
                    },
                    Note {
                        octaves: vec![],
                        length: 0,
                    },
                    Note {
                        octaves: vec![],
                        length: 0,
                    },
                    Note {
                        octaves: vec![],
                        length: 0,
                    },
                ],
                instrument: "Beep Boops".to_string(),
            }],
        }],
        player: Player {
            play_mode: PlayMode::Paused,
            active_layer_index: 0,
            active_section: 0,
        },
    }
}
