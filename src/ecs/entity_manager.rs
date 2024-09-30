use crate::ecs::component::*;
use crate::ecs::entity::*;
use crate::ecs::query::*;
use crate::rendering::data::TextureMap;
use crate::rendering::mesh::Mesh;
use std::any::{Any, TypeId};
use std::collections::hash_map::Keys;
use std::collections::HashMap;

/// create a component list for entity creation
#[macro_export]
macro_rules! components {
    ($($T:expr),+) => {
        vec![$(Box::new($T)),+]
    };
}

pub(crate) use components;

/// the main ressource manager holding both the ECS and the asset data
pub struct EntityManager {
    pub(crate) ecs: ECS,
    asset_register: HashMap<MeshType, Mesh>,
    pub(crate) texture_map: TextureMap,
}

impl EntityManager {
    /// creates a new entitiy manager
    pub fn new() -> Self {
        Self {
            ecs: ECS::new(),
            asset_register: HashMap::new(),
            texture_map: TextureMap::new(),
        }
    }

    /// stores a new entity and returns the id of the new entity
    pub fn create_entity(&mut self, components: Vec<Box<dyn Any>>) -> EntityID {
        if components.contains_component::<MeshType>() {
            // following unwrap is safe because of contains check
            let mesh_type = components.component_data::<MeshType>().unwrap();
            self.try_add_mesh(*mesh_type);
            if let Some(MeshAttribute::Textured(file)) =
                components.component_data::<MeshAttribute>()
            {
                self.texture_map.add_texture(file);
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
            Scale::default(),
            mesh_type,
            mesh_attribute,
            Velocity::zero(),
            Acceleration::zero(),
            TouchTime::now()
        ))
    }

    /// deletes an entity from the register by id and returns the removed entity
    pub fn delete_entity(&mut self, entity: EntityID) -> Result<(), ()> {
        if let Some(mesh_type) = self.ecs.get_component::<MeshType>(entity) {
            if self
                .query1::<MeshType>(vec![])
                .filter(|component| *mesh_type == **component)
                .count()
                == 1
            {
                self.asset_register.remove(mesh_type);
            }
            if let Some(MeshAttribute::Textured(path)) =
                self.ecs.get_component::<MeshAttribute>(entity)
            {
                if self
                    .query1::<MeshAttribute>(vec![])
                    .filter(|component| *path == component.texture_path().unwrap_or(""))
                    .count()
                    == 1
                {
                    self.texture_map.delete_texture(path);
                }
            }
        }
        self.ecs.delete_entity(entity)
    }

    /// yields the component data reference of an entity if present
    pub fn get_component<T: Any>(&self, entity: EntityID) -> Option<&T> {
        self.ecs.get_component::<T>(entity)
    }

    /// yields the mutable component data reference of an entity if present
    pub fn get_component_mut<T: Any>(&mut self, entity: EntityID) -> Option<&mut T> {
        self.ecs.get_component_mut::<T>(entity)
    }

    /// adds a component to an existing entity
    pub fn add_component<T: Any>(&mut self, entity: EntityID, component: T) {
        if let Some(mesh_type) = (&component as &dyn Any).downcast_ref::<MeshType>() {
            self.try_add_mesh(*mesh_type);
        }
        self.ecs.add_component::<T>(entity, component)
    }

    /// checks wether or not an entity has a component of given type associated with it
    pub fn has_component<T: Any>(&self, entity: EntityID) -> bool {
        self.ecs.has_component::<T>(entity)
    }

    /// removes a component from an entity and returns the component data if present
    pub fn remove_component<T: Any>(&mut self, entity: EntityID) -> Option<T> {
        let removed = self.ecs.remove_component::<T>(entity);
        if let Some(component) = removed.as_ref() {
            if let Some(mesh_type) = (component as &dyn Any).downcast_ref::<MeshType>() {
                if !self
                    .query1::<MeshType>(vec![])
                    .any(|component| mesh_type == component)
                {
                    self.asset_register.remove(mesh_type);
                }
            }
            if let Some(MeshAttribute::Textured(path)) =
                (component as &dyn Any).downcast_ref::<MeshAttribute>()
            {
                if !self
                    .query1::<MeshAttribute>(vec![])
                    .any(|component| *path == component.texture_path().unwrap_or(""))
                {
                    self.texture_map.delete_texture(path);
                }
            }
        }
        removed
    }

    /// makes mesh data available for a given entity id
    pub fn asset_from_id(&self, entity: EntityID) -> Option<&Mesh> {
        let mesh_type = self.ecs.get_component::<MeshType>(entity)?;
        self.asset_register.get(mesh_type)
    }

