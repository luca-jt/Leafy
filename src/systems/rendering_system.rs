use crate::rendering::data::TextureMap;
use crate::state::video_state::VideoState;
use std::collections::HashMap;

pub trait Renderer {
    fn render(&mut self);
}

pub struct RenderingSystem {
    pub video_state: VideoState,
    texture_map: TextureMap,
    renderers: HashMap<String, Box<dyn Renderer>>,
    // just do hardcoded renderer lists for every type?
}

impl RenderingSystem {
    /// creates a new rendering system
    pub fn new() -> Self {
        Self {
            video_state: VideoState::new(),
            texture_map: TextureMap::new(),
            renderers: HashMap::new(),
        }
    }

    /// add a renderer to the system
    pub fn add_renderer(&mut self, name: String, renderer: impl Renderer + 'static) {
        self.renderers.insert(name, Box::new(renderer));
    }

    /// start the rendering for all renderers
    pub fn render(&mut self) {
        for renderer in self.renderers.values_mut() {
            renderer.render();
        }
    }
}
