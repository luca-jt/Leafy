use falling_leaf::prelude::*;
use falling_leaf::winit::keyboard::KeyCode;
use std::path::Path;

/// example app
pub struct App {
    sprite: EntityID,
    using_fullscreen: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            sprite: NO_ENTITY,
            using_fullscreen: false,
        }
    }
}

impl FallingLeafApp for App {
    fn init(&mut self, engine: &Engine<Self>) {
        engine
            .rendering_system_mut()
            .sprite_grid_mut(SpriteLayer::Layer0)
            .scale = 0.5;

        let mut entity_manager = engine.entity_manager_mut();

        let sprite_path = Path::new("examples/2D/sprite.png").into();
        assert!(entity_manager.load_sprite(&sprite_path));

        self.sprite = entity_manager.create_entity(components!(
            Sprite {
                source: SpriteSource::Single(sprite_path),
                position: SpritePosition::Grid(vec2(-1.0, 0.0)),
                layer: SpriteLayer::Layer0,
            },
            Scale::from_factor(1.0)
        ));

        engine.event_system_mut().add_modifier(quit_app);
        engine.event_system_mut().add_modifier(toggle_fullscreen);
    }

    fn on_frame_update(&mut self, _engine: &Engine<Self>) {}
}

fn quit_app(event: &KeyPress, engine: &Engine<App>) {
    if event.key == KeyCode::Escape {
        engine.quit();
    }
}

fn toggle_fullscreen(event: &KeyPress, engine: &Engine<App>) {
    if event.key == KeyCode::F11 {
        let current_fullscreen_state = engine.app().using_fullscreen;
        engine.app_mut().using_fullscreen = !current_fullscreen_state;
        engine
            .video_system()
            .set_fullscreen(!current_fullscreen_state);
    }
}
