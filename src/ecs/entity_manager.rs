use crate::ecs::component::*;
use crate::ecs::entity::*;
use crate::rendering::data::TextureMap;
use crate::rendering::mesh::{Hitbox, HitboxType, Mesh};
use crate::utils::constants::ORIGIN;
use crate::utils::file::*;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::{HashMap, VecDeque};

/// create a component list for entity creation
#[macro_export]
macro_rules! components {
    ($($T:expr),+) => {
        vec![$(Box::new($T)),+]
    };
}

/// the main ressource manager holding both the ECS and the asset data
pub struct EntityManager {
    pub(crate) ecs: UnsafeCell<ECS>,
    asset_register: HashMap<MeshType, Mesh>,
    pub(crate) texture_map: TextureMap,
    hitbox_register: HashMap<(MeshType, HitboxType), Hitbox>,
    commands: VecDeque<AssetCommand>,
}

impl EntityManager {
    /// creates a new entitiy manager
    pub fn new() -> Self {
        Self {
            ecs: UnsafeCell::new(ECS::new()),
            asset_register: HashMap::new(),
            texture_map: TextureMap::new(),
            hitbox_register: HashMap::new(),
            commands: VecDeque::new(),
        }
    }

    /// stores a new entity and returns the id of the new entity
    pub fn create_entity(&mut self, components: Vec<Box<dyn Any>>) -> EntityID {
        let entity = self.ecs.get_mut().create_entity(components);
        // load mesh if necessary
        self.add_command(AssetCommand::AddMesh(entity));
        // load texture if necessary
        self.add_command(AssetCommand::AddTexture(entity));
        // compute hitbox if necessary
        self.add_command(AssetCommand::AddHitbox(entity));
        // do rigid body calculations if necessary
        self.add_command(AssetCommand::UpdateRigidBody(entity));
        // add the light id if necessary
        self.add_command(AssetCommand::AddLightID(entity));
        self.exec_commands();
        entity
    }

