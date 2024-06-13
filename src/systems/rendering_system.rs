pub trait Renderer {
    fn render(&mut self);
}

pub struct RenderingSystem {
    renderers: Vec<Box<dyn Renderer>>,
}

impl RenderingSystem {
    /// start the rendering for all renderers
    pub fn render(&mut self) {
        for renderer in self.renderers.iter_mut() {
            renderer.render();
        }
    }
}
