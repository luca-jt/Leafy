use crate::ecs::component::Component;

/// abstract model of any thing in the game
pub struct Entity {
    id: u32,
    components: Vec<Box<dyn Component>>,
}

impl Entity {
    pub fn add_component(&mut self, component: impl Component) {
        self.components.push(Box::new(component));
    }
}
