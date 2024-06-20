use crate::ecs::entity::{Entity, EntityID};
use std::collections::HashMap;

pub struct EntityManager {
    next_id: EntityID,
    entities: HashMap<EntityID, Entity>,
}

impl EntityManager {
    /// creates a new entitiy manager
    pub fn new() -> Self {
        Self {
            next_id: 0,
            entities: HashMap::new(),
        }
    }

    /// stores a new entity and returns the id of the new entity
    pub fn add_entity(&mut self, entity: Entity) -> EntityID {
        self.entities.insert(self.next_id, entity);
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// deletes an entity from the register by id and returns the removed entity
    pub fn rm_entity(&mut self, id: EntityID) -> Entity {
        self.entities.remove(&id).unwrap()
    }

    /// get the reference of a stored entity
    pub fn get_entity(&mut self, id: EntityID) -> &Entity {
        self.entities.get(&id).unwrap()
    }

    /// get the mutable reference of a stored entity
    pub fn get_entity_mut(&mut self, id: EntityID) -> &mut Entity {
        self.entities.get_mut(&id).unwrap()
    }
}
