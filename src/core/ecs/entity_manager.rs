use crate::ecs::component::{
    Acceleration, MeshAttribute, MeshType, MotionState, Position, Renderable, TouchTime, Velocity,
};
use crate::ecs::entity::{
    Archetype, ArchetypeID, ComponentStorage, EntityID, EntityRecord, EntityType,
};
use crate::ecs::query::{ExcludeFilter, IncludeFilter};
use crate::rendering::mesh::{Mesh, SharedMesh};
use crate::{exclude_filter, include_filter};
use fyrox_resource::core::num_traits::Zero;
use std::any::{Any, TypeId};
use std::collections::hash_map::Keys;
use std::collections::HashMap;

/// create a component list for entity creation
#[macro_export]
macro_rules! components {
    ($($T:expr),*) => {
        vec![$(Box::new($T), )*]
    };
}

pub(crate) use components;

/// the main ressource manager holding both the ECS and the asset data
pub struct EntityManager {
    pub ecs: ECS,
    asset_register: HashMap<MeshType, SharedMesh>,
}

impl EntityManager {
    /// creates a new entitiy manager
    pub fn new() -> Self {
        Self {
            ecs: ECS::new(),
            asset_register: HashMap::new(),
        }
    }

    /// stores a new entity and returns the id of the new entity
    pub fn create_entity(&mut self, components: Vec<Box<dyn Any>>) -> EntityID {
        if components.contains_component::<Renderable>() {
            // following unwrap is safe because of contains check
            let mesh_type = components.component_data::<Renderable>().unwrap().mesh_type;
            if !self.asset_register.keys().any(|t| *t == mesh_type) {
                let mesh = match mesh_type {
                    MeshType::Sphere => Mesh::new("sphere.obj"),
                    MeshType::Cube => Mesh::new("cube.obj"),
                    MeshType::Plane => Mesh::new("plane.obj"),
                };
                self.asset_register
                    .insert(mesh_type, SharedMesh::from_mesh(mesh));
            }
        }

        self.ecs.create_entity(components)
    }

    /// creates a new entity with all basic data needed for physics and rendering
    pub fn create_regular_moving(
        &mut self,
        at: Position,
        mesh_type: MeshType,
        mesh_attribute: MeshAttribute,
    ) -> EntityID {
        self.create_entity(components!(
            at,
            Renderable {
                scale: 1f32.into(),
                mesh_type,
                mesh_attribute,
            },
            MotionState {
                velocity: Velocity::zeros(),
                acceleration: Acceleration::zeros()
            },
            TouchTime::now()
        ))
    }

    /// creates a new fixed entity with all basic data needed for rendering
    pub fn create_regular_fixed(
        &mut self,
        at: Position,
        mesh_type: MeshType,
        mesh_attribute: MeshAttribute,
    ) -> EntityID {
        self.create_entity(components!(
            at,
            Renderable {
                scale: 1f32.into(),
                mesh_type,
                mesh_attribute,
            },
            TouchTime::now() // needed?
        ))
    }

    /// deletes an entity from the register by id and returns the removed entity
    pub fn delete_entity(&mut self, entity: EntityID) -> Result<(), ()> {
        if let Some(renderable) = self.ecs.get_component::<Renderable>(entity) {
            if !self
                .ecs
                .query1::<Renderable>(include_filter!(), exclude_filter!())
                .any(|component| renderable.mesh_type == component.mesh_type)
            {
                self.asset_register.remove(&renderable.mesh_type);
            }
        }
        self.ecs.delete_entity(entity)
    }

    /// makes mesh data available for a given entity id
    pub fn asset_from_id(&self, entity: EntityID) -> Option<SharedMesh> {
        let mesh_type = self.ecs.get_component::<MeshType>(entity)?;
        Some(
            self.asset_register
                .get(mesh_type)
                .expect("asset not in the register")
                .clone(),
        )
    }

    /// makes mesh data available for a given mesh type
    pub fn asset_from_type(&self, mesh_type: MeshType) -> SharedMesh {
        self.asset_register
            .get(&mesh_type)
            .expect("asset not in the register")
            .clone()
    }

    /// iterate over all of the stored entities
    pub fn all_ids_iter(&self) -> Keys<'_, EntityID, EntityRecord> {
        self.ecs.entity_index.keys()
    }
}

/// the entity component system that manages all the data associated with an entity
pub struct ECS {
    pub(crate) next_entity: EntityID,
    pub(crate) next_archetype_id: ArchetypeID,
    pub(crate) entity_index: HashMap<EntityID, EntityRecord>,
    pub(crate) archetypes: HashMap<ArchetypeID, Archetype>,
    pub(crate) type_to_archetype: HashMap<EntityType, ArchetypeID>,
}

