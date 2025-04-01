use falling_leaf::prelude::bits::user_level::{DOPPLER_EFFECT, FLOATING};
use falling_leaf::prelude::*;
use falling_leaf::rendering::data::Skybox;
use falling_leaf::systems::audio_system::{SoundType, VolumeType};
use falling_leaf::winit::keyboard::KeyCode;
use std::f32::consts::FRAC_PI_2;
use std::path::Path;

const CAM_MOVE_SPEED: f32 = 5.0;
const CAM_MOUSE_SPEED: f32 = 3.0;

// wrapper for a component
struct TouchTime(TimePoint);
impl Component for TouchTime {}

/// example app
pub struct App {
    player: EntityID,
    sphere: EntityID,
    collision_point: EntityID,
    using_mouse_control: bool,
    using_fullscreen: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            player: NO_ENTITY,
            sphere: NO_ENTITY,
            collision_point: NO_ENTITY,
            using_mouse_control: true,
            using_fullscreen: false,
        }
    }
}

impl FallingLeafApp for App {
    fn init(&mut self, engine: &Engine<Self>) {
        //
        // initial settings
        //
        let start_pos = vec3(0.0, 5.0, -5.0);
        engine.trigger_event(CamPositionChange {
            new_pos: start_pos,
            new_look: ORIGIN - start_pos,
            new_up: Y_AXIS,
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

        engine.audio_system_mut().enable_hrtf();
        engine.rendering_system_mut().set_msaa(Some(4));
        engine
            .rendering_system_mut()
            .set_3d_render_resolution((1920, 1080));

        engine.rendering_system_mut().set_skybox(Skybox::try_new([
            "examples/3D/skybox/right.jpg",
            "examples/3D/skybox/left.jpg",
            "examples/3D/skybox/top.jpg",
            "examples/3D/skybox/bottom.jpg",
            "examples/3D/skybox/front.jpg",
            "examples/3D/skybox/back.jpg",
        ]));

        //
        // asset loading
        //
        let mut entity_manager = engine.entity_manager_mut();

        let hammer_mesh = entity_manager.load_asset_file("examples/3D/hammer.obj")[0];
        let sphere_mesh = entity_manager.load_asset_file("examples/3D/sphere.obj")[0];

        let wall_texture = Texture {
            path: Path::new("examples/3D/wall.png").into(),
            filtering: Filtering::Nearest,
            wrapping: Wrapping::Repeat,
            color_space: ColorSpace::RGBA8,
        };
        assert!(entity_manager.load_texture(&wall_texture));

        assert!(
            entity_manager.load_hitbox(HitboxType::ConvexHull, Some(MeshType::Cube.mesh_handle()))
        );
        assert!(entity_manager.load_hitbox(HitboxType::Box, Some(MeshType::Cube.mesh_handle())));

        let hit_sound =
            engine
                .audio_system_mut()
                .load_sound("examples/3D/hit.wav", SoundType::SFX, false);

        let heli_sound = engine.audio_system_mut().load_sound(
            "examples/3D/helicopter.wav",
            SoundType::SFX,
            true,
        );
        engine.audio_system_mut().set_looping(heli_sound, true);
        engine.audio_system_mut().play(heli_sound);

        //
        // entities
        //
        let _light1 = entity_manager.create_entity(components!(
            Position::new(-1.0, 6.0, -1.0),
            PointLight::default(),
            Renderable {
                mesh_type: MeshType::Cube,
                mesh_attribute: MeshAttribute::Colored(Color32::from_rgb(255, 255, 200)),
                material: MaterialSource::default(),
            },
            Scale::from_factor(0.1)
        ));

        let _light2 = entity_manager.create_entity(components!(
            Position::new(1.0, 6.0, 1.0),
            PointLight::default(),
            Renderable {
                mesh_type: MeshType::Cube,
                mesh_attribute: MeshAttribute::Colored(Color32::from_rgb(255, 255, 200)),
                material: MaterialSource::default(),
            },
            Scale::from_factor(0.1)
        ));

        let _floor = entity_manager.create_entity(components!(
            Position::origin(),
            Scale::new(5.0, 0.1, 5.0),
            Renderable {
                mesh_type: MeshType::Cube,
                mesh_attribute: MeshAttribute::Textured(wall_texture.clone()),
                material: MaterialSource::default(),
            },
            Collider::new(HitboxType::Box)
        ));

        let _ceiling = entity_manager.create_entity(components!(
            Position::new(0.0, 10.0, 0.0),
            Scale::new(5.0, 0.1, 5.0),
            Renderable {
                mesh_type: MeshType::Cube,
                mesh_attribute: MeshAttribute::Textured(wall_texture),
                material: MaterialSource::default(),
            },
            Collider::new(HitboxType::Box)
        ));

        let _hammer = entity_manager.create_entity(components!(
            Position::new(6.0, 2.0, 0.0),
            Renderable {
                mesh_type: MeshType::Custom(hammer_mesh),
                mesh_attribute: MeshAttribute::Colored(Color32::GREY),
                material: MaterialSource::default(),
            },
            Velocity::zero(),
            RigidBody::default().with_density(5.0),
            Orientation::default(),
            Scale::from_factor(0.7),
            AngularMomentum::from_axis(X_AXIS * 3.0 + Z_AXIS * 4.0),
            EntityFlags::from_flags(&[FLOATING])
        ));

        self.player = entity_manager.create_entity(components!(
            Position::new(0.0, 4.0, 0.0),
            Scale::from_factor(0.2),
            Renderable {
                mesh_type: MeshType::Cube,
                mesh_attribute: MeshAttribute::Colored(Color32::RED),
                material: MaterialSource::default(),
            },
            Velocity::new(-1.0, 0.0, 0.0),
            Orientation::new(45.0, Y_AXIS + Z_AXIS),
            AngularMomentum::zero(),
            Acceleration::zero(),
            Collider::new(HitboxType::ConvexHull),
            RigidBody::default(),
            EntityFlags::default(),
            SoundController::from_handles(&[hit_sound])
        ));

        self.sphere = entity_manager.create_entity(components!(
            Position::new(0.0, 1.0, 1.0),
            Scale::from_factor(0.2),
            Renderable {
                mesh_type: MeshType::Custom(sphere_mesh),
                mesh_attribute: MeshAttribute::Colored(Color32::BLUE),
                material: MaterialSource::default(),
            },
            SoundController::from_handles(&[heli_sound]),
            TouchTime(TimePoint::now()),
            EntityFlags::from_flags(&[DOPPLER_EFFECT])
        ));

        self.collision_point = entity_manager.create_entity(components!(
            Position::origin(),
            Scale::from_factor(0.05),
            Renderable {
                mesh_type: MeshType::Custom(sphere_mesh),
                mesh_attribute: MeshAttribute::Colored(Color32::YELLOW),
                material: MaterialSource::default(),
            }
        ));

        //
        // event functions
        //
        engine.event_system_mut().add_modifier(jump);
        engine.event_system_mut().add_modifier(quit_app);
        engine.event_system_mut().add_modifier(toggle_cursor);
        engine.event_system_mut().add_modifier(toggle_fullscreen);
    }

    fn on_frame_update(&mut self, engine: &Engine<Self>) {
        let mut entity_manager = engine.entity_manager_mut();
        let secs = entity_manager
            .get_component_mut::<TouchTime>(self.sphere)
            .unwrap()
            .0
            .delta_time();

        let pos = entity_manager
            .get_component_mut::<Position>(self.sphere)
            .unwrap();

        let av = FRAC_PI_2;
        pos.data_mut().x = (secs * av).0.sin() * 3.0;
        pos.data_mut().z = (secs * av).0.cos() * 3.0;

        let hit_handle = entity_manager
            .get_component::<SoundController>(self.player)
            .unwrap()
            .handles[0];

        if let Some(info) = entity_manager
            .get_component::<Collider>(self.player)
            .unwrap()
            .collision_info()
            .get(0)
        {
            if info.momentum.norm() > 0.1 {
                engine.audio_system().play(hit_handle);
            }
            *entity_manager
                .get_component_mut::<Position>(self.collision_point)
                .unwrap()
                .data_mut() = info.point;
        }
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
