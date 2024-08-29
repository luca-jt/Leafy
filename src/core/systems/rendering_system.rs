use crate::ecs::component::MeshAttribute::*;
use crate::ecs::component::{MeshAttribute, MeshType, Position, Renderable};
use crate::ecs::entity_manager::EntityManager;
use crate::engine::EngineMode;
use crate::glm;
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::data::{OrthoCamera, PerspectiveCamera, ShadowMap, TextureMap};
use crate::rendering::font_renderer::FontRenderer;
use crate::rendering::instance_renderer::InstanceRenderer;
use crate::rendering::sprite_renderer::SpriteRenderer;
use crate::rendering::voxel_renderer::VoxelRenderer;
use crate::systems::event_system::events::{CamPositionChange, EngineModeChange};
use crate::systems::event_system::EventObserver;
use RendererType::*;

/// responsible for the automated rendering of all entities
pub struct RenderingSystem {
    current_mode: EngineMode,
    texture_map: TextureMap,
    shadow_map: ShadowMap,
    renderers: Vec<RendererType>,
    perspective_camera: PerspectiveCamera,
    ortho_camera: OrthoCamera,
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
            texture_map: TextureMap::new(),
            shadow_map: ShadowMap::new(2048, 2048, glm::Vec3::new(1.0, 10.0, 1.0)),
            renderers: Vec::new(),
            perspective_camera: PerspectiveCamera::new(cam_pos, cam_focus),
            ortho_camera: OrthoCamera::from_size(1.0),
        }
    }

    /// start the rendering for all renderers
    pub(crate) fn render(&mut self, entity_manager: &EntityManager) {
        clear_gl_screen();
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                Batch(_, renderer) => renderer.begin_batch(),
                Instance(..) => {}
                Font(renderer) => renderer.init(),
                Sprite(renderer) => renderer.init(),
                Voxel(renderer) => renderer.init(),
            }
        }
        for (position, renderable) in entity_manager.query2::<Position, Renderable>() {
            let mesh = entity_manager.asset_from_type(renderable.mesh_type);

            let mut found = false;
            for r_type in self.renderers.iter_mut() {
                if let Batch(m_type, renderer) = r_type {
                    if *m_type == renderable.mesh_type {
                        match renderable.mesh_attribute {
                            Textured(id) => {
                                renderer.draw_tex_mesh(
                                    position.data_clone(),
                                    renderable.scale.0,
                                    id,
                                    &self.perspective_camera,
                                    &mut self.shadow_map,
                                );
                                found = true;
                                break;
                            }
                            Colored(color) => {
                                renderer.draw_color_mesh(
                                    position.data_clone(),
                                    renderable.scale.0,
                                    color,
                                    &self.perspective_camera,
                                    &mut self.shadow_map,
                                );
                                found = true;
                                break;
                            }
                        }
                    }
                } else if let Instance(m_type, m_attr, renderer) = r_type {
                    if *m_type == renderable.mesh_type && *m_attr == renderable.mesh_attribute {
                        match renderable.mesh_attribute {
                            Textured(id) => {
                                if id == renderer.tex_id {
                                    renderer.add_position(position.data_clone());
                                    found = true;
                                    break;
                                }
                            }
                            Colored(color) => {
                                if color == renderer.color {
                                    renderer.add_position(position.data_clone());
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            // add new renderer if needed
            if !found {
                match renderable.mesh_type {
                    MeshType::Plane | MeshType::Cube => {
                        self.renderers
                            .push(Batch(renderable.mesh_type, BatchRenderer::new(mesh, 10)));
                    }
                    MeshType::Sphere => {
                        let mut renderer = InstanceRenderer::new(mesh, 10);
                        match renderable.mesh_attribute {
                            Textured(tex_id) => {
                                renderer.tex_id = tex_id;
                            }
                            Colored(color) => {
                                renderer.color = color;
                            }
                        }
                        self.renderers.push(Instance(
                            renderable.mesh_type,
                            renderable.mesh_attribute,
                            renderer,
                        ));
                    }
                }
            }
        }

        // render shadows
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

        // render geometry
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                Batch(_, renderer) => {
                    renderer.flush(&self.perspective_camera, &self.shadow_map);
                }
                Instance(_, _, renderer) => {
                    renderer.draw_all(&self.perspective_camera, &self.shadow_map);
                }
                Font(renderer) => renderer.end(),
                Sprite(renderer) => renderer.end(),
                Voxel(renderer) => renderer.end(),
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
