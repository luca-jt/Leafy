use crate::ecs::entity_manager::EntityManager;
use std::cell::RefMut;
use std::ops::{Deref, DerefMut};
use std::time::{Duration, Instant};

use crate::systems::animation_system::AnimationSystem;
use crate::systems::audio_system::AudioSystem;
use crate::systems::event_system::*;
use crate::systems::rendering_system::RenderingSystem;
use crate::systems::video_system::VideoSystem;
use crate::utils::constants::FPS_CAP;
use crate::utils::tools::{shared_ptr, SharedPtr};

/// main engine
pub struct Engine {
    pub video_system: SharedPtr<VideoSystem>,
    rendering_system: RenderingSystem,
    pub audio_system: AudioSystem,
    pub event_system: EventSystem,
    pub animation_system: AnimationSystem,
}

impl Engine {
    /// engine setup on startup
    pub fn new() -> Self {
        let video_system = shared_ptr(VideoSystem::new());
        let rendering_system = RenderingSystem::new();
        let audio_system = AudioSystem::new(&video_system.borrow().sdl_context);
        let mut event_system = EventSystem::new(&video_system.borrow().sdl_context);
        let animation_system = AnimationSystem::new();

        event_system.add_listener::<FLKeyPress>(&video_system);
        event_system.add_listener::<FLWindowResize>(&video_system);

        Self {
            video_system,
            rendering_system,
            audio_system,
            event_system,
            animation_system,
        }
    }

    /// runs the main loop
    pub fn run(&mut self, app: &mut impl FLApp) {
        app.init(self);
        let mut fps = 0f64;
        let mut frame_start_time = Instant::now();

        'running: loop {
            self.audio_system.update();
            self.animation_system
                .apply_physics(app.entity_manager().deref_mut());
            self.rendering_system.render(app.entity_manager().deref());
            app.on_frame_update(self);
            if self.event_system.parse_sdl_events().is_err() {
                break 'running;
            }
            self.video_system.borrow().swap_window();
            Self::cap_fps(&mut frame_start_time, &mut fps);
        }
    }

    /// caps the fps of the main loop
    fn cap_fps(frame_start_time: &mut Instant, fps: &mut f64) {
        let elapsed_frame_time = frame_start_time.elapsed();
        let max_frame_time = Duration::from_secs_f64(1.0 / FPS_CAP);
        if elapsed_frame_time < max_frame_time {
            std::thread::sleep(max_frame_time - elapsed_frame_time);
        }
        *fps = (1.0 / frame_start_time.elapsed().as_secs_f64()).round();
        *frame_start_time = Instant::now();
    }
}

/// all necessary app functionality to run the engine with
pub trait FLApp {
    /// initialize the app (e.g. add event handling)
    fn init(&mut self, engine: &mut Engine);
    /// run this update code every frame
    fn on_frame_update(&mut self, engine: &mut Engine);
    /// allows for access to the entity manager to be used for all engine operations
    fn entity_manager(&mut self) -> RefMut<EntityManager>;
}
