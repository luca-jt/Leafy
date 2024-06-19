use crate::state::audio_state::AudioState;

/// system managing the audio
pub struct AudioSystem {
    audio_state: AudioState,
}

impl AudioSystem {
    /// creates a new audio system
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        Self {
            audio_state: AudioState::new(sdl_context),
        }
    }

    /// update audio volume etc
    pub fn update(&mut self) {
        //...
    }
}
