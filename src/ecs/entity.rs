use crate::ecs::component::Component;

/// abstract model of any thing in the game
pub struct Entity {
    components: Vec<Box<dyn Component>>,
}

impl Entity {
    pub fn add_component(&mut self, component: impl Component + 'static) {
        self.components.push(Box::new(component));
    }
}

// maybe kein Component Trait mit vector sondern einfach alle komponenten mit option hardgecoded
