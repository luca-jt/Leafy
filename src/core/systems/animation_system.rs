use crate::ecs::component::{MotionState, Position, TouchTime};
use crate::ecs::entity_manager::EntityManager;
use crate::systems::event_system::events::AnimationSpeedChange;
use crate::systems::event_system::EventObserver;
use crate::utils::constants::G;

pub struct AnimationSystem {
    animation_speed: f32,
}

impl AnimationSystem {
    /// creates a new animation system
    pub(crate) fn new() -> Self {
        Self {
            animation_speed: 1.0,
        }
    }

    /// applys all of the physics to all of the entities
    pub(crate) fn apply_physics(&self, entity_manager: &mut EntityManager) {
        // apply physics
        for (p, m, t) in entity_manager.query3_mut::<Position, MotionState, TouchTime>() {
            let dt = t.delta_time_f32() * self.animation_speed;

            p.add(0.5 * m.acceleration.data() * dt.powi(2) + m.velocity.data() * dt);
            m.velocity.add(m.acceleration.data() * dt);
            m.acceleration = G; // ?

            t.reset();
        }
    }
}

impl EventObserver<AnimationSpeedChange> for AnimationSystem {
    fn on_event(&mut self, event: &AnimationSpeedChange) {
        self.animation_speed = event.new_animation_speed;
    }
}

pub struct SampledAnimation {}