    /// makes mesh data available for a given mesh type
    pub fn asset_from_type(&self, mesh_type: MeshType) -> Option<&Mesh> {
        self.asset_register.get(&mesh_type)
    }

    /// iterate over all of the stored entities
    pub fn all_ids_iter(&self) -> Keys<'_, EntityID, EntityRecord> {
        self.ecs.entity_index.keys()
    }

    /// adds a new mesh to the asset register if necessary
    fn try_add_mesh(&mut self, mesh_type: MeshType) {
        if !self.asset_register.keys().any(|t| *t == mesh_type) {
            let mesh = match mesh_type {
                MeshType::Triangle => Mesh::new("triangle.obj"),
                MeshType::Plane => Mesh::new("plane.obj"),
                MeshType::Cube => Mesh::new("cube.obj"),
                MeshType::Sphere => Mesh::new("sphere.obj"),
                MeshType::Custom(path) => Mesh::new(path),
            };
            self.asset_register.insert(mesh_type, mesh);
        }
    }

    // ----------------------------------------------------------------------------------------------------------------
    // QUERY WRAPPER

    /// create immutable query for 1 component, iterable
    pub fn query1<T: Any>(&self, filter: Vec<Box<dyn QueryFilter>>) -> Query1<'_, T> {
        self.ecs.query1::<T>(filter)
    }

    /// create mutable query for 1 component, iterable
    pub fn query1_mut<T: Any>(&mut self, filter: Vec<Box<dyn QueryFilter>>) -> Query1Mut<'_, T> {
        self.ecs.query1_mut::<T>(filter)
    }

    /// create immutable query for 2 components, iterable
    pub fn query2<A: Any, B: Any>(&self, filter: Vec<Box<dyn QueryFilter>>) -> Query2<'_, A, B> {
        self.ecs.query2::<A, B>(filter)
    }

    /// create immutable query for 2 components, 1 optional, iterable
    pub fn query2_opt1<A: Any, B: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query2Opt1<'_, A, B> {
        self.ecs.query2_opt1::<A, B>(filter)
    }

    /// create mutable query for 2 components, iterable
    pub fn query2_mut<A: Any, B: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query2Mut<'_, A, B> {
        self.ecs.query2_mut::<A, B>(filter)
    }

    /// create mutable query for 2 components, 1 optional, iterable
    pub fn query2_mut_opt1<A: Any, B: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query2MutOpt1<'_, A, B> {
        self.ecs.query2_mut_opt1::<A, B>(filter)
    }

    /// create immutable query for 3 components, iterable
    pub fn query3<A: Any, B: Any, C: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3<'_, A, B, C> {
        self.ecs.query3::<A, B, C>(filter)
    }

    /// create immutable query for 3 components, 1 optional, iterable
    pub fn query3_opt1<A: Any, B: Any, C: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3Opt1<'_, A, B, C> {
        self.ecs.query3_opt1::<A, B, C>(filter)
    }

    /// create immutable query for 3 components, 2 optional, iterable
    pub fn query3_opt2<A: Any, B: Any, C: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3Opt2<'_, A, B, C> {
        self.ecs.query3_opt2::<A, B, C>(filter)
    }

    /// create mutable query for 3 components, iterable
    pub fn query3_mut<A: Any, B: Any, C: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3Mut<'_, A, B, C> {
        self.ecs.query3_mut::<A, B, C>(filter)
    }

    /// create mutable query for 3 components, 1 optional, iterable
    pub fn query3_mut_opt1<A: Any, B: Any, C: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3MutOpt1<'_, A, B, C> {
        self.ecs.query3_mut_opt1::<A, B, C>(filter)
    }

    /// create mutable query for 3 components, 2 optional, iterable
    pub fn query3_mut_opt2<A: Any, B: Any, C: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3MutOpt2<'_, A, B, C> {
        self.ecs.query3_mut_opt2::<A, B, C>(filter)
    }

    /// create immutable query for 4 components, iterable
    pub fn query4<A: Any, B: Any, C: Any, D: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4<'_, A, B, C, D> {
        self.ecs.query4::<A, B, C, D>(filter)
    }

    /// create immutable query for 4 components, 1 optional, iterable
    pub fn query4_opt1<A: Any, B: Any, C: Any, D: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4Opt1<'_, A, B, C, D> {
        self.ecs.query4_opt1::<A, B, C, D>(filter)
    }

    /// create immutable query for 4 components, 2 optional, iterable
    pub fn query4_opt2<A: Any, B: Any, C: Any, D: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4Opt2<'_, A, B, C, D> {
        self.ecs.query4_opt2::<A, B, C, D>(filter)
    }

    /// create immutable query for 4 components, 3 optional, iterable
    pub fn query4_opt3<A: Any, B: Any, C: Any, D: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4Opt3<'_, A, B, C, D> {
        self.ecs.query4_opt3::<A, B, C, D>(filter)
    }

    /// create mutable query for 4 components, iterable
    pub fn query4_mut<A: Any, B: Any, C: Any, D: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4Mut<'_, A, B, C, D> {
        self.ecs.query4_mut::<A, B, C, D>(filter)
    }

    /// create mutable query for 4 components, 2 optional, iterable
    pub fn query4_mut_opt2<A: Any, B: Any, C: Any, D: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4MutOpt2<'_, A, B, C, D> {
        self.ecs.query4_mut_opt2::<A, B, C, D>(filter)
    }

    /// create mutable query for 4 components, 3 optional, iterable
    pub fn query4_mut_opt3<A: Any, B: Any, C: Any, D: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4MutOpt3<'_, A, B, C, D> {
        self.ecs.query4_mut_opt3::<A, B, C, D>(filter)
    }

    /// create immutable query for 5 components, iterable
    pub fn query5<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5<'_, A, B, C, D, E> {
        self.ecs.query5::<A, B, C, D, E>(filter)
    }

    /// create immutable query for 5 components, 1 optional, iterable
    pub fn query5_opt1<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5Opt1<'_, A, B, C, D, E> {
        self.ecs.query5_opt1::<A, B, C, D, E>(filter)
    }

    /// create immutable query for 5 components, 2 optional, iterable
    pub fn query5_opt2<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5Opt2<'_, A, B, C, D, E> {
        self.ecs.query5_opt2::<A, B, C, D, E>(filter)
    }

    /// create immutable query for 5 components, 4 optional, iterable
    pub fn query5_opt4<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5Opt4<'_, A, B, C, D, E> {
        self.ecs.query5_opt4::<A, B, C, D, E>(filter)
    }

    /// create mutable query for 5 components, iterable
    pub fn query5_mut<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5Mut<'_, A, B, C, D, E> {
        self.ecs.query5_mut::<A, B, C, D, E>(filter)
    }

    /// create mutable query for 5 components, 1 optional, iterable
    pub fn query5_mut_opt1<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5MutOpt1<'_, A, B, C, D, E> {
        self.ecs.query5_mut_opt1::<A, B, C, D, E>(filter)
    }

    /// create mutable query for 5 components, 2 optional, iterable
    pub fn query5_mut_opt2<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5MutOpt2<'_, A, B, C, D, E> {
        self.ecs.query5_mut_opt2::<A, B, C, D, E>(filter)
    }

    /// create mutable query for 5 components, 3 optional, iterable
    pub fn query5_mut_opt3<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5MutOpt3<'_, A, B, C, D, E> {
        self.ecs.query5_mut_opt3::<A, B, C, D, E>(filter)
    }

    /// create mutable query for 5 components, 4 optional, iterable
    pub fn query5_mut_opt4<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5MutOpt4<'_, A, B, C, D, E> {
        self.ecs.query5_mut_opt4::<A, B, C, D, E>(filter)
    }
}

