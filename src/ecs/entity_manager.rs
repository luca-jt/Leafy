use crate::ecs::component::utils::*;
use crate::ecs::component::*;
use crate::ecs::entity::*;
use crate::rendering::data::*;
use crate::rendering::mesh::{Hitbox, Mesh};
use crate::utils::constants::*;
use crate::utils::file::*;
use crate::utils::tools::types_eq;
use crate::{BumpBox, BumpVec};
use ahash::{AHashMap, AHashSet};
use bumpalo::Bump;
use itertools::Itertools;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex};
use tobj::{load_obj, GPU_LOAD_OPTIONS};

/// Identifier for a loaded mesh in the entity manager.
pub type MeshHandle = u64;

/// create a component list for entity creation (must use)
#[macro_export]
macro_rules! components {
    ($($T:expr),+) => {
        vec![$crate::ecs::entity_manager::_component_alloc($crate::utils::constants::NO_ENTITY), $($crate::ecs::entity_manager::_component_alloc($T)), +]
    };
}

/// internal arena allocator used for entity data
#[rustfmt::skip]
pub(crate) static ENTITY_ARENA_ALLOC: LazyLock<Mutex<UnsafeCell<Bump>>> = LazyLock::new(|| Mutex::new(UnsafeCell::new(Bump::with_capacity(ARENA_ALLOCATOR_CHUNK_SIZE))));

/// internal allocation function for entity data
pub fn _component_alloc<T: Component>(component: T) -> BumpBox<'static, dyn Component> {
    let arena_lock = ENTITY_ARENA_ALLOC.lock().unwrap();
    unsafe {
        let data = (*arena_lock.get()).alloc(component) as *mut T;
        BumpBox::from_raw(data as *mut dyn Component)
    }
}

/// the main ressource manager holding both the ECS and the asset data
pub struct EntityManager {
    pub(crate) ecs: UnsafeCell<ECS>,
    mesh_register: AHashMap<MeshHandle, Mesh>,
    lod_register: AHashMap<MeshHandle, [Mesh; 4]>,
    material_register: AHashMap<Rc<str>, Material>,
    pub(crate) texture_map: TextureMap,
    hitbox_register: AHashMap<HitboxHandle, Hitbox>,
    next_mesh_handle: MeshHandle,
}

impl EntityManager {
    /// creates a new entitiy manager
    pub fn new() -> Self {
        let mut mesh_register = AHashMap::new();
        mesh_register.insert(1, Mesh::from_bytes(TRIANGLE_MESH));
        mesh_register.insert(2, Mesh::from_bytes(PLANE_MESH));
        mesh_register.insert(3, Mesh::from_bytes(CUBE_MESH));

        Self {
            ecs: UnsafeCell::new(ECS::new()),
            mesh_register: AHashMap::new(),
            lod_register: AHashMap::new(),
            material_register,
            texture_map: TextureMap::new(),
            hitbox_register: AHashMap::new(),
            next_mesh_handle: 4,
        }
    }

    /// stores a new entity and returns the id of the new entity
    pub fn create_entity(&mut self, components: Vec<BumpBox<'static, dyn Component>>) -> EntityID {
        let entity = self.ecs.get_mut().create_entity(components);
        *self.get_component_mut::<EntityID>(entity).unwrap() = entity;
        self.recompute_rigid_body_data(entity);
        entity
    }

    /// deletes an entity from the register by id and returns the removed entity
    pub fn delete_entity(&mut self, entity: EntityID) -> FLResult {
        if unsafe { &*self.ecs.get() }.has_component::<Renderable>(entity) {
            todo!("material textures");
        }
        self.ecs.get_mut().delete_entity(entity)?;
        self.exec_commands();
        Ok(())
    }

    /// yields the component data reference of an entity if present (also returns ``None`` if the entity ID is invalid)
    pub fn get_component<T: Component>(&self, entity: EntityID) -> Option<&T> {
        unsafe { &*self.ecs.get() }.get_component::<T>(entity)
    }

    /// Yields the mutable component data reference of an entity if present (also returns ``None`` if the entity ID is invalid). If data is modified that influences loaded asset data, you have to recompute it manually with the managers methods.
    pub fn get_component_mut<T: Component>(&mut self, entity: EntityID) -> Option<&mut T> {
        self.ecs.get_mut().get_component_mut::<T>(entity)
    }

