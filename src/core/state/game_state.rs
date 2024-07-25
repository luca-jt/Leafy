use crate::ecs::component::{
    Color32, MeshAttribute, MeshType, MotionState, Position, Scale, Velocity,
};
use crate::ecs::entity::{Entity, EntityID};
use crate::ecs::entity_manager::EntityManager;
use crate::systems::event_system::{EventObserver, FLEventData};
use sdl2::keyboard::Keycode;
use std::collections::HashSet;

/// state for the game logic
pub struct GameState {
    pub entity_manager: EntityManager,
    pub moving_entities: HashSet<EntityID>,
    pub player: EntityID,
    // TODO: scene files (initialize the right renderers)?
}

impl GameState {
    /// creates a new game state
    pub fn new() -> Self {
        let mut entity_manager = EntityManager::new();
        let moving_entities: HashSet<EntityID> = HashSet::new();

        let mut floor = Entity::new_fixed(
            MeshType::Plane,
            MeshAttribute::Colored(Color32::GREEN),
            Position::zeros(),
        );
        floor.scale = Scale::from(5.0);
        let _ = entity_manager.add_entity(floor);

        let test_entity = Entity::new_moving(
            MeshType::Sphere,
            MeshAttribute::Colored(Color32::RED),
            Position::new(0.0, 2.0, 0.0),
        );
        let player = entity_manager.add_entity(test_entity);

        Self {
            entity_manager,
            moving_entities,
            player,
        }
    }

    /// turns the entity into a fixated one if not already
    pub fn fix_entity(&mut self, entity_id: EntityID) {
        let entity = self.entity_manager.get_entity_mut(entity_id);
        entity.motion_state = MotionState::Fixed;
        self.moving_entities.remove(&entity_id);
    }

    /// turns the entity into a moving one if not already
    pub fn unfix_entity(&mut self, entity_id: EntityID) {
        let entity = self.entity_manager.get_entity_mut(entity_id);
        if let MotionState::Fixed = entity.motion_state {
            entity.motion_state = MotionState::default();
            self.moving_entities.insert(entity_id);
        }
    }
}

impl EventObserver for GameState {
    fn on_event(&mut self, event: &FLEventData) {
        if let FLEventData::KeyPress(key) = event {
            if *key == Keycode::SPACE {
                let entity = self.entity_manager.get_entity_mut(self.player);
                entity.set_velocity(Velocity::new(0.0, 3.0, 0.0));
            }
        }
    }
}
