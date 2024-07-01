use crate::ecs::entity::{EntityType, MeshType};
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::data::{OrthoCamera, PerspectiveCamera, ShadowMap, TextureMap};
use crate::rendering::font_renderer::FontRenderer;
use crate::rendering::instance_renderer::InstanceRenderer;
use crate::rendering::sprite_renderer::SpriteRenderer;
use crate::rendering::voxel_renderer::VoxelRenderer;
use crate::state::game_state::GameState;
use nalgebra_glm as glm;
use MeshType::*;
use RendererType::*;

pub struct RenderingSystem {
    texture_map: TextureMap,
    shadow_map: ShadowMap,
    renderers: Vec<RendererType>,
    // TODO: scene files (initialize the right renderers)?
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
                .asset_from_type(entity_ref.entity_type);

            for r_type in self.renderers.iter_mut() {
                if let Batch(e_type, renderer) = r_type {
                    if *e_type == entity_ref.entity_type {
                        match entity_ref.mesh_type {
                            Textured(id) => {
                                renderer.draw_tex_mesh(
                                    entity_ref.position,
                                    1.0,
                                    id,
                                    &self.perspective_camera,
                                    &mut self.shadow_map,
                                );
                            }
                            Colored(color) => {
                                renderer.draw_color_mesh(
                                    entity_ref.position,
                                    1.0,
                                    color.to_vec4(),
                                    &self.perspective_camera,
                                    &mut self.shadow_map,
                                );
                            }
                        }
                    }
                } else if let Instance(e_type, m_type, renderer) = r_type {
                    if *e_type == entity_ref.entity_type && *m_type == entity_ref.mesh_type {
                        match entity_ref.mesh_type {
                            Textured(id) => {
                                // add entity to renderer
                            }
                            Colored(color) => {
                                // add entity to renderer
                            }
                        }
                    }
                }
            }
            // TODO: add new renderer if needed
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
    Batch(EntityType, BatchRenderer),
    Instance(EntityType, MeshType, InstanceRenderer<10>), // TODO: instance number
    Font(FontRenderer),
    Sprite(SpriteRenderer),
    Voxel(VoxelRenderer),
}

impl RendererType {
    /// yields the entity type of a renderer type if present
    pub fn entity_type(&self) -> Option<EntityType> {
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