    /// adds a component to an existing entity (returns ``Err`` if the component is already present or the entity ID is invalid)
    pub fn add_component<T: Component>(&mut self, entity: EntityID, component: T) -> FLResult {
        self.ecs.get_mut().add_component::<T>(entity, component)?;
        if types_eq::<T, Renderable>() {
            todo!("material textures");
        } else if types_eq::<T, Scale>() || types_eq::<T, RigidBody>() {
            self.recompute_rigid_body_data(entity);
        }
        Ok(())
    }

    /// checks wether or not an entity has a component of given type associated with it (also returns ``false`` if the entity ID is invalid)
    pub fn has_component<T: Component>(&self, entity: EntityID) -> bool {
        unsafe { &*self.ecs.get() }.has_component::<T>(entity)
    }

    /// removes a component from an entity and returns the component data if present
    pub fn remove_component<T: Component>(&mut self, entity: EntityID) -> Option<T> {
        let removed = self.ecs.get_mut().remove_component::<T>(entity);
        if removed.is_some() {
            if types_eq::<T, Renderable>() {
                todo!("material textures");
            } else if types_eq::<T, Scale>() {
                self.recompute_rigid_body_data(entity);
            }
        }
        removed
    }

    /// Iterator of all the currently stored entity IDs.
    pub fn all_ids_iter(&self) -> impl Iterator<Item = EntityID> {
        unsafe { &*self.ecs.get() }.entity_index.keys().copied()
    }

    /// Loads all the meshes in the ``.obj`` file and all the mentioned materials.
    pub fn load_asset_file(
        &mut self,
        file_path: impl AsRef<Path>,
    ) -> Result<Vec<MeshHandle>, String> {
        let (models, materials) = load_obj(file_path, &GPU_LOAD_OPTIONS)?;
        let materials = materials?;

        let mut handles = Vec::with_capacity(models.len());

        for model in models {
            log::debug!("Loaded mesh {:?} from file {:?}.", model.name, file_path);

            let mtl_name = model
                .mesh
                .material_id
                .map(|index| materials[index].name.clone().into());

            let handle = self.next_mesh_handle;
            self.next_mesh_handle += 1;
            self.mesh_register
                .insert(handle, Mesh::from_obj_data(&model.mesh, mtl_name));
            load_data.handles.push(handle);
        }

        for material in materials {
            let mtl_name = material.name.clone().into();
            self.material_register
                .insert(mtl_name, Material::from_mtl(&material));
            log::debug!("Loaded material {:?}.", material.name);

            todo!("load the material textures");
        }

        Ok(handles)
    }

    /// Deletes a loaded mesh from the internal register. Returns wether or not the mesh existed. Also deletes potentially generated LODs for that mesh if present.
    pub fn delete_mesh(&mut self, handle: MeshHandle) -> bool {
        todo!("material and -texture removal");
        let success = self.mesh_register.remove(handle).is_some();
        if success {
            self.lod_register.remove(handle);
            log::debug!("deleted mesh and associated LODs from register: {mesh_type:?}");
        }
        success
    }

    /// Loads all the material data in a ``.mtl`` file and returns wether or not the file could be loaded.
    pub fn load_materials(&mut self, path: impl AsRef<Path> + Debug) -> bool {
        match load_mtl(file) {
            Ok((materials, _)) => {
                for material in materials {
                    let mtl_name = material.name.clone().into();
                    self.material_register
                        .insert(mtl_name, Material::from_mtl(&material));
                    log::debug!("Loaded material {:?}.", material.name);

                    todo!("load the material textures");
                }
                true
            }
            Err(msg) => {
                log::error!("{msg:?}");
                false
            }
        }
    }

    /// Deletes a stored material and returns wether or not the material was present.
    pub fn delete_material(&mut self, name: &Rc<str>) -> bool {
        let success = self.material_register.remove(name).is_some();
        if success {
            log::debug!("Deleted material {name:?}.");
            todo!("delete the material textures if needed");
        }
        success
    }

    /// Generates all LODs for a loaded mesh. Returns wether or not the given mesh was present and LODs were loaded.
    pub fn load_lods(&mut self, handle: MeshHandle) -> bool {
        if let Some(mesh) = self.mesh_from_type(handle, LOD::None) {
            let lod_array = mesh.generate_lods();
            self.lod_register.insert(mesh_type.clone(), lod_array);
            log::debug!("loaded LODs in register for mesh: {mesh_type:?}");
            true
        } else {
            false
        }
    }

    /// Deletes the stored LODs for a given mesh from the internal registers and returns wether or not that mesh was present.
    pub fn delete_lods(&mut self, handle: MeshHandle) -> bool {
        let success = self.lod_register.remove(handle).is_some();
        todo!("have some way of looking up mesh meta data from a handle so we can display it here in the log for example");
        if success {
            log::debug!("Deleted LODs from register for mesh: {mesh_type:?}.");
        }
        success
    }

