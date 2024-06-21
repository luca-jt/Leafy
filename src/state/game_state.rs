use crate::ecs::component::Color32;
use crate::ecs::entity::{Entity, EntityID, EntityType, MeshType};
use crate::ecs::entity_manager::EntityManager;

/// state for the game logic
pub struct GameState {
    pub entity_manager: EntityManager,
    pub entities: Vec<EntityID>,
}

impl GameState {
    /// creates a new game state
    pub fn new() -> Self {
        let mut entity_manager = EntityManager::new();

        let test_entity = Entity::new(EntityType::Sphere, MeshType::Colored(Color32::RED));
        let test_id = entity_manager.create_entity(test_entity);

        Self {
            entity_manager,
            entities: vec![test_id],
        }
    }

    /// updates the current game state
    pub fn update(&mut self) {
        //...
    }
}
