use sdl2::mixer::{self, InitFlag, Sdl2MixerContext, AUDIO_S16LSB, DEFAULT_CHANNELS};

/// holds the audio backend attributes
pub struct AudioState {
    _audio_subsystem: sdl2::AudioSubsystem,
    _mixer_context: Sdl2MixerContext,
}

impl AudioState {
    /// creates a new audio state
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let _audio_subsystem = sdl_context.audio().unwrap();
        mixer::open_audio(44100, AUDIO_S16LSB, DEFAULT_CHANNELS, 1024).unwrap();
        let _mixer_context = mixer::init(InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD).unwrap();

        mixer::allocate_channels(5);

        Self {
            _audio_subsystem,
            _mixer_context,
        }
    }
}

/// converts 0-100% volume sliders to absolute volume
pub fn convert_volume(master: f64, specific: f64) -> i32 {
    (128f64 * master * specific) as i32
}
