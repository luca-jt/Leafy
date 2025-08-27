use leafy::prelude::*;
use leafy::winit::keyboard::KeyCode;
use std::path::Path;

const GRID_SCALE: f32 = 0.3;

/// example app
pub struct App {
    sprite1: EntityID,
    sprite2: EntityID,
    using_fullscreen: bool,
    start_time: TimePoint,
}

impl App {
    pub fn new() -> Self {
        Self {
            sprite1: NO_ENTITY,
            sprite2: NO_ENTITY,
            using_fullscreen: false,
            start_time: TimePoint::now(),
        }
    }
}

impl LeafyApp for App {
    fn init(&mut self, engine: &Engine<Self>) {
        engine
            .rendering_system_mut()
            .sprite_grid_mut(SpriteLayer::Layer0)
            .scale = GRID_SCALE;

        engine
            .rendering_system_mut()
            .sprite_grid_mut(SpriteLayer::Layer1)
            .scale = GRID_SCALE / 16.0; // one pixel of the sprite

        let mut entity_manager = engine.entity_manager_mut();

        let sprite_path = Path::new("examples/2D/sprite.png").into();
        assert!(entity_manager.load_sprite(&sprite_path));

        self.sprite1 = entity_manager.create_entity(components!(Sprite {
            source: SpriteSource::Single(sprite_path.clone()),
            position: SpritePosition::Grid(vec2(0.0, 1.0)),
            ..Default::default()
        }));
        self.sprite2 = entity_manager.create_entity(components!(
            Sprite {
                source: SpriteSource::Single(sprite_path),
                position: SpritePosition::Absolute(vec2(0.0, -GRID_SCALE)),
                layer: SpriteLayer::Layer0,
                projection_layer: Some(SpriteLayer::Layer1),
            },
            Scale::from_factor(GRID_SCALE)
        ));

        engine.event_system_mut().add_modifier(quit_app);
        engine.event_system_mut().add_modifier(toggle_fullscreen);
    }

    fn on_frame_update(&mut self, engine: &Engine<Self>) {
        let elapsed = self.start_time.delta_time();
        let x_position = (elapsed.0 * 2.0).sin() * 2.0;
        let mut manager = engine.entity_manager_mut();

        manager
            .get_component_mut::<Sprite>(self.sprite1)
            .unwrap()
            .position
            .vector_mut()
            .x = x_position;

        manager
            .get_component_mut::<Sprite>(self.sprite2)
            .unwrap()
            .position
            .vector_mut()
            .x = x_position * GRID_SCALE;
    }
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
