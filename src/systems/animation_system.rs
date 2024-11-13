use crate::ecs::component::*;
use crate::ecs::entity_manager::EntityManager;
use crate::engine::{Engine, EngineMode, FallingLeafApp};
use crate::glm;
use crate::systems::event_system::events::*;
use crate::systems::event_system::EventObserver;
use crate::utils::constants::{G, Y_AXIS};
use std::ops::DerefMut;
use winit::keyboard::KeyCode;

pub struct AnimationSystem {
    current_mode: EngineMode,
    animation_speed: f32,
    gravity: Acceleration,
    time_step_size: TimeDuration,
    time_accumulated: TimeDuration,
    pub(crate) time_of_last_sim: TouchTime,
    pub(crate) flying_cam_dir: Option<(glm::Vec3, f32)>,
    pub(crate) flying_cam_keys: MovementKeys,
}

impl AnimationSystem {
    /// creates a new animation system
    pub(crate) fn new() -> Self {
        Self {
            current_mode: EngineMode::Running,
            animation_speed: 1.0,
            gravity: G,
            time_step_size: TimeDuration(0.001),
            time_accumulated: TimeDuration(0.0),
            time_of_last_sim: TouchTime::now(),
            flying_cam_dir: None,
            flying_cam_keys: MovementKeys {
                up: KeyCode::Space,
                down: KeyCode::ShiftLeft,
                forward: KeyCode::KeyW,
                backward: KeyCode::KeyS,
                left: KeyCode::KeyA,
                right: KeyCode::KeyD,
            },
        }
    }

    /// applys all of the physics to all of the entities
    pub(crate) fn update<T: FallingLeafApp>(&mut self, engine: &Engine<T>) {
        let dt = self.time_of_last_sim.delta_time() * self.animation_speed;
        self.time_accumulated += dt;
        while self.time_accumulated >= self.time_step_size {
            self.simulate_timestep(engine);
            self.time_accumulated -= self.time_step_size;
        }
        self.time_of_last_sim.reset();
    }

    /// simulate one full time step in the system
    fn simulate_timestep<T: FallingLeafApp>(&mut self, engine: &Engine<T>) {
        if self.current_mode == EngineMode::Running {
            self.apply_physics(engine.entity_manager_mut().deref_mut(), self.time_step_size);
            self.handle_collisions(engine.entity_manager_mut().deref_mut(), self.time_step_size);
        }
        self.update_cam(engine, self.time_step_size);
    }

    /// checks for collision between entities with hitboxes and resolves them
    fn handle_collisions(&self, entity_manager: &mut EntityManager, time_step: TimeDuration) {
        let objects = entity_manager
            .query7_mut_opt6::<Position, Velocity, AngularVelocity, MeshType, Scale, RigidBody, HitboxType>(vec![])
            .collect::<Vec<_>>();
        // two collision cases: two edges touching or one vertex anywhere on a side
        // first do macro level checks where you construct a axis alligned box around the mesh that is as big as the max reach of the mesh in each direction
        // match these into groups where hits are able to occur and then construct the detailed colliders for them
        // repeat the collision detection and resolution until the entire group is resolved
        // then the collision detection algorithms depend on the hitbox type:
        // if the hitboxes are convex, use GJK for detection and EPA for penetration depth calculation, ellipsiods and box colliders are trivial, and the rest is triangle intersection tests wich are expensive
        // calculate the minimum translation vector to seperate the two colliders and calculate the collision normal
        // seperate the colliders and create an impulse to move the underlying objects
    }

    /// performs all relevant physics calculations on entity data
    fn apply_physics(&self, entity_manager: &mut EntityManager, time_step: TimeDuration) {
        for (p, v, a_opt, rb_opt, o_opt, av_opt) in entity_manager
            .query6_mut_opt4::<Position, Velocity, Acceleration, RigidBody, Orientation, AngularVelocity>(vec![])
        {
            let total_a = rb_opt.is_some().then_some(self.gravity).unwrap_or_default() + a_opt.copied().unwrap_or_default();
            *v += total_a * time_step;
            *p += *v * time_step;

            if let (Some(av), Some(o)) = (av_opt, o_opt) {
                let inertia_mat = rb_opt.copied().unwrap_or_default().inertia_tensor;
                let corr_av = inertia_mat.try_inverse().unwrap() * av.0;
                o.0 += 0.5 * o.0 * glm::quat(corr_av.x, corr_av.y, corr_av.z, 0.0) * time_step.0;
                o.0.normalize_mut();
            }
        }
    }

