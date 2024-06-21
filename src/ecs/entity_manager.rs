use crate::ecs::entity::{Entity, EntityID, EntityType};
use crate::rendering::mesh::{Mesh, SharedMesh};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct EntityManager {
    entity_register: HashMap<EntityID, Entity>,
    next_entity_id: EntityID,
    asset_register: HashMap<EntityType, SharedMesh>,
}

impl EntityManager {
    /// creates a new entitiy manager
    pub fn new() -> Self {
        Self {
            entity_register: HashMap::new(),
            next_entity_id: 0,
            asset_register: HashMap::new(),
        }
    }

    /// stores a new entity and returns the id of the new entity
    pub fn create_entity(&mut self, entity: Entity) -> EntityID {
        if !self.asset_register.keys().any(|t| *t == entity.entity_type) {
            let mesh = match entity.entity_type {
                EntityType::Sphere => Mesh::new("sphere.obj"),
                EntityType::Cube => Mesh::new("cube.obj"),
            };
            self.asset_register
                .insert(entity.entity_type, Rc::new(RefCell::new(mesh)));
        }

        self.entity_register.insert(self.next_entity_id, entity);
        let id = self.next_entity_id;
        self.next_entity_id += 1;

        id
    }

    /// deletes an entity from the register by id and returns the removed entity
    pub fn rm_entity(&mut self, entity_id: EntityID) -> Entity {
        self.entity_register.remove(&entity_id).unwrap()
    }

    /// get the reference of a stored entity
    pub fn get_entity(&self, entity_id: EntityID) -> &Entity {
        self.entity_register.get(&entity_id).unwrap()
    }

    /// get the mutable reference of a stored entity
    pub fn get_entity_mut(&mut self, entity_id: EntityID) -> &mut Entity {
        self.entity_register.get_mut(&entity_id).unwrap()
    }

    /// makes mesh data available for a given asset id
    pub fn get_asset(&self, entity_id: EntityID) -> SharedMesh {
        let entity = self.entity_register.get(&entity_id).unwrap();
        self.asset_register
            .get(&entity.entity_type)
            .unwrap()
            .clone()
    }
}
