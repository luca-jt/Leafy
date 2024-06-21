use crate::utils::file::get_audio_path;
use sdl2::mixer::{InitFlag, Sdl2MixerContext, AUDIO_S16LSB, DEFAULT_CHANNELS};
use std::collections::VecDeque;
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;

/// holds the audio backend attributes
pub struct AudioState {
    _audio_subsystem: sdl2::AudioSubsystem,
    _mixer_context: Sdl2MixerContext,
    master_volume: f64,
    music_volume: f64,
    sfx_volume: f64,
    threads: VecDeque<JoinHandle<()>>,
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
            master_volume: 0.0,
            music_volume: 0.0,
            sfx_volume: 0.0,
            threads: VecDeque::new(),
        }
    }

    /// plays a sound effect from a given file
    pub fn play_sfx(&mut self, file_name: &str) {
        let full_path = get_audio_path(file_name);
        let volume = convert_volume(self.master_volume, self.sfx_volume);

        let sfx_thread = thread::spawn(move || {
            let mut sfx = sdl2::mixer::Chunk::from_file(full_path).unwrap();
            sfx.set_volume(volume);
            sdl2::mixer::Channel::all().play(&sfx, 0).ok();
            sleep(Duration::from_secs(2));
        });
        self.threads.push_back(sfx_thread);
    }
}

impl Drop for AudioState {
    fn drop(&mut self) {
        while self.threads.is_empty() {
            let thread = self.threads.pop_front().unwrap();
            thread.join().unwrap();
        }
    }
}

/// converts 0-100% volume sliders to absolute volume
fn convert_volume(master: f64, specific: f64) -> i32 {
    (128f64 * master * specific) as i32
}
