use crate::ecs::component::MeshAttribute::*;
use crate::ecs::component::*;
use crate::ecs::entity::EntityID;
use crate::ecs::entity_manager::EntityManager;
use crate::glm;
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::data::*;
use crate::rendering::instance_renderer::InstanceRenderer;
use crate::rendering::mesh::Mesh;
use crate::rendering::shader::ShaderCatalog;
use crate::rendering::sprite_renderer::SpriteRenderer;
use crate::systems::event_system::events::CamPositionChange;
use crate::systems::event_system::EventObserver;
use crate::utils::constants::{MAX_LIGHT_SRC_COUNT, ORIGIN, Z_AXIS};
use crate::utils::tools::padding;
use gl::types::*;
use RendererType::*;

/// responsible for the automated rendering of all entities
pub struct RenderingSystem {
    light_sources: Vec<(EntityID, ShadowMap)>,
    renderers: Vec<RendererType>,
    perspective_camera: PerspectiveCamera,
    ortho_camera: OrthoCamera,
    shader_catalog: ShaderCatalog,
    used_renderer_indeces: Vec<usize>,
    cam_position_link: Option<EntityID>,
    clear_color: Color32,
    render_distance: Option<f32>,
    shadow_resolution: ShadowResolution,
    current_cam_config: (glm::Vec3, glm::Vec3),
    ambient_light: LightSource,
}