impl ECS {
    /// creates a new ecs
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            next_archetype_id: 0,
            entity_index: HashMap::new(),
            archetypes: HashMap::new(),
            type_to_archetype: HashMap::new(),
        }
    }

    /// creates a new entity with given components, stores the given data and returns the id
    pub(crate) fn create_entity(&mut self, components: Vec<Box<dyn Any>>) -> EntityID {
        let new_entity = self.next_entity;
        self.next_entity += 1;

        let entity_type = EntityType::from(&components);
        let archetype_id = self.get_arch_id(&entity_type);

        let archetype = self.archetypes.get_mut(&archetype_id).unwrap();
        let row = archetype.components.get_mut(&entity_type[0]).unwrap().len();

        for component in components {
            archetype
                .components
                .get_mut(&component.type_id())
                .unwrap()
                .push(component);
        }

        self.entity_index
            .insert(new_entity, EntityRecord { archetype_id, row });

        new_entity
    }

    /// deletes a stored entity and all the associated component data
    pub(crate) fn delete_entity(&mut self, entity: EntityID) -> Result<(), ()> {
        let record = self.entity_index.remove(&entity).ok_or(())?;
        let archetype = self.archetypes.get_mut(&record.archetype_id).ok_or(())?;
        for column in archetype.components.values_mut() {
            column.remove(record.row);
        }
        if archetype
            .components
            .values()
            .nth(0)
            .unwrap()
            .len()
            .is_zero()
        {
            self.archetypes.remove(&record.archetype_id);
            self.type_to_archetype
                .retain(|_, arch_id| *arch_id != record.archetype_id);
        }
        Ok(())
    }

    /// yields the component data reference of an entity if present
    pub fn get_component<T: Any>(&self, entity: EntityID) -> Option<&T> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get(&record.archetype_id)?;
        let component_vec = archetype.components.get(&TypeId::of::<T>())?;
        let component = component_vec.get(record.row)?;
        component.downcast_ref::<T>()
    }

    /// yields the mutable component data reference of an entity if present
    pub fn get_component_mut<T: Any>(&mut self, entity: EntityID) -> Option<&mut T> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get_mut(&record.archetype_id)?;
        let component_vec = archetype.components.get_mut(&TypeId::of::<T>())?;
        let component = component_vec.get_mut(record.row)?;
        component.downcast_mut::<T>()
    }

    /// gets the vector of all associated component TypeId's
    pub fn get_entity_type(&self, entity: EntityID) -> EntityType {
        let record = self.entity_index.get(&entity).expect("entity doesnt exist");
        let archetype = self.archetypes.get(&record.archetype_id).unwrap();

        EntityType(archetype.components.keys().cloned().collect())
    }

    /// adds a component to an existing entity
    pub fn add_component<T: Any>(&mut self, entity: EntityID, component: T) {
        let row = self.entity_index.get(&entity).unwrap().row;
        let archetype_id = self.entity_index.get(&entity).unwrap().archetype_id;
        let old_archetype = self.archetypes.get_mut(&archetype_id).unwrap();

        // Remove the entity's components from the old archetype
        let mut components: Vec<Box<dyn Any>> = old_archetype
            .components
            .values_mut()
            .map(|vec| vec.swap_remove(row))
            .collect();

        // Add the new component
        components.push(Box::new(component));

        // Create a new entity type
        let new_entity_type = EntityType::from(&components);

        // Find or create the new archetype
        let new_archetype_id = self.get_arch_id(&new_entity_type);

        let new_archetype = self.archetypes.get_mut(&new_archetype_id).unwrap();
        let new_row = new_archetype
            .components
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .len();

        for component in components {
            new_archetype
                .components
                .get_mut(&component.type_id())
                .unwrap()
                .push(component);
        }

        // Update the entity record
        let record = self.entity_index.get_mut(&entity).unwrap();
        record.archetype_id = new_archetype_id;
        record.row = new_row;
    }

    /// checks wether or not an entity has a component of given type associated with it
    pub fn has_component<T: Any>(&self, entity: EntityID) -> bool {
        let record = self.entity_index.get(&entity).unwrap();
        let archetype = self.archetypes.get(&record.archetype_id).unwrap();
        archetype.components.contains_key(&TypeId::of::<T>())
    }

    /// removes a component from an entity and returns the component data if present
    pub fn remove_component<T: Any>(&mut self, entity: EntityID) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let archetype_id = self.entity_index.get(&entity).unwrap().archetype_id;
        let row = self.entity_index.get(&entity).unwrap().row;
        let old_archetype = self.archetypes.get_mut(&archetype_id).unwrap();

        // Remove the entity's components from the old archetype
        let mut components: Vec<Box<dyn Any>> = old_archetype
            .components
            .values_mut()
            .map(|vec| vec.swap_remove(row))
            .collect();

        // Remove the specific component
        let component_index = components.iter().position(|c| c.type_id() == type_id)?;
        let component = components.swap_remove(component_index);

        // Create a new entity type
        let new_entity_type = EntityType::from(&components);

        // Find or create the new archetype
        let new_archetype_id = self.get_arch_id(&new_entity_type);

        let new_archetype = self.archetypes.get_mut(&new_archetype_id).unwrap();
        let new_row = new_archetype
            .components
            .get_mut(&new_entity_type[0])
            .unwrap()
            .len();

        for component in components {
            new_archetype
                .components
                .get_mut(&component.type_id())
                .unwrap()
                .push(component);
        }

        // Update the entity record
        let record = self.entity_index.get_mut(&entity).unwrap();
        record.archetype_id = new_archetype_id;
        record.row = new_row;

        component.downcast::<T>().ok().map(|b| *b)
    }

    /// gets the archetype id of an entity type and creates a new archetype if necessary
    fn get_arch_id(&mut self, entity_type: &EntityType) -> ArchetypeID {
        *self
            .type_to_archetype
            .entry(entity_type.clone())
            .or_insert_with(|| {
                let id = self.next_archetype_id;
                self.next_archetype_id += 1;
                self.archetypes.insert(
                    id,
                    Archetype {
                        id,
                        components: entity_type
                            .iter()
                            .map(|&type_id| (type_id, Vec::new()))
                            .collect(),
                    },
                );
                id
            })
    }
}
