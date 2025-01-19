use crate::ecs::component::{Position, SoundController};
use crate::ecs::entity_manager::EntityManager;
use crate::engine::EngineMode;
use crate::systems::event_system::events::*;
use crate::systems::event_system::EventObserver;
use crate::utils::file::HRTF_SPHERE;
use crate::utils::tools::vec3_to_vector3;
use fyrox_resource::io::FsResourceIo;
use fyrox_resource::untyped::ResourceKind;
use fyrox_sound::algebra::Vector3;
use fyrox_sound::buffer::{DataSource, SoundBufferResource, SoundBufferResourceExtension};
use fyrox_sound::context::{self, SoundContext};
use fyrox_sound::effects::{reverb::Reverb, Effect};
use fyrox_sound::engine::SoundEngine;
use fyrox_sound::futures::executor::block_on;
use fyrox_sound::hrtf::HrirSphere;
use fyrox_sound::pool::Handle;
use fyrox_sound::renderer::hrtf::{HrirSphereResource, HrirSphereResourceExt, HrtfRenderer};
use fyrox_sound::renderer::Renderer;
use fyrox_sound::source::{SoundSource, SoundSourceBuilder, Status};
use std::collections::HashSet;
use std::path::Path;

/// system managing the audio
pub struct AudioSystem {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    current_speed_pitch: f64,
    _sound_engine: SoundEngine,
    sound_context: SoundContext,
    pitch_on_speed_change: bool,
    active_effect_handles: HashSet<Handle<SoundSource>>,
    active_music_handles: HashSet<Handle<SoundSource>>,
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
            current_speed_pitch: 1.0,
            _sound_engine: sound_engine,
            sound_context,
            pitch_on_speed_change: true,
            active_effect_handles: HashSet::new(),
            active_music_handles: HashSet::new(),
        }
    }

    /// update entity sound positions etc (runs every frame)
    pub(crate) fn update(&mut self, entity_manager: &EntityManager) {
        let mut state = self.sound_context.state();
        for (sound, pos) in entity_manager.query2::<SoundController, Position>((None, None)) {
            for handle in sound.handles.iter().copied() {
                let source = state.source_mut(handle);
                source.set_position(vec3_to_vector3(pos.data()));
            }
        }
    }

    /// access to the sound source corresponding to a handle
    pub fn alter_source<F>(&self, handle: Handle<SoundSource>, mut f: F)
    where
        F: FnMut(&mut SoundSource),
    {
        let mut state = self.sound_context.state();
        f(state.source_mut(handle));
    }

    /// loads a sound from file and caches it (default state of the source is stopped and not looping)
    pub fn load_sound(
        &mut self,
        file_path: impl AsRef<Path>,
        sound_type: SoundType,
        spatial: bool,
    ) -> Handle<SoundSource> {
        let volume = self.absolute_volume(sound_type);
        let buffer = SoundBufferResource::new_generic(
            block_on(DataSource::from_file(file_path, &FsResourceIo)).unwrap(),
        )
        .unwrap();

        let mut sb = SoundSourceBuilder::new()
            .with_buffer(buffer)
            .with_looping(false)
            .with_status(Status::Stopped)
            .with_gain(volume);

        if !spatial {
            sb = sb.with_spatial_blend_factor(0.0);
        }
        let source = sb.build().unwrap();
        let handle = self.sound_context.state().add_source(source);
        match sound_type {
            SoundType::SFX => {
                self.active_effect_handles.insert(handle);
            }
            SoundType::Music => {
                self.active_music_handles.insert(handle);
            }
        }
        handle
    }

    /// removes a sound source from the system
    pub fn remove_sound(&mut self, handle: Handle<SoundSource>) {
        self.active_effect_handles.remove(&handle);
        self.active_music_handles.remove(&handle);
        let mut state = self.sound_context.state();
        state.remove_source(handle);
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

    /// add a reverb effect (enables hrtf)
    pub fn add_reverb(&self, decay_time: f32) {
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
            .remove_effect(0);
    }

    /// changes the volume of the given type to the specified value
    pub fn set_volume(&mut self, volume_type: VolumeType, new_volume: f32) {
        log::trace!("set volume {:?} to {:?}", volume_type, new_volume);
        match volume_type {
            VolumeType::Master => {
                self.master_volume = new_volume;
                self.update_sfx_volumes();
                self.update_music_volumes();
            }
            VolumeType::SFX => {
                self.sfx_volume = new_volume;
                self.update_sfx_volumes()
            }
            VolumeType::Music => {
                self.music_volume = new_volume;
                self.update_music_volumes();
            }
        }
    }

    /// changes the pitch of an entity's sound
    pub fn set_pitch(&self, handle: Handle<SoundSource>, pitch: f64) {
        log::trace!("set pitch of handle {:?} to {:?}", handle, pitch);
        todo!()
    }

    /// enables/disbles the pitch change on animation speed change (default is true)
    pub fn set_pitch_on_speed_change(&mut self, flag: bool) {
        log::debug!("set pitch on speed change: {:?}", flag);
        self.pitch_on_speed_change = flag;
        let mut state = self.sound_context.state();
        for source in state.sources_mut().iter_mut() {
            source.set_pitch(source.pitch() / self.current_speed_pitch);
        }
        self.current_speed_pitch = 1.0;
    }

    /// updates the SFX volumes of all known handles
    fn update_sfx_volumes(&self) {
        let mut state = self.sound_context.state();
        let sfx_volume = self.absolute_volume(SoundType::SFX);
        for sfx_handle in self.active_effect_handles.iter().copied() {
            let sfx_source = state.source_mut(sfx_handle);
            sfx_source.set_gain(sfx_volume);
        }
    }

    /// updates the music volumes of all known handles
    fn update_music_volumes(&self) {
        let mut state = self.sound_context.state();
        let music_volume = self.absolute_volume(SoundType::Music);
        for music_handle in self.active_music_handles.iter().copied() {
            let music_source = state.source_mut(music_handle);
            music_source.set_gain(music_volume);
        }
    }

    /// calculate the total resulting volume for either the sfx or music
    fn absolute_volume(&self, sound_type: SoundType) -> f32 {
        match sound_type {
            SoundType::SFX => self.master_volume * self.sfx_volume,
            SoundType::Music => self.master_volume * self.music_volume,
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
            let new_speed_pitch = event.new_animation_speed as f64;
            let mut state = self.sound_context.state();
            for source in state.sources_mut().iter_mut() {
                source.set_pitch(source.pitch() / self.current_speed_pitch * new_speed_pitch);
            }
            self.current_speed_pitch = new_speed_pitch;
        }
    }
}

impl EventObserver<EngineModeChange> for AudioSystem {
    fn on_event(&mut self, event: &EngineModeChange) {
        match event.new_mode {
            EngineMode::Running => {
                let mut state = self.sound_context.state();
                for source in state.sources_mut().iter_mut() {
                    source.play();
                }
            }
            EngineMode::Paused | EngineMode::Editor => {
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

/// type of sound
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoundType {
    SFX,
    Music,
}
