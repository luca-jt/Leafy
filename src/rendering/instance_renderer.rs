use crate::systems::rendering_system::Renderer;

pub struct InstanceRenderer {}

impl InstanceRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for InstanceRenderer {
    fn render(&mut self) {
        todo!()
    }
}
