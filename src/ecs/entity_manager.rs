use crate::ecs::entity::{Entity, EntityID, MeshType};
use crate::rendering::mesh::{Mesh, SharedMesh};
use std::collections::hash_map::{Keys, Values, ValuesMut};
use std::collections::HashMap;

pub struct EntityManager {
    entity_register: HashMap<EntityID, Entity>,
    next_entity_id: EntityID,
    asset_register: HashMap<MeshType, SharedMesh>,
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
    pub fn add_entity(&mut self, entity: Entity) -> EntityID {
        if !self.asset_register.keys().any(|t| *t == entity.mesh_type) {
            let mesh = match entity.mesh_type {
                MeshType::Sphere => Mesh::new("sphere.obj"),
                MeshType::Cube => Mesh::new("cube.obj"),
                MeshType::Plane => Mesh::new("plane.obj"),
            };
            self.asset_register
                .insert(entity.mesh_type, SharedMesh::from_mesh(mesh));
        }

        self.entity_register.insert(self.next_entity_id, entity);
        let id = self.next_entity_id;
        self.next_entity_id += 1;

        id
    }

    /// deletes an entity from the register by id and returns the removed entity
    pub fn rm_entity(&mut self, entity_id: EntityID) -> Entity {
        self.entity_register
            .remove(&entity_id)
            .expect("entity not in the register")
    }

    /// get the reference of a stored entity
    pub fn get_entity(&self, entity_id: EntityID) -> &Entity {
        self.entity_register
            .get(&entity_id)
            .expect("entity not in the register")
    }

    /// get the mutable reference of a stored entity
    pub fn get_entity_mut(&mut self, entity_id: EntityID) -> &mut Entity {
        self.entity_register
            .get_mut(&entity_id)
            .expect("entity not in the register")
    }

    /// makes mesh data available for a given entity id
    pub fn asset_from_id(&self, entity_id: EntityID) -> SharedMesh {
        let entity = self
            .entity_register
            .get(&entity_id)
            .expect("entity not in the register");
        self.asset_register
            .get(&entity.mesh_type)
            .expect("asset not in the register")
            .clone()
    }

    /// makes mesh data available for a given mesh type
    pub fn asset_from_type(&self, mesh_type: MeshType) -> SharedMesh {
        self.asset_register
            .get(&mesh_type)
            .expect("asset not in the register")
            .clone()
    }

    /// iterate over all of the stored entities
    pub fn all_ids_iter(&self) -> Keys<'_, EntityID, Entity> {
        self.entity_register.keys()
    }

    /// iterate over all of the stored entities
    pub fn all_entities_iter(&self) -> Values<'_, EntityID, Entity> {
        self.entity_register.values()
    }

    /// iterate over all of the stored entities mutably
    pub fn all_entities_iter_mut(&mut self) -> ValuesMut<'_, EntityID, Entity> {
        self.entity_register.values_mut()
    }
}
