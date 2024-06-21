use crate::ecs::entity::EntityType;
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::data::TextureMap;
use crate::rendering::font_renderer::FontRenderer;
use crate::rendering::instance_renderer::InstanceRenderer;
use crate::rendering::sprite_renderer::SpriteRenderer;
use crate::state::game_state::GameState;
use RendererType::*;

/// stores the renderer type with rendered entity type + renderer
pub enum RendererType {
    Batch(EntityType, BatchRenderer),
    Instance(EntityType, InstanceRenderer),
    Font(FontRenderer),
    Sprite(SpriteRenderer),
}

impl RendererType {
    /// yields the entity type of a renderer type if present
    pub fn entity_type(&self) -> Option<EntityType> {
        match self {
            Batch(ntt_type, _) => Some(*ntt_type),
            Instance(ntt_type, _) => Some(*ntt_type),
            _ => None,
        }
    }
}

pub struct RenderingSystem {
    texture_map: TextureMap,
    renderers: Vec<RendererType>,
    // TODO: scene files (initialize the right renderers)
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

            for renderer_type in self.renderers.iter_mut() {
                match renderer_type {
                    Batch(_, _) => {
                        // TODO
                    }
                    Instance(_, _) => {
                        // TODO
                    }
                    _ => {}
                }
            }
        }
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                _ => {} // TODO
            }
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
