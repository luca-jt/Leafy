use crate::ecs::entity::{Entity, EntityID, EntityType};
use crate::ecs::entity_manager::EntityManager;

/// state for the game logic
pub struct GameState {
    entity_manager: EntityManager,
    entities: Vec<EntityID>,
}

impl GameState {
    /// creates a new game state
    pub fn new() -> Self {
        let mut entity_manager = EntityManager::new();

        let test_entity = Entity::new(EntityType::Sphere);
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
