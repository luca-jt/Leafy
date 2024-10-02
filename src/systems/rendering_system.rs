use crate::ecs::component::MeshAttribute::*;
use crate::ecs::component::{Color32, MeshAttribute, MeshType, Orientation, Position, Scale};
use crate::ecs::entity::EntityID;
use crate::ecs::entity_manager::EntityManager;
use crate::engine::EngineMode;
use crate::glm;
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::data::{OrthoCamera, PerspectiveCamera, ShadowMap, TextureMap};
use crate::rendering::instance_renderer::InstanceRenderer;
use crate::rendering::mesh::Mesh;
use crate::rendering::shader::ShaderCatalog;
use crate::rendering::sprite_renderer::SpriteRenderer;
use crate::rendering::voxel_renderer::VoxelRenderer;
use crate::systems::event_system::events::{CamPositionChange, EngineModeChange};
use crate::systems::event_system::EventObserver;
use crate::utils::constants::{ORIGIN, Z_AXIS};
use RendererType::*;

/// responsible for the automated rendering of all entities
pub struct RenderingSystem {
    current_mode: EngineMode,
    shadow_map: ShadowMap,
    renderers: Vec<RendererType>,
    perspective_camera: PerspectiveCamera,
    ortho_camera: OrthoCamera,
    shader_catalog: ShaderCatalog,
    used_renderer_indeces: Vec<usize>,
    cam_position_link: Option<EntityID>,
    render_shadows: bool,
    clear_color: Color32,
    render_distance: Option<f32>,
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
            current_mode: EngineMode::Running,
            shadow_map: ShadowMap::new(2048, 2048, glm::Vec3::new(1.0, 8.0, 1.0)),
            renderers: Vec::new(),
            perspective_camera: PerspectiveCamera::new(glm::Vec3::new(0.0, 1.0, -2.0), ORIGIN),
            ortho_camera: OrthoCamera::from_size(1.0),
            shader_catalog: ShaderCatalog::new(),
            used_renderer_indeces: Vec::new(),
            cam_position_link: None,
            render_shadows: true,
            clear_color: Color32::WHITE,
            render_distance: None,
        }
    }

    /// start the 3D rendering for all renderers
    pub(crate) fn render(&mut self, entity_manager: &EntityManager) {
        if let Some(entity) = self.cam_position_link {
            let pos = entity_manager
                .get_component::<Position>(entity)
                .expect("entity has no position");
            self.perspective_camera
                .update_cam(pos.data(), &(pos.data() + Z_AXIS));
        }
        self.used_renderer_indeces.clear();
        clear_gl_screen(self.clear_color);
        self.init_renderers();
        // add entity data
        for (position, mesh_type, mesh_attr, scale, orientation) in entity_manager
            .query5_opt3::<Position, MeshType, MeshAttribute, Scale, Orientation>(vec![])
        {
            let mesh = entity_manager.asset_from_type(*mesh_type).unwrap();

            let is_added = self.try_add_data(
                position,
                mesh_type,
                mesh_attr.unwrap_or(&Colored(Color32::WHITE)),
                scale.unwrap_or(&Scale::default()),
                orientation.unwrap_or(&Orientation::default()),
                mesh,
                &entity_manager.texture_map,
            );
            // add new renderer if needed
            if !is_added {
                self.add_new_renderer(
                    position,
                    mesh_type,
                    mesh_attr.unwrap_or(&Colored(Color32::WHITE)),
                    scale.unwrap_or(&Scale::default()),
                    orientation.unwrap_or(&Orientation::default()),
                    mesh,
                    &entity_manager.texture_map,
                );
            }
        }
        self.render_shadows();
        self.render_geometry();
        self.cleanup_renderers();
    }

    /// initialize all the stored renderers
    fn init_renderers(&mut self) {
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                Batch(_, renderer) => renderer.begin_batch(),
                Instance(..) => {}
                Voxel(renderer) => renderer.init(),
                _ => {}
            }
        }
    }

    /// render all the geometry data stored in the renderers
    fn render_geometry(&mut self) {
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                Batch(_, renderer) => {
                    renderer.flush(
                        &self.perspective_camera,
                        &self.shadow_map,
                        self.shader_catalog.batch_basic(),
                    );
                }
                Instance(_, _, renderer) => {
                    renderer.draw_all(
                        &self.perspective_camera,
                        &self.shadow_map,
                        self.shader_catalog.instance_basic(),
                    );
                }
                Sprite(renderer) => renderer.end(),
                Voxel(renderer) => renderer.end(),
            }
        }
    }

    /// renders the shadows to the shadow map
    fn render_shadows(&mut self) {
        self.shadow_map.bind_writing();
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                Batch(_, renderer) => {
                    renderer.end_batch();
                    renderer.render_shadows(&self.shadow_map);
                }
                Instance(_, _, renderer) => {
                    renderer.confirm_positions();
                    renderer.render_shadows(&self.shadow_map);
                }
                _ => {}
            }
        }
        self.shadow_map.unbind_writing();
        self.shadow_map.depth_buffer_cleared = false;
    }

    /// try to add the render data to an existing renderer
    fn try_add_data(
        &mut self,
        position: &Position,
        mesh_type: &MeshType,
        mesh_attr: &MeshAttribute,
        scale: &Scale,
        orientation: &Orientation,
        mesh: &Mesh,
        texture_map: &TextureMap,
    ) -> bool {
        for (i, r_type) in self.renderers.iter_mut().enumerate() {
            if let Batch(m_type, renderer) = r_type {
                if m_type == mesh_type {
                    self.used_renderer_indeces.push(i);
                    return match mesh_attr {
                        Textured(path) => {
                            renderer.draw_tex_mesh(
                                position,
                                scale,
                                orientation,
                                texture_map.get_tex_id(path).unwrap(),
                                &self.perspective_camera,
                                &mut self.shadow_map,
                                self.shader_catalog.batch_basic(),
                                mesh,
                            );
                            true
                        }
                        Colored(color) => {
                            renderer.draw_color_mesh(
                                position,
                                scale,
                                orientation,
                                *color,
                                &self.perspective_camera,
                                &mut self.shadow_map,
                                self.shader_catalog.batch_basic(),
                                mesh,
                            );
                            true
                        }
                    };
                }
            } else if let Instance(m_type, m_attr, renderer) = r_type {
                if m_type == mesh_type && m_attr == mesh_attr {
                    match mesh_attr {
                        Textured(path) => {
                            if texture_map.get_tex_id(path).unwrap() == renderer.tex_id {
                                renderer.add_position(position, scale, orientation, mesh);
                                self.used_renderer_indeces.push(i);
                                return true;
                            }
                        }
                        Colored(color) => {
                            if *color == renderer.color {
                                renderer.add_position(position, scale, orientation, mesh);
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
    fn add_new_renderer(
        &mut self,
        position: &Position,
        mesh_type: &MeshType,
        mesh_attr: &MeshAttribute,
        scale: &Scale,
        orientation: &Orientation,
        mesh: &Mesh,
        texture_map: &TextureMap,
    ) {
        match mesh_type {
            MeshType::Triangle | MeshType::Plane | MeshType::Cube => {
                let mut renderer = BatchRenderer::new(mesh, self.shader_catalog.batch_basic());
                match mesh_attr {
                    Colored(color) => {
                        renderer.draw_color_mesh(
                            position,
                            scale,
                            orientation,
                            *color,
                            &self.perspective_camera,
                            &mut self.shadow_map,
                            self.shader_catalog.batch_basic(),
                            mesh,
                        );
                    }
                    Textured(path) => {
                        renderer.draw_tex_mesh(
                            position,
                            scale,
                            orientation,
                            texture_map.get_tex_id(path).unwrap(),
                            &self.perspective_camera,
                            &mut self.shadow_map,
                            self.shader_catalog.batch_basic(),
                            mesh,
                        );
                    }
                }
                self.renderers.push(Batch(*mesh_type, renderer));
            }
            _ => {
                let mut renderer =
                    InstanceRenderer::new(mesh, self.shader_catalog.instance_basic());
                match mesh_attr {
                    Textured(path) => {
                        renderer.tex_id = texture_map.get_tex_id(path).unwrap();
                    }
                    Colored(color) => {
                        renderer.color = *color;
                    }
                }
                renderer.add_position(position, scale, orientation, mesh);
                self.renderers
                    .push(Instance(*mesh_type, mesh_attr.clone(), renderer));
            }
        }
        self.used_renderer_indeces
            .push(self.used_renderer_indeces.len());
    }

    /// drop renderers that are not used anymore
    fn cleanup_renderers(&mut self) {
        for i in 0..self.renderers.len() {
            if !self.used_renderer_indeces.contains(&i) {
                self.renderers.remove(i);
            }
        }
    }

    /// change OpenGL's background clear color
    pub fn set_gl_clearcolor(&mut self, color: Color32) {
        self.clear_color = color;
    }

    /// engables/dislables the shadow rendering of all entities for 3D rendering
    pub fn enable_shadows(&mut self, flag: bool) {
        self.render_shadows = flag;
    }

    /// set the FOV for 3D rendering (in degrees)
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
}

impl EventObserver<EngineModeChange> for RenderingSystem {
    fn on_event(&mut self, event: &EngineModeChange) {
        self.current_mode = event.new_mode;
    }
}

impl EventObserver<CamPositionChange> for RenderingSystem {
    fn on_event(&mut self, event: &CamPositionChange) {
        self.perspective_camera
            .update_cam(&event.new_pos, &event.new_focus);
    }
}

/// stores the renderer type with rendered entity type + renderer
pub(crate) enum RendererType {
    Batch(MeshType, BatchRenderer),
    Instance(MeshType, MeshAttribute, InstanceRenderer),
    Sprite(SpriteRenderer),
    Voxel(VoxelRenderer),
}

impl RendererType {
    /// yields the mesh type of a renderer type if present
    pub(crate) fn mesh_type(&self) -> Option<MeshType> {
        match self {
            Batch(mesh_type, _) => Some(*mesh_type),
            Instance(mesh_type, _, _) => Some(*mesh_type),
            _ => None,
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
