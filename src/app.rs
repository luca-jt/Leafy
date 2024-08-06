use std::time::{Duration, Instant};

use fl_core::state::game_state::GameState;
use fl_core::state::video_state::VideoState;
use fl_core::systems::animation_system::AnimationSystem;
use fl_core::systems::audio_system::AudioSystem;
use fl_core::systems::event_system::*;
use fl_core::systems::rendering_system::RenderingSystem;
use fl_core::utils::constants::FPS_CAP;
use fl_core::utils::tools::{shared_ptr, SharedPtr};

/// main app
pub struct App {
    game_state: SharedPtr<GameState>,
    video_state: SharedPtr<VideoState>,
    rendering_system: RenderingSystem,
    audio_system: AudioSystem,
    event_system: EventSystem,
    animation_system: AnimationSystem,
}

impl App {
    /// app setup on startup
    pub fn new() -> Self {
        let game_state = shared_ptr(GameState::new());
        let video_state = shared_ptr(VideoState::new());
        let rendering_system = RenderingSystem::new();
        let audio_system = AudioSystem::new(&video_state.borrow().sdl_context);
        let mut event_system = EventSystem::new(&video_state.borrow().sdl_context);
        let animation_system = AnimationSystem::new();

        // maybe do some of them in some constuctor
        event_system.add_listener::<FLKeyPress>(video_state.clone());
        event_system.add_listener::<FLWindowResize>(video_state.clone());
        event_system.add_listener::<FLKeyPress>(game_state.clone());

        Self {
            game_state,
            video_state,
            rendering_system,
            audio_system,
            event_system,
            animation_system,
        }
    }

    /// runs the main loop
    pub fn run(&mut self) {
        self.audio_system.play_music("bg_music.mp3");
        let mut fps: f64 = 0.0;
        let mut frame_start_time = Instant::now();

        'running: loop {
            self.audio_system.update();
            self.animation_system
                .apply_physics(&mut self.game_state.borrow_mut().entity_manager);
            self.rendering_system.render(&self.game_state.borrow());
            if self.event_system.parse_sdl_events().is_err() {
                break 'running;
            }
            self.video_state.borrow().swap_window();
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
