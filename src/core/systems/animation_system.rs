use crate::state::game_state::GameState;
use crate::utils::constants::G;

pub struct AnimationSystem {
    animation_speed: f32,
}

impl AnimationSystem {
    /// creates a new animation system
    pub fn new() -> Self {
        Self {
            animation_speed: 1.0,
        }
    }

    /// applys all of the physics to all of the entities
    pub fn apply_physics(&self, game_state: &mut GameState) {
        // apply physics
        // TODO(luca): collision checking
        // TODO(luca): friction
        for id in game_state.moving_entities.iter() {
            let entity_ref = game_state.entity_manager.get_entity_mut(*id);
            debug_assert!(!entity_ref.is_fixed());

            let dt = entity_ref.elapsed_time_f32() * self.animation_speed;
            let a = entity_ref.acceleration();
            let v = entity_ref.velocity();

            entity_ref.position += 0.5 * a * dt.powi(2) + v * dt;
            entity_ref.set_velocity(v + a * dt);
            entity_ref.set_acceleration(G); // ?

            entity_ref.reset_time();
        }
    }
    //...
}

pub struct SampledAnimation {}
