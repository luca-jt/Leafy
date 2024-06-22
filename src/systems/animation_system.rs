use super::event_system::{PhysicsEvent, SharedEventQueue};
use crate::state::game_state::GameState;
use PhysicsEvent::*;

/// system managing the animations
pub struct AnimationSystem {
    event_queue: SharedEventQueue,
}

impl AnimationSystem {
    /// creates a new animation system
    pub fn new(event_queue: SharedEventQueue) -> Self {
        Self { event_queue }
    }

    /// updates the animations
    pub fn update(&mut self, game_state: &mut GameState) {
        // apply queued events
        let events = self.event_queue.drain();
        for event in events {
            match event {
                ChangeVelocity { e_id, v } => {
                    let entity = game_state.entity_manager.get_entity_mut(e_id);
                    entity.velocity = Some(v);
                }
                ChangeAcceleration { e_id, a } => {
                    let entity = game_state.entity_manager.get_entity_mut(e_id);
                    entity.acceleration = Some(a);
                }
            }
        }
        // apply physics
        for id in game_state.entities.iter() {
            let _entity_ref = game_state.entity_manager.get_entity(*id);
            //...
        }
        //...
    }
}

pub struct SampledAnimation {}