    /// creates a new entity with all basic data needed for physics and rendering
    pub fn create_basic_dynamic(
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
            Orientation::default(),
            AngularVelocity::zero(),
            Acceleration::zero(),
            Collider {
                hitbox_type: HitboxType::ConvexHull,
                offset: ORIGIN,
                scale: Scale::default()
            },
            RigidBody::default(),
            EntityFlags::default()
        ))
    }

    /// creates a new default point light source for the rendering system without other components attached (invisible)
    pub fn create_point_light(&mut self, position: Position) -> EntityID {
        let light = self.create_entity(components!(position, PointLight::default()));
        light
    }

    /// creates a new default point light source for the rendering system with Scale attached (visible)
    pub fn create_point_light_visible(&mut self, position: Position) -> EntityID {
        let light = self.create_entity(components!(
            position,
            PointLight::default(),
            MeshType::Cube,
            MeshAttribute::Colored(Color32::from_rgb(255, 255, 200)),
            Scale::from_factor(0.1)
        ));
        light
    }

    /// deletes an entity from the register by id and returns the removed entity
    pub fn delete_entity(&mut self, entity: EntityID) -> Result<(), ()> {
        if unsafe { &*self.ecs.get() }.has_component::<MeshType>(entity) {
            self.add_command(AssetCommand::CleanMeshes);
            self.add_command(AssetCommand::CleanHitboxes);
            self.add_command(AssetCommand::CleanTextures);
        }
        self.ecs.get_mut().delete_entity(entity)?;
        self.exec_commands();
        Ok(())
    }

    /// yields the component data reference of an entity if present
    pub fn get_component<T: Any>(&self, entity: EntityID) -> Option<&T> {
        unsafe { &*self.ecs.get() }.get_component::<T>(entity)
    }

    /// yields the mutable component data reference of an entity if present
    pub fn get_component_mut<T: Any>(&mut self, entity: EntityID) -> Option<&mut T> {
        if TypeId::of::<T>() == TypeId::of::<MeshType>() {
            self.add_command(AssetCommand::UpdateRigidBody(entity));
            self.add_command(AssetCommand::AddMesh(entity));
            self.add_command(AssetCommand::CleanMeshes);
            self.add_command(AssetCommand::AddHitbox(entity));
            self.add_command(AssetCommand::CleanHitboxes);
        } else if TypeId::of::<T>() == TypeId::of::<Scale>()
            || TypeId::of::<T>() == TypeId::of::<RigidBody>()
        {
            self.add_command(AssetCommand::UpdateRigidBody(entity));
        } else if TypeId::of::<T>() == TypeId::of::<Collider>() {
            self.add_command(AssetCommand::AddHitbox(entity));
            self.add_command(AssetCommand::CleanHitboxes);
        } else if TypeId::of::<T>() == TypeId::of::<MeshAttribute>() {
            self.add_command(AssetCommand::AddTexture(entity));
            self.add_command(AssetCommand::CleanTextures);
        }
        self.exec_commands();
        self.ecs.get_mut().get_component_mut::<T>(entity)
    }

    /// adds a component to an existing entity (returns Err(()) if the component is already present)
    pub fn add_component<T: Any>(&mut self, entity: EntityID, component: T) -> Result<(), ()> {
        self.ecs.get_mut().add_component::<T>(entity, component)?;
        // add mesh to the register if necessary
        if TypeId::of::<T>() == TypeId::of::<MeshType>() {
            self.add_command(AssetCommand::AddMesh(entity));
        }
        // add the texture if necessary
        self.add_command(AssetCommand::AddTexture(entity));
        // add hitbox to the register if necessary
        self.add_command(AssetCommand::AddHitbox(entity));
        // add the light source id component if necessary
        if TypeId::of::<T>() == TypeId::of::<PointLight>() {
            self.add_command(AssetCommand::AddLightID(entity));
        }
        // recalculate the inertia tensor if necessary
        if TypeId::of::<T>() == TypeId::of::<Scale>()
            || TypeId::of::<T>() == TypeId::of::<RigidBody>()
        {
            self.add_command(AssetCommand::UpdateRigidBody(entity));
        }
        self.exec_commands();
        Ok(())
    }

    /// checks wether or not an entity has a component of given type associated with it
    pub fn has_component<T: Any>(&self, entity: EntityID) -> bool {
        unsafe { &*self.ecs.get() }.has_component::<T>(entity)
    }

    /// removes a component from an entity and returns the component data if present
    pub fn remove_component<T: Any>(&mut self, entity: EntityID) -> Option<T> {
        let removed = self.ecs.get_mut().remove_component::<T>(entity);
        if removed.is_some() {
            if TypeId::of::<T>() == TypeId::of::<MeshType>() {
                self.add_command(AssetCommand::CleanMeshes);
                self.add_command(AssetCommand::CleanHitboxes);
            } else if TypeId::of::<T>() == TypeId::of::<MeshAttribute>() {
                self.add_command(AssetCommand::CleanTextures);
            } else if TypeId::of::<T>() == TypeId::of::<Scale>() {
                self.add_command(AssetCommand::UpdateRigidBody(entity));
            } else if TypeId::of::<T>() == TypeId::of::<Collider>() {
                self.add_command(AssetCommand::CleanHitboxes);
            } else if TypeId::of::<T>() == TypeId::of::<PointLight>() {
                self.add_command(AssetCommand::DeleteLightID(entity));
            }
            self.exec_commands();
        }
        removed
    }

    /// iterate over all of the stored entities
    pub fn all_ids_iter(&self) -> impl Iterator<Item = &EntityID> {
        unsafe { &*self.ecs.get() }.entity_index.keys()
    }

    /// makes mesh data available for a given mesh type
    pub(crate) fn asset_from_type(&self, mesh_type: &MeshType) -> Option<&Mesh> {
        self.asset_register.get(mesh_type)
    }

    /// makes hitbox data available for given entity data
    pub(crate) fn hitbox_from_data(
        &self,
        mesh_type: &MeshType,
        hitbox: &HitboxType,
    ) -> Option<&Hitbox> {
        self.hitbox_register.get(&(mesh_type.clone(), *hitbox))
    }

    /// executes all the commands in the queue and clears it
    fn exec_commands(&mut self) {
        while let Some(command) = self.commands.pop_front() {
            match command {
                AssetCommand::AddMesh(entity) => {
                    if let Some(mesh_type) =
                        unsafe { &*self.ecs.get() }.get_component::<MeshType>(entity)
                    {
                        if !self.asset_register.keys().any(|t| t == mesh_type) {
                            let mesh = match mesh_type {
                                MeshType::Triangle => Mesh::from_bytes(TRIANGLE_MESH),
                                MeshType::Plane => Mesh::from_bytes(PLANE_MESH),
                                MeshType::Cube => Mesh::from_bytes(CUBE_MESH),
                                MeshType::Sphere => Mesh::from_bytes(SPHERE_MESH),
                                MeshType::Cylinder => Mesh::from_bytes(CYLINDER_MESH),
                                MeshType::Cone => Mesh::from_bytes(CONE_MESH),
                                MeshType::Torus => Mesh::from_bytes(TORUS_MESH),
                                MeshType::Custom(path) => Mesh::from_path(path),
                            };
                            self.asset_register.insert(mesh_type.clone(), mesh);
                            log::debug!("inserted mesh in register: '{:?}'", mesh_type);
                        }
                    }
                }
                AssetCommand::CleanMeshes => {
                    self.asset_register.retain(|mesh_type, _| {
                        let delete = unsafe { &*self.ecs.get() }
                            .query1::<MeshType>(vec![])
                            .filter(|&mt| mt == mesh_type)
                            .count()
                            == 0;
                        if delete {
                            log::debug!("deleted mesh from register: '{:?}'", mesh_type);
                        }
                        delete
                    });
                }
                AssetCommand::AddHitbox(entity) => {
                    if let (Some(mesh_type), Some(collider)) = (
                        unsafe { &*self.ecs.get() }.get_component::<MeshType>(entity),
                        unsafe { &*self.ecs.get() }.get_component::<Collider>(entity),
                    ) {
                        self.hitbox_register
                            .entry((mesh_type.clone(), collider.hitbox_type))
                            .or_insert_with(|| {
                                log::debug!(
                                    "inserted hitbox '{:?}' in register for mesh '{:?}'",
                                    collider.hitbox_type,
                                    mesh_type
                                );
                                self.asset_register
                                    .get(mesh_type)
                                    .unwrap()
                                    .generate_hitbox(&collider.hitbox_type)
                            });
                    }
                }
                AssetCommand::CleanHitboxes => {
                    self.hitbox_register.retain(|(mesh_type, box_type), _| {
                        let delete = unsafe { &*self.ecs.get() }
                            .query2::<MeshType, Collider>(vec![])
                            .filter(|(mt, ht)| mesh_type == *mt && *box_type == ht.hitbox_type)
                            .count()
                            == 0;
                        if delete {
                            log::debug!(
                                "deleted hitbox '{:?}' from register for mesh '{:?}'",
                                box_type,
                                mesh_type
                            );
                        }
                        delete
                    });
                }
                AssetCommand::UpdateRigidBody(entity) => {
                    if unsafe { &*self.ecs.get() }.has_component::<MeshType>(entity)
                        && unsafe { &*self.ecs.get() }.has_component::<RigidBody>(entity)
                    {
                        let mt = unsafe { &*self.ecs.get() }
                            .get_component::<MeshType>(entity)
                            .unwrap();
                        let mesh = self.asset_from_type(mt).unwrap();
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
                AssetCommand::AddLightID(entity) => {
                    if unsafe { &*self.ecs.get() }.has_component::<PointLight>(entity) {
                        self.ecs
                            .get_mut()
                            .add_component(entity, LightSrcID(entity))
                            .unwrap();
                    }
                }
                AssetCommand::DeleteLightID(entity) => {
                    self.ecs.get_mut().remove_component::<LightSrcID>(entity);
                }
                AssetCommand::AddTexture(entity) => {
                    if unsafe { &*self.ecs.get() }.has_component::<MeshType>(entity) {
                        if let Some(MeshAttribute::Textured(texture)) =
                            unsafe { &*self.ecs.get() }.get_component::<MeshAttribute>(entity)
                        {
                            if self.texture_map.get_tex_id(texture).is_none() {
                                self.texture_map.add_texture(texture);
                            }
                        }
                    }
                }
                AssetCommand::CleanTextures => {
                    self.texture_map.retain(|texture| {
                        unsafe { &*self.ecs.get() }
                            .query1::<MeshAttribute>(vec![])
                            .filter(|&ma| ma.texture().map(|t| t == texture).unwrap_or(false))
                            .count()
                            == 0
                    });
                }
            }
        }
    }

    /// adds a command to the managers' queue
    fn add_command(&mut self, command: AssetCommand) {
        self.commands.push_back(command);
    }

    /// fully recompute all internal asset data that is influenced by entities' components (performance heavy)
    /// (might be useful when modifying specific component data in queries)
    pub fn full_recompute(&mut self) {
        let ids = self.all_ids_iter().copied().collect::<Vec<_>>();
        for entity in ids {
            self.add_command(AssetCommand::AddMesh(entity));
            self.add_command(AssetCommand::CleanMeshes);
            self.add_command(AssetCommand::UpdateRigidBody(entity));
            self.add_command(AssetCommand::AddHitbox(entity));
            self.add_command(AssetCommand::CleanHitboxes);
            self.add_command(AssetCommand::AddTexture(entity));
            self.add_command(AssetCommand::CleanTextures);
        }
        self.exec_commands();
    }
}

/// allows for additional entity data or asset data to be added
enum AssetCommand {
    AddMesh(EntityID),
    CleanMeshes,
    AddHitbox(EntityID),
    CleanHitboxes,
    UpdateRigidBody(EntityID),
    AddLightID(EntityID),
    DeleteLightID(EntityID),
    AddTexture(EntityID),
    CleanTextures,
}

/// the entity component system that manages all the data associated with an entity
pub(crate) struct ECS {
    next_entity: EntityID,
    next_archetype_id: ArchetypeID,
    entity_index: HashMap<EntityID, EntityRecord>,
    pub(crate) archetypes: HashMap<ArchetypeID, Archetype>,
    type_to_archetype: HashMap<EntityType, ArchetypeID>,
}

impl ECS {
    /// creates a new ecs
    pub(crate) fn new() -> Self {
        Self {
            next_entity: 1,
            next_archetype_id: 1,
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
        if archetype.components.values().nth(0).unwrap().is_empty() {
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
        EntityType::from(archetype.components.keys().copied().collect::<Vec<_>>())
    }

    /// adds a component to an existing entity
    pub(crate) fn add_component<T: Any>(
        &mut self,
        entity: EntityID,
        component: T,
    ) -> Result<(), ()> {
        if self.has_component::<T>(entity) {
            return Err(());
        }
        let mut entity_type = self.get_entity_type(entity);
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
        if old_archetype.components.values().nth(0).unwrap().is_empty() {
            self.archetypes.remove(&record.archetype_id);
            self.type_to_archetype.remove(&entity_type);
        } else {
            self.shift_rows(old_arch_id, record.row);
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
            .push(Box::new(component));

        // Update the entity record
        let record = self.entity_index.get_mut(&entity).unwrap();
        record.archetype_id = new_archetype_id;
        record.row = new_row;
        Ok(())
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
        let mut entity_type = self.get_entity_type(entity);
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
        if old_archetype.components.values().nth(0).unwrap().is_empty() {
            self.archetypes.remove(&record.archetype_id);
            self.type_to_archetype.remove(&entity_type);
        } else {
            self.shift_rows(old_arch_id, record.row);
        }

        // Remove the specific component
        let index_to_remove = old_components
            .iter()
            .position(|c| (**c).type_id() == TypeId::of::<T>())?;
        let component = old_components
            .remove(index_to_remove)
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
