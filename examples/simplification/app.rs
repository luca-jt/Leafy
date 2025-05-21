use falling_leaf::prelude::*;
use winit::keyboard::KeyCode;

const CAM_MOVE_SPEED: f32 = 4.5;
const CAM_MOUSE_SPEED: f32 = 4.0;

pub struct App {
    mesh: EntityID,
}

impl App {
    pub fn new() -> Self {
        Self { mesh: NO_ENTITY }
    }
}

impl FallingLeafApp for App {
    fn init(&mut self, engine: &Engine<Self>) {
        let start_pos = vec3(0.0, 3.0, 5.0);
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
            .rendering_system_mut()
            .post_processing_params
            .background_as_scene_element = false;

        let mut entity_manager = engine.entity_manager_mut();

        let torus_mesh = entity_manager.load_asset_file("examples/simplification/torus.obj")[0];
        assert!(entity_manager.load_lods(torus_mesh));

        let _light = entity_manager.create_entity(components!(
            Position::new(1.0, 10.0, 1.0),
            DirectionalLight::default()
        ));

        let _floor = entity_manager.create_entity(components!(
            Position::origin(),
            Scale::from_factor(5.0),
            Renderable {
                mesh_type: MeshType::Plane,
                mesh_attribute: MeshAttribute::Colored(Color32::GREEN),
                material_source: MaterialSource::default(),
                shader_type: ShaderType::Basic,
                added_brightness: 0.0
            }
        ));

        self.mesh = entity_manager.create_entity(components!(
            Position::new(0.0, 2.0, 0.0),
            Renderable {
                mesh_type: MeshType::Custom(torus_mesh),
                mesh_attribute: MeshAttribute::Colored(Color32::YELLOW),
                material_source: MaterialSource::default(),
                shader_type: ShaderType::Basic,
                added_brightness: 0.0
            },
            LOD::None
        ));

        engine.event_system_mut().add_modifier(change_mesh_lod);
        engine.event_system_mut().add_modifier(quit_app);
    }

    fn on_frame_update(&mut self, _engine: &Engine<Self>) {}
}

fn change_mesh_lod(event: &KeyPress, engine: &Engine<App>) {
    let mesh_entity = engine.app().mesh;
    let mut entity_manager = engine.entity_manager_mut();
    let lod = entity_manager
        .get_component_mut::<LOD>(mesh_entity)
        .unwrap();

    if event.key == KeyCode::ArrowRight {
        *lod = i32_to_lod((*lod as i32 + 1) % NUM_LODS).unwrap()
    } else if event.key == KeyCode::ArrowLeft {
        *lod = i32_to_lod((*lod as i32 + NUM_LODS - 1) % NUM_LODS).unwrap()
    }
}

fn quit_app(event: &KeyPress, engine: &Engine<App>) {
    if event.key == KeyCode::Escape {
        engine.quit();
    }
}

fn i32_to_lod(value: i32) -> Result<LOD, ()> {
    match value {
        x if x == LOD::None as i32 => Ok(LOD::None),
        x if x == LOD::LVL1 as i32 => Ok(LOD::LVL1),
        x if x == LOD::LVL2 as i32 => Ok(LOD::LVL2),
        x if x == LOD::LVL3 as i32 => Ok(LOD::LVL3),
        x if x == LOD::LVL4 as i32 => Ok(LOD::LVL4),
        _ => Err(()),
    }
}
