use crate::state::game_state::GameState;
use crate::systems::animation_system::AnimationSystem;
use crate::systems::audio_system::AudioSystem;
use crate::systems::event_system::EventSystem;
use crate::systems::rendering_system::RenderingSystem;

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
        'running: loop {
            // ...
            self.rendering_system.render();
            // ...
            break 'running;
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        todo!()
    }
}
