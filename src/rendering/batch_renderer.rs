use crate::systems::rendering_system::Renderer;

pub struct BatchRenderer {}

impl BatchRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for BatchRenderer {
    fn render(&mut self) {
        todo!()
    }
}
