use sdl2::mixer::{InitFlag, Sdl2MixerContext, AUDIO_S16LSB, DEFAULT_CHANNELS};

/// holds the audio backend attributes
pub struct AudioState {
    _audio_subsystem: sdl2::AudioSubsystem,
    _mixer_context: Sdl2MixerContext,
}

impl AudioState {
    /// creates a new audio state
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let _audio_subsystem = sdl_context.audio().unwrap();
        sdl2::mixer::open_audio(44100, AUDIO_S16LSB, DEFAULT_CHANNELS, 1024).unwrap();
        let _mixer_context =
            sdl2::mixer::init(InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD).unwrap();

        sdl2::mixer::allocate_channels(5);

        Self {
            _audio_subsystem,
            _mixer_context,
        }
    }
}
