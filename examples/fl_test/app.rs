use fl_core::components;
use fl_core::ecs::component::*;
use fl_core::ecs::entity::EntityID;
use fl_core::ecs::entity_manager::EntityManager;
use fl_core::engine::{app_downcast, Engine, FallingLeafApp};
use fl_core::glm;
use fl_core::systems::audio_system::VolumeKind;
use fl_core::systems::event_system::events::*;
use fl_core::winit::keyboard::KeyCode;
use std::cell::RefMut;
use std::f32::consts::PI;

/// example app
pub struct App {
    player: Option<EntityID>,
    cube: Option<EntityID>,
}

impl App {
    pub fn new() -> Self {
        Self {
            player: None,
            cube: None,
        }
    }
}

impl FallingLeafApp for App {
    fn init(&mut self, engine: &Engine) {
        engine.event_system().trigger(CamPositionChange {
            new_pos: glm::Vec3::new(0.0, 5.0, -5.0),
            new_focus: glm::Vec3::zeros(),
        });
        engine.event_system().trigger(AudioVolumeChanged {
            kind: VolumeKind::Master,
            new_volume: 0.5,
        });

        let mut entity_manager = engine.entity_manager();
        let _floor = entity_manager.create_entity(components!(
            Position::origin(),
            Scale::from_factor(5.0),
            MeshType::Plane,
            MeshAttribute::Textured("wall.png")
        ));
        let player = entity_manager.create_regular_moving(
            Position::new(0.0, 4.0, 0.0),
            MeshType::Sphere,
            MeshAttribute::Colored(Color32::RED),
        );
        *entity_manager.get_component_mut::<Scale>(player).unwrap() = Scale::from_factor(0.2);

        let sound = engine.audio_system().new_sound_controller();
        let heli_position = Position::new(0.0, 1.0, 1.0);

        engine
            .audio_system()
            .play_sfx_at("helicopter.wav", true, &sound, &heli_position);

        let cube = entity_manager.create_entity(components!(
            heli_position,
            Scale::from_factor(0.1),
            Orientation::new(45.0, 0.0, 1.0, 0.0),
            MeshType::Cube,
            MeshAttribute::Colored(Color32::BLUE),
            sound,
            TouchTime::now()
        ));

        engine.event_system().add_modifier(jump);
        engine.audio_system().enable_hrtf();
        engine.audio_system().play_background_music("drop.wav");

        self.cube = Some(cube);
        self.player = Some(player);
    }

    fn on_frame_update(&mut self, engine: &Engine) {
        let mut entity_manager = engine.entity_manager();
        let secs = entity_manager
            .get_component_mut::<TouchTime>(self.cube.unwrap())
            .unwrap()
            .delta_time();
        let pos = entity_manager
            .get_component_mut::<Position>(self.cube.unwrap())
            .unwrap();
        let av = PI / 2.0;
        pos.data_mut().x = (secs * av).0.sin() * 3.0;
        pos.data_mut().z = (secs * av).0.cos() * 3.0;
    }
}

fn jump(
    event: &KeyPress,
    app: RefMut<Box<dyn FallingLeafApp>>,
    entity_manager: &mut EntityManager,
) {
    let app = app_downcast::<App>(app);
    if event.key == KeyCode::Space {
        let v_ref = entity_manager
            .get_component_mut::<Velocity>(app.player.unwrap())
            .unwrap();
        *v_ref = Velocity::new(0.0, 3.0, 0.0);
    }
}
