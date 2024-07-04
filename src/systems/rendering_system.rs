use crate::ecs::entity::{MeshAttribute, MeshType};
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::data::{OrthoCamera, PerspectiveCamera, ShadowMap, TextureMap};
use crate::rendering::font_renderer::FontRenderer;
use crate::rendering::instance_renderer::InstanceRenderer;
use crate::rendering::sprite_renderer::SpriteRenderer;
use crate::rendering::voxel_renderer::VoxelRenderer;
use crate::state::game_state::GameState;
use nalgebra_glm as glm;
use MeshAttribute::*;
use RendererType::*;

pub struct RenderingSystem {
    texture_map: TextureMap,
    shadow_map: ShadowMap,
    renderers: Vec<RendererType>,
    perspective_camera: PerspectiveCamera,
    ortho_camera: OrthoCamera,
}

impl RenderingSystem {
    /// creates a new rendering system
    pub fn new() -> Self {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::SCISSOR_TEST);
        }

        Self {
            texture_map: TextureMap::new(),
            shadow_map: ShadowMap::new(1024, 1024, glm::Vec3::new(1.0, 1.0, 1.0)),
            renderers: Vec::new(),
            perspective_camera: PerspectiveCamera::new(
                glm::Vec3::new(0.0, 1.0, -1.0),
                glm::Vec3::zeros(),
            ),
            ortho_camera: OrthoCamera::from_size(1.0),
        }
    }

    /// start the rendering for all renderers
    pub fn render(&mut self, game_state: &GameState) {
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
        for entity_ref in game_state.entity_manager.all_entities_iter() {
            let mesh = game_state
                .entity_manager
                .asset_from_type(entity_ref.mesh_type);

            let mut found = false;
            for r_type in self.renderers.iter_mut() {
                if let Batch(m_type, renderer) = r_type {
                    if *m_type == entity_ref.mesh_type {
                        match entity_ref.mesh_attribute {
                            Textured(id) => {
                                renderer.draw_tex_mesh(
                                    entity_ref.position,
                                    1.0,
                                    id,
                                    &self.perspective_camera,
                                    &mut self.shadow_map,
                                );
                                found = true;
                                break;
                            }
                            Colored(color) => {
                                renderer.draw_color_mesh(
                                    entity_ref.position,
                                    1.0,
                                    color.to_vec4(),
                                    &self.perspective_camera,
                                    &mut self.shadow_map,
                                );
                                found = true;
                                break;
                            }
                        }
                    }
                } else if let Instance(m_type, m_attr, renderer) = r_type {
                    if *m_type == entity_ref.mesh_type && *m_attr == entity_ref.mesh_attribute {
                        match entity_ref.mesh_attribute {
                            Textured(id) => {
                                // TODO: instance renderer has to be for textured meshes
                                if id == renderer.tex_id {
                                    renderer.add_position(entity_ref.position);
                                    found = true;
                                    break;
                                }
                            }
                            Colored(color) => {
                                if color == renderer.color {
                                    renderer.add_position(entity_ref.position);
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
                match entity_ref.mesh_type {
                    MeshType::Plane | MeshType::Cube => {
                        self.renderers
                            .push(Batch(entity_ref.mesh_type, BatchRenderer::new(mesh, 10)));
                    }
                    MeshType::Sphere => {
                        self.renderers.push(Instance(
                            entity_ref.mesh_type,
                            entity_ref.mesh_attribute,
                            InstanceRenderer::new(mesh, 10), // TODO: 10? oben auch
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

/// stores the renderer type with rendered entity type + renderer
pub enum RendererType {
    Batch(MeshType, BatchRenderer),
    Instance(MeshType, MeshAttribute, InstanceRenderer),
    Font(FontRenderer),
    Sprite(SpriteRenderer),
    Voxel(VoxelRenderer),
}

impl RendererType {
    /// yields the mesh type of a renderer type if present
    pub fn mesh_type(&self) -> Option<MeshType> {
        match self {
            Batch(ntt_type, _) => Some(*ntt_type),
            Instance(ntt_type, _, _) => Some(*ntt_type),
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
