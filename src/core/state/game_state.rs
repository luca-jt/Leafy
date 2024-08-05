use crate::ecs::component::{
    Acceleration, Color32, MeshAttribute, MeshType, MotionState, Position, Renderable, TouchTime,
    Velocity,
};
use crate::ecs::entity::EntityID;
use crate::ecs::entity_manager::{components, EntityManager};
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

        let _floor = entity_manager.create_entity(components!(
            Position::zeros(),
            Renderable {
                scale: 5f32.into(),
                mesh_type: MeshType::Plane,
                mesh_attribute: MeshAttribute::Colored(Color32::GREEN),
            }
        ));

        let player = entity_manager.create_entity(components!(
            Position::new(0.0, 2.0, 0.0),
            Renderable {
                scale: 1f32.into(),
                mesh_type: MeshType::Sphere,
                mesh_attribute: MeshAttribute::Colored(Color32::RED),
            },
            MotionState {
                velocity: Velocity::zeros(),
                acceleration: Acceleration::zeros()
            },
            TouchTime::now()
        ));

        Self {
            entity_manager,
            moving_entities,
            player,
        }
    }

    /*pub fn fix_entity(&mut self, entity_id: EntityID) {
        let entity = self.entity_manager.get_entity_mut(entity_id);
        entity.motion_state = MotionState::Fixed;
        self.moving_entities.remove(&entity_id);
    }

    pub fn unfix_entity(&mut self, entity_id: EntityID) {
        let entity = self.entity_manager.get_entity_mut(entity_id);
        if let MotionState::Fixed = entity.motion_state {
            entity.motion_state = MotionState::default();
            self.moving_entities.insert(entity_id);
        }
    }*/
}

impl EventObserver for GameState {
    fn on_event(&mut self, event: &FLEventData) {
        if let FLEventData::KeyPress(key) = event {
            if *key == Keycode::SPACE {
                let v_ref = &mut self
                    .entity_manager
                    .ecs
                    .get_component_mut::<MotionState>(self.player)
                    .unwrap()
                    .velocity;
                *v_ref = Velocity::new(0.0, 3.0, 0.0);
            }
        }
    }
}
