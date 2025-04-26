use crate::ecs::entity::*;
use crate::internal_prelude::*;
use crate::rendering::data::*;
use crate::rendering::mesh::{Hitbox, Mesh};
use tobj::{load_mtl, load_obj, GPU_LOAD_OPTIONS};

/// Identifier for a loaded mesh in the entity manager.
pub type MeshHandle = u64;

/// Creates a component list for entity creation (must use).
#[macro_export]
macro_rules! components {
    ($($T:expr),+) => {
        vec![$crate::ecs::entity::MetaDataComponentEntry::from_component($crate::utils::constants::NO_ENTITY), $($crate::ecs::entity::MetaDataComponentEntry::from_component($T)), +]
    };
}

/// internal arena allocator used for entity data
#[rustfmt::skip]
pub(crate) static ENTITY_ARENA: GlobalArenaAllocator = global_arena_allocator::<ENTITY_ALLOCATOR_CHUNK_SIZE>();

/// The main manager holding both the ECS containing the enitity data and the asset data ressource registers.
pub struct EntityManager {
    pub(crate) ecs: UnsafeCell<ECS>,
    mesh_register: AHashMap<MeshHandle, Mesh>,
    lod_register: AHashMap<MeshHandle, [Mesh; 4]>,
    material_register: AHashMap<String, Material>,
    pub(crate) texture_map: TextureMap,
    hitbox_register: AHashMap<(HitboxType, Option<MeshHandle>), Hitbox>,
    next_mesh_handle: MeshHandle,
}

impl EntityManager {
    /// Creates a new entitiy manager.
    pub(crate) fn new() -> Self {
        let mut mesh_register = AHashMap::new();
        mesh_register.insert(1, Mesh::from_bytes(TRIANGLE_MESH));
        mesh_register.insert(2, Mesh::from_bytes(PLANE_MESH));
        mesh_register.insert(3, Mesh::from_bytes(CUBE_MESH));

        Self {
            ecs: UnsafeCell::new(ECS::new()),
            mesh_register,
            lod_register: AHashMap::new(),
            material_register: AHashMap::new(),
            texture_map: TextureMap::new(),
            hitbox_register: AHashMap::new(),
            next_mesh_handle: 4,
        }
    }

    /// Stores the components, creates a new entity and returns the id of the that entity.
    pub fn create_entity(&mut self, components: Vec<MetaDataComponentEntry>) -> EntityID {
        assert!(
            components
                .iter()
                .map(|entry| entry.meta_data.type_id)
                .tuple_combinations()
                .all(|(id1, id2)| id1 != id2),
            "All component types have to be different."
        );
        let entity = self.ecs.get_mut().create_entity(components);
        *self.get_component_mut::<EntityID>(entity).unwrap() = entity;
        self.recompute_rigid_body_data(entity);
        entity
    }

    /// Deletes an entity from the register by ``EntityID`` and returns wether or not the removal was successful.
    pub fn delete_entity(&mut self, entity: EntityID) -> bool {
        self.ecs.get_mut().delete_entity(entity)
    }

    /// Yields the component data reference of an entity if present (also returns ``None`` if the entity ID is invalid).
    pub fn get_component<T: Component>(&self, entity: EntityID) -> Option<&T> {
        unsafe { &*self.ecs.get() }.get_component::<T>(entity)
    }

    /// Yields the mutable component data reference of an entity if present (also returns ``None`` if the entity ID is invalid). If data is modified that influences engine behavior and requires internal recomputations, you have to do that manually with the managers methods.
    pub fn get_component_mut<T: Component>(&mut self, entity: EntityID) -> Option<&mut T> {
        self.ecs.get_mut().get_component_mut::<T>(entity)
    }

    /// Adds a component to an existing entity (returns ``false`` if the component is already present or the ``EntityID`` is invalid).
    pub fn add_component<T: Component>(&mut self, entity: EntityID, component: T) -> bool {
        let success = self.ecs.get_mut().add_component::<T>(entity, component);
        if (types_eq::<T, Renderable>() || types_eq::<T, Scale>() || types_eq::<T, RigidBody>())
            && success
        {
            self.recompute_rigid_body_data(entity);
        }
        success
    }

