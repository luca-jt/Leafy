use crate::rendering::data::TextureMap;
use crate::state::game_state::GameState;
use std::collections::HashMap;

pub trait Renderer {
    fn render(&mut self);
}

pub struct RenderingSystem {
    texture_map: TextureMap,
    renderers: HashMap<String, Box<dyn Renderer>>,
    // just do hardcoded renderer lists for every type?
}

impl RenderingSystem {
    /// creates a new rendering system
    pub fn new() -> Self {
        Self {
            texture_map: TextureMap::new(),
            renderers: HashMap::new(),
        }
    }

    /// add a renderer to the system
    pub fn add_renderer(&mut self, name: String, renderer: impl Renderer + 'static) {
        self.renderers.insert(name, Box::new(renderer));
    }

    /// start the rendering for all renderers
    pub fn render(&mut self, game_state: &GameState) {
        clear_gl_screen();

        for id in game_state.entities.iter() {
            let _entity_ref = game_state.entity_manager.get_entity(*id);
            let _mesh = game_state.entity_manager.get_asset(*id);
            //...
        }

        for renderer in self.renderers.values_mut() {
            renderer.render();
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