    /// Loads a hitbox that optionally depends on a loaded mesh and returns wether or not the loading was successful.
    pub fn load_hitbox(&mut self, hitbox_type: HitboxType, handle: Option<MeshHandle>) -> bool {
        todo!("store the name of a mesh in the struct so that handles are enough to get some information -> same for materials");
        if !self.hitbox_register.contains_key(&(hitbox_type, mesh_type)) {
            let hitbox = if let Some(mesh_type) = opt_mesh_type {
                self.mesh_register
                    .get(mesh_type)
                    .unwrap()
                    .generate_hitbox(&hitbox_type)
            } else {
                Hitbox::from_generic_type(hitbox_type)
            };

            self.hitbox_register
                .insert((hitbox_type, mesh_type), hitbox);

            log::debug!(
                "loaded hitbox {:?} in register for MeshType {:?}",
                hitbox_type,
                mesh_type
            );
            true
        } else {
            false
        }
    }

    /// Deletes a loaded hitbox and returns wether or not the hitbox was actually present.
    pub fn delete_hitbox(&mut self, hitbox_type: HitboxType, handle: MeshHandle) -> bool {
        let success = self
            .hitbox_register
            .remove(&(hitbox_type, mesh_type))
            .is_some();

        if success {
            log::debug!("deleted hitbox {hitbox_type:?} from register for mesh {mesh_type:?}");
        }
        success
    }

    /// Loads a texture in the internal register and returns wether or not the loading was successful.
    pub fn load_texture(&mut self, texture: &Texture) -> bool {
        self.texture_map.add_texture(texture)
    }

    /// Deletes a stored texture and returns wether or not the texture was present.
    pub fn delete_texture(&mut self, texture: &Texture) -> bool {
        self.texture_map.delete_texture(texture)
    }

    /// Loads a material texture in the internal register and returns wether or not the loading was successful.
    pub fn load_material_texture(&mut self, path: &Rc<Path>) -> bool {
        todo!("with these textures the path function argument here and in other similar cases should be the full path");
        self.texture_map.add_material_texture(path)
    }

    /// Deletes a stored material texture and returns wether or not the texture was present.
    pub fn delete_material_texture(&mut self, path: &Rc<Path>) -> bool {
        self.texture_map.delete_material_texture(path)
    }

    /// Loads the texture data for a sprite and makes
    pub fn load_sprite(&mut self, path: &Rc<Path>) -> bool {
        self.texture_map.add_sprite(path.clone())
    }

    /// Deletes a stored sprite and returns wether or not the deletion was successful.
    pub fn delete_sprite(&mut self, path: &Rc<Path>) -> bool {
        self.texture_map.delete_sprite(path)
    }

    /// Loads a sprite sheet and returns wether or not the load was successful.
    pub fn load_sprite_sheet(&mut self, path: &Rc<Path>) -> bool {
        self.texture_map.add_sheet(path)
    }

    /// Deletes a stored sprite sheet and returns wether or not the deletion was successful.
    pub fn delete_sprite_sheet(&mut self, path: &Rc<Path>) -> bool {
        self.texture_map.delete_sheet(path)
    }

    /// Computes the rigid body physics data from component data and stores it for physics sim. When you update component data that influences this, you can call this function to refresh the state. Relevant components are ``RigidBody``, ``Scale`` and ``Renderable``.
    pub fn recompute_rigid_body_data(&mut self, entity: EntityID) {
        if unsafe { &*self.ecs.get() }.has_component::<Renderable>(entity)
            && unsafe { &*self.ecs.get() }.has_component::<RigidBody>(entity)
        {
            let mt = &unsafe { &*self.ecs.get() }
                .get_component::<Renderable>(entity)
                .unwrap()
                .mesh_type;

            let mesh = self
                .mesh_from_type(mt, LOD::None)
                .expect("mesh data missing");
            let scale = unsafe { &*self.ecs.get() }
                .get_component::<Scale>(entity)
                .copied();

            let density = unsafe { &*self.ecs.get() }
                .get_component::<RigidBody>(entity)
                .unwrap()
                .density;

            let (inv_inertia_tensor, center_of_mass, mass) =
                mesh.intertia_data(density, &scale.unwrap_or_default());

            let body = self
                .ecs
                .get_mut()
                .get_component_mut::<RigidBody>(entity)
                .unwrap();

            body.inv_inertia_tensor = inv_inertia_tensor;
            body.center_of_mass = center_of_mass;
            body.mass = mass;
        }
    }

