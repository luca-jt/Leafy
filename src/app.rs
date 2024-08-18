use fl_core::components;
use fl_core::ecs::component::{
    Acceleration, Color32, MeshAttribute, MeshType, MotionState, Position, Renderable, TouchTime,
    Velocity,
};
use fl_core::ecs::entity::EntityID;
use fl_core::ecs::entity_manager::EntityManager;
use fl_core::engine::Engine;
use fl_core::systems::event_system::{EventObserver, FLKeyPress};
use fl_core::utils::tools::{shared_ptr, SharedPtr};
use sdl2::keyboard::Keycode;

/// example app
pub struct App {
    engine: Engine,
    game_state: SharedPtr<GameState>,
}

impl App {
    /// creates a new app and initializes it
    pub fn init() -> Self {
        let mut engine = Engine::new();
        let game_state = GameState::init();

        engine.event_system.add_listener::<FLKeyPress>(&game_state);
        engine.audio_system.play_music("bg_music.mp3");

        Self { engine, game_state }
    }

    /// runs the main engine loop
    pub fn run(&mut self) {
        self.engine
            .run(&mut self.game_state.borrow_mut().entity_manager);
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
