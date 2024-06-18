use crate::state::audio_state::AudioState;
use crate::state::game_state::GameState;
use crate::state::video_state::VideoState;
use crate::systems::animation_system::AnimationSystem;
use crate::systems::audio_system::AudioSystem;
use crate::systems::event_system::EventSystem;
use crate::systems::rendering_system::RenderingSystem;

/// main app
pub struct App {
    video_state: VideoState,
    audio_state: AudioState,
    game_state: GameState,
    audio_system: AudioSystem,
    event_system: EventSystem,
    rendering_system: RenderingSystem,
    animation_system: AnimationSystem,
}

impl App {
    /// app setup on startup
    pub fn new() -> Self {
        Self {
            video_state: VideoState::new(),
            audio_state: AudioState::new(),
            game_state: GameState::new(),
            audio_system: AudioSystem::new(),
            event_system: EventSystem::new(),
            rendering_system: RenderingSystem::new(),
            animation_system: AnimationSystem::new(),
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
        // ...
    }
}
