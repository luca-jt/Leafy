use fl_core::components;
use fl_core::ecs::component::{
    Acceleration, Color32, MeshAttribute, MeshType, MotionState, Position, Renderable, TouchTime,
    Velocity,
};
use fl_core::ecs::entity::EntityID;
use fl_core::ecs::entity_manager::EntityManager;
use fl_core::engine::{Engine, FLApp};
use fl_core::systems::event_system::{EventObserver, FLKeyPress};
use fl_core::utils::tools::{shared_ptr, SharedPtr};
use sdl2::keyboard::Keycode;
use std::cell::RefMut;

/// example app
pub struct App {
    game_state: SharedPtr<GameState>,
}

impl App {
    pub fn new() -> Self {
        Self {
            game_state: GameState::init(),
        }
    }
}

impl FLApp for App {
    fn init(&mut self, engine: &mut Engine) {
        engine
            .event_system
            .add_listener::<FLKeyPress>(&self.game_state);
        engine.audio_system.play_music("bg_music.mp3");
    }

    fn on_frame_update(&mut self, _engine: &mut Engine) {
        //...
    }

    fn entity_manager(&mut self) -> RefMut<EntityManager> {
        RefMut::map(self.game_state.borrow_mut(), |game_state| {
            &mut game_state.entity_manager
        })
    }
}

/// example game state that holds all the entity data and other stuff
struct GameState {
    pub entity_manager: EntityManager,
    player: EntityID,
}

impl GameState {
    /// initialize some example data
    pub fn init() -> SharedPtr<Self> {
        let mut entity_manager = EntityManager::new();

        let _floor = entity_manager.create_entity(components!(
            Position::zeros(),
            Renderable {
                scale: 5f32.into(),
                mesh_type: MeshType::Plane,
                mesh_attribute: MeshAttribute::Colored(Color32::GREEN),
            }
        ));

        let player = entity_manager.create_entity(components!(
            Position::new(0.0, 2.0, 0.0),
            Renderable {
                scale: 1f32.into(),
                mesh_type: MeshType::Sphere,
                mesh_attribute: MeshAttribute::Colored(Color32::RED),
            },
            MotionState {
                velocity: Velocity::zeros(),
                acceleration: Acceleration::zeros()
            },
            TouchTime::now()
        ));

        shared_ptr(Self {
            entity_manager,
            player,
        })
    }
}

impl EventObserver<FLKeyPress> for GameState {
    fn on_event(&mut self, event: &FLKeyPress) {
        if event.key == Keycode::SPACE {
            let v_ref = &mut self
                .entity_manager
                .ecs
                .get_component_mut::<MotionState>(self.player)
                .unwrap()
                .velocity;
            *v_ref = Velocity::new(0.0, 3.0, 0.0);
        }
    }
}
