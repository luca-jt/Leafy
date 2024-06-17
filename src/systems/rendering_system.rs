use std::collections::HashMap;

pub trait Renderer {
    fn render(&mut self);
}

pub struct RenderingSystem {
    renderers: HashMap<String, Box<dyn Renderer>>,
    // just do hardcoded renderer lists for every type?
}

impl RenderingSystem {
    /// add a renderer to the system
    pub fn add_renderer(&mut self, name: String, renderer: impl Renderer) {
        self.renderers.insert(name, Box::new(renderer));
    }

    /// start the rendering for all renderers
    pub fn render(&mut self) {
        for renderer in self.renderers.values_mut() {
            renderer.render();
        }
    }
}
