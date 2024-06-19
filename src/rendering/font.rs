use crate::systems::rendering_system::Renderer;

pub struct FontRenderer {}

impl FontRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for FontRenderer {
    fn render(&mut self) {
        //...
    }
}
