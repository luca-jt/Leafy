use crate::ecs::component::{Color32, MotionState, Position};
use crate::ecs::entity::{Entity, EntityID, MeshAttribute, MeshType};
use crate::ecs::entity_manager::EntityManager;
use std::collections::HashSet;

/// state for the game logic
pub struct GameState {
    pub entity_manager: EntityManager,
    pub moving_entities: HashSet<EntityID>,
    // TODO: scene files (initialize the right renderers)?
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

    /// turns the entity into a fixated one if not already
    fn fix_entity(&mut self, entity_id: EntityID) {
        let entity = self.entity_manager.get_entity_mut(entity_id);
        entity.motion_state = MotionState::Fixed;
        self.moving_entities.remove(&entity_id);
    }

    /// turns the entity into a moving one if not already
    fn unfix_entity(&mut self, entity_id: EntityID) {
        let entity = self.entity_manager.get_entity_mut(entity_id);
        if let MotionState::Fixed = entity.motion_state {
            entity.motion_state = MotionState::zeros();
            self.moving_entities.insert(entity_id);
        }
    }

    /// initialize the game state
    fn init(&mut self) {
        let mut floor = Entity::new_fixed(
            MeshType::Plane,
            MeshAttribute::Colored(Color32::GREEN),
            Position::zeros(),
        );
        floor.scale = 5.0;
        let _ = self.entity_manager.add_entity(floor);

        let test_entity = Entity::new_fixed(
            MeshType::Sphere,
            MeshAttribute::Colored(Color32::RED),
            Position::new(0.0, 2.0, 0.0),
        );
        let _ = self.entity_manager.add_entity(test_entity);
    }

    /// updates the game state
    pub fn update(&mut self) {
        //...
    }
}