    /// makes mesh data available for a given MeshType
    pub(crate) fn mesh_from_type(&self, mesh_type: &MeshType, lod: LOD) -> Option<&Mesh> {
        match lod {
            LOD::None => self.mesh_register.get(mesh_type),
            _ => Some(
                self.lod_register
                    .get(mesh_type)?
                    .get(lod as usize - 1)
                    .unwrap(),
            ),
        }
    }

    /// makes hitbox data available for given entity data
    pub(crate) fn hitbox_from_data(
        &self,
        hitbox: HitboxType,
        opt_handle: Option<MeshHandle>,
    ) -> Option<&Hitbox> {
        self.hitbox_register.get(&(hitbox, opt_handle))
    }

    /// clears all of the stored entites and their associated data and invalidates all of the IDs yielded from the system up to this point
    pub fn clear(&mut self) {
        self.ecs.get_mut().clear();
        self.mesh_register.clear();
        self.lod_register.clear();
        self.texture_map.clear();
        self.texture_map.clear();
        self.hitbox_register.clear();
        self.commands.clear();
    }
}

/// the entity component system that manages all the data associated with an entity
pub(crate) struct ECS {
    next_entity: EntityID,
    next_archetype_id: ArchetypeID,
    entity_index: AHashMap<EntityID, EntityRecord>,
    pub(crate) archetypes: AHashMap<ArchetypeID, Archetype>,
    type_to_archetype: AHashMap<EntityType, ArchetypeID>,
}

impl ECS {
    /// creates a new ecs
    pub(crate) fn new() -> Self {
        Self {
            next_entity: 1,
            next_archetype_id: 1,
            entity_index: AHashMap::new(),
            archetypes: AHashMap::new(),
            type_to_archetype: AHashMap::new(),
        }
    }

    /// Creates a new entity with given components, stores the given data and returns the id.
    pub(crate) fn create_entity(
        &mut self,
        components: Vec<BumpBox<'static, dyn Component>>,
    ) -> EntityID {
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
    pub(crate) fn delete_entity(&mut self, entity: EntityID) -> FLResult {
        let record = self
            .entity_index
            .remove(&entity)
            .ok_or(String::from("entity ID not found"))?;

        let archetype = self.archetypes.get_mut(&record.archetype_id).unwrap();
        for column in archetype.components.values_mut() {
            column.swap_remove(record.row);
        }
        if archetype.components.values().nth(0).unwrap().is_empty() {
            self.archetypes.remove(&record.archetype_id);
            self.type_to_archetype
                .retain(|_, arch_id| *arch_id != record.archetype_id);
        } else {
            self.edit_record_after_delete(record.archetype_id, record.row);
        }
        Ok(())
    }

    /// yields the component data reference of an entity if present (also returns ``None`` if the entity ID is invalid)
    pub(crate) fn get_component<T: Component>(&self, entity: EntityID) -> Option<&T> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get(&record.archetype_id).unwrap();
        let component_vec = archetype.components.get(&TypeId::of::<T>())?;
        let component = component_vec.get(record.row).unwrap();
        (&**component as &dyn Any).downcast_ref::<T>()
    }