    /// Checks wether or not an entity has a component of given type associated with it (also returns ``false`` if the entity ID is invalid).
    pub fn has_component<T: Component>(&self, entity: EntityID) -> bool {
        unsafe { &*self.ecs.get() }.has_component::<T>(entity)
    }

    /// Removes a component from an entity and returns the component data if present.
    pub fn remove_component<T: Component>(&mut self, entity: EntityID) -> Option<T> {
        let removed = self.ecs.get_mut().remove_component::<T>(entity);
        if removed.is_some() && types_eq::<T, Scale>() {
            self.recompute_rigid_body_data(entity);
        }
        removed
    }

    /// Iterator of all the currently stored entity IDs.
    pub fn all_ids_iter(&self) -> impl Iterator<Item = EntityID> {
        unsafe { &*self.ecs.get() }.entity_index.keys().copied()
    }

    /// Iterator of all the currently stored mesh handles.
    pub fn all_mesh_handles(&self) -> impl Iterator<Item = MeshHandle> + use<'_> {
        self.mesh_register.keys().copied()
    }

    /// Iterator of all the currently stored material names.
    pub fn all_material_names(&self) -> impl Iterator<Item = &str> {
        self.material_register.keys().map(|s| s.as_str())
    }

    /// Loads all the meshes in the ``.obj`` file and all the mentioned materials. Returns all the handles to the loaded meshes. If the loading fails, the returned ``Vec<MeshHandle>`` will be empty.
    pub fn load_asset_file(&mut self, file_path: impl AsRef<Path>) -> Vec<MeshHandle> {
        let file_path = file_path.as_ref();

        let (models, materials) = match load_obj(file_path, &GPU_LOAD_OPTIONS) {
            Ok((ms, mtl_load_result)) => match mtl_load_result {
                Ok(mtls) => (ms, mtls),
                Err(msg) => {
                    log::error!("Error loading asset file: {msg:?}.");
                    return Vec::new();
                }
            },
            Err(msg) => {
                log::error!("Error loading asset file: {msg:?}.");
                return Vec::new();
            }
        };

        let mut handles = Vec::with_capacity(models.len());

        for model in models {
            let mtl_name = model
                .mesh
                .material_id
                .map(|index| materials[index].name.clone());

            let handle = self.next_mesh_handle;
            self.next_mesh_handle += 1;

            self.mesh_register.insert(
                handle,
                Mesh::from_obj_data(&model, Rc::from(file_path), mtl_name),
            );
            log::debug!("Loaded mesh {:?} from file {:?}.", model.name, file_path);

            handles.push(handle);
        }

        for mtl in materials {
            let mtl_name = mtl.name.clone();
            if self.material_register.contains_key(&mtl_name) {
                log::warn!("Material '{mtl_name:?}' is already loaded and is overwritten.");
            }

            self.material_register
                .insert(mtl_name, Material::from_mtl(&mtl));

            log::debug!("Loaded material {:?}.", mtl.name);
            self.load_material_textures_from_mtl(&mtl, file_path);
        }

        handles
    }

    /// Deletes a loaded mesh from the internal register. Returns wether or not the mesh existed. Also deletes potentially generated LODs for that mesh if present.
    pub fn delete_mesh(&mut self, handle: MeshHandle) -> bool {
        if let Some(mesh) = self.mesh_register.remove(&handle) {
            self.lod_register.remove(&handle);
            log::debug!(
                "Deleted mesh and associated LODs from register: {:?}",
                mesh.name
            );
            true
        } else {
            log::warn!("Required mesh data not present.");
            false
        }
    }

    /// Loads all the material data in a ``.mtl`` file and returns wether or not the file could be loaded.
    pub fn load_materials(&mut self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        match load_mtl(path) {
            Ok((materials, _)) => {
                for mtl in materials {
                    let mtl_name = mtl.name.clone();
                    if self.material_register.contains_key(&mtl_name) {
                        log::warn!("Material '{mtl_name:?}' is already loaded and is overwritten.");
                    }

                    self.material_register
                        .insert(mtl_name, Material::from_mtl(&mtl));

                    log::debug!("Loaded material {:?}.", mtl.name);
                    self.load_material_textures_from_mtl(&mtl, path);
                }
                true
            }
            Err(msg) => {
                log::error!("{msg:?}");
                false
            }
        }
    }

    /// Loads all of the necessary material textures from mtl data.
    fn load_material_textures_from_mtl(&mut self, mtl: &tobj::Material, file_path: &Path) {
        if let Some(ambient_texture) = mtl.ambient_texture.as_ref() {
            let mut full_texture_path = PathBuf::from(file_path);
            full_texture_path.set_file_name(ambient_texture);
            self.texture_map
                .add_material_texture(full_texture_path.as_path());
        }
        if let Some(diffuse_texture) = mtl.diffuse_texture.as_ref() {
            let mut full_texture_path = PathBuf::from(file_path);
            full_texture_path.set_file_name(diffuse_texture);
            self.texture_map
                .add_material_texture(full_texture_path.as_path());
        }
        if let Some(specular_texture) = mtl.specular_texture.as_ref() {
            let mut full_texture_path = PathBuf::from(file_path);
            full_texture_path.set_file_name(specular_texture);
            self.texture_map
                .add_material_texture(full_texture_path.as_path());
        }
        if let Some(normal_texture) = mtl.normal_texture.as_ref() {
            let mut full_texture_path = PathBuf::from(file_path);
            full_texture_path.set_file_name(normal_texture);
            self.texture_map.add_material_texture(full_texture_path);
        }
    }

    /// Deletes a stored material and returns wether or not the material was present.
    pub fn delete_material(&mut self, name: impl AsRef<str>) -> bool {
        let name = name.as_ref();
        if let Some(mtl) = self.material_register.remove(name) {
            log::debug!("Deleted material {name:?}.");
            self.delete_material_textures_from_material(mtl);
            true
        } else {
            log::warn!("Required material data '{name:?}' not present.");
            false
        }
    }

    /// Deletes all the referenced material textures in a stored material.
    fn delete_material_textures_from_material(&mut self, mtl: Material) {
        if let Ambient::Texture(file_name) = mtl.ambient {
            self.texture_map.delete_material_texture(file_name);
        }
        if let Diffuse::Texture(file_name) = mtl.diffuse {
            self.texture_map.delete_material_texture(file_name);
        }
        if let Specular::Texture(file_name) = mtl.specular {
            self.texture_map.delete_material_texture(file_name);
        }
        if let Some(file_name) = mtl.normal_texture {
            self.texture_map.delete_material_texture(file_name);
        }
    }

    /// Generates all LODs for a loaded mesh. Returns wether or not the given mesh was present and LODs were loaded.
    pub fn load_lods(&mut self, handle: MeshHandle) -> bool {
        let opt_mesh = self.mesh_from_handle(handle, LOD::None);
        if opt_mesh.is_some() {
            let mesh = opt_mesh.unwrap();
            if self.lod_register.contains_key(&handle) {
                log::warn!("LOD data already present for mesh {:?}.", mesh.name);
                return false;
            }
            let lod_array = mesh.generate_lods();
            log::debug!("Loaded LODs in register for mesh: {:?}.", mesh.name);
            self.lod_register.insert(handle, lod_array);
            true
        } else {
            log::warn!("Required mesh data not present.");
            false
        }
    }

    /// Deletes the stored LODs for a given mesh from the internal registers and returns wether or not that mesh was present.
    pub fn delete_lods(&mut self, handle: MeshHandle) -> bool {
        let success = self.lod_register.remove(&handle).is_some();
        if success {
            let mesh_name = self.mesh_name_from_handle(handle).unwrap();
            log::debug!("Deleted LODs from register for mesh: {mesh_name:?}.");
        } else {
            log::warn!("Required LOD data not present.");
        }
        success
    }

    /// Loads a hitbox that optionally depends on a loaded mesh and returns wether or not the loading was successful.
    pub fn load_hitbox(&mut self, hitbox_type: HitboxType, opt_handle: Option<MeshHandle>) -> bool {
        if !self
            .hitbox_register
            .contains_key(&(hitbox_type, opt_handle))
        {
            let hitbox = if let Some(handle) = opt_handle {
                if let Some(mesh) = self.mesh_register.get(&handle) {
                    mesh.generate_hitbox(&hitbox_type)
                } else {
                    log::warn!("Mesh data not present for loading the hitbox {hitbox_type:?}.");
                    return false;
                }
            } else {
                Hitbox::from_generic_type(hitbox_type)
            };

            self.hitbox_register
                .insert((hitbox_type, opt_handle), hitbox);

            let mesh_name = opt_handle.map(|handle| self.mesh_name_from_handle(handle).unwrap());
            log::debug!(
                "Loaded hitbox {:?} in register for mesh {:?}.",
                hitbox_type,
                mesh_name
            );
            true
        } else {
            false
        }
    }

    /// Deletes a loaded hitbox and returns wether or not the hitbox was actually present.
    pub fn delete_hitbox(
        &mut self,
        hitbox_type: HitboxType,
        opt_handle: Option<MeshHandle>,
    ) -> bool {
        if let Some(handle) = opt_handle {
            if !self.mesh_register.contains_key(&handle) {
                log::warn!("Mesh data not present for hitbox {hitbox_type:?}.");
                return false;
            }
        }
        if self
            .hitbox_register
            .remove(&(hitbox_type, opt_handle))
            .is_some()
        {
            let mesh_name = opt_handle.map(|handle| self.mesh_name_from_handle(handle).unwrap());
            log::debug!("Deleted hitbox {hitbox_type:?} from register for mesh {mesh_name:?}");
            true
        } else {
            false
        }
    }

    /// Loads a texture in the internal register and returns wether or not the loading was successful.
    pub fn load_texture(&mut self, texture: &Texture) -> bool {
        self.texture_map.add_texture(texture)
    }

    /// Deletes a stored texture and returns wether or not the texture was present.
    pub fn delete_texture(&mut self, texture: &Texture) -> bool {
        self.texture_map.delete_texture(texture)
    }

    /// Loads a material texture in the internal register and returns wether or not the loading was successful. Shininess textures are not supported.
    pub fn load_material_texture(&mut self, path: impl AsRef<Path>) -> bool {
        self.texture_map.add_material_texture(path)
    }

    /// Deletes a stored material texture and returns wether or not the texture was present.
    pub fn delete_material_texture(&mut self, name: &Rc<str>) -> bool {
        self.texture_map.delete_material_texture(name)
    }

    /// Loads the texture data for a sprite and makes
    pub fn load_sprite(&mut self, path: &Rc<Path>) -> bool {
        self.texture_map.add_sprite(path)
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

    /// Checks the current state of the entities that exist and deletes all the assets that are not currently used by any of them.
    pub fn delete_unused_assets(&mut self) {
        todo!();
    }

    /// Deletes all the asset data that was loaded from the given file path.
    pub fn delete_data_from_file_origin(&mut self, path: impl AsRef<Path>) {
        todo!();
    }

    /// Computes the rigid body physics data from component data and stores it for physics sim. When you update component data that influences this, you can call this function to refresh the state. Relevant components are ``RigidBody``, ``Scale`` and ``Renderable``. When creating a new entity or adding/removing a relevant component, this will be called automatically if necessary.
    pub fn recompute_rigid_body_data(&mut self, entity: EntityID) {
        if unsafe { &*self.ecs.get() }.has_component::<Renderable>(entity)
            && unsafe { &*self.ecs.get() }.has_component::<RigidBody>(entity)
        {
            let handle = unsafe { &*self.ecs.get() }
                .get_component::<Renderable>(entity)
                .unwrap()
                .mesh_type
                .mesh_handle();

            let opt_mesh = self.mesh_from_handle(handle, LOD::None);
            if opt_mesh.is_none() {
                log::error!("No mesh data present to use for computing rigid body data.");
                return;
            }
            let mesh = opt_mesh.unwrap();

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

    /// Access to the name of a mesh with a handle.
    pub fn mesh_name_from_handle(&self, handle: MeshHandle) -> Option<&str> {
        if let Some(mesh) = self.mesh_register.get(&handle) {
            Some(mesh.name.as_str())
        } else {
            log::warn!("Mesh data not present.");
            None
        }
    }

    /// Access to the source file path of a loaded mesh with a handle.
    pub fn mesh_source_file_from_handle(&self, handle: MeshHandle) -> Option<&Path> {
        if let Some(mesh) = self.mesh_register.get(&handle) {
            Some(mesh.source_file.as_ref())
        } else {
            log::warn!("Mesh data not present.");
            None
        }
    }

    /// Access to the optional name of the native material of a loaded mesh with a handle.
    pub fn mesh_native_material_name_from_handle(&self, handle: MeshHandle) -> Option<&str> {
        if let Some(mesh) = self.mesh_register.get(&handle) {
            Some(mesh.material_name.as_ref()?.as_str())
        } else {
            log::warn!("Mesh data not present.");
            None
        }
    }

    /// Makes mesh data available for a given ``MeshHandle`` and ``LOD`` if it is stored.
    pub(crate) fn mesh_from_handle(&self, handle: MeshHandle, lod: LOD) -> Option<&Mesh> {
        match lod {
            LOD::None => self.mesh_register.get(&handle),
            _ => Some(
                self.lod_register
                    .get(&handle)?
                    .get(lod as usize - 1)
                    .unwrap(),
            ),
        }
    }

    /// makes material data available for a given name
    pub(crate) fn material_from_name(&self, name: impl AsRef<str>) -> Option<&Material> {
        self.material_register.get(name.as_ref())
    }

    /// makes hitbox data available for given entity data
    #[rustfmt::skip]
    pub(crate) fn hitbox_from_data(&self, hitbox: HitboxType, opt_handle: Option<MeshHandle>) -> Option<&Hitbox> {
        self.hitbox_register.get(&(hitbox, opt_handle))
    }

    /// Clears all of the stored entites and their associated data and invalidates all of the IDs and Handles yielded from the system up to this point.
    pub fn clear(&mut self) {
        self.ecs.get_mut().clear();
        self.mesh_register.clear();
        self.lod_register.clear();
        self.texture_map.clear();
        self.texture_map.clear();
        self.hitbox_register.clear();
        log::debug!("Cleared the entity manager.");
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
    pub(crate) fn create_entity(&mut self, components: Vec<MetaDataComponentEntry>) -> EntityID {
        let new_entity = self.next_entity;
        self.next_entity += 1;

        let entity_type = EntityType::from(components.iter().map(|entry| entry.meta_data));
        let archetype_id = self.get_arch_id(&entity_type);

        let archetype = self.archetypes.get_mut(&archetype_id).unwrap();
        let row = archetype
            .components
            .values()
            .nth(0)
            .unwrap()
            .component_count();

        for component in components {
            archetype
                .components
                .get_mut(&component.meta_data.type_id)
                .unwrap()
                .push_component_entry(component.entry);
        }

        self.entity_index
            .insert(new_entity, EntityRecord { archetype_id, row });

        new_entity
    }

    /// Deletes a stored entity and all the associated component data. Returns wether or not the removal was successful.
    pub(crate) fn delete_entity(&mut self, entity: EntityID) -> bool {
        if let Some(record) = self.entity_index.remove(&entity) {
            let archetype = self.archetypes.get_mut(&record.archetype_id).unwrap();
            for column in archetype.components.values_mut() {
                column.swap_delete_nth_component(record.row);
            }
            if !archetype.components.values().nth(0).unwrap().is_empty() {
                self.edit_record_after_delete(record.archetype_id, record.row);
            }
            true
        } else {
            log::warn!("EntityID {entity:?} not found.");
            false
        }
    }

    /// yields the component data reference of an entity if present (also returns ``None`` if the entity ID is invalid)
    pub(crate) fn get_component<T: Component>(&self, entity: EntityID) -> Option<&T> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get(&record.archetype_id).unwrap();
        let storage = archetype.components.get(&TypeId::of::<T>())?;
        let component = storage.get_nth_component(record.row);
        Some(component)
    }

    /// yields the mutable component data reference of an entity if present (also returns ``None`` if the entity ID is invalid)
    pub(crate) fn get_component_mut<T: Component>(&mut self, entity: EntityID) -> Option<&mut T> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get_mut(&record.archetype_id).unwrap();
        let storage = archetype.components.get_mut(&TypeId::of::<T>())?;
        let component = storage.get_nth_component_mut(record.row);
        Some(component)
    }

    /// gets the vector of all associated component TypeId's (returns ``None`` if the entity ID is invalid)
    pub(crate) fn get_entity_type(&self, entity: EntityID) -> Option<EntityType> {
        let record = self.entity_index.get(&entity)?;
        let archetype = self.archetypes.get(&record.archetype_id).unwrap();
        Some(EntityType::from(
            archetype
                .components
                .values()
                .map(|storage| storage.meta_data),
        ))
    }

    /// Adds a component to an existing entity and returns ``false`` if the component was already present.
    pub(crate) fn add_component<T: Component>(&mut self, entity: EntityID, component: T) -> bool {
        if self.has_component::<T>(entity) {
            let component_name = type_name::<T>();
            log::warn!("The entity {entity:?} already has a component of type {component_name:?}.");
            return false;
        }
        let entity_type = self.get_entity_type(entity);
        if entity_type.is_none() {
            log::warn!("EntityID not found.");
            return false;
        }
        let mut entity_type = entity_type.unwrap();
        let record = self.entity_index.get(&entity).unwrap();
        let old_archetype = self.archetypes.get_mut(&record.archetype_id).unwrap();
        let old_arch_id = old_archetype.id;

        // remove the entity's components from the old archetype
        let old_components: Vec<ComponentEntry> = old_archetype
            .components
            .values_mut()
            .map(|storage| storage.swap_remove_nth_component_entry(record.row))
            .collect();

        if !old_archetype.components.values().nth(0).unwrap().is_empty() {
            self.edit_record_after_delete(old_arch_id, record.row);
        }

        // find or create the new archetype
        entity_type.add_component::<T>();
        let new_archetype_id = self.get_arch_id(&entity_type);

        let new_archetype = self.archetypes.get_mut(&new_archetype_id).unwrap();
        let new_row = new_archetype
            .components
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .component_count();

        // add all components to new archetype
        for old_component in old_components {
            new_archetype
                .components
                .get_mut(&old_component.type_id)
                .unwrap()
                .push_component_entry(old_component);
        }
        new_archetype
            .components
            .get_mut(&component.type_id())
            .unwrap()
            .push_component(component);

        // Update the entity record
        let record = self.entity_index.get_mut(&entity).unwrap();
        record.archetype_id = new_archetype_id;
        record.row = new_row;

        true
    }

    /// checks wether or not an entity has a component of given type associated with it (also returns ``false`` if the entity ID is invalid)
    pub(crate) fn has_component<T: Component>(&self, entity: EntityID) -> bool {
        if let Some(record) = self.entity_index.get(&entity) {
            let archetype = self.archetypes.get(&record.archetype_id).unwrap();
            return archetype.components.contains_key(&TypeId::of::<T>());
        }
        log::warn!("No entity found with ID: {entity:?}.");
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

        // Remove the entity's components from the old archetype and save the component
        let num_old_components = old_archetype
            .components
            .values()
            .nth(0)
            .unwrap()
            .component_count();

        let mut tmp_components = Vec::with_capacity(num_old_components - 1);
        let mut component: Option<T> = None;

        for storage in old_archetype.components.values_mut() {
            if storage.meta_data.type_id == TypeId::of::<T>() {
                // this is safe because we check for the type ids
                component = Some(storage.swap_remove_nth_component::<T>(record.row));
            } else {
                tmp_components.push(storage.swap_remove_nth_component_entry(record.row));
            }
        }

        if !old_archetype.components.values().nth(0).unwrap().is_empty() {
            self.edit_record_after_delete(old_arch_id, record.row);
        }

        if tmp_components.is_empty() {
            self.entity_index.remove(&entity).unwrap();
            return component;
        }

        // Find or create the new archetype
        entity_type.rm_component::<T>();
        let new_archetype_id = self.get_arch_id(&entity_type);

        let new_archetype = self.archetypes.get_mut(&new_archetype_id).unwrap();
        let new_row = new_archetype
            .components
            .values()
            .nth(0)
            .unwrap()
            .component_count();

        // add the old components to the new archetype
        for component_entry in tmp_components {
            new_archetype
                .components
                .get_mut(&component_entry.type_id)
                .unwrap()
                .push_component_entry(component_entry);
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
        reset_global_arena(&ENTITY_ARENA);
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
                            .map(|meta_data| {
                                (
                                    meta_data.type_id,
                                    ComponentStorage::from_meta_data(*meta_data),
                                )
                            })
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
            .component_count();

        let record = self
            .entity_index
            .values_mut()
            .find(|record| record.archetype_id == archetype_id && record.row == last_index)
            .unwrap();

        record.row = changed_index;
    }
}
