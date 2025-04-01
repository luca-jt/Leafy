use crate::ecs::entity_manager::EntityManager;
use crate::internal_prelude::*;
use fyrox_resource::io::FsResourceIo;
use fyrox_resource::untyped::ResourceKind;
use fyrox_sound::buffer::{DataSource, SoundBufferResource, SoundBufferResourceExtension};
use fyrox_sound::context::{SoundContext, SAMPLE_RATE};
use fyrox_sound::effects::{reverb::Reverb, Effect};
use fyrox_sound::engine::SoundEngine;
use fyrox_sound::futures::executor::block_on;
use fyrox_sound::hrtf::HrirSphere;
use fyrox_sound::pool::Handle;
use fyrox_sound::renderer::hrtf::{HrirSphereResource, HrirSphereResourceExt, HrtfRenderer};
use fyrox_sound::renderer::Renderer;
use fyrox_sound::source::{SoundSource, SoundSourceBuilder, Status};

/// The system managing the audio state of the engine. You can control general settings and create new handles to use audio with entities.
pub struct AudioSystem {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    current_speed_pitch: f64,
    _sound_engine: SoundEngine,
    sound_context: SoundContext,
    pitch_on_speed_change: bool,
    active_effect_handles: AHashSet<Handle<SoundSource>>,
    active_music_handles: AHashSet<Handle<SoundSource>>,
    using_reverb: bool,
    using_hrtf: bool,
    removed_handles: AHashSet<Handle<SoundSource>>,
}