    /// yields the mutable component data reference of an entity if present (also returns ``None`` if the entity ID is invalid)
    pub(crate) fn get_component_mut<T: Component>(&mut self, entity: EntityID) -> Option<&mut T> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get_mut(&record.archetype_id).unwrap();
        let component_vec = archetype.components.get_mut(&TypeId::of::<T>())?;
        let component = component_vec.get_mut(record.row).unwrap();
        (&mut **component as &mut dyn Any).downcast_mut::<T>()
    }

    /// gets the vector of all associated component TypeId's (returns ``None`` if the entity ID is invalid)
    pub(crate) fn get_entity_type(&self, entity: EntityID) -> Option<EntityType> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get(&record.archetype_id).unwrap();
        Some(EntityType::from(
            archetype.components.keys().copied().collect_vec(),
        ))
    }

    /// adds a component to an existing entity
    pub(crate) fn add_component<T: Component>(
        &mut self,
        entity: EntityID,
        component: T,
    ) -> FLResult {
        if self.has_component::<T>(entity) {
            return Err(String::from("entity already has this component"));
        }
        let mut entity_type = self.get_entity_type(entity).ok_or("entity ID not found")?;
        let record = self.entity_index.get(&entity).unwrap();
        let old_archetype = self.archetypes.get_mut(&record.archetype_id).unwrap();
        let old_arch_id = old_archetype.id;

        // Remove the entity's components from the old archetype
        let old_components: Vec<BumpBox<dyn Component>> = old_archetype
            .components
            .values_mut()
            .map(|vec| vec.swap_remove(record.row))
            .collect();

        // remove the old archetype if there are no more components in it
        if old_archetype.components.values().nth(0).unwrap().is_empty() {
            self.archetypes.remove(&record.archetype_id);
            self.type_to_archetype.remove(&entity_type);
        } else {
            self.edit_record_after_delete(old_arch_id, record.row);
        }

        // Find or create the new archetype
        entity_type.add_component::<T>();
        let new_archetype_id = self.get_arch_id(&entity_type);

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
            .push(_component_alloc(component));

        // Update the entity record
        let record = self.entity_index.get_mut(&entity).unwrap();
        record.archetype_id = new_archetype_id;
        record.row = new_row;
        Ok(())
    }

    /// checks wether or not an entity has a component of given type associated with it (also returns ``false`` if the entity ID is invalid)
    pub(crate) fn has_component<T: Component>(&self, entity: EntityID) -> bool {
        if let Some(record) = self.entity_index.get(&entity) {
            let archetype = self.archetypes.get(&record.archetype_id).unwrap();
            return archetype.components.contains_key(&TypeId::of::<T>());
        }
        false
    }

    /// Removes a component from an entity and returns the component data if present (also returns ``None`` if the entity ID is invalid). Deletes the entity if there are no more components after the deletion.
    pub(crate) fn remove_component<T: Component>(&mut self, entity: EntityID) -> Option<T> {
        if !self.has_component::<T>(entity) {
            return None;
        }
        let mut entity_type = self.get_entity_type(entity)?;
        let record = self.entity_index.get(&entity)?;
        let old_archetype = self.archetypes.get_mut(&record.archetype_id).unwrap();
        let old_arch_id = old_archetype.id;

        // Remove the entity's components from the old archetype
        let mut old_components: Vec<BumpBox<dyn Component>> = old_archetype
            .components
            .values_mut()
            .map(|vec| vec.swap_remove(record.row))
            .collect();

        // Remove the old archetype if there are no more components in it
        if old_archetype.components.values().nth(0).unwrap().is_empty() {
            self.archetypes.remove(&record.archetype_id);
            self.type_to_archetype.remove(&entity_type);
        } else {
            self.edit_record_after_delete(old_arch_id, record.row);
        }

        // Remove the specific component
        let index_to_remove = old_components
            .iter()
            .position(|c| (**c).type_id() == TypeId::of::<T>())?;

        let component = unsafe {
            Box::from_raw(BumpBox::leak(old_components.remove(index_to_remove))
                as *mut dyn Component as *mut dyn Any)
        }
        .downcast::<T>()
        .ok()
        .map(|b| *b);

        if old_components.is_empty() {
            self.entity_index.remove(&entity).unwrap();
            return component;
        }

        // Find or create the new archetype
        entity_type.rm_component::<T>();
        let new_archetype_id = self.get_arch_id(&entity_type);

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

        component
    }

    /// erases all of the stored entity data
    pub(crate) fn clear(&mut self) {
        self.entity_index.clear();
        self.archetypes.clear();
        self.type_to_archetype.clear();
        let arena_lock = ENTITY_ARENA_ALLOC.lock().unwrap();
        unsafe { &mut *(arena_lock.get()) }.reset();
    }

    /// gets the archetype id of an entity type and creates a new archetype if necessary
    fn get_arch_id(&mut self, entity_type: &EntityType) -> ArchetypeID {
        let arena_lock = ENTITY_ARENA_ALLOC.lock().unwrap();
        let arena = unsafe { &*(arena_lock.get()) };

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
                            .map(|&type_id| (type_id, BumpVec::new_in(arena)))
                            .collect(),
                    },
                );
                id
            })
    }

    /// edit the row value that is now in the old spot in the entity records after an entity was removed from an archetype
    fn edit_record_after_delete(&mut self, archetype_id: ArchetypeID, changed_index: usize) {
        let last_index = self
            .archetypes
            .get(&archetype_id)
            .unwrap()
            .components
            .values()
            .nth(0)
            .unwrap()
            .len();

        let record = self
            .entity_index
            .values_mut()
            .find(|record| record.archetype_id == archetype_id && record.row == last_index)
            .unwrap();

        record.row = changed_index;
    }
}
