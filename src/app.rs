use crate::state::game_state::GameState;
use crate::systems::animation_system::AnimationSystem;
use crate::systems::audio_system::AudioSystem;
use crate::systems::event_system::EventSystem;
use crate::systems::rendering_system::RenderingSystem;
use crate::utils::constants::FPS_CAP;
use std::time::{Duration, Instant};

/// main app
pub struct App {
    game_state: GameState,
    rendering_system: RenderingSystem,
    animation_system: AnimationSystem,
    audio_system: AudioSystem,
    event_system: EventSystem,
}

impl App {
    /// app setup on startup
    pub fn new() -> Self {
        let game_state = GameState::new();
        let rendering_system = RenderingSystem::new();
        let animation_system = AnimationSystem::new();
        let sdl_context = &rendering_system.video_state.sdl_context;
        let audio_system = AudioSystem::new(sdl_context);
        let event_system = EventSystem::new(sdl_context);

        Self {
            game_state,
            rendering_system,
            animation_system,
            audio_system,
            event_system,
        }
    }

    /// runs the main loop
    pub fn run(&mut self) {
        let mut fps: f64 = 0.0;
        let mut frame_start_time = Instant::now();

        'running: loop {
            self.audio_system.update();
            self.game_state.update();
            self.animation_system.update();
            self.rendering_system.render();
            if self.event_system.parse_sdl_events().is_err() {
                break 'running;
            }
            self.rendering_system.swap_window();
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

impl Drop for App {
    fn drop(&mut self) {
        sdl2::mixer::Music::halt();
        sdl2::mixer::close_audio();
    }
}
