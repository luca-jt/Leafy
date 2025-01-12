use falling_leaf::components;
use falling_leaf::ecs::component::utils::Filtering;
use falling_leaf::ecs::component::utils::*;
use falling_leaf::ecs::component::*;
use falling_leaf::ecs::entity::EntityID;
use falling_leaf::engine::{Engine, FallingLeafApp};
use falling_leaf::glm;
use falling_leaf::rendering::data::Skybox;
use falling_leaf::systems::audio_system::VolumeType;
use falling_leaf::systems::event_system::events::*;
use falling_leaf::utils::constants::bits::user_level::FLOATING;
use falling_leaf::utils::constants::{NO_ENTITY, ORIGIN, X_AXIS, Y_AXIS, Z_AXIS};
use falling_leaf::winit::keyboard::KeyCode;
use std::f32::consts::FRAC_PI_2;
use std::path::Path;

const CAM_MOVE_SPEED: f32 = 4.5;
const CAM_MOUSE_SPEED: f32 = 4.0;

/// example app
pub struct App {
    player: EntityID,
    sphere: EntityID,
    using_mouse_control: bool,
    using_fullscreen: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            player: NO_ENTITY,
            sphere: NO_ENTITY,
            using_mouse_control: true,
            using_fullscreen: false,
        }
    }
}

impl FallingLeafApp for App {
    fn init(&mut self, engine: &Engine<Self>) {
        let start_pos = glm::vec3(0.0, 5.0, -5.0);
        engine.trigger_event(CamPositionChange {
            new_pos: start_pos,
            new_look: ORIGIN - start_pos,
        });
        engine
            .video_system_mut()
            .set_mouse_fpp_cam_control(Some(CAM_MOUSE_SPEED));
        engine
            .animation_system_mut()
            .set_flying_cam_movement(Some(CAM_MOVE_SPEED));
        engine
            .audio_system_mut()
            .set_volume(VolumeType::Master, 0.5);

        engine
            .rendering_system_mut()
            .set_skybox(Some(Skybox::new(["", "", "", "", "", ""])));

        let mut entity_manager = engine.entity_manager_mut();
        entity_manager.create_point_light(Position::new(1.0, 6.0, 1.0));
        entity_manager.create_point_light_visible(Position::new(-1.0, 6.0, -1.0));

        let _floor = entity_manager.create_entity(components!(
            Position::origin(),
            Scale::new(5.0, 0.1, 5.0),
            MeshType::Cube,
            MeshAttribute::Textured(Texture {
                path: Path::new("./examples/3D/wall.png").into(),
                filtering: Filtering::Nearest,
                wrapping: Wrapping::Repeat,
            }),
            Collider::new(HitboxType::Box)
        ));

        let _hammer = entity_manager.create_entity(components!(
            Position::new(6.0, 2.0, 0.0),
            MeshType::Custom(Path::new("./examples/3D/hammer.obj").into()),
            MeshAttribute::Colored(Color32::GREY),
            Velocity::zero(),
            RigidBody::default().with_density(40.0),
            Orientation::default(),
            AngularMomentum::from_axis(X_AXIS * 20.0),
            EntityFlags::from_flags(&[FLOATING])
        ));

        self.player = entity_manager.create_entity(components!(
            Position::new(0.0, 4.0, 0.0),
            Scale::from_factor(0.2),
            MeshType::Cube,
            MeshAttribute::Colored(Color32::RED),
            Velocity::new(-1.0, 0.0, 0.0),
            Orientation::new(45.0, Y_AXIS + Z_AXIS),
            AngularMomentum::zero(),
            Acceleration::zero(),
            Collider::new(HitboxType::ConvexHull),
            RigidBody::default(),
            EntityFlags::default()
        ));

        let sound = engine.audio_system_mut().new_sound_controller();
        let heli_position = Position::new(0.0, 1.0, 1.0);

        engine.audio_system_mut().play_sfx_at(
            "examples/3D/helicopter.wav",
            true,
            &sound,
            &heli_position,
        );

        self.sphere = entity_manager.create_entity(components!(
            heli_position,
            Scale::from_factor(0.2),
            MeshType::Sphere,
            MeshAttribute::Colored(Color32::BLUE),
            sound,
            TouchTime::now()
        ));

        engine.event_system_mut().add_modifier(jump);
        engine.event_system_mut().add_modifier(quit_app);
        engine.event_system_mut().add_modifier(toggle_cursor);
        engine.event_system_mut().add_modifier(toggle_fullscreen);

        engine.audio_system().enable_hrtf();
    }

    fn on_frame_update(&mut self, engine: &Engine<Self>) {
        let mut entity_manager = engine.entity_manager_mut();
        let secs = entity_manager
            .get_component_mut::<TouchTime>(self.sphere)
            .unwrap()
            .delta_time();
        let pos = entity_manager
            .get_component_mut::<Position>(self.sphere)
            .unwrap();
        let av = FRAC_PI_2;
        pos.data_mut().x = (secs * av).0.sin() * 3.0;
        pos.data_mut().z = (secs * av).0.cos() * 3.0;
    }
}

fn jump(event: &KeyPress, engine: &Engine<App>) {
    if event.key == KeyCode::KeyE && !event.is_repeat {
        let mut entity_manager = engine.entity_manager_mut();
        let v_ref = entity_manager
            .get_component_mut::<Velocity>(engine.app().player)
            .unwrap();
        v_ref.data_mut().y = 5.0;
    }
}

fn quit_app(event: &KeyPress, engine: &Engine<App>) {
    if event.key == KeyCode::Escape {
        engine.quit();
    }
}

fn toggle_cursor(event: &KeyPress, engine: &Engine<App>) {
    if event.key == KeyCode::Tab {
        if engine.app().using_mouse_control {
            engine.video_system_mut().set_mouse_fpp_cam_control(None);
            engine.app_mut().using_mouse_control = false;
        } else {
            engine
                .video_system_mut()
                .set_mouse_fpp_cam_control(Some(CAM_MOUSE_SPEED));
            engine.app_mut().using_mouse_control = true;
        }
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
