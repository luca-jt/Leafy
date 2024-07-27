use entity::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub mod component;
pub mod entity;
pub mod entity_manager;
pub mod query;

/// the entity component system that manages all the data associated with an entity
pub struct ECS {
    next_entity: EntityID,
    next_archetype_id: ArchetypeID,
    entity_index: HashMap<EntityID, EntityRecord>,
    archetypes: HashMap<ArchetypeID, Archetype>,
    type_to_archetype: HashMap<EntityType, ArchetypeID>,
}

/// create a component list for entity creation
macro_rules! components {
    ($($T:expr),*) => {
        vec![$(Box::new($T), )*]
    };
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
    pub fn create_entity(&mut self, components: Vec<Box<dyn Any>>) -> EntityID {
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

    /// yields the component data of an entity if present
    pub fn get_component<T: Any>(&self, entity: EntityID) -> Option<&T> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get(&record.archetype_id)?;
        let component_vec = archetype.components.get(&TypeId::of::<T>())?;
        let component = component_vec.get(record.row)?;
        component.downcast_ref::<T>()
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
}
