use crate::ecs::component::*;
use crate::ecs::entity_manager::EntityManager;
use crate::engine::{Engine, EngineMode, FallingLeafApp};
use crate::glm;
use crate::rendering::data::calc_model_matrix;
use crate::rendering::mesh::{Hitbox, HitboxMesh};
use crate::systems::event_system::events::*;
use crate::systems::event_system::EventObserver;
use crate::utils::constants::bits::internal::*;
use crate::utils::constants::bits::user_level::*;
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
    flying_cam_dir: Option<(glm::Vec3, f32)>,
    flying_cam_keys: MovementKeys,
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
                .query9_mut_opt6::<Position, Collider, MeshType, Velocity, AngularVelocity, Scale, RigidBody, EntityFlags, Orientation>(vec![])
                .map(|(p, hb, mt, v, av, s, rb, f, o)| {
                    (
                        p,
                        entity_manager.hitbox_from_data(mt, &hb.hitbox_type).unwrap(),
                        entity_manager.asset_from_type(mt).unwrap(),
                        hb,
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
                        (copied_or_default(&tuple.6).scale_matrix() * to_vec4(&tuple.2.max_reach))
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
                    // check what entities are fixed
                    let is_dynamic_1 = entity_data[i].4.is_some();
                    let is_dynamic_2 = entity_data[j].4.is_some();

                    if !is_dynamic_1 && !is_dynamic_2 {
                        // both are immovable
                        continue;
                    }

                    // rigid bodies
                    let rb_1 = copied_or_default(&entity_data[i].7);
                    let rb_2 = copied_or_default(&entity_data[j].7);

                    //  translate the colliders to the positional offset of the hitbox
                    let collider_1 = ColliderData {
                        hitbox: entity_data[i].1,
                        model: calc_model_matrix(
                            entity_data[i].0,
                            &copied_or_default(&entity_data[i].6),
                            &copied_or_default(&entity_data[i].9),
                            &rb_1.center_of_mass,
                        ),
                        collider: entity_data[i].3,
                    };
                    let collider_2 = ColliderData {
                        hitbox: entity_data[j].1,
                        model: calc_model_matrix(
                            entity_data[j].0,
                            &copied_or_default(&entity_data[j].6),
                            &copied_or_default(&entity_data[j].9),
                            &rb_2.center_of_mass,
                        ),
                        collider: entity_data[j].3,
                    };
                    // check for collision
                    if let Some(collison_data) = collider_1.collides_with(&collider_2) {
                        // set collision flags
                        if let Some(flags) = &mut entity_data[i].8 {
                            flags.set_bit(COLLISION, true);
                        }
                        if let Some(flags) = &mut entity_data[j].8 {
                            flags.set_bit(COLLISION, true);
                        }

                        // to resolve the collision seperate normal and tangential components of the motion in relation to the collision
                        // INFO: normal, velocity, and translation vector are pov 1 -> 2

                        // vectors from the center of mass to the contact point
                        let mass_center_coll_point_1 =
                            collison_data.collision_point - rb_1.center_of_mass;
                        let mass_center_coll_point_2 =
                            collison_data.collision_point - rb_2.center_of_mass;

                        // relative velocity
                        let v_rel = copied_or_default(&entity_data[i].4).data()
                            - copied_or_default(&entity_data[j].4).data();

                        // components of the relative velocity
                        let normal_component = collison_data.collision_normal.dot(&v_rel) * v_rel;
                        let tangential_component = v_rel - normal_component;

                        let coll_normal = collison_data.collision_normal;
                        let coll_tangent = tangential_component.normalize();

                        let restitution_coefficient = 0.0; // TODO: change that in the future with component data
                        let min_friction = rb_1.friction.min(rb_2.friction).clamp(0.0, 1.0);

                        //
                        // do all of the impulse computations for both the normal and tangential components
                        //

                        //
                        // normal impulse calculations
                        //
                        // k's are responsible for the impact the angular motion has on the collision response
                        // they are used in a more efficient calculation of K1=[r1]_× * I_world,1^−1 * [r1]_×^T (using the cross matrix)
                        let r1_cross_n = mass_center_coll_point_1.cross(&coll_normal);
                        let r2_cross_n = mass_center_coll_point_2.cross(&coll_normal);
                        let k1_normal = rb_1.inv_inertia_tensor * r1_cross_n;
                        let k2_normal = rb_2.inv_inertia_tensor * r2_cross_n;

                        //
                        // tangential impulse calculations
                        //
                        // k's are responsible for the impact the angular motion has on the collision response
                        let r1_cross_t = mass_center_coll_point_1.cross(&coll_tangent);
                        let r2_cross_t = mass_center_coll_point_2.cross(&coll_tangent);
                        let k1_tang = rb_1.inv_inertia_tensor * r1_cross_t;
                        let k2_tang = rb_2.inv_inertia_tensor * r2_cross_t;

                        // resolve the collision depending on what enities are movable
                        // if only one is movable, treat the mass of the immovable entity as infinite
                        if is_dynamic_1 && is_dynamic_2 {
                            // both have velocity and therefore are movable

                            // seperate the two objects
                            *entity_data[i].0.data_mut() += 0.5 * collison_data.translation_vec;
                            *entity_data[j].0.data_mut() += -0.5 * collison_data.translation_vec;

                            // normal impulse
                            let effective_mass_n = 1.0 / rb_1.mass
                                + 1.0 / rb_2.mass
                                + coll_normal.dot(
                                    &(r1_cross_n.cross(&k1_normal) + r2_cross_n.cross(&k2_normal)),
                                );
                            let normal_impulse = -((1.0 + restitution_coefficient)
                                * normal_component)
                                / effective_mass_n;

                            // tangential impulse
                            // TODO: maybe just use the result from the normal impulse to compute Jt = -friction * m * tangential_component? -> performance
                            let effective_mass_t = 1.0 / rb_1.mass
                                + 1.0 / rb_2.mass
                                + coll_tangent.dot(
                                    &(r1_cross_t.cross(&k1_tang) + r2_cross_t.cross(&k2_tang)),
                                );
                            let tang_impulse = -((1.0 + restitution_coefficient)
                                * tangential_component)
                                / effective_mass_t
                                * min_friction;

                            //
                            // apply impulses
                            //
                            *entity_data[i].4.as_mut().unwrap().data_mut() +=
                                (normal_impulse + tang_impulse) / rb_1.mass;
                            *entity_data[j].4.as_mut().unwrap().data_mut() +=
                                -(normal_impulse + tang_impulse) / rb_2.mass;

                            if let Some(angular_vel) = entity_data[i].5.as_mut() {
                                angular_vel.0 += rb_1.inv_inertia_tensor
                                    * mass_center_coll_point_1.cross(&tang_impulse);
                            }
                            if let Some(angular_vel) = entity_data[j].5.as_mut() {
                                angular_vel.0 += rb_2.inv_inertia_tensor
                                    * mass_center_coll_point_2.cross(&tang_impulse);
                            }
                        } else if is_dynamic_1 {
                            // only 1 is movable
                            *entity_data[i].0.data_mut() += collison_data.translation_vec;

                            // normal impulse
                            let effective_mass_n = 1.0 / rb_1.mass
                                + coll_normal.dot(
                                    &(r1_cross_n.cross(&k1_normal) + r2_cross_n.cross(&k2_normal)),
                                );
                            let normal_impulse = -((1.0 + restitution_coefficient)
                                * normal_component)
                                / effective_mass_n;

                            // tangential impulse
                            let effective_mass_t = 1.0 / rb_1.mass
                                + coll_tangent.dot(
                                    &(r1_cross_t.cross(&k1_tang) + r2_cross_t.cross(&k2_tang)),
                                );
                            let tang_impulse = -((1.0 + restitution_coefficient)
                                * tangential_component)
                                / effective_mass_t
                                * min_friction;

                            //
                            // apply impulses
                            //
                            *entity_data[i].4.as_mut().unwrap().data_mut() +=
                                (normal_impulse + tang_impulse) / rb_1.mass;

                            if let Some(angular_vel) = entity_data[i].5.as_mut() {
                                angular_vel.0 += rb_1.inv_inertia_tensor
                                    * mass_center_coll_point_1.cross(&tang_impulse);
                            }
                        } else if is_dynamic_2 {
                            // only 2 is movable
                            *entity_data[j].0.data_mut() += -collison_data.translation_vec;

                            // normal impulse
                            let effective_mass_n = 1.0 / rb_2.mass
                                + coll_normal.dot(
                                    &(r1_cross_n.cross(&k1_normal) + r2_cross_n.cross(&k2_normal)),
                                );
                            let normal_impulse = -((1.0 + restitution_coefficient)
                                * normal_component)
                                / effective_mass_n;

                            // tangential impulse
                            let effective_mass_t = 1.0 / rb_2.mass
                                + coll_tangent.dot(
                                    &(r1_cross_t.cross(&k1_tang) + r2_cross_t.cross(&k2_tang)),
                                );
                            let tang_impulse = -((1.0 + restitution_coefficient)
                                * tangential_component)
                                / effective_mass_t
                                * min_friction;

                            //
                            // apply impulses
                            //
                            *entity_data[j].4.as_mut().unwrap().data_mut() +=
                                -(normal_impulse + tang_impulse) / rb_2.mass;

                            if let Some(angular_vel) = entity_data[j].5.as_mut() {
                                angular_vel.0 += rb_2.inv_inertia_tensor
                                    * mass_center_coll_point_2.cross(&tang_impulse);
                            }
                        } else {
                            // both are immovable (should not happen due to the check earlier)
                            panic!("tried to seperate two immovable objects");
                        }
                        any_hits = true;
                    }
                }
            }
        }
    }

    /// performs all relevant physics calculations on entity data
    fn apply_physics(&self, entity_manager: &mut EntityManager, time_step: TimeDuration) {
        for (p, v, a_opt, rb_opt, o_opt, av_opt, flags) in unsafe {
            entity_manager
                .query7_mut_opt5::<Position, Velocity, Acceleration, RigidBody, Orientation, AngularVelocity, EntityFlags>(vec![])
        } {
            let total_a = rb_opt
                .is_some()
                .then(|| {
                    flags.map_or(Acceleration::zero(), |f| {
                        if f.get_bit(FLOATING) {
                            Acceleration::zero()
                        } else {
                            self.gravity
                        }
                    })
                })
                .unwrap_or_default()
                + a_opt.copied().unwrap_or_default();

            *v += total_a * time_step;
            *p += *v * time_step;

            if let (Some(av), Some(o)) = (av_opt, o_opt) {
                let inv_inertia_mat = rb_opt.copied().unwrap_or_default().inv_inertia_tensor;
                let corr_av = inv_inertia_mat * av.0;
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

/// contains collision plane normal vector, the point of the collision and the minimal translation vector
struct CollisionData {
    collision_normal: glm::Vec3,
    collision_point: glm::Vec3,
    translation_vec: glm::Vec3,
}

/// collider data that is used in collision checking
struct ColliderData<'a> {
    hitbox: &'a Hitbox,
    model: glm::Mat4,
    collider: &'a Collider,
}

impl ColliderData<'_> {
    /// checks if two hitboxes collide with each other
    pub(crate) fn collides_with(&self, other: &Self) -> Option<CollisionData> {
        match self.hitbox {
            Hitbox::ConvexMesh(mesh1) => match other.hitbox {
                Hitbox::ConvexMesh(mesh2) => seperating_axis(
                    mesh1,
                    mesh2,
                    &self.model,
                    &other.model,
                    self.collider,
                    other.collider,
                ),
                Hitbox::Ellipsoid(dim) => mesh_ellipsoid_collision(
                    mesh1,
                    dim,
                    &self.model,
                    &other.model,
                    self.collider,
                    other.collider,
                ),
                Hitbox::Box(mesh2) => seperating_axis(
                    mesh1,
                    mesh2,
                    &self.model,
                    &other.model,
                    self.collider,
                    other.collider,
                ),
            },
            Hitbox::Ellipsoid(dim1) => match other.hitbox {
                Hitbox::ConvexMesh(mesh) => {
                    mesh_ellipsoid_collision(
                        mesh,
                        dim1,
                        &other.model,
                        &self.model,
                        other.collider,
                        self.collider,
                    )
                    .map(|mut cd| {
                        // invert the vectors because the order changed in the function parameters
                        cd.translation_vec = -cd.translation_vec;
                        cd.collision_normal = -cd.collision_normal;
                        cd
                    })
                }
                Hitbox::Ellipsoid(dim2) => ellipsoid_collision(
                    dim1,
                    dim2,
                    &self.model,
                    &other.model,
                    self.collider,
                    other.collider,
                ),
                Hitbox::Box(mesh) => {
                    mesh_ellipsoid_collision(
                        mesh,
                        dim1,
                        &other.model,
                        &self.model,
                        other.collider,
                        self.collider,
                    )
                    .map(|mut cd| {
                        // invert the vectors because the order changed in the function parameters
                        cd.translation_vec = -cd.translation_vec;
                        cd.collision_normal = -cd.collision_normal;
                        cd
                    })
                }
            },
            Hitbox::Box(mesh1) => match other.hitbox {
                Hitbox::ConvexMesh(mesh2) => seperating_axis(
                    mesh1,
                    mesh2,
                    &self.model,
                    &other.model,
                    self.collider,
                    other.collider,
                ),
                Hitbox::Ellipsoid(dim) => mesh_ellipsoid_collision(
                    mesh1,
                    dim,
                    &self.model,
                    &other.model,
                    self.collider,
                    other.collider,
                ),
                Hitbox::Box(mesh2) => seperating_axis(
                    mesh1,
                    mesh2,
                    &self.model,
                    &other.model,
                    self.collider,
                    other.collider,
                ),
            },
        }
    }
}

// for convex GJK for detection and EPA for penetration depth calculation, ellipsiods and box colliders trivial, the rest is triangle intersection tests
// calculate the minimum translation vector to seperate the two colliders and calculate the collision normal
// assume the normal vector is form the point of view of 1
// the normal vector in the collision data should be normalized

fn ellipsoid_collision(
    ell1: &glm::Vec3,
    ell2: &glm::Vec3,
    m1: &glm::Mat4,
    m2: &glm::Mat4,
    coll1: &Collider,
    coll2: &Collider,
) -> Option<CollisionData> {
    None
}

fn mesh_ellipsoid_collision(
    bx: &HitboxMesh,
    ell: &glm::Vec3,
    m1: &glm::Mat4,
    m2: &glm::Mat4,
    coll1: &Collider,
    coll2: &Collider,
) -> Option<CollisionData> {
    None
}

fn seperating_axis(
    mesh1: &HitboxMesh,
    mesh2: &HitboxMesh,
    m1: &glm::Mat4,
    m2: &glm::Mat4,
    coll1: &Collider,
    coll2: &Collider,
) -> Option<CollisionData> {
    None
}
