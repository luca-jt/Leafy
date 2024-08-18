use crate::ecs::component::{MotionState, Position, TouchTime};
use crate::ecs::entity_manager::EntityManager;
use crate::ecs::query::{exclude_filter, include_filter, ExcludeFilter, IncludeFilter};
use crate::utils::constants::G;

pub struct AnimationSystem {
    pub animation_speed: f32,
}

impl AnimationSystem {
    /// creates a new animation system
    pub fn new() -> Self {
        Self {
            animation_speed: 1.0,
        }
    }

    /// applys all of the physics to all of the entities
    pub fn apply_physics(&self, entity_manager: &mut EntityManager) {
        // apply physics
        // TODO(luca): collision checking
        // TODO(luca): friction
        for (p, m, t) in entity_manager
            .ecs
            .query3_mut::<Position, MotionState, TouchTime>(include_filter!(), exclude_filter!())
        {
            let dt = t.delta_time_f32() * self.animation_speed;

            p.add(0.5 * m.acceleration.data() * dt.powi(2) + m.velocity.data() * dt);
            m.velocity.add(m.acceleration.data() * dt);
            m.acceleration = G; // ?

            t.reset();
        }
    }
}

pub struct SampledAnimation {}
