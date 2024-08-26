use crate::ecs::component::{Position, SoundControl};
use crate::ecs::entity::EntityID;
use crate::ecs::entity_manager::EntityManager;
use crate::ecs::query::{ExcludeFilter, IncludeFilter};
use crate::utils::file::get_audio_path;
use crate::{exclude_filter, include_filter};
use std::collections::HashMap;
use std::path::PathBuf;

use fyrox_resource::io::FsResourceIo;
use fyrox_sound::buffer::SoundBufferResourceExtension;
use fyrox_sound::renderer::hrtf::{HrirSphereResource, HrirSphereResourceExt};
use fyrox_sound::{
    buffer::{DataSource, SoundBufferResource},
    context::{self, SoundContext},
    engine::SoundEngine,
    futures::executor::block_on,
    hrtf::HrirSphere,
    pool::Handle,
    renderer::{hrtf::HrtfRenderer, Renderer},
    source::{SoundSource, SoundSourceBuilder, Status},
};

/// system managing the audio
pub struct AudioSystem {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    sound_engine: SoundEngine,
    sound_context: SoundContext,
    background_music: Option<Handle<SoundSource>>,
    sounds: HashMap<EntityID, Vec<Handle<SoundSource>>>,
}

impl AudioSystem {
    /// creates a new audio system
    pub fn new() -> Self {
        let sound_engine = SoundEngine::new().unwrap();
        let sound_context = SoundContext::new();
        sound_engine.state().add_context(sound_context.clone());

        Self {
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            sound_engine,
            sound_context,
            background_music: None,
            sounds: HashMap::new(),
        }
    }

    /// update audio volume etc (runs every frame)
    pub fn update(&mut self, entity_manager: &EntityManager) {
        for (pos, sound) in entity_manager
            .ecs
            .query2::<Position, SoundControl>(include_filter!(), exclude_filter!())
        {
            // todo: adjust volume + positions + listener pos etc.
        }
    }

    /// use sound rendering on the hrtf sphere
    pub fn enable_hrtf(&self) {
        let hrir_path = PathBuf::from(get_audio_path("IRC_1002_C.bin"));
        let hrir_sphere = HrirSphere::from_file(&hrir_path, context::SAMPLE_RATE).unwrap();

        self.sound_context
            .state()
            .set_renderer(Renderer::HrtfRenderer(HrtfRenderer::new(
                HrirSphereResource::from_hrir_sphere(hrir_sphere, hrir_path.into()),
            )));
    }

    /// plays a sound effect from a given file
    pub fn play_sfx(&self, file_name: &str, looping: bool) -> Handle<SoundSource> {
        let full_path = get_audio_path(file_name);
        let volume = convert_volume(self.master_volume, self.sfx_volume);

        let buffer = SoundBufferResource::new_generic(
            block_on(DataSource::from_file(full_path, &FsResourceIo)).unwrap(),
        )
        .unwrap();

        let source = SoundSourceBuilder::new()
            .with_buffer(buffer)
            .with_looping(looping)
            .with_status(Status::Playing)
            .with_spatial_blend_factor(0.0)
            .build()
            .unwrap();

        self.sound_context.state().add_source(source)
    }

    /// plays a sound effect from a given file for a specific entity
    pub fn play_sfx_on_at(
        &mut self,
        file_name: &str,
        looping: bool,
        entity: EntityID,
        position: &Position,
    ) {
        let full_path = get_audio_path(file_name);
        let volume = convert_volume(self.master_volume, self.sfx_volume);

        let buffer = SoundBufferResource::new_generic(
            block_on(DataSource::from_file(full_path, &FsResourceIo)).unwrap(),
        )
        .unwrap();

        let source = SoundSourceBuilder::new()
            .with_buffer(buffer)
            .with_looping(looping)
            .with_status(Status::Playing)
            .build()
            .unwrap();

        let handle = self.sound_context.state().add_source(source);
        // todo: add handle to register
    }

    /// plays the music from a given file in a loop
    pub fn play_background_music(&mut self, file_name: &str) {
        self.stop_background_music();
        let handle = self.play_sfx(file_name, true);
        self.background_music = Some(handle);
    }

    /// stops the background music from playing
    pub fn stop_background_music(&mut self) {
        if let Some(sound) = self.background_music.take() {
            // todo
        }
    }
}

/// converts 0-200% volume settings to the resulting volume
pub fn convert_volume(master: f32, specific: f32) -> f32 {
    master * specific
}
