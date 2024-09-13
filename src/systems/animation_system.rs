use crate::ecs::component::{Acceleration, Position, TouchTime, Velocity};
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
    pub(crate) fn update(&self, entity_manager: &mut EntityManager) {
        // apply physics
        for (p, t, v, a_opt) in
            entity_manager.query4_mut_opt1::<Position, TouchTime, Velocity, Acceleration>(vec![])
        {
            let dt = t.delta_time() * self.animation_speed;

            *p += *v * dt;
            if let Some(a) = a_opt {
                *p += *a * dt * dt * 0.5;
                *v += *a * dt;
                *a = G; // ?
            }

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