    /// updates the camera position based on the current movement key induced camera movement
    fn update_cam<T: FallingLeafApp>(&mut self, engine: &Engine<T>, time_step: TimeDuration) {
        if let Some(cam_move_config) = self.flying_cam_dir {
            if cam_move_config.0 != glm::Vec3::zeros() {
                let cam_config = engine.rendering_system().current_cam_config();
                let move_vector = cam_move_config.0.normalize();
                let changed = move_vector * time_step.0 * cam_move_config.1;

                let mut look_z = cam_config.1;
                look_z.y = 0.0;
                look_z.normalize_mut();
                let look_x = look_z.cross(&Y_AXIS).normalize();
                let look_space_matrix = glm::Mat3::from_columns(&[look_x, Y_AXIS, look_z]);

                engine.trigger_event(CamPositionChange {
                    new_pos: cam_config.0 + look_space_matrix * changed,
                    new_look: cam_config.1,
                });
            }
        }
    }

    /// enables/disables the built-in flying cam movement with a movement speed
    /// to change the movement keys use ``define_movement_keys()``
    pub fn set_flying_cam_movement(&mut self, speed: Option<f32>) {
        log::debug!("set flying cam movement: {:?}", speed);
        match speed {
            None => {
                self.flying_cam_dir = None;
            }
            Some(s) => {
                self.flying_cam_dir = Some((glm::Vec3::zeros(), s));
            }
        }
    }

    /// changes the movement keys used for the built-in flying camera movement
    /// defaults: up - Space, down - LeftShift, directions - WASD
    pub fn define_movement_keys(&mut self, keys: MovementKeys) {
        self.flying_cam_keys = keys;
    }

    /// changes the gravity value used for physics computations (default is ``constants::G``)
    pub fn set_gravity(&mut self, a: Acceleration) {
        log::debug!("set gravity: {:?}", a);
        self.gravity = a;
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

/// defines a set of movement keys for camera control
#[derive(Debug, Copy, Clone)]
pub struct MovementKeys {
    pub up: KeyCode,
    pub down: KeyCode,
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
}

/// starts moving the camera in the direction the key was pressed for
pub(crate) fn move_cam<T: FallingLeafApp>(event: &KeyPress, engine: &Engine<T>) {
    if event.is_repeat {
        return;
    }
    let keys = engine.animation_system().flying_cam_keys;
    if let Some((cam_move_direction, _)) = engine.animation_system_mut().flying_cam_dir.as_mut() {
        if event.key == keys.down {
            cam_move_direction.y -= 1.0;
        }
        if event.key == keys.up {
            cam_move_direction.y += 1.0;
        }
        if event.key == keys.forward {
            cam_move_direction.z += 1.0;
        }
        if event.key == keys.left {
            cam_move_direction.x -= 1.0;
        }
        if event.key == keys.backward {
            cam_move_direction.z -= 1.0;
        }
        if event.key == keys.right {
            cam_move_direction.x += 1.0;
        }
    }
}

/// stops the cam form moving in the direction the key was released for
pub(crate) fn stop_cam<T: FallingLeafApp>(event: &KeyRelease, engine: &Engine<T>) {
    if event.is_repeat {
        return;
    }
    let keys = engine.animation_system().flying_cam_keys;
    if let Some((cam_move_direction, _)) = engine.animation_system_mut().flying_cam_dir.as_mut() {
        if event.key == keys.down {
            cam_move_direction.y += 1.0;
        }
        if event.key == keys.up {
            cam_move_direction.y -= 1.0;
        }
        if event.key == keys.forward {
            cam_move_direction.z -= 1.0;
        }
        if event.key == keys.left {
            cam_move_direction.x += 1.0;
        }
        if event.key == keys.backward {
            cam_move_direction.z += 1.0;
        }
        if event.key == keys.right {
            cam_move_direction.x -= 1.0;
        }
    }
}

pub struct SampledAnimation {}
