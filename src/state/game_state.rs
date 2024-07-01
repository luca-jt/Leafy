use crate::ecs::component::{Color32, Position};
use crate::ecs::entity::{Entity, EntityID, EntityType, MeshType};
use crate::ecs::entity_manager::EntityManager;
use std::collections::HashSet;

/// state for the game logic
pub struct GameState {
    pub entity_manager: EntityManager,
    pub moving_entities: HashSet<EntityID>,
}

impl GameState {
    /// creates a new game state
    pub fn new() -> Self {
        let entity_manager = EntityManager::new();
        let moving_entities: HashSet<EntityID> = HashSet::new();

        let mut instance = Self {
            entity_manager,
            moving_entities,
        };
        instance.init();

        instance
    }

    /// initialize the game state
    fn init(&mut self) {
        let test_entity = Entity::new_moving(
            EntityType::Sphere,
            MeshType::Colored(Color32::RED),
            Position::zeros(),
        );
        let test_id = self.entity_manager.add_entity(test_entity);
        self.moving_entities.insert(test_id);
    }

    /// updates the game state
    pub fn update(&mut self) {
        //...
    }
}