/// the entity component system that manages all the data associated with an entity
pub struct ECS {
    next_entity: EntityID,
    next_archetype_id: ArchetypeID,
    entity_index: HashMap<EntityID, EntityRecord>,
    pub(crate) archetypes: HashMap<ArchetypeID, Archetype>,
    type_to_archetype: HashMap<EntityType, ArchetypeID>,
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
        let row = archetype.components.values().nth(0).unwrap().len();

        for component in components {
            archetype
                .components
                .get_mut(&(*component).type_id())
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
        if archetype.components.values().nth(0).unwrap().len() == 0 {
            self.archetypes.remove(&record.archetype_id);
            self.type_to_archetype
                .retain(|_, arch_id| *arch_id != record.archetype_id);
        }
        Ok(())
    }

    /// yields the component data reference of an entity if present
    pub(crate) fn get_component<T: Any>(&self, entity: EntityID) -> Option<&T> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get(&record.archetype_id)?;
        let component_vec = archetype.components.get(&TypeId::of::<T>())?;
        let component = component_vec.get(record.row)?;
        component.downcast_ref::<T>()
    }

    /// yields the mutable component data reference of an entity if present
    pub(crate) fn get_component_mut<T: Any>(&mut self, entity: EntityID) -> Option<&mut T> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get_mut(&record.archetype_id)?;
        let component_vec = archetype.components.get_mut(&TypeId::of::<T>())?;
        let component = component_vec.get_mut(record.row)?;
        component.downcast_mut::<T>()
    }

    /// gets the vector of all associated component TypeId's
    pub(crate) fn get_entity_type(&self, entity: EntityID) -> EntityType {
        let record = self.entity_index.get(&entity).expect("entity doesnt exist");
        let archetype = self.archetypes.get(&record.archetype_id).unwrap();

        EntityType(archetype.components.keys().cloned().collect())
    }

    /// adds a component to an existing entity
    pub(crate) fn add_component<T: Any>(&mut self, entity: EntityID, component: T) {
        if self.has_component::<T>(entity) {
            return;
        }
        let record = self.entity_index.get(&entity).unwrap();
        let old_archetype = self.archetypes.get_mut(&record.archetype_id).unwrap();
        let old_arch_id = old_archetype.id;

        // Remove the entity's components from the old archetype
        let old_components: Vec<Box<dyn Any>> = old_archetype
            .components
            .values_mut()
            .map(|vec| vec.remove(record.row))
            .collect();

        // remove the old archetype if there are no more components in it
        if old_archetype.components.values().nth(0).unwrap().len() == 0 {
            self.archetypes.remove(&record.archetype_id);
        }

        self.shift_rows(old_arch_id, record.row);

        let mut new_entity_type = self.get_entity_type(entity);
        new_entity_type.add_component::<T>();

        // Find or create the new archetype
        let new_archetype_id = self.get_arch_id(&new_entity_type);

        let new_archetype = self.archetypes.get_mut(&new_archetype_id).unwrap();
        let new_row = new_archetype
            .components
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .len();

        // add all components to new archetype
        for old_component in old_components {
            new_archetype
                .components
                .get_mut(&(*old_component).type_id())
                .unwrap()
                .push(old_component);
        }
        new_archetype
            .components
            .get_mut(&component.type_id())
            .unwrap()
            .push(Box::new(component));

        // Update the entity record
        let record = self.entity_index.get_mut(&entity).unwrap();
        record.archetype_id = new_archetype_id;
        record.row = new_row;
    }

    /// checks wether or not an entity has a component of given type associated with it
    pub(crate) fn has_component<T: Any>(&self, entity: EntityID) -> bool {
        let record = self.entity_index.get(&entity).unwrap();
        let archetype = self.archetypes.get(&record.archetype_id).unwrap();
        archetype.components.contains_key(&TypeId::of::<T>())
    }

    /// removes a component from an entity and returns the component data if present
    pub(crate) fn remove_component<T: Any>(&mut self, entity: EntityID) -> Option<T> {
        if !self.has_component::<T>(entity) {
            return None;
        }
        let record = self.entity_index.get(&entity).unwrap();
        let old_archetype = self.archetypes.get_mut(&record.archetype_id).unwrap();
        let old_arch_id = old_archetype.id;

        // Remove the entity's components from the old archetype
        let mut old_components: Vec<Box<dyn Any>> = old_archetype
            .components
            .values_mut()
            .map(|vec| vec.remove(record.row))
            .collect();

        // remove the old archetype if there are no more components in it
        if old_archetype.components.values().nth(0).unwrap().len() == 0 {
            self.archetypes.remove(&record.archetype_id);
        }

        // Remove the specific component
        let index_to_remove = old_components
            .iter()
            .position(|c| (**c).type_id() == TypeId::of::<T>())?;
        let component = old_components.remove(index_to_remove);

        self.shift_rows(old_arch_id, record.row);

        // Find or create the new archetype
        let mut new_entity_type = self.get_entity_type(entity);
        new_entity_type.rm_component::<T>();
        let new_archetype_id = self.get_arch_id(&new_entity_type);

        let new_archetype = self.archetypes.get_mut(&new_archetype_id).unwrap();
        let new_row = new_archetype.components.values().nth(0).unwrap().len();

        // add the old components to the new archetype
        for old_component in old_components {
            new_archetype
                .components
                .get_mut(&(*old_component).type_id())
                .unwrap()
                .push(old_component);
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

    /// shift down all row values bigger than the given row in the entity records
    fn shift_rows(&mut self, archetype_id: ArchetypeID, bigger_than: usize) {
        debug_assert!(bigger_than > 0);
        for record in self
            .entity_index
            .values_mut()
            .filter(|record| record.archetype_id == archetype_id && record.row > bigger_than)
        {
            record.row -= 1;
        }
    }
}
