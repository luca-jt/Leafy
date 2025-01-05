use crate::ecs::component::{Position, SoundController};
use crate::ecs::entity_manager::EntityManager;
use crate::engine::EngineMode;
use crate::systems::event_system::events::*;
use crate::systems::event_system::EventObserver;
use crate::utils::file::HRTF_SPHERE;
use crate::utils::tools::vec3_to_vector3;
use fyrox_resource::io::FsResourceIo;
use fyrox_resource::untyped::ResourceKind;
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
use std::collections::HashMap;
use std::path::Path;

pub type SoundHandleID = u64;

/// system managing the audio
pub struct AudioSystem {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    _sound_engine: SoundEngine,
    sound_context: SoundContext,
    background_music: Option<Handle<SoundSource>>,
    next_handle_id: SoundHandleID,
    sound_register: HashMap<SoundHandleID, Vec<Handle<SoundSource>>>,
    pitch_on_speed_change: bool,
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
            _sound_engine: sound_engine,
            sound_context,
            background_music: None,
            next_handle_id: 1,
            sound_register: HashMap::new(),
            pitch_on_speed_change: true,
        }
    }

    /// update audio volume etc (runs every frame)
    pub(crate) fn update(&mut self, entity_manager: &EntityManager) {
        // update music volume
        let music_volume = self.calc_absolute_volume(VolumeType::Music);
        if let Some(music_handle) = self.background_music {
            let mut state = self.sound_context.state();
            let music_source = state.source_mut(music_handle);
            music_source.set_gain(music_volume);
        }
        // update entity sound positions
        let sfx_volume = self.calc_absolute_volume(VolumeType::SFX);
        let mut active_ids = vec![];
        for (pos, sound) in entity_manager.query2::<Position, SoundController>((None, None)) {
            active_ids.push(sound.id);
            let handles = self.sound_register.get(&sound.id).unwrap();
            for handle in handles {
                let mut state = self.sound_context.state();
                let source = state.source_mut(*handle);
                source.set_gain(sfx_volume);
                let pos_data = pos.data();
                source.set_position(vec3_to_vector3(pos_data));
            }
        }
        // clean up unused controllers
        self.sound_register.retain(|id, _| active_ids.contains(id));
    }

    /// spawns a new sound controller component
    pub fn new_sound_controller(&mut self) -> SoundController {
        let new_id = self.next_handle_id;
        self.next_handle_id += 1;
        self.sound_register.insert(new_id, Vec::new());
        SoundController { id: new_id }
    }

    /// plays a sound effect from a given file
    pub fn play_sfx(&self, file_path: impl AsRef<Path>, looping: bool) -> Handle<SoundSource> {
        let volume = self.calc_absolute_volume(VolumeType::SFX);

        let buffer = SoundBufferResource::new_generic(
            block_on(DataSource::from_file(file_path, &FsResourceIo)).unwrap(),
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
        file_path: impl AsRef<Path>,
        looping: bool,
        controller: &SoundController,
        position: &Position,
    ) {
        let volume = self.calc_absolute_volume(VolumeType::SFX);

        let buffer = SoundBufferResource::new_generic(
            block_on(DataSource::from_file(file_path, &FsResourceIo)).unwrap(),
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
            .expect("no entity with this controller")
            .push(handle);
        let pos_data = position.data();
        self.sound_context
            .state()
            .source_mut(handle)
            .set_position(vec3_to_vector3(pos_data));
    }

    /// plays the music from a given file in a loop (overwrites the bg music playing before)
    pub fn play_background_music(&mut self, file_path: impl AsRef<Path>) {
        let volume = self.calc_absolute_volume(VolumeType::Music);
        self.stop_background_music();

        let buffer = SoundBufferResource::new_generic(
            block_on(DataSource::from_file(file_path, &FsResourceIo)).unwrap(),
        )
        .unwrap();

        let source = SoundSourceBuilder::new()
            .with_buffer(buffer)
            .with_looping(true)
            .with_status(Status::Playing)
            .with_spatial_blend_factor(0.0)
            .with_gain(volume)
            .build()
            .unwrap();

        let handle = self.sound_context.state().add_source(source);
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
        log::trace!("enabled HRTF");
        let hrir_sphere = HrirSphere::new(HRTF_SPHERE, context::SAMPLE_RATE).unwrap();

        self.sound_context
            .state()
            .set_renderer(Renderer::HrtfRenderer(HrtfRenderer::new(
                HrirSphereResource::from_hrir_sphere(hrir_sphere, ResourceKind::Embedded),
            )));
    }

    /// enable a reverb effect (enables hrtf)
    pub fn enable_reverb(&self, decay_time: f32) {
        log::trace!("enabled reverb");
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
        log::trace!("disabled reverb");
        self.sound_context
            .state()
            .bus_graph_mut()
            .primary_bus_mut()
            .remove_effect(0); // might need specification later on with more effects
    }

    /// changes the volume of the given type to the specified value
    pub fn set_volume(&mut self, volume_type: VolumeType, new_volume: f32) {
        log::trace!("set volume {:?} to {:?}", volume_type, new_volume);
        match volume_type {
            VolumeType::Master => {
                self.master_volume = new_volume;
            }
            VolumeType::SFX => {
                self.sfx_volume = new_volume;
            }
            VolumeType::Music => {
                self.music_volume = new_volume;
            }
        }
    }

    /// enables/disbles the pitch change on animation speed change (default is true)
    pub fn set_pitch_on_speed_change(&mut self, flag: bool) {
        log::trace!("set pitch on speed change: {:?}", flag);
        self.pitch_on_speed_change = flag;
    }

    /// calculate the total resulting volume for either the sfx or music
    fn calc_absolute_volume(&self, volume: VolumeType) -> f32 {
        match volume {
            VolumeType::Master => {
                panic!("master volume is not supported");
            }
            VolumeType::SFX => self.master_volume * self.sfx_volume,
            VolumeType::Music => self.master_volume * self.music_volume,
        }
    }
}

impl EventObserver<CamPositionChange> for AudioSystem {
    fn on_event(&mut self, event: &CamPositionChange) {
        let mut state = self.sound_context.state();
        let listener = state.listener_mut();
        listener.set_orientation_lh(vec3_to_vector3(&event.new_look), *Vector3::y_axis());
        listener.set_position(vec3_to_vector3(&event.new_pos));
    }
}

impl EventObserver<AnimationSpeedChange> for AudioSystem {
    fn on_event(&mut self, event: &AnimationSpeedChange) {
        if self.pitch_on_speed_change {
            let mut state = self.sound_context.state();
            for source in state.sources_mut().iter_mut() {
                source.set_pitch(event.new_animation_speed as f64);
            }
        }
    }
}

impl EventObserver<EngineModeChange> for AudioSystem {
    fn on_event(&mut self, event: &EngineModeChange) {
        match event.new_mode {
            EngineMode::Running | EngineMode::Editor => {
                let mut state = self.sound_context.state();
                for source in state.sources_mut().iter_mut() {
                    source.play();
                }
            }
            EngineMode::Paused => {
                let mut state = self.sound_context.state();
                for source in state.sources_mut().iter_mut() {
                    source.pause();
                }
            }
        }
    }
}

/// all versions of audio volume
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VolumeType {
    Master,
    SFX,
    Music,
}
