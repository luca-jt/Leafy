use crate::utils::file::get_audio_path;
use sdl2::mixer;
use sdl2::mixer::{InitFlag, Sdl2MixerContext, AUDIO_S16LSB, DEFAULT_CHANNELS};
use std::collections::VecDeque;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

/// system managing the audio
pub struct AudioSystem {
    _audio_state: AudioState,
    master_volume: f64,
    music_volume: f64,
    sfx_volume: f64,
    threads: VecDeque<JoinHandle<()>>,
}

impl AudioSystem {
    /// creates a new audio system
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        Self {
            _audio_state: AudioState::new(sdl_context),
            master_volume: 0.0,
            music_volume: 0.0,
            sfx_volume: 0.0,
            threads: VecDeque::new(),
        }
    }

    /// update audio volume etc
    pub fn update(&mut self) {
        //...
        mixer::Music::set_volume(convert_volume(self.master_volume, self.music_volume));
    }

    /// plays a sound effect from a given file
    pub fn play_sfx(&mut self, file_name: &str) {
        let full_path = get_audio_path(file_name);
        let volume = convert_volume(self.master_volume, self.sfx_volume);

        let sfx_thread = thread::spawn(move || {
            let mut sfx = mixer::Chunk::from_file(full_path).unwrap();
            sfx.set_volume(volume);
            mixer::Channel::all().play(&sfx, 0).ok();
            sleep(Duration::from_secs(2));
        });
        self.threads.push_back(sfx_thread);
    }

    /// plays the music from a given file in a loop
    pub fn play_music(&mut self, file_name: &str) {
        let full_path = get_audio_path(file_name);
        let volume = convert_volume(self.master_volume, self.music_volume);
        mixer::Music::halt();

        let music_thread = thread::spawn(move || {
            let music = mixer::Music::from_file(full_path).unwrap();
            mixer::Music::set_volume(volume);
            music.play(-1).unwrap();
            while mixer::Music::is_playing() {
                sleep(Duration::from_millis(100));
            }
        });
        self.threads.push_back(music_thread);
    }
}

impl Drop for AudioSystem {
    fn drop(&mut self) {
        mixer::Music::halt();
        mixer::close_audio();
        while self.threads.is_empty() {
            let thread = self.threads.pop_front().unwrap();
            thread.join().expect("could not join thread");
        }
    }
}

/// holds the audio backend attributes
pub struct AudioState {
    _audio_subsystem: sdl2::AudioSubsystem,
    _mixer_context: Sdl2MixerContext,
}

impl AudioState {
    /// creates a new audio state
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let _audio_subsystem = sdl_context.audio().expect("sdl audio failed");
        mixer::open_audio(44100, AUDIO_S16LSB, DEFAULT_CHANNELS, 1024).expect("audio open failed");
        let _mixer_context =
            mixer::init(InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD).expect("mixer init failed");

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
