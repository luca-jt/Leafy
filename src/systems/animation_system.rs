use crate::ecs::component::*;
use crate::ecs::entity_manager::EntityManager;
use crate::engine::EngineMode;
use crate::include_filter;
use crate::systems::event_system::events::{AnimationSpeedChange, EngineModeChange};
use crate::systems::event_system::EventObserver;
use crate::utils::constants::G;

pub struct AnimationSystem {
    current_mode: EngineMode,
    animation_speed: f32,
    time_step_size: TimeDuration,
}

impl AnimationSystem {
    /// creates a new animation system
    pub(crate) fn new() -> Self {
        Self {
            current_mode: EngineMode::Running,
            animation_speed: 1.0,
            time_step_size: TimeDuration(0.001),
        }
    }

    /// applys all of the physics to all of the entities
    pub(crate) fn update(&self, entity_manager: &mut EntityManager) {
        self.apply_physics(entity_manager);
        self.handle_collisions(entity_manager);
    }

    /// checks for collision between entities with hitboxes and resolves them
    fn handle_collisions(&self, entity_manager: &mut EntityManager) {
        let objects = entity_manager
            .query4_mut_opt3::<Position, Velocity, AngularVelocity, MeshType>(vec![
                include_filter!(Hitbox),
            ])
            .collect::<Vec<_>>();
        // two collision cases: two edges touching or one vertex anywhere on a side
        // compute center of mass and hitboxes in mesh constructor
        // use steps for animations, not only delta time
        // angular velocity (+ operations) -> use also in apply_physics()
    }

    /// performs all relevant physics calculations on entity data
    fn apply_physics(&self, entity_manager: &mut EntityManager) {
        for (p, t, v, a_opt) in
            entity_manager.query4_mut_opt1::<Position, TouchTime, Velocity, Acceleration>(vec![])
        {
            if self.current_mode == EngineMode::Running {
                let dt = t.delta_time() * self.animation_speed;

                *p += *v * dt;
                if let Some(a) = a_opt {
                    *p += *a * dt * dt * 0.5;
                    *v += *a * dt;
                    *a = G; // ?
                }
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

impl EventObserver<EngineModeChange> for AnimationSystem {
    fn on_event(&mut self, event: &EngineModeChange) {
        self.current_mode = event.new_mode;
    }
}

pub struct SampledAnimation {}
