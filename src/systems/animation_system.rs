use super::event_system::{PhysicsEvent, SharedEventQueue};
use crate::ecs::component::MotionState;
use crate::state::game_state::GameState;
use crate::utils::constants::G;
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
                    match &mut entity.motion_state {
                        MotionState::Moving(velocity, _) => {
                            *velocity = v;
                        }
                        MotionState::Fixed => {}
                    }
                }
                ChangeAcceleration { e_id, a } => {
                    let entity = game_state.entity_manager.get_entity_mut(e_id);
                    match &mut entity.motion_state {
                        MotionState::Moving(_, acceleration) => {
                            *acceleration = a;
                        }
                        MotionState::Fixed => {}
                    }
                } // TODO(luca): maybe change motion state on events
            }
        }
        // apply physics
        // TODO(luca): collision checking
        // TODO(luca): friction
        for id in game_state.moving_entities.iter() {
            let entity_ref = game_state.entity_manager.get_entity_mut(*id);

            let dt = entity_ref.elapsed_time_f32();
            let a = entity_ref.acceleration();
            let v = entity_ref.velocity();

            entity_ref.position += 0.5 * a * dt.powi(2) + v * dt;
            entity_ref.set_velocity(v + a * dt);
            entity_ref.set_acceleration(G); // ?

            entity_ref.reset_time();
        }
        //...
    }
}

pub struct SampledAnimation {}
