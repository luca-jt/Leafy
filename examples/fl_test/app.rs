use fl_core::components;
use fl_core::ecs::component::*;
use fl_core::ecs::entity::EntityID;
use fl_core::engine::{Engine, FallingLeafApp};
use fl_core::glm;
use fl_core::systems::audio_system::VolumeType;
use fl_core::systems::event_system::events::*;
use fl_core::utils::constants::{ORIGIN, Y_AXIS};
use fl_core::winit::keyboard::KeyCode;
use std::f32::consts::PI;

pub const CAM_MOVE_SPEED: f32 = 4.5;

/// example app
pub struct App {
    player: Option<EntityID>,
    cube: Option<EntityID>,
    cam_move_direction: glm::Vec3,
    time_of_cam_update: TouchTime,
}

impl App {
    pub fn new() -> Self {
        Self {
            player: None,
            cube: None,
            cam_move_direction: glm::Vec3::zeros(),
            time_of_cam_update: TouchTime::now(),
        }
    }

    fn update_cam_veloctiy(&mut self, engine: &Engine<Self>) {
        let cam_config = engine.rendering_system().current_cam_config();
        let elapsed = self.time_of_cam_update.delta_time();
        self.time_of_cam_update.reset();

        let move_vector = if self.cam_move_direction != glm::Vec3::zeros() {
            self.cam_move_direction.normalize()
        } else {
            self.cam_move_direction
        };
        let changed = move_vector * elapsed.0 * CAM_MOVE_SPEED;

        let mut look_z = cam_config.1;
        look_z.y = 0.0;
        look_z.normalize_mut();
        let look_x = look_z.cross(&Y_AXIS).normalize();
        let look_space_matrix = glm::Mat3::from_columns(&[look_x, Y_AXIS, look_z]);

        engine.trigger_event(CamPositionChange {
            new_pos: cam_config.0 + look_space_matrix * changed,
            new_look: cam_config.1,
        });
    }
}

impl FallingLeafApp for App {
    fn init(&mut self, engine: &Engine<Self>) {
        let start_pos = glm::Vec3::new(0.0, 5.0, -5.0);
        engine.trigger_event(CamPositionChange {
            new_pos: start_pos,
            new_look: ORIGIN - start_pos,
        });
        engine.video_system_mut().set_mouse_cam_control(Some(0.001));
        engine
            .audio_system_mut()
            .set_volume(VolumeType::Master, 0.5);

        let mut entity_manager = engine.entity_manager_mut();
        entity_manager.add_light_src(Position::new(1.0, 6.0, 1.0));
        entity_manager.add_light_src(Position::new(-1.0, 6.0, -1.0));

        let _floor = entity_manager.create_entity(components!(
            Position::origin(),
            Scale::from_factor(5.0),
            MeshType::Plane,
            MeshAttribute::Textured(Texture::Wall),
            Hitbox
        ));
        let player = entity_manager.create_regular_moving(
            Position::new(0.0, 4.0, 0.0),
            MeshType::Sphere,
            MeshAttribute::Colored(Color32::RED),
        );
        *entity_manager.get_component_mut::<Scale>(player).unwrap() = Scale::from_factor(0.2);

        let sound = engine.audio_system_mut().new_sound_controller();
        let heli_position = Position::new(0.0, 1.0, 1.0);

        engine.audio_system_mut().play_sfx_at(
            "examples/fl_test/helicopter.wav",
            true,
            &sound,
            &heli_position,
        );

        let cube = entity_manager.create_entity(components!(
            heli_position,
            Scale::from_factor(0.1),
            Orientation::new(45.0, 0.0, 1.0, 0.0),
            MeshType::Cube,
            MeshAttribute::Colored(Color32::BLUE),
            sound,
            TouchTime::now()
        ));

        engine.event_system_mut().add_modifier(jump);
        engine.event_system_mut().add_modifier(move_cam);
        engine.event_system_mut().add_modifier(stop_cam);
        engine.event_system_mut().add_modifier(quit_app);

        engine.audio_system().enable_hrtf();
        engine
            .audio_system_mut()
            .play_background_music("examples/fl_test/drop.wav");

        self.cube = Some(cube);
        self.player = Some(player);
    }

    fn on_frame_update(&mut self, engine: &Engine<Self>) {
        self.update_cam_veloctiy(engine);
        let mut entity_manager = engine.entity_manager_mut();
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

fn jump(event: &KeyPress, engine: &Engine<App>) {
    if event.key == KeyCode::KeyE && !event.is_repeat {
        let mut entity_manager = engine.entity_manager_mut();
        let v_ref = entity_manager
            .get_component_mut::<Velocity>(engine.app().player.unwrap())
            .unwrap();
        *v_ref = Velocity::new(0.0, 3.0, 0.0);
    }
}

fn move_cam(event: &KeyPress, engine: &Engine<App>) {
    if event.is_repeat {
        return;
    }
    if event.key == KeyCode::ShiftLeft {
        engine.app_mut().cam_move_direction.y -= 1.0;
    }
    if event.key == KeyCode::Space {
        engine.app_mut().cam_move_direction.y += 1.0;
    }
    if event.key == KeyCode::KeyW {
        engine.app_mut().cam_move_direction.z += 1.0;
    }
    if event.key == KeyCode::KeyA {
        engine.app_mut().cam_move_direction.x -= 1.0;
    }
    if event.key == KeyCode::KeyS {
        engine.app_mut().cam_move_direction.z -= 1.0;
    }
    if event.key == KeyCode::KeyD {
        engine.app_mut().cam_move_direction.x += 1.0;
    }
}

fn stop_cam(event: &KeyRelease, engine: &Engine<App>) {
    if event.is_repeat {
        return;
    }
    if event.key == KeyCode::ShiftLeft {
        engine.app_mut().cam_move_direction.y += 1.0;
    }
    if event.key == KeyCode::Space {
        engine.app_mut().cam_move_direction.y -= 1.0;
    }
    if event.key == KeyCode::KeyW {
        engine.app_mut().cam_move_direction.z -= 1.0;
    }
    if event.key == KeyCode::KeyA {
        engine.app_mut().cam_move_direction.x += 1.0;
    }
    if event.key == KeyCode::KeyS {
        engine.app_mut().cam_move_direction.z += 1.0;
    }
    if event.key == KeyCode::KeyD {
        engine.app_mut().cam_move_direction.x -= 1.0;
    }
}

fn quit_app(event: &KeyPress, engine: &Engine<App>) {
    if event.key == KeyCode::Escape {
        engine.quit();
    }
}