impl RenderingSystem {
    /// creates a new rendering system with initial cam data
    pub(crate) fn new() -> Self {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::SCISSOR_TEST);
        }

        Self {
            light_sources: Vec::new(),
            renderers: Vec::new(),
            perspective_camera: PerspectiveCamera::new(-Z_AXIS, ORIGIN),
            ortho_camera: OrthoCamera::from_size(1.0),
            shader_catalog: ShaderCatalog::new(),
            used_renderer_indeces: Vec::new(),
            cam_position_link: None,
            clear_color: Color32::WHITE,
            render_distance: None,
            shadow_resolution: ShadowResolution::High,
            current_cam_config: (-Z_AXIS, Z_AXIS),
            ambient_light: LightSource {
                color: Color32::WHITE,
                intensity: 0.3,
            },
        }
    }

    /// adds and removes light sources according to entity data
    pub(crate) fn update_light_sources(&mut self, entity_manager: &EntityManager) {
        let lights = entity_manager
            .query3_opt1::<Position, LightSource, LightSrcID>(vec![])
            .map(|(p, s, l)| (p, s, l.unwrap().0))
            .collect::<Vec<_>>();
        // remove deleted shadow maps
        self.light_sources
            .retain(|src| lights.iter().any(|(_, _, id)| *id == src.0));
        // update positions of existing ones
        self.light_sources.iter_mut().for_each(|(entity, map)| {
            let correct_light = lights.iter().find(|(_, _, id)| id == entity).unwrap();
            map.update_light(
                correct_light.0.data(),
                &correct_light.1.color,
                correct_light.1.intensity,
            );
        });
        // add new light sources
        let new_lights = lights
            .into_iter()
            .filter(|&(_, _, entity)| !self.light_sources.iter().any(|(id, _)| entity == *id))
            .collect::<Vec<_>>();

        for (pos, src, entity) in new_lights {
            if self.light_sources.len() == MAX_LIGHT_SRC_COUNT {
                panic!(
                    "no more light source slots available (max is {})",
                    MAX_LIGHT_SRC_COUNT
                );
            }
            self.light_sources.push((
                entity,
                ShadowMap::new(
                    self.shadow_resolution.map_res(),
                    *pos.data(),
                    &src.color,
                    src.intensity,
                ),
            ));
        }
    }

    /// start the 3D rendering for all renderers
    pub(crate) fn render(&mut self, entity_manager: &EntityManager) {
        self.update_entity_cam(entity_manager);
        self.used_renderer_indeces.clear();
        clear_gl_screen(self.clear_color);
        self.init_renderers();
        // add entity data
        let (render_dist, cam_pos) = (self.render_distance, self.current_cam_config.0);
        for (position, mesh_type, mesh_attr, scale, orientation) in entity_manager
            .query5_opt3::<Position, MeshType, MeshAttribute, Scale, Orientation>(vec![])
            .filter(|(pos, ..)| match render_dist {
                None => true,
                Some(dist) => (pos.data() - cam_pos).norm() <= dist,
            })
        {
            let default_attr = Colored(Color32::WHITE);
            let trafo = calc_model_matrix(
                position,
                scale.unwrap_or(&Scale::default()),
                orientation.unwrap_or(&Orientation::default()),
            );

            let render_data = RenderData {
                trafo: &trafo,
                m_type: mesh_type,
                m_attr: mesh_attr.unwrap_or(&default_attr),
                mesh: entity_manager.asset_from_type(mesh_type).unwrap(),
                tex_map: &entity_manager.texture_map,
            };

            let is_added = self.try_add_data(&render_data);
            // add new renderer if needed
            if !is_added {
                self.add_new_renderer(&render_data);
            }
        }
        self.confirm_data();
        self.render_shadows();
        self.render_geometry();
        self.cleanup_renderers();
    }

    /// initialize all the stored renderers
    fn init_renderers(&mut self) {
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                Batch(_, renderer) => renderer.begin_first_batch(),
                Instance(..) => {}
                _ => {}
            }
        }
    }

    /// confirms all of the added data in the renderers
    fn confirm_data(&mut self) {
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                Batch(_, renderer) => {
                    renderer.end_batch();
                }
                Instance(_, _, renderer) => {
                    renderer.confirm_positions();
                }
                _ => {}
            }
        }
    }

    /// render all the geometry data stored in the renderers
    fn render_geometry(&mut self) {
        self.update_uniform_buffers();
        let shadow_maps = self
            .light_sources
            .iter()
            .map(|(_, map)| map)
            .collect::<Vec<_>>();

        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                Batch(_, renderer) => {
                    renderer.flush(&shadow_maps, self.shader_catalog.batch_basic());
                }
                Instance(_, _, renderer) => {
                    renderer.draw_all(&shadow_maps, self.shader_catalog.instance_basic());
                }
                Sprite(renderer) => renderer.end(),
            }
        }
    }

    /// renders the shadows to the shadow map
    fn render_shadows(&mut self) {
        for (_, shadow_map) in self.light_sources.iter_mut() {
            shadow_map.bind_writing();
            for renderer_type in self.renderers.iter_mut() {
                match renderer_type {
                    Batch(_, renderer) => {
                        renderer.render_shadows(shadow_map);
                    }
                    Instance(_, _, renderer) => {
                        renderer.render_shadows(shadow_map);
                    }
                    _ => {}
                }
            }
            shadow_map.unbind_writing();
        }
    }

    /// try to add the render data to an existing renderer
    fn try_add_data(&mut self, rd: &RenderData) -> bool {
        for (i, r_type) in self.renderers.iter_mut().enumerate() {
            if let Batch(m_type, renderer) = r_type {
                if m_type == rd.m_type {
                    self.used_renderer_indeces.push(i);
                    return match rd.m_attr {
                        Textured(path) => {
                            renderer.draw_tex_mesh(
                                rd.trafo,
                                rd.tex_map.get_tex_id(path).unwrap(),
                                rd.mesh,
                                self.shader_catalog.batch_basic(),
                            );
                            true
                        }
                        Colored(color) => {
                            renderer.draw_color_mesh(rd.trafo, *color, rd.mesh);
                            true
                        }
                    };
                }
            } else if let Instance(m_type, m_attr, renderer) = r_type {
                if m_type == rd.m_type && m_attr == rd.m_attr {
                    match rd.m_attr {
                        Textured(path) => {
                            if rd.tex_map.get_tex_id(path).unwrap() == renderer.tex_id {
                                renderer.add_position(rd.trafo, rd.mesh);
                                self.used_renderer_indeces.push(i);
                                return true;
                            }
                        }
                        Colored(color) => {
                            if *color == renderer.color {
                                renderer.add_position(rd.trafo, rd.mesh);
                                self.used_renderer_indeces.push(i);
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// add a new renderer to the system and add the render data to it
    fn add_new_renderer(&mut self, rd: &RenderData) {
        match rd.m_type {
            MeshType::Triangle | MeshType::Plane | MeshType::Cube => {
                let mut renderer = BatchRenderer::new(rd.mesh, self.shader_catalog.batch_basic());
                match rd.m_attr {
                    Colored(color) => {
                        renderer.draw_color_mesh(rd.trafo, *color, rd.mesh);
                    }
                    Textured(path) => {
                        renderer.draw_tex_mesh(
                            rd.trafo,
                            rd.tex_map.get_tex_id(path).unwrap(),
                            rd.mesh,
                            self.shader_catalog.batch_basic(),
                        );
                    }
                }
                self.renderers.push(Batch(rd.m_type.clone(), renderer));
            }
            _ => {
                let mut renderer =
                    InstanceRenderer::new(rd.mesh, self.shader_catalog.instance_basic());
                match rd.m_attr {
                    Textured(path) => {
                        renderer.tex_id = rd.tex_map.get_tex_id(path).unwrap();
                    }
                    Colored(color) => {
                        renderer.color = *color;
                    }
                }
                renderer.add_position(rd.trafo, rd.mesh);
                self.renderers
                    .push(Instance(rd.m_type.clone(), rd.m_attr.clone(), renderer));
            }
        }
        self.used_renderer_indeces
            .push(self.used_renderer_indeces.len());
    }

    /// updates the camera if it is attached to an entity
    fn update_entity_cam(&mut self, entity_manager: &EntityManager) {
        if let Some(entity) = self.cam_position_link {
            let pos = entity_manager
                .get_component::<Position>(entity)
                .expect("entity has no position");
            self.perspective_camera
                .update_cam(pos.data(), &(pos.data() + Z_AXIS));
        }
    }

    /// updates all the uniform buffers
    fn update_uniform_buffers(&self) {
        self.shader_catalog.matrix_buffer.upload_data(
            0,
            size_of::<glm::Mat4>(),
            self.perspective_camera.projection() as *const glm::Mat4 as *const GLvoid,
        );
        self.shader_catalog.matrix_buffer.upload_data(
            size_of::<glm::Mat4>(),
            size_of::<glm::Mat4>(),
            self.perspective_camera.view() as *const glm::Mat4 as *const GLvoid,
        );

        let light_data = self
            .light_sources
            .iter()
            .map(|(_, map)| LightData {
                light_src: glm::Vec4::new(map.light_pos.x, map.light_pos.y, map.light_pos.z, 1.0),
                light_matrix: map.light_matrix,
                color: map.light_color.to_vec4(),
                intensity: map.light_intensity,
                padding_12bytes: Default::default(),
            })
            .collect::<Vec<_>>();

        let ambient_config = LightConfig {
            color: self.ambient_light.color.to_vec4(),
            intensity: self.ambient_light.intensity,
        };
        self.shader_catalog.light_buffer.upload_data(
            0,
            size_of::<LightConfig>(),
            &ambient_config as *const LightConfig as *const GLvoid,
        );
        if !light_data.is_empty() {
            self.shader_catalog.light_buffer.upload_data(
                size_of::<LightConfig>() + padding::<LightConfig>(),
                MAX_LIGHT_SRC_COUNT * size_of::<LightData>(),
                light_data.as_ptr() as *const GLvoid,
            );
        }
    }

    /// drop renderers that are not used anymore
    fn cleanup_renderers(&mut self) {
        for i in 0..self.renderers.len() {
            if !self.used_renderer_indeces.contains(&i) {
                self.renderers.remove(i);
            }
        }
    }

    /// gets the current camera position and look direction vector
    pub fn current_cam_config(&self) -> (glm::Vec3, glm::Vec3) {
        self.current_cam_config
    }

    /// change OpenGL's background clear color (default is white)
    pub fn set_gl_clearcolor(&mut self, color: Color32) {
        self.clear_color = color;
    }

    /// set the FOV for 3D rendering in degrees (default is 45°)
    pub fn set_fov(&mut self, fov: f32) {
        self.perspective_camera.update_fov(fov);
    }

    /// link/unlink the 3D camera position to some enities' position
    pub fn link_cam_to_entity(&mut self, link: Option<EntityID>) {
        self.cam_position_link = link;
    }

    /// changes the render distance to `distance` units from the current camera position
    pub fn set_render_distance(&mut self, distance: Option<f32>) {
        self.render_distance = distance;
    }

    /// changes the shadow map resolution (default is normal)
    pub fn set_shadow_resolution(&mut self, resolution: ShadowResolution) {
        self.shadow_resolution = resolution;
        self.light_sources.iter_mut().for_each(|(_, map)| {
            *map = ShadowMap::new(
                self.shadow_resolution.map_res(),
                map.light_pos,
                &map.light_color,
                map.light_intensity,
            )
        });
    }

    /// changes the ambient light (default is white and 0.3)
    pub fn set_ambient_light(&mut self, light: LightSource) {
        self.ambient_light = light;
    }
}

impl EventObserver<CamPositionChange> for RenderingSystem {
    fn on_event(&mut self, event: &CamPositionChange) {
        let new_focus = event.new_pos + event.new_look;
        self.perspective_camera
            .update_cam(&event.new_pos, &new_focus);
        self.current_cam_config = (event.new_pos, event.new_look);
    }
}

/// stores the renderer type with rendered entity type + renderer
enum RendererType {
    Batch(MeshType, BatchRenderer),
    Instance(MeshType, MeshAttribute, InstanceRenderer),
    Sprite(SpriteRenderer),
}

/// data bundle for rendering
struct RenderData<'a> {
    trafo: &'a glm::Mat4,
    m_type: &'a MeshType,
    m_attr: &'a MeshAttribute,
    mesh: &'a Mesh,
    tex_map: &'a TextureMap,
}

/// all possible settings for shadow map resolution
pub enum ShadowResolution {
    Ultra,
    High,
    Normal,
    Low,
}

impl ShadowResolution {
    /// yields the actual corresponding map resolution to the setting
    fn map_res(&self) -> (GLsizei, GLsizei) {
        match self {
            ShadowResolution::Ultra => (4096, 4096),
            ShadowResolution::High => (2048, 2048),
            ShadowResolution::Normal => (1024, 1024),
            ShadowResolution::Low => (512, 512),
        }
    }
}

/// clears the opengl viewport
fn clear_gl_screen(color: Color32) {
    let float_color = color.to_vec4();
    unsafe {
        gl::ClearColor(float_color.x, float_color.y, float_color.z, float_color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
