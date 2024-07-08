use component::Component;
use entity::EntityID;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

pub mod component;
pub mod entity;
pub mod entity_manager;

pub struct TestComponent(i32);

impl Component for TestComponent {}

pub struct ECS {
    next_entity: EntityID,
    pub components: HashMap<TypeId, HashMap<EntityID, Box<dyn Any>>>,
    entities: HashSet<EntityID>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            components: HashMap::new(),
            entities: HashSet::new(),
        }
    }

    pub fn create_entity(&mut self) -> EntityID {
        let entity = self.next_entity;
        self.entities.insert(entity);
        self.next_entity += 1;
        entity
    }

    pub fn add_component<T: Component>(&mut self, entity: EntityID, component: T) {
        let type_id = TypeId::of::<T>();
        self.components
            .entry(type_id)
            .or_insert_with(HashMap::new)
            .insert(entity, Box::new(component));
    }

    pub fn get_component<T: Component>(&self, entity: EntityID) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)?
            .get(&entity)?
            .downcast_ref::<T>()
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: EntityID) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)?
            .get_mut(&entity)?
            .downcast_mut::<T>()
    }
}

macro_rules! spawn_entity {
    ($ecs:expr, $( $component:expr ),* ) => {
        let entity = $ecs.create_entity();
        $(
            $ecs.add_component(entity, $component);
        )*
        entity
    };
}

pub type Query<'c> = Vec<(EntityID, Vec<&'c mut dyn Component>)>;

macro_rules! query {
    ($ecs:expr, $( $component:ident ), *) => {
        let mut filters: Vec<TypeId> = Vec::new();
        $(
            filters.push(TypeId::of::<$component>());
        )*
        let possible_keys: HashSet<&EntityID> = filters
            .iter()
            .map(|filter| HashSet::from_iter($ecs.components.get(filter)?.keys()))
            .reduce(|a, b| a.intersection(&b).collect())?;
        if possible_keys.is_empty() {
            return None;
        };
        let mut query = Query::new()
        for key in possible_keys {
            let components: Vec<&'c mut dyn Component> = Vec::new();
            $(
                components.push($ecs.get_component_mut::<$component>(*key));
            )*
            query.push(*key, components);
        }
        Some(query)
    };
}

/*
fn print_position(query: Query<TestComponent, >) {
    for (entity, c) in query.iter() {
        println!("Entity {:?} is {:?}", entity, c);
    }
}

struct Schedule<T> {
    systems: Vec<Box<dyn Fn(&Query<T>)>>,
}

impl<T> Schedule<T> {
    fn default() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    fn add_system<F>(&mut self, system: F)
    where
        F: Fn(Query<T>) + 'static,
    {
        self.systems.push(Box::new(system));
    }

    fn run(&self, ecs: &mut ECS) {
        for system in &self.systems {
            system(ecs);
        }
    }
}
*/
