use fl_core::components;
use fl_core::ecs::component::{
    Color32, MeshAttribute, MeshType, MotionState, Position, Renderable, TouchTime, Velocity,
};
use fl_core::ecs::entity::EntityID;
use fl_core::ecs::entity_manager::EntityManager;
use fl_core::engine::{Engine, FallingLeafApp};
use fl_core::glm;
use fl_core::systems::audio_system::{AudioSystem, VolumeKind};
use fl_core::systems::event_system::events::*;
use fl_core::systems::event_system::{EventObserver, EventSystem};
use fl_core::utils::tools::{shared_ptr, SharedPtr};
use fl_core::winit::keyboard::KeyCode;
use std::any::Any;
use std::cell::RefMut;
use std::f32::consts::PI;

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

impl FallingLeafApp for App {
    fn init(&mut self, engine: &Engine) {
        engine.event_system().trigger(CamPositionChange {
            new_pos: glm::Vec3::new(0.0, 1.0, -2.0),
            new_focus: glm::Vec3::zeros(),
        });

        let _floor = self.entity_manager().create_entity(components!(
            Position::zeros(),
            Renderable {
                scale: 5f32.into(),
                mesh_type: MeshType::Plane,
                mesh_attribute: MeshAttribute::Textured("wall.png"),
            }
        ));

        engine
            .event_system()
            .add_listener::<KeyPress>(&self.game_state);

        engine.audio_system().play_background_music("drop.wav");

        engine.event_system().trigger(AudioVolumeChanged {
            kind: VolumeKind::Master,
            new_volume: 0.1,
        });

        let sound = engine.audio_system().new_sound_controller();
        engine.audio_system().enable_hrtf();

        let position = Position::new(0.0, 1.0, 1.0);
        engine
            .audio_system()
            .play_sfx_at("helicopter.wav", true, &sound, &position);

        let cube = self.entity_manager().create_entity(components!(
            position,
            Renderable {
                scale: 0.5f32.into(),
                mesh_type: MeshType::Cube,
                mesh_attribute: MeshAttribute::Colored(Color32::BLUE),
            },
            sound,
            TouchTime::now()
        ));

        self.game_state.borrow_mut().cube = Some(cube);
    }

    fn on_frame_update(
        &mut self,
        _event_system: &mut EventSystem,
        _audio_system: &SharedPtr<AudioSystem>,
    ) {
        let mut game_state = self.game_state.borrow_mut();
        let cube = game_state.cube.unwrap();
        let tt = game_state
            .entity_manager
            .get_component_mut::<TouchTime>(cube)
            .unwrap();
        let secs = tt.delta_time_f32();
        let pos = game_state
            .entity_manager
            .get_component_mut::<Position>(cube)
            .unwrap();
        let av = PI / 2.0;
        pos.data_mut().x = (av * secs).sin();
        pos.data_mut().z = (av * secs).cos();
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
    cube: Option<EntityID>,
}

impl GameState {
    /// initialize some example data
    pub fn init() -> SharedPtr<Self> {
        let mut entity_manager = EntityManager::new();

        let player = entity_manager.create_regular_moving(
            Position::new(0.0, 2.0, 0.0),
            MeshType::Sphere,
            MeshAttribute::Colored(Color32::RED),
        );

        shared_ptr(Self {
            entity_manager,
            player,
            cube: None,
        })
    }
}

impl EventObserver<KeyPress> for GameState {
    fn on_event(&mut self, event: &KeyPress) {
        if event.key == KeyCode::Space {
            let v_ref = &mut self
                .entity_manager
                .get_component_mut::<MotionState>(self.player)
                .unwrap()
                .velocity;
            *v_ref = Velocity::new(0.0, 3.0, 0.0);
        }
    }
}

/*fn jump(event: &KeyPress, app: &mut Box<dyn Any>, entity_manager: &mut EntityManager) {
    let app = app.downcast_mut::<App>().unwrap();
    if event.key == KeyCode::Space {
        let v_ref = &mut entity_manager
            .get_component_mut::<MotionState>(app.player)
            .unwrap()
            .velocity;
        *v_ref = Velocity::new(0.0, 3.0, 0.0);
    }
}
*/
