use crate::ecs::component::MeshAttribute::*;
use crate::ecs::component::{MeshAttribute, MeshType, Position, Renderable};
use crate::ecs::entity_manager::EntityManager;
use crate::engine::EngineMode;
use crate::glm;
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::data::{OrthoCamera, PerspectiveCamera, ShadowMap, TextureMap};
use crate::rendering::font_renderer::FontRenderer;
use crate::rendering::instance_renderer::InstanceRenderer;
use crate::rendering::mesh::Mesh;
use crate::rendering::shader::ShaderCatalog;
use crate::rendering::sprite_renderer::SpriteRenderer;
use crate::rendering::voxel_renderer::VoxelRenderer;
use crate::systems::event_system::events::{CamPositionChange, EngineModeChange};
use crate::systems::event_system::EventObserver;
use crate::utils::tools::SharedPtr;
use RendererType::*;

/// responsible for the automated rendering of all entities
pub struct RenderingSystem {
    current_mode: EngineMode,
    shadow_map: ShadowMap,
    renderers: Vec<RendererType>,
    perspective_camera: PerspectiveCamera,
    ortho_camera: OrthoCamera,
    shader_catalog: ShaderCatalog,
}

impl RenderingSystem {
    /// creates a new rendering system with initial cam data
    pub(crate) fn new(cam_pos: glm::Vec3, cam_focus: glm::Vec3) -> Self {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::SCISSOR_TEST);
        }

        Self {
            current_mode: EngineMode::Running,
            shadow_map: ShadowMap::new(2048, 2048, glm::Vec3::new(1.0, 10.0, 1.0)),
            renderers: Vec::new(),
            perspective_camera: PerspectiveCamera::new(cam_pos, cam_focus),
            ortho_camera: OrthoCamera::from_size(1.0),
            shader_catalog: ShaderCatalog::new(),
        }
    }

    /// start the rendering for all renderers
    pub(crate) fn render(&mut self, entity_manager: &EntityManager) {
        clear_gl_screen();
        self.init_renderers();
        // add entity data
        for (position, renderable) in entity_manager.query2::<Position, Renderable>() {
            let is_added = self.try_add_data(position, renderable, &entity_manager.texture_map);
            // add new renderer if needed
            if !is_added {
                let mesh = entity_manager
                    .asset_from_type(renderable.mesh_type)
                    .unwrap();

                self.add_new_renderer(position, renderable, mesh, &entity_manager.texture_map);
            }
        }
        self.render_shadows();
        self.render_geometry();
    }

    /// initialize all the stored renderers
    fn init_renderers(&mut self) {
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                Batch(_, renderer) => renderer.begin_batch(),
                Instance(..) => {}
                Font(renderer) => renderer.init(),
                Sprite(renderer) => renderer.init(),
                Voxel(renderer) => renderer.init(),
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
                Font(renderer) => renderer.end(),
                Sprite(renderer) => renderer.end(),
                Voxel(renderer) => renderer.end(),
            }
        }
    }

    /// renders the shadows to the shadow map
    fn render_shadows(&mut self) {
        self.shadow_map.bind_writing(&self.perspective_camera);
        self.shadow_map.try_clear_depth();
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                Batch(_, renderer) => {
                    renderer.end_batch();
                    renderer.render_shadows();
                }
                Instance(_, _, renderer) => {
                    renderer.confirm_positions();
                    renderer.render_shadows();
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
        renderable: &Renderable,
        texture_map: &TextureMap,
    ) -> bool {
        for r_type in self.renderers.iter_mut() {
            if let Batch(m_type, renderer) = r_type {
                if *m_type == renderable.mesh_type {
                    return match renderable.mesh_attribute {
                        Textured(path) => {
                            renderer.draw_tex_mesh(
                                position,
                                renderable.scale.0,
                                texture_map.get_tex_id(path).unwrap(),
                                &self.perspective_camera,
                                &mut self.shadow_map,
                                self.shader_catalog.batch_basic(),
                            );
                            true
                        }
                        Colored(color) => {
                            renderer.draw_color_mesh(
                                position,
                                renderable.scale.0,
                                color,
                                &self.perspective_camera,
                                &mut self.shadow_map,
                                self.shader_catalog.batch_basic(),
                            );
                            true
                        }
                    };
                }
            } else if let Instance(m_type, m_attr, renderer) = r_type {
                if *m_type == renderable.mesh_type && *m_attr == renderable.mesh_attribute {
                    match renderable.mesh_attribute {
                        Textured(path) => {
                            if texture_map.get_tex_id(path).unwrap() == renderer.tex_id {
                                renderer.add_position(position);
                                return true;
                            }
                        }
                        Colored(color) => {
                            if color == renderer.color {
                                renderer.add_position(position);
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
        renderable: &Renderable,
        mesh: SharedPtr<Mesh>,
        texture_map: &TextureMap,
    ) {
        match renderable.mesh_type {
            MeshType::Plane | MeshType::Cube => {
                let mut renderer = BatchRenderer::new(mesh, 10, self.shader_catalog.batch_basic());
                match renderable.mesh_attribute {
                    Colored(color) => {
                        renderer.draw_color_mesh(
                            position,
                            renderable.scale.0,
                            color,
                            &self.perspective_camera,
                            &mut self.shadow_map,
                            self.shader_catalog.batch_basic(),
                        );
                    }
                    Textured(path) => {
                        renderer.draw_tex_mesh(
                            position,
                            renderable.scale.0,
                            texture_map.get_tex_id(path).unwrap(),
                            &self.perspective_camera,
                            &mut self.shadow_map,
                            self.shader_catalog.batch_basic(),
                        );
                    }
                }
                self.renderers.push(Batch(renderable.mesh_type, renderer));
            }
            MeshType::Sphere => {
                let mut renderer =
                    InstanceRenderer::new(mesh, 10, self.shader_catalog.instance_basic());
                match renderable.mesh_attribute {
                    Textured(path) => {
                        renderer.tex_id = texture_map.get_tex_id(path).unwrap();
                    }
                    Colored(color) => {
                        renderer.color = color;
                    }
                }
                renderer.add_position(position);
                self.renderers.push(Instance(
                    renderable.mesh_type,
                    renderable.mesh_attribute,
                    renderer,
                ));
            }
        }
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
    Font(FontRenderer),
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
fn clear_gl_screen() {
    unsafe {
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
