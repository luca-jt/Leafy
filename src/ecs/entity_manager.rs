use crate::ecs::entity::Entity;
use std::collections::HashMap;

pub struct EntityManager {
    id_count: u64,
    entities: HashMap<u64, Entity>,
}

impl EntityManager {
    /// creates a new entitiy manager
    pub fn new() -> Self {
        Self {
            id_count: 0,
            entities: HashMap::new(),
        }
    }

    /// stores a new entity and returns the id of the new entity
    pub fn add_entity(&mut self, entity: Entity) -> u64 {
        self.entities.insert(self.id_count, entity);
        self.id_count += 1;
        self.id_count
    }

    /// deletes an entity from the register by id and returns the removed entity
    pub fn rm_entity(&mut self, id: u64) -> Entity {
        self.entities.remove(&id).unwrap()
    }

    /// get the reference of a stored entity
    pub fn get_entity(&mut self, id: u64) -> &Entity {
        self.entities.get(&id).unwrap()
    }

    /// get the mutable reference of a stored entity
    pub fn get_entity_mut(&mut self, id: u64) -> &mut Entity {
        self.entities.get_mut(&id).unwrap()
    }
}