impl AudioSystem {
    /// creates a new audio system
    pub(crate) fn new() -> Self {
        let sound_engine = SoundEngine::new().expect("Error creating sound engine.");
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
            active_effect_handles: AHashSet::new(),
            active_music_handles: AHashSet::new(),
            using_reverb: false,
            using_hrtf: false,
            removed_handles: AHashSet::new(),
        }
    }

    /// update entity sound positions etc (runs every frame)
    pub(crate) fn update(&mut self, entity_manager: &mut EntityManager) {
        let mut state = self.sound_context.state();
        for (sound, pos) in
            unsafe { entity_manager.query2::<&mut SoundController, &Position>((None, None)) }
        {
            // remove invalid handles from components
            sound
                .handles
                .retain(|handle| !self.removed_handles.contains(handle));
            self.removed_handles.clear();
            // update position
            for handle in sound.handles.iter().copied() {
                let source = state.source_mut(handle);
                source.set_position(vec3_to_vector3(pos.data()));
            }
        }
    }

    /// updates the doppler effect pitch for all handles of a sound controller
    pub(crate) fn set_doppler_pitch(
        &self,
        controller: &SoundController,
        old_pitch: f64,
        new_pitch: f64,
    ) {
        let mut state = self.sound_context.state();
        for handle in controller.handles.iter().copied() {
            let source = state.source_mut(handle);
            let base_pitch = source.pitch() / old_pitch;
            source.set_pitch(base_pitch * new_pitch);
        }
    }

    /// Loads a sound from file and caches it (default state of the source is stopped and not looping). Sounds loaded this way should only be attached to at most one entity!
    pub fn load_sound(
        &mut self,
        file_path: impl AsRef<Path>,
        sound_type: SoundType,
        spatial: bool,
    ) -> Handle<SoundSource> {
        let file_path = file_path.as_ref();

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
        log::debug!("Loaded {sound_type:?} sound: {file_path:?}.");
        handle
    }

    /// Removes a sound source from the system.
    pub fn remove_sound(&mut self, handle: Handle<SoundSource>) {
        self.active_effect_handles.remove(&handle);
        self.active_music_handles.remove(&handle);
        let mut state = self.sound_context.state();
        state.remove_source(handle);
        self.removed_handles.insert(handle);
    }

    /// Plays a sound.
    pub fn play(&self, handle: Handle<SoundSource>) {
        let mut state = self.sound_context.state();
        state.source_mut(handle).play();
    }

    /// Pauses a sound.
    pub fn pause(&self, handle: Handle<SoundSource>) {
        let mut state = self.sound_context.state();
        state.source_mut(handle).pause();
    }

    /// Stops a sound.
    pub fn stop(&self, handle: Handle<SoundSource>) {
        let mut state = self.sound_context.state();
        state.source_mut(handle).stop().unwrap();
    }

    /// Sets wether or not a sound should loop.
    pub fn set_looping(&self, handle: Handle<SoundSource>, looping: bool) {
        let mut state = self.sound_context.state();
        state.source_mut(handle).set_looping(looping);
    }

    /// Sets the playback time for a sound.
    pub fn set_playback_time(&self, handle: Handle<SoundSource>, time: Duration) {
        let mut state = self.sound_context.state();
        state.source_mut(handle).set_playback_time(time);
    }

    /// Sets the panning for a sound (must be in range ``-1..=1`` where -1 = only left, 0 = both, 1 = only right).
    pub fn set_panning(&self, handle: Handle<SoundSource>, panning: f32) {
        let mut state = self.sound_context.state();
        state.source_mut(handle).set_panning(panning);
    }

    /// Sets the radius around the sound in which no distance attenuation is applied.
    pub fn set_radius(&self, handle: Handle<SoundSource>, radius: f32) {
        let mut state = self.sound_context.state();
        state.source_mut(handle).set_radius(radius);
    }

    /// Sets the roll-off factor for a sound in distance attenuation.
    pub fn set_rolloff_factor(&self, handle: Handle<SoundSource>, rolloff: f32) {
        let mut state = self.sound_context.state();
        state.source_mut(handle).set_rolloff_factor(rolloff);
    }

    /// Sets maximum distance for a sound until which distance gain will be applicable.
    pub fn set_max_distance(&self, handle: Handle<SoundSource>, distance: f32) {
        let mut state = self.sound_context.state();
        state.source_mut(handle).set_max_distance(distance);
    }

    /// Sets the relative gain for a sound (does not influence the volume settings).
    pub fn set_gain(&self, handle: Handle<SoundSource>, gain: f32) {
        let mut state = self.sound_context.state();
        if self.active_effect_handles.contains(&handle) {
            state
                .source_mut(handle)
                .set_gain(gain * self.absolute_volume(SoundType::SFX));
        } else if self.active_music_handles.contains(&handle) {
            state
                .source_mut(handle)
                .set_gain(gain * self.absolute_volume(SoundType::Music));
        } else {
            log::warn!("Handle not in storage.");
            return;
        }
    }

    /// Use sound rendering on the HRTF sphere.
    pub fn enable_hrtf(&mut self) {
        log::trace!("Enabled HRTF.");
        let hrir_sphere = HrirSphere::new(HRTF_SPHERE, SAMPLE_RATE).unwrap();
        self.sound_context
            .state()
            .set_renderer(Renderer::HrtfRenderer(HrtfRenderer::new(
                HrirSphereResource::from_hrir_sphere(hrir_sphere, ResourceKind::Embedded),
            )));
        self.using_hrtf = true;
    }

    /// Disable sound rendering on the HRTF sphere.
    pub fn disable_hrtf(&mut self) {
        log::trace!("Disabled HRTF.");
        self.sound_context.state().set_renderer(Renderer::Default);
        self.using_hrtf = false;
    }

    /// Add a reverb effect (requires HRTF).
    pub fn add_reverb(&mut self, decay_time: f32) {
        if !self.using_hrtf {
            log::error!("No HRTF enabled. Required for reverb.");
            return;
        }
        log::trace!("Enabled reverb with decay time {decay_time:?}.");
        if self.using_reverb {
            self.disable_reverb();
        }
        let mut reverb = Reverb::new();
        reverb.set_decay_time(decay_time);
        self.sound_context
            .state()
            .bus_graph_mut()
            .primary_bus_mut()
            .add_effect(Effect::Reverb(reverb));

        self.using_reverb = true;
    }

    /// Disable the reverb effect.
    pub fn disable_reverb(&mut self) {
        log::trace!("Disabled reverb.");
        if !self.using_reverb {
            return;
        }
        self.sound_context
            .state()
            .bus_graph_mut()
            .primary_bus_mut()
            .remove_effect(0);
        self.using_reverb = false;
    }

    /// Changes the volume of the given type to the specified value.
    pub fn set_volume(&mut self, volume_type: VolumeType, new_volume: f32) {
        log::trace!("Set volume {volume_type:?} to {new_volume:?}.");
        let mut state = self.sound_context.state();
        let old_sfx_volume = self.absolute_volume(SoundType::SFX);
        let old_music_volume = self.absolute_volume(SoundType::Music);
        match volume_type {
            VolumeType::Master => {
                self.master_volume = new_volume;
                for sfx_handle in self.active_effect_handles.iter().copied() {
                    let sfx_source = state.source_mut(sfx_handle);
                    sfx_source.set_gain(
                        sfx_source.gain() / old_sfx_volume * self.absolute_volume(SoundType::SFX),
                    );
                }
                for music_handle in self.active_music_handles.iter().copied() {
                    let music_source = state.source_mut(music_handle);
                    music_source.set_gain(
                        music_source.gain() / old_music_volume
                            * self.absolute_volume(SoundType::Music),
                    );
                }
            }
            VolumeType::SFX => {
                self.sfx_volume = new_volume;
                for sfx_handle in self.active_effect_handles.iter().copied() {
                    let sfx_source = state.source_mut(sfx_handle);
                    let new_gain =
                        sfx_source.gain() / old_sfx_volume * self.absolute_volume(SoundType::SFX);
                    sfx_source.set_gain(new_gain);
                }
            }
            VolumeType::Music => {
                self.music_volume = new_volume;
                for music_handle in self.active_music_handles.iter().copied() {
                    let music_source = state.source_mut(music_handle);
                    let new_gain = music_source.gain() / old_music_volume
                        * self.absolute_volume(SoundType::Music);
                    music_source.set_gain(new_gain);
                }
            }
        }
    }

    /// Changes the pitch of an entity's sound independantly of other pitch changes.
    pub fn set_pitch(&self, handle: Handle<SoundSource>, pitch: f64) {
        log::trace!("Set pitch of handle {handle:?} to {pitch:?}.");
        let mut state = self.sound_context.state();
        let source = state.source_mut(handle);
        if self.pitch_on_speed_change {
            source.set_pitch(pitch * self.current_speed_pitch);
        } else {
            source.set_pitch(pitch);
        }
    }

    /// Enables/disbles the pitch change on animation speed change (default is ``true``).
    pub fn set_pitch_on_speed_change(&mut self, flag: bool) {
        log::debug!("Set pitch on speed change: {flag:?}.");
        if self.pitch_on_speed_change == flag {
            return;
        }
        self.pitch_on_speed_change = flag;
        if flag {
            let mut state = self.sound_context.state();
            for source in state.sources_mut().iter_mut() {
                source.set_pitch(source.pitch() * self.current_speed_pitch);
            }
        } else {
            let mut state = self.sound_context.state();
            for source in state.sources_mut().iter_mut() {
                source.set_pitch(source.pitch() / self.current_speed_pitch);
            }
        }
    }

    /// Calculate the total resulting volume for either the SFX or music.
    fn absolute_volume(&self, sound_type: SoundType) -> f32 {
        match sound_type {
            SoundType::SFX => self.master_volume * self.sfx_volume,
            SoundType::Music => self.master_volume * self.music_volume,
        }
    }

    /// Changes the audio playback state based on the engine mode.
    pub(crate) fn on_mode_change(&mut self, event: &EngineModeChange) {
        match event.new_mode {
            EngineMode::Running => {
                let mut state = self.sound_context.state();
                for source in state.sources_mut().iter_mut() {
                    source.play();
                }
            }
            EngineMode::Editor => {
                let mut state = self.sound_context.state();
                for source in state.sources_mut().iter_mut() {
                    source.pause();
                }
            }
        }
    }

    /// general event handling function for the camera position change
    pub(crate) fn on_cam_position_change(&mut self, event: &CamPositionChange) {
        let mut state = self.sound_context.state();
        let listener = state.listener_mut();
        listener.set_orientation_lh(
            vec3_to_vector3(&event.new_look),
            vec3_to_vector3(&event.new_up),
        );
        listener.set_position(vec3_to_vector3(&event.new_pos));
    }

    /// general event handling function for the camera position change
    pub(crate) fn on_animation_speed_change(&mut self, event: &AnimationSpeedChange) {
        let new_speed_pitch = event.new_animation_speed as f64;
        if self.pitch_on_speed_change {
            let mut state = self.sound_context.state();
            for source in state.sources_mut().iter_mut() {
                source.set_pitch(source.pitch() / self.current_speed_pitch * new_speed_pitch);
            }
        }
        self.current_speed_pitch = new_speed_pitch;
    }
}

/// All versions of audio volume that can be controlled in the audio system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VolumeType {
    Master,
    SFX,
    Music,
}

/// Specifies what type a sound should be. This influences e.g. volume settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoundType {
    SFX,
    Music,
}
