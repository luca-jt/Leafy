use crate::ecs::component::*;
use crate::ecs::entity_manager::EntityManager;
use crate::engine::{Engine, EngineMode, FallingLeafApp};
use crate::glm;
use crate::rendering::data::calc_model_matrix;
use crate::rendering::mesh::Hitbox;
use crate::systems::event_system::events::*;
use crate::systems::event_system::EventObserver;
use crate::utils::constants::bits::COLLISION;
use crate::utils::constants::{G, Y_AXIS};
use crate::utils::tools::{copied_or_default, to_vec4};
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
            self.handle_collisions(engine.entity_manager_mut().deref_mut());
        }
        self.update_cam(engine, self.time_step_size);
    }

    /// checks for collision between entities with hitboxes and resolves them
    fn handle_collisions(&self, entity_manager: &mut EntityManager) {
        let mut entity_data = unsafe {
            entity_manager
                .query9_mut_opt6::<Position, HitboxType, MeshType, Velocity, AngularVelocity, Scale, RigidBody, EntityFlags, Orientation>(vec![])
                .map(|(p, hb, mt, v, av, s, rb, f, o)| {
                    (
                        p,
                        entity_manager.hitbox_from_data(mt, hb).unwrap(),
                        entity_manager.asset_from_type(mt).unwrap(),
                        v,
                        av,
                        s,
                        rb,
                        f.map(|flags| {
                            flags.set_bit(COLLISION, false);
                            flags
                        }),
                        o
                    )
                })
                .collect::<Vec<_>>()
        };
        if entity_data.len() <= 1 {
            return;
        }
        // repeat collision checks until all of them are resolved
        let mut any_hits = true;
        while any_hits {
            any_hits = false;
            // bounding spheres of the mesh for macro level checks
            let spheres = entity_data
                .iter()
                .map(|tuple| {
                    (
                        *tuple.0.data(),
                        (copied_or_default(&tuple.5).scale_matrix() * to_vec4(tuple.2.max_reach()))
                            .xyz()
                            .norm(),
                    )
                })
                .collect::<Vec<_>>();

            for i in 0..spheres.len() {
                for j in i..spheres.len() {
                    if !spheres_collide(&spheres[i].0, spheres[i].1, &spheres[j].0, spheres[j].1) {
                        continue;
                    }
                    // objects 1 and 2
                    let data_1 = &entity_data[i];
                    let data_2 = &entity_data[j];

                    let rigid_body_1 = copied_or_default(&data_1.6);
                    let rigid_body_2 = copied_or_default(&data_2.6);

                    let collider_1 = Collider {
                        hitbox: data_1.1,
                        model_matrix: calc_model_matrix(
                            data_1.0,
                            data_1.5.as_ref().unwrap_or(&&mut Scale::default()),
                            data_1.8.as_ref().unwrap_or(&&mut Orientation::default()),
                            &rigid_body_1.center_of_mass,
                        ),
                    };
                    let collider_2 = Collider {
                        hitbox: data_2.1,
                        model_matrix: calc_model_matrix(
                            data_2.0,
                            data_2.5.as_ref().unwrap_or(&&mut Scale::default()),
                            data_2.8.as_ref().unwrap_or(&&mut Orientation::default()),
                            &rigid_body_2.center_of_mass,
                        ),
                    };
                    if let Some(collison_data) = collider_1.collides_with(&collider_2) {
                        // set collision flags
                        if let Some(flags) = &mut entity_data[i].7 {
                            flags.set_bit(COLLISION, true);
                        }
                        if let Some(flags) = &mut entity_data[j].7 {
                            flags.set_bit(COLLISION, true);
                        }
                        // resolve the collision
                        let data_1 = &entity_data[i];
                        let data_2 = &entity_data[j];
                        // seperate normal and tangential components of the motion in relation to the collision
                        // INFO: normal vector is 1 -> 2

                        // vectors from the center of mass to the contact point
                        let mass_center_coll_point_1 =
                            collison_data.collision_point - rigid_body_1.center_of_mass;
                        let mass_center_coll_point_2 =
                            collison_data.collision_point - rigid_body_2.center_of_mass;

                        // components = (normal_component, tangential_component)
                        let components_1 = data_1.3.as_ref().and_then(|v| {
                            let normal_component =
                                mass_center_coll_point_1.dot(v.data()) * v.data();
                            Some((normal_component, v.data() - normal_component))
                        });
                        let components_2 = data_2.3.as_ref().and_then(|v| {
                            let normal_component =
                                mass_center_coll_point_2.dot(v.data()) * v.data();
                            Some((normal_component, v.data() - normal_component))
                        });

                        // TODO:
                        // https://de.wikipedia.org/wiki/Sto%C3%9F_(Physik)
                        // tangential für reibung relevant und normal für impulsänderung
                        // beachte rotationsänderungen die durch offsets zwischen normalkomponente und schwerpunkten auftreten können
                        // betrachte dann inelastische collisions und elastische nur wenn es nicht viel mehr aufwand ist (vielleicht mit einer flag oder so)
                        // -> diese unterscheiden sich durch den coefficient of restitution welcher in der berechnung der impulse verwendet wird
                        // (sollte in der fallunterscheidung nicht all zu viel aufwand sein)

                        let restitution_coefficient = 0; // change that in the future with component data

                        let collision_resolved =
                            if let (Some(comp_1), Some(comp_2)) = (components_1, components_2) {
                                // both have velocity and therefore are movable
                                //*data_1.0.data_mut() += 0.5 * collison_data.translation_vec;
                                //*data_2.0.data_mut() += -0.5 * collison_data.translation_vec;
                                true
                            } else if let Some(comp_1) = components_1 {
                                // only 1 is movable
                                //*data_1.0.data_mut() += collison_data.translation_vec;
                                true
                            } else if let Some(comp_2) = components_2 {
                                // only 2 is movable
                                //*data_2.0.data_mut() += -collison_data.translation_vec;
                                true
                            } else {
                                // both are immovable
                                false
                            };
                        if collision_resolved {
                            any_hits = true;
                        }
                    }
                }
            }
        }
    }

    /// performs all relevant physics calculations on entity data
    fn apply_physics(&self, entity_manager: &mut EntityManager, time_step: TimeDuration) {
        for (p, v, a_opt, rb_opt, o_opt, av_opt) in unsafe {
            entity_manager
                .query6_mut_opt4::<Position, Velocity, Acceleration, RigidBody, Orientation, AngularVelocity>(vec![])
        } {
            let total_a = rb_opt.is_some().then_some(self.gravity).unwrap_or_default()
                + a_opt.copied().unwrap_or_default();
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

/// checks if two broad oject areas represented as spheres at two positions collide
fn spheres_collide(pos1: &glm::Vec3, box1: f32, pos2: &glm::Vec3, box2: f32) -> bool {
    (pos1 - pos2).norm() <= box1 + box2
}

/// contains collision plane normal vector and the point of the collision
struct CollisionData {
    collision_normal: glm::Vec3,
    collision_point: glm::Vec3,
}

/// collider that is used in collision checking
struct Collider<'a> {
    hitbox: &'a Hitbox,
    model_matrix: glm::Mat4,
}

impl Collider<'_> {
    /// checks if two hitboxes collide with each other
    pub(crate) fn collides_with(&self, other: &Self) -> Option<CollisionData> {
        // TODO:
        // for convex GJK for detection and EPA for penetration depth calculation, ellipsiods and box colliders trivial, the rest is triangle intersection tests
        // calculate the minimum translation vector to seperate the two colliders and calculate the collision normal
        // assume the normal vector is form the point of view of 1
        // the normal vector in the collision data should be normalized
        return match self.hitbox {
            Hitbox::Mesh(_) => match other.hitbox {
                Hitbox::Mesh(_) => None,
                Hitbox::ConvexMesh(_) => None,
                Hitbox::Ellipsoid(_) => None,
                Hitbox::Box(_) => None,
            },
            Hitbox::ConvexMesh(_) => match other.hitbox {
                Hitbox::Mesh(_) => None,
                Hitbox::ConvexMesh(_) => None,
                Hitbox::Ellipsoid(_) => None,
                Hitbox::Box(_) => None,
            },
            Hitbox::Ellipsoid(dim1) => match other.hitbox {
                Hitbox::Mesh(_) => None,
                Hitbox::ConvexMesh(_) => None,
                Hitbox::Ellipsoid(dim2) => None,
                Hitbox::Box(dim2) => None,
            },
            Hitbox::Box(dim1) => match other.hitbox {
                Hitbox::Mesh(_) => None,
                Hitbox::ConvexMesh(_) => None,
                Hitbox::Ellipsoid(dim2) => None,
                Hitbox::Box(dim2) => None,
            },
        };
    }
}

pub struct SampledAnimation {}
