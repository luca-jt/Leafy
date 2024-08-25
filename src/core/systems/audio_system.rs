use crate::ecs::component::{Position, SoundID};
use crate::ecs::entity::EntityID;
use crate::ecs::entity_manager::EntityManager;
use crate::ecs::query::{ExcludeFilter, IncludeFilter};
use crate::utils::file::get_audio_path;
use crate::{exclude_filter, include_filter};
use ambisonic::rodio::Source;
use ambisonic::{rodio, Ambisonic, AmbisonicBuilder, SoundController};
use std::collections::HashMap;
use std::io::BufReader;

pub type SoundControllerID = u64;

/// system managing the audio
pub struct AudioSystem {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    scene: Ambisonic,
    background_music: Option<SoundController>,
    sounds: HashMap<EntityID, Vec<SoundController>>,
}

impl AudioSystem {
    /// creates a new audio system
    pub fn new() -> Self {
        let scene = AmbisonicBuilder::default().build();

        Self {
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            scene,
            background_music: None,
            sounds: HashMap::new(),
        }
    }

    /// update audio volume etc
    pub fn update(&mut self, entity_manager: &EntityManager) {
        for (pos, sound) in entity_manager
            .ecs
            .query2::<Position, SoundID>(include_filter!(), exclude_filter!())
        {
            // todo: adjust volume + positions etc.
        }
    }

    /// plays a sound effect from a given file
    pub fn play_sfx(&mut self, file_name: &str) -> SoundController {
        let full_path = get_audio_path(file_name);
        let volume = convert_volume(self.master_volume, self.sfx_volume);

        let file = std::fs::File::open(full_path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        let source = source.stoppable().pausable(false).amplify(volume);

        self.scene.play_omni(source.convert_samples())
    }

    /// plays a sound effect from a given file for a specific entity
    pub fn play_sfx_on_at(&mut self, file_name: &str, entity: EntityID, position: &Position) {
        let full_path = get_audio_path(file_name);
        let volume = convert_volume(self.master_volume, self.sfx_volume);

        let file = std::fs::File::open(full_path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        let source = source.stoppable().pausable(false).amplify(volume);
        let play_position = [position.data().x, position.data().y, position.data().z];
        let sound = self.scene.play_at(source.convert_samples(), play_position);

        self.sounds.entry(entity).or_insert(Vec::new()).push(sound);
    }

    /// plays the music from a given file in a loop
    pub fn play_music(&mut self, file_name: &str) {
        let full_path = get_audio_path(file_name);
        let volume = convert_volume(self.master_volume, self.music_volume);

        let file = std::fs::File::open(full_path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        let source = source
            .repeat_infinite()
            .stoppable()
            .pausable(false)
            .amplify(volume);
        let sound = self.scene.play_omni(source.convert_samples());

        self.stop_music();
        self.background_music = Some(sound);
    }

    /// stops the background music from playing
    pub fn stop_music(&mut self) {
        if let Some(sound) = self.background_music.take() {
            sound.stop();
        }
    }
}

/// converts 0-200% volume settings to the resulting volume
pub fn convert_volume(master: f32, specific: f32) -> f32 {
    master * specific
}
