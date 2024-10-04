use crate::ecs::component::MeshAttribute::*;
use crate::ecs::component::{Color32, MeshAttribute, MeshType, Orientation, Position, Scale};
use crate::ecs::entity::EntityID;
use crate::ecs::entity_manager::EntityManager;
use crate::glm;
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::data::{OrthoCamera, PerspectiveCamera, ShadowMap, TextureMap};
use crate::rendering::instance_renderer::InstanceRenderer;
use crate::rendering::mesh::Mesh;
use crate::rendering::shader::ShaderCatalog;
use crate::rendering::sprite_renderer::SpriteRenderer;
use crate::rendering::voxel_renderer::VoxelRenderer;
use crate::systems::event_system::events::CamPositionChange;
use crate::systems::event_system::EventObserver;
use crate::utils::constants::{ORIGIN, Z_AXIS};
use RendererType::*;

/// responsible for the automated rendering of all entities
pub struct RenderingSystem {
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
    current_cam_pos: glm::Vec3,
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
        let start_cam_pos = glm::Vec3::new(0.0, 1.0, -2.0);

        Self {
            shadow_map: ShadowMap::new(2048, 2048, glm::Vec3::new(1.0, 8.0, 1.0)),
            renderers: Vec::new(),
            perspective_camera: PerspectiveCamera::new(start_cam_pos, ORIGIN),
            ortho_camera: OrthoCamera::from_size(1.0),
            shader_catalog: ShaderCatalog::new(),
            used_renderer_indeces: Vec::new(),
            cam_position_link: None,
            render_shadows: true,
            clear_color: Color32::WHITE,
            render_distance: None,
            current_cam_pos: start_cam_pos,
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
        let (render_dist, cam_pos) = (self.render_distance, self.current_cam_pos);
        for (position, mesh_type, mesh_attr, scale, orientation) in entity_manager
            .query5_opt3::<Position, MeshType, MeshAttribute, Scale, Orientation>(vec![])
            .filter(|(pos, ..)| match render_dist {
                None => true,
                Some(dist) => (pos.data() - cam_pos).norm() <= dist,
            })
        {
            let default_attr = Colored(Color32::WHITE);
            let default_scale = Scale::default();
            let default_orient = Orientation::default();

            let render_data = RenderData {
                pos: position,
                m_type: mesh_type,
                m_attr: mesh_attr.unwrap_or(&default_attr),
                scale: scale.unwrap_or(&default_scale),
                orient: orientation.unwrap_or(&default_orient),
                mesh: entity_manager.asset_from_type(mesh_type).unwrap(),
                tex_map: &entity_manager.texture_map,
            };

            let is_added = self.try_add_data(&render_data);
            // add new renderer if needed
            if !is_added {
                self.add_new_renderer(&render_data);
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
                Batch(_, renderer) => renderer.begin_first_batch(),
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
                    renderer.end_batches();
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
    fn try_add_data(&mut self, rd: &RenderData) -> bool {
        for (i, r_type) in self.renderers.iter_mut().enumerate() {
            if let Batch(m_type, renderer) = r_type {
                if m_type == rd.m_type {
                    self.used_renderer_indeces.push(i);
                    return match rd.m_attr {
                        Textured(path) => {
                            renderer.draw_tex_mesh(
                                rd.pos,
                                rd.scale,
                                rd.orient,
                                rd.tex_map.get_tex_id(path).unwrap(),
                                rd.mesh,
                                self.shader_catalog.batch_basic(),
                            );
                            true
                        }
                        Colored(color) => {
                            renderer.draw_color_mesh(rd.pos, rd.scale, rd.orient, *color, rd.mesh);
                            true
                        }
                    };
                }
            } else if let Instance(m_type, m_attr, renderer) = r_type {
                if m_type == rd.m_type && m_attr == rd.m_attr {
                    match rd.m_attr {
                        Textured(path) => {
                            if rd.tex_map.get_tex_id(path).unwrap() == renderer.tex_id {
                                renderer.add_position(rd.pos, rd.scale, rd.orient, rd.mesh);
                                self.used_renderer_indeces.push(i);
                                return true;
                            }
                        }
                        Colored(color) => {
                            if *color == renderer.color {
                                renderer.add_position(rd.pos, rd.scale, rd.orient, rd.mesh);
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
                        renderer.draw_color_mesh(rd.pos, rd.scale, rd.orient, *color, rd.mesh);
                    }
                    Textured(path) => {
                        renderer.draw_tex_mesh(
                            rd.pos,
                            rd.scale,
                            rd.orient,
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
                renderer.add_position(rd.pos, rd.scale, rd.orient, rd.mesh);
                self.renderers
                    .push(Instance(rd.m_type.clone(), rd.m_attr.clone(), renderer));
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

impl EventObserver<CamPositionChange> for RenderingSystem {
    fn on_event(&mut self, event: &CamPositionChange) {
        self.perspective_camera
            .update_cam(&event.new_pos, &event.new_focus);
        self.current_cam_pos = event.new_pos;
    }
}

/// stores the renderer type with rendered entity type + renderer
enum RendererType {
    Batch(MeshType, BatchRenderer),
    Instance(MeshType, MeshAttribute, InstanceRenderer),
    Sprite(SpriteRenderer),
    Voxel(VoxelRenderer),
}

/// data bundle for rendering
struct RenderData<'a> {
    pos: &'a Position,
    m_type: &'a MeshType,
    m_attr: &'a MeshAttribute,
    scale: &'a Scale,
    orient: &'a Orientation,
    mesh: &'a Mesh,
    tex_map: &'a TextureMap,
}

/// clears the opengl viewport
fn clear_gl_screen(color: Color32) {
    let float_color = color.to_vec4();
    unsafe {
        gl::ClearColor(float_color.x, float_color.y, float_color.z, float_color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
