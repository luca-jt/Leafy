use crate::state::audio_state::AudioState;

/// system managing the audio
pub struct AudioSystem {
    audio_state: AudioState,
}

impl AudioSystem {
    /// creates a new audio system
    pub fn new() -> Self {
        Self {
            audio_state: AudioState::new(),
        }
    }
}
