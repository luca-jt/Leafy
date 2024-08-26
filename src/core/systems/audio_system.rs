use crate::ecs::component::{Position, SoundController};
use crate::ecs::entity_manager::EntityManager;
use crate::ecs::query::{ExcludeFilter, IncludeFilter};
use crate::utils::file::get_audio_path;
use crate::{exclude_filter, include_filter};
use nalgebra_glm as glm;
use std::collections::HashMap;
use std::path::PathBuf;

use fyrox_resource::io::FsResourceIo;
use fyrox_sound::{
    algebra::Vector3,
    buffer::{DataSource, SoundBufferResource, SoundBufferResourceExtension},
    context::{self, SoundContext},
    effects::{reverb::Reverb, Effect},
    engine::SoundEngine,
    futures::executor::block_on,
    hrtf::HrirSphere,
    pool::Handle,
    renderer::{
        hrtf::{HrirSphereResource, HrirSphereResourceExt, HrtfRenderer},
        Renderer,
    },
    source::{SoundSource, SoundSourceBuilder, Status},
};

pub type SoundHandleID = u64;

/// system managing the audio
pub struct AudioSystem {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    sound_engine: SoundEngine,
    sound_context: SoundContext,
    background_music: Option<Handle<SoundSource>>,
    next_handle_id: SoundHandleID,
    sound_register: HashMap<SoundHandleID, Vec<Handle<SoundSource>>>,
}

impl AudioSystem {
    /// creates a new audio system
    pub(crate) fn new() -> Self {
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
            next_handle_id: 0,
            sound_register: HashMap::new(),
        }
    }

    /// update audio volume etc (runs every frame)
    pub(crate) fn update(
        &mut self,
        entity_manager: &EntityManager,
        listener_pos: glm::Vec3,
        listener_look: glm::Vec3,
    ) {
        // update music volume
        let music_volume = convert_volume(self.master_volume, self.music_volume);
        if let Some(music_handle) = self.background_music {
            let mut state = self.sound_context.state();
            let music_source = state.source_mut(music_handle);
            music_source.set_gain(music_volume);
        }
        // update entity sound positions
        let sfx_volume = convert_volume(self.master_volume, self.sfx_volume);
        for (pos, sound) in entity_manager
            .ecs
            .query2::<Position, SoundController>(include_filter!(), exclude_filter!())
        {
            let handles = self.sound_register.get(&sound.id).unwrap();
            for handle in handles {
                let mut state = self.sound_context.state();
                let source = state.source_mut(*handle);
                source.set_gain(sfx_volume);
                let pos_data = pos.data();
                source.set_position(Vector3::new(pos_data.x, pos_data.y, pos_data.z));
            }
        }
        // update the listeners position
        let mut state = self.sound_context.state();
        let listener = state.listener_mut();
        listener.set_orientation_lh(
            Vector3::new(listener_look.x, listener_look.y, listener_look.z),
            *Vector3::y_axis(),
        );
        listener.set_position(Vector3::new(listener_pos.x, listener_pos.y, listener_pos.z));
    }

    /// spawns a new sound controller component
    pub fn new_sound_controller(&mut self) -> SoundController {
        let new_id = self.next_handle_id;
        self.next_handle_id += 1;
        self.sound_register.insert(new_id, Vec::new());
        SoundController { id: new_id }
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
            .with_gain(volume)
            .build()
            .unwrap();

        self.sound_context.state().add_source(source)
    }

    /// plays a sound effect from a given file for a specific entity
    pub fn play_sfx_at(
        &mut self,
        file_name: &str,
        looping: bool,
        controller: &SoundController,
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
            .with_gain(volume)
            .build()
            .unwrap();

        let handle = self.sound_context.state().add_source(source);
        self.sound_register
            .get_mut(&controller.id)
            .unwrap()
            .push(handle);
        let pos_data = position.data();
        self.sound_context
            .state()
            .source_mut(handle)
            .set_position(Vector3::new(pos_data.x, pos_data.y, pos_data.z));
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
            let mut state = self.sound_context.state();
            let source = state.source_mut(sound);
            source.stop().unwrap();
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

    /// enable a reverb effect (enables hrtf)
    pub fn enable_reverb(&self, decay_time: f32) {
        self.enable_hrtf();
        let mut reverb = Reverb::new();
        reverb.set_decay_time(decay_time);

        self.sound_context
            .state()
            .bus_graph_mut()
            .primary_bus_mut()
            .add_effect(Effect::Reverb(reverb));
    }

    /// disable the reverb effect (does not disable hrtf)
    pub fn disable_reverb(&self) {
        self.sound_context
            .state()
            .bus_graph_mut()
            .primary_bus_mut()
            .remove_effect(0); // might need specification later on with more effects
    }
}

/// converts 0-200% volume settings to the resulting volume
pub(crate) fn convert_volume(master: f32, specific: f32) -> f32 {
    master * specific
}
