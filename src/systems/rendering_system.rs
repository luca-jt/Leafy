use crate::ecs::entity::{EntityType, MeshType};
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::data::TextureMap;
use crate::rendering::font_renderer::FontRenderer;
use crate::rendering::instance_renderer::InstanceRenderer;
use crate::rendering::sprite_renderer::SpriteRenderer;
use crate::state::game_state::GameState;
use MeshType::*;
use RendererType::*;

pub struct RenderingSystem {
    texture_map: TextureMap,
    renderers: Vec<RendererType>,
    // TODO: scene files (initialize the right renderers)?
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
            renderers: Vec::new(),
        }
    }

    /// start the rendering for all renderers
    pub fn render(&mut self, game_state: &GameState) {
        clear_gl_screen();
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                _ => {} // TODO
            }
        }
        for id in game_state.entities.iter() {
            let entity_ref = game_state.entity_manager.get_entity(*id);
            let mesh = game_state.entity_manager.get_asset(*id);

            for r_type in self.renderers.iter() {
                match r_type {
                    Batch(e_type, renderer) => {
                        if *e_type == entity_ref.entity_type {
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
                    Instance(e_type, m_type, renderer) => {
                        if *e_type == entity_ref.entity_type {
                            if *m_type == entity_ref.mesh_type {
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
                    _ => {}
                }
            }
            // add new renderer if needed
        }
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                _ => {} // TODO
            }
        }
    }
}

/// stores the renderer type with rendered entity type + renderer
pub enum RendererType {
    Batch(EntityType, BatchRenderer),
    Instance(EntityType, MeshType, InstanceRenderer),
    Font(FontRenderer),
    Sprite(SpriteRenderer),
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
