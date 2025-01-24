use crate::ecs::component::utils::*;
use crate::ecs::component::*;
use crate::ecs::entity_manager::EntityManager;
use crate::engine::{Engine, FallingLeafApp};
use crate::rendering::data::calc_model_matrix;
use crate::rendering::mesh::Hitbox;
use crate::systems::event_system::events::*;
use crate::systems::event_system::EventObserver;
use crate::utils::constants::bits::internal::*;
use crate::utils::constants::bits::user_level::*;
use crate::utils::constants::*;
use crate::utils::tools::*;
use crate::{glm, include_filter};
use fyrox_sound::math::get_barycentric_coords;
use itertools::Itertools;
use std::collections::HashSet;
use std::ops::DerefMut;
use winit::keyboard::KeyCode;

pub struct AnimationSystem {
    pub(crate) animation_speed: f32,
    gravity: Acceleration,
    pub(crate) flying_cam_dir: Option<(glm::Vec3, f32)>,
    pub(crate) flying_cam_keys: MovementKeys,
    pub(crate) curr_cam_pos: glm::Vec3,
    pub(crate) prev_cam_pos: glm::Vec3,
}

impl AnimationSystem {
    /// creates a new animation system
    pub(crate) fn new() -> Self {
        Self {
            animation_speed: 1.0,
            gravity: G,
            flying_cam_dir: None,
            flying_cam_keys: MovementKeys {
                up: KeyCode::Space,
                down: KeyCode::ShiftLeft,
                forward: KeyCode::KeyW,
                backward: KeyCode::KeyS,
                left: KeyCode::KeyA,
                right: KeyCode::KeyD,
            },
            curr_cam_pos: ORIGIN,
            prev_cam_pos: ORIGIN,
        }
    }

    /// applys all animaion updates to all entities (called once per time step)
    pub(crate) fn update<T: FallingLeafApp>(&mut self, engine: &Engine<T>) {
        self.apply_physics(engine.entity_manager_mut().deref_mut());
        self.handle_collisions(engine.entity_manager_mut().deref_mut());
        self.damp_velocities(engine.entity_manager_mut().deref_mut());
    }

    /// stops velocities near zero to make behavior more realistic
    fn damp_velocities(&self, entity_manager: &mut EntityManager) {
        for velocity in unsafe {
            entity_manager.query1_mut::<Velocity>((Some(include_filter!(Position)), None))
        } {
            let ub = 0.1;
            let vel_norm = velocity.data().norm();
            let factor = map_range((0.0, ub), (0.999, 1.0), vel_norm.clamp(0.0, ub));
            *velocity *= factor;
        }
        for momentum in unsafe {
            entity_manager
                .query1_mut::<AngularMomentum>((Some(include_filter!(Position, Orientation)), None))
        } {
            let ub = 0.1;
            let mom_norm = momentum.data().norm();
            let factor = map_range((0.0, ub), (0.999, 1.0), mom_norm.clamp(0.0, ub));
            *momentum *= factor;
        }
    }

    /// checks for collision between entities with hitboxes and resolves them
    fn handle_collisions(&self, entity_manager: &mut EntityManager) {
        let mut entity_data = unsafe {
            entity_manager
                .query9_mut_opt6::<Position, Collider, MeshType, Velocity, AngularMomentum, Scale, RigidBody, EntityFlags, Orientation>((None, None))
                .map(|(p, coll, mt, v, am, s, rb, f, o)| {
                    let mesh = entity_manager.asset_from_type(mt, LOD::None).unwrap();
                    let hitbox = entity_manager.hitbox_from_data(mt, &coll.hitbox_type).unwrap();
                    let scale_matrix = copied_or_default(&s).scale_matrix() * coll.scale.scale_matrix();
                    let coll_reach = mesh.max_reach + coll.offset.abs();
                    coll.last_collisions.clear();
                    (
                        p,
                        hitbox,
                        mesh,
                        coll,
                        v,
                        am,
                        s,
                        rb,
                        f.map(|flags| {
                            flags.set_bit(COLLIDED, false);
                            flags
                        }),
                        o,
                        mult_mat4_vec3(&scale_matrix, &coll_reach).norm()
                    )
                })
                .collect_vec()
        };
        if entity_data.len() <= 1 {
            return;
        }
        // repeat collision checks until all of them are resolved
        let mut any_hits = true;
        while any_hits {
            any_hits = false;
            for (i, j) in (0..entity_data.len()).tuple_combinations() {
                // check bounding spheres of the mesh for macro level filtering
                if !spheres_collide(
                    entity_data[i].0.data(),
                    entity_data[i].10,
                    entity_data[j].0.data(),
                    entity_data[j].10,
                ) {
                    continue;
                }

                // check what entities do not have velocity
                let is_dynamic_1 = entity_data[i].4.is_some();
                let is_dynamic_2 = entity_data[j].4.is_some();

                // check what entities are still colliding statically
                let static_collision_1 = entity_data[i]
                    .8
                    .as_ref()
                    .map(|flags| flags.get_bit(STATIC_COLLISION))
                    .unwrap_or(false);
                let static_collision_2 = entity_data[j]
                    .8
                    .as_ref()
                    .map(|flags| flags.get_bit(STATIC_COLLISION))
                    .unwrap_or(false);

                // check what entities ignore collision resolvement
                let ingores_collision_1 = entity_data[i]
                    .8
                    .as_ref()
                    .map(|flags| flags.get_bit(IGNORING_COLLISION))
                    .unwrap_or(false);
                let ingores_collision_2 = entity_data[j]
                    .8
                    .as_ref()
                    .map(|flags| flags.get_bit(IGNORING_COLLISION))
                    .unwrap_or(false);

                // ignore collision resolvement if a relevant flag is set
                if ingores_collision_1 || ingores_collision_2 {
                    continue;
                }

                // check what entities should be seperated
                let should_seperate_1 = is_dynamic_1 || static_collision_1;
                let should_seperate_2 = is_dynamic_2 || static_collision_2;

                // rigid bodies
                let rb_1 = copied_or_default(&entity_data[i].7);
                let rb_2 = copied_or_default(&entity_data[j].7);
                // rotations
                let rot1 = copied_or_default(&entity_data[i].9);
                let rot2 = copied_or_default(&entity_data[j].9);
                // scales
                let scale1 = copied_or_default(&entity_data[i].6);
                let scale2 = copied_or_default(&entity_data[j].6);

                //  translate the colliders to the positional offset of the hitbox
                let collider_1 = ColliderData {
                    position: *entity_data[i].0,
                    scale: scale1,
                    orientation: rot1,
                    center_of_mass: rb_1.center_of_mass,
                    hitbox: entity_data[i].1,
                    collider: entity_data[i].3,
                    is_dynamic: should_seperate_1,
                };
                let collider_2 = ColliderData {
                    position: *entity_data[j].0,
                    scale: scale2,
                    orientation: rot2,
                    center_of_mass: rb_2.center_of_mass,
                    hitbox: entity_data[j].1,
                    collider: entity_data[j].3,
                    is_dynamic: should_seperate_2,
                };
                // check for collision
                if let Some(collision_data) = collider_1.collides_with(&collider_2) {
                    // INFO: normal, velocity, and translation vector, etc are POV 1 -> 2

                    // set collision flags/data
                    if let Some(flags) = &mut entity_data[i].8 {
                        flags.set_bit(COLLIDED, true);
                    }
                    if let Some(flags) = &mut entity_data[j].8 {
                        flags.set_bit(COLLIDED, true);
                    }

                    // rotation matrices + local inertia matrices
                    let rotation_mat1 = glm::mat4_to_mat3(&rot1.rotation_matrix());
                    let rotation_mat2 = glm::mat4_to_mat3(&rot2.rotation_matrix());
                    let local_inertia_inv_1 =
                        rotation_mat1 * rb_1.inv_inertia_tensor * rotation_mat1.transpose();
                    let local_inertia_inv_2 =
                        rotation_mat2 * rb_2.inv_inertia_tensor * rotation_mat2.transpose();

                    // vectors from the center of mass to the contact point
                    let mass_center_coll_point_1 = collision_data.collision_point
                        - (rb_1.center_of_mass + entity_data[i].0.data());
                    let mass_center_coll_point_2 = collision_data.collision_point
                        - (rb_2.center_of_mass + entity_data[j].0.data());

                    // angular velocities
                    let av1 = local_inertia_inv_1 * copied_or_default(&entity_data[i].5).data();
                    let av2 = local_inertia_inv_2 * copied_or_default(&entity_data[j].5).data();

                    // relative velocity
                    let v_rel = copied_or_default(&entity_data[i].4).data()
                        + av1.cross(&mass_center_coll_point_1)
                        - copied_or_default(&entity_data[j].4).data()
                        - av2.cross(&mass_center_coll_point_2);

                    // set collision info
                    entity_data[i].3.last_collisions.push(CollisionInfo {
                        momentum: v_rel * rb_1.mass,
                        point: collision_data.collision_point,
                        normal: collision_data.collision_normal,
                    });
                    entity_data[j].3.last_collisions.push(CollisionInfo {
                        momentum: -v_rel * rb_2.mass,
                        point: collision_data.collision_point,
                        normal: -collision_data.collision_normal,
                    });

                    // seperate the two objects
                    if should_seperate_1 && should_seperate_2 {
                        // both have velocity and therefore are movable
                        *entity_data[i].0.data_mut() += 0.5 * collision_data.translation_vec;
                        *entity_data[j].0.data_mut() += -0.5 * collision_data.translation_vec;
                    } else if should_seperate_1 {
                        // only 1 is movable
                        *entity_data[i].0.data_mut() += collision_data.translation_vec;
                    } else if should_seperate_2 {
                        // only 2 is movable
                        *entity_data[j].0.data_mut() += -collision_data.translation_vec;
                    }

                    // ignore the rest of the collision response if both of entities are static or the flag is set
                    if (!is_dynamic_1 || static_collision_1)
                        && (!is_dynamic_2 || static_collision_2)
                    {
                        continue;
                    }

                    // collision normal and tangent frame
                    let coll_normal = collision_data.collision_normal;
                    let coll_tangent1 = if coll_normal.y != 0.0 || coll_normal.z != 0.0 {
                        normalize_non_zero(coll_normal.cross(&X_AXIS)).unwrap_or(ORIGIN)
                    } else {
                        normalize_non_zero(coll_normal.cross(&Z_AXIS)).unwrap_or(ORIGIN)
                    };
                    let coll_tangent2 =
                        normalize_non_zero(coll_normal.cross(&coll_tangent1)).unwrap_or(ORIGIN);

                    // components of the relative velocity
                    let normal_component = coll_normal.dot(&v_rel) * coll_normal;
                    let tangential_component1 = coll_tangent1.dot(&v_rel) * coll_tangent1;
                    let tangential_component2 = coll_tangent2.dot(&v_rel) * coll_tangent2;
                    let total_friction = (rb_1.friction * rb_2.friction).sqrt();

                    // resolve the collision depending on what enities are movable
                    // if only one is movable, treat the mass of the immovable entity as infinite
                    if is_dynamic_1 && is_dynamic_2 {
                        // normal impulse
                        let effective_mass_n = 1.0 / rb_1.mass
                            + 1.0 / rb_2.mass
                            + coll_normal.cross(&mass_center_coll_point_1).dot(
                                &(local_inertia_inv_1
                                    * coll_normal.cross(&mass_center_coll_point_1)),
                            )
                            + coll_normal.cross(&mass_center_coll_point_2).dot(
                                &(local_inertia_inv_2
                                    * coll_normal.cross(&mass_center_coll_point_2)),
                            );
                        let normal_impulse = -normal_component / effective_mass_n;

                        // tangential impulse
                        let effective_mass_t1 = 1.0 / rb_1.mass
                            + 1.0 / rb_2.mass
                            + coll_tangent1.cross(&mass_center_coll_point_1).dot(
                                &(local_inertia_inv_1
                                    * coll_tangent1.cross(&mass_center_coll_point_1)),
                            )
                            + coll_tangent1.cross(&mass_center_coll_point_2).dot(
                                &(local_inertia_inv_2
                                    * coll_tangent1.cross(&mass_center_coll_point_2)),
                            );
                        let effective_mass_t2 = 1.0 / rb_1.mass
                            + 1.0 / rb_2.mass
                            + coll_tangent2.cross(&mass_center_coll_point_1).dot(
                                &(local_inertia_inv_1
                                    * coll_tangent2.cross(&mass_center_coll_point_1)),
                            )
                            + coll_tangent2.cross(&mass_center_coll_point_2).dot(
                                &(local_inertia_inv_2
                                    * coll_tangent2.cross(&mass_center_coll_point_2)),
                            );
                        let tang_impulse1 = -tangential_component1 / effective_mass_t1;
                        let tang_impulse2 = -tangential_component2 / effective_mass_t2;
                        let tang_impulse = (tang_impulse1 + tang_impulse2) * total_friction;

                        // friction
                        let max_friction_impulse = total_friction * normal_impulse.norm();
                        let tang_impulse_mag = tang_impulse.norm();
                        let clamped_tang_impulse = if tang_impulse_mag > max_friction_impulse {
                            tang_impulse * (max_friction_impulse / tang_impulse_mag)
                        } else {
                            tang_impulse
                        };

                        // apply impulse
                        let impulse1 =
                            (normal_impulse + clamped_tang_impulse) * (1.0 + rb_1.restitution);
                        let impulse2 =
                            (normal_impulse + clamped_tang_impulse) * (1.0 + rb_2.restitution);

                        *entity_data[i].4.as_mut().unwrap().data_mut() += impulse1 / rb_1.mass;
                        *entity_data[j].4.as_mut().unwrap().data_mut() += -impulse2 / rb_2.mass;

                        if let Some(angular_mom) = entity_data[i].5.as_mut() {
                            *angular_mom.data_mut() += mass_center_coll_point_1.cross(&impulse1);
                        }
                        if let Some(angular_mom) = entity_data[j].5.as_mut() {
                            *angular_mom.data_mut() += mass_center_coll_point_2.cross(&(-impulse2));
                        }
                    } else if is_dynamic_1 {
                        // normal impulse
                        let effective_mass_n = 1.0 / rb_1.mass
                            + coll_normal.cross(&mass_center_coll_point_1).dot(
                                &(local_inertia_inv_1
                                    * coll_normal.cross(&mass_center_coll_point_1)),
                            );
                        let normal_impulse = -normal_component / effective_mass_n;

                        // tangential impulse
                        let effective_mass_t1 = 1.0 / rb_1.mass
                            + coll_tangent1.cross(&mass_center_coll_point_1).dot(
                                &(local_inertia_inv_1
                                    * coll_tangent1.cross(&mass_center_coll_point_1)),
                            );
                        let effective_mass_t2 = 1.0 / rb_1.mass
                            + coll_tangent2.cross(&mass_center_coll_point_1).dot(
                                &(local_inertia_inv_1
                                    * coll_tangent2.cross(&mass_center_coll_point_1)),
                            );
                        let tang_impulse1 =
                            -tangential_component1 / effective_mass_t1 * total_friction;
                        let tang_impulse2 =
                            -tangential_component2 / effective_mass_t2 * total_friction;
                        let tang_impulse = (tang_impulse1 + tang_impulse2) * total_friction;

                        // friction
                        let max_friction_impulse = total_friction * normal_impulse.norm();
                        let tang_impulse_mag = tang_impulse.norm();
                        let clamped_tang_impulse = if tang_impulse_mag > max_friction_impulse {
                            tang_impulse * (max_friction_impulse / tang_impulse_mag)
                        } else {
                            tang_impulse
                        };

                        // apply impulse
                        let impulse1 =
                            (normal_impulse + clamped_tang_impulse) * (1.0 + rb_1.restitution);

                        *entity_data[i].4.as_mut().unwrap().data_mut() += impulse1 / rb_1.mass;

                        if let Some(angular_mom) = entity_data[i].5.as_mut() {
                            *angular_mom.data_mut() += mass_center_coll_point_1.cross(&impulse1);
                        }
                    } else if is_dynamic_2 {
                        // normal impulse
                        let effective_mass_n = 1.0 / rb_2.mass
                            + coll_normal.cross(&mass_center_coll_point_2).dot(
                                &(local_inertia_inv_2
                                    * coll_normal.cross(&mass_center_coll_point_2)),
                            );
                        let normal_impulse = -normal_component / effective_mass_n;

                        // tangential impulse
                        let effective_mass_t1 = 1.0 / rb_2.mass
                            + coll_tangent1.cross(&mass_center_coll_point_2).dot(
                                &(local_inertia_inv_2
                                    * coll_tangent1.cross(&mass_center_coll_point_2)),
                            );
                        let effective_mass_t2 = 1.0 / rb_2.mass
                            + coll_tangent2.cross(&mass_center_coll_point_2).dot(
                                &(local_inertia_inv_2
                                    * coll_tangent2.cross(&mass_center_coll_point_2)),
                            );
                        let tang_impulse1 =
                            -tangential_component1 / effective_mass_t1 * total_friction;
                        let tang_impulse2 =
                            -tangential_component2 / effective_mass_t2 * total_friction;
                        let tang_impulse = (tang_impulse1 + tang_impulse2) * total_friction;

                        // friction
                        let max_friction_impulse = total_friction * normal_impulse.norm();
                        let tang_impulse_mag = tang_impulse.norm();
                        let clamped_tang_impulse = if tang_impulse_mag > max_friction_impulse {
                            tang_impulse * (max_friction_impulse / tang_impulse_mag)
                        } else {
                            tang_impulse
                        };

                        // apply impulse
                        let impulse2 =
                            (normal_impulse + clamped_tang_impulse) * (1.0 + rb_2.restitution);

                        *entity_data[j].4.as_mut().unwrap().data_mut() += -impulse2 / rb_2.mass;

                        if let Some(angular_mom) = entity_data[j].5.as_mut() {
                            *angular_mom.data_mut() += mass_center_coll_point_2.cross(&(-impulse2));
                        }
                    }
                    // register that a collsion was resolved
                    any_hits = true;
                }
            }
        }
    }

    /// performs all relevant physics calculations on entity data
    fn apply_physics(&self, entity_manager: &mut EntityManager) {
        for (p, v, a_opt, rb_opt, o_opt, am_opt, flags) in unsafe {
            entity_manager
                .query7_mut_opt5::<Position, Velocity, Acceleration, RigidBody, Orientation, AngularMomentum, EntityFlags>((None, None))
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
                + copied_or_default(&a_opt);

            *v += total_a * TIME_STEP;
            *p += *v * TIME_STEP;

            if let (Some(am), Some(o)) = (am_opt, o_opt) {
                let inv_inertia_mat = copied_or_default(&rb_opt).inv_inertia_tensor;
                let rot_mat = glm::mat4_to_mat3(&o.rotation_matrix());
                let local_inertia_mat = rot_mat * inv_inertia_mat * rot_mat.transpose();
                let ang_vel = local_inertia_mat * am.data();
                o.0 += 0.5 * o.0 * glm::quat(ang_vel.x, ang_vel.y, ang_vel.z, 0.0) * TIME_STEP.0;
                o.0.normalize_mut();
            }
        }
    }

    /// enables/disables the built-in flying cam movement with a movement speed
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
    /// (default: up - Space, down - LeftShift, directions - WASD)
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

impl EventObserver<CamPositionChange> for AnimationSystem {
    fn on_event(&mut self, event: &CamPositionChange) {
        self.prev_cam_pos = self.curr_cam_pos;
        self.curr_cam_pos = event.new_pos;
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

/// checks if two broad oject areas represented as spheres at two positions collide
fn spheres_collide(pos1: &glm::Vec3, radius1: f32, pos2: &glm::Vec3, radius2: f32) -> bool {
    (pos1 - pos2).norm() < radius1 + radius2
}

/// contains collision plane normal vector, the point of the collision and the minimal translation vector
#[derive(Debug, Copy, Clone)]
struct CollisionData {
    collision_normal: glm::Vec3,
    collision_point: glm::Vec3,
    translation_vec: glm::Vec3,
}

/// collider data that is used in collision checking
struct ColliderData<'a> {
    position: Position,
    scale: Scale,
    orientation: Orientation,
    center_of_mass: glm::Vec3,
    hitbox: &'a Hitbox,
    collider: &'a Collider,
    is_dynamic: bool,
}

impl ColliderData<'_> {
    /// builds the sphere collider specification data if the hitbox type matches
    fn sphere_spec(&self) -> Option<SphereColliderSpec> {
        match self.hitbox {
            Hitbox::ConvexMesh(_) => None,
            Hitbox::Sphere(radius) => {
                let mass_offset = glm::translate(&glm::Mat4::identity(), &self.center_of_mass);
                let inv_mass_offset = mass_offset.try_inverse().unwrap();
                Some(SphereColliderSpec {
                    scale_dimensions: glm::Vec3::from_element(*radius)
                        .component_mul(self.scale.data())
                        .component_mul(self.collider.scale.data()),
                    collider_pos: self.position.data() + self.collider.offset,
                    rotation_mat: mass_offset
                        * self.orientation.rotation_matrix()
                        * inv_mass_offset,
                })
            }
        }
    }

    /// builds the mesh collider specification data if the hitbox type matches
    fn mesh_spec(&self) -> Option<MeshColliderSpec> {
        match self.hitbox {
            Hitbox::ConvexMesh(mesh) => {
                let model = calc_model_matrix(
                    &self.position,
                    &self.scale,
                    &self.orientation,
                    &self.center_of_mass,
                );
                Some(MeshColliderSpec {
                    transform: model
                        * self.collider.scale.scale_matrix()
                        * glm::translate(&glm::Mat4::identity(), &self.collider.offset),
                    points: &mesh.vertices,
                })
            }
            Hitbox::Sphere(_) => None,
        }
    }

    /// checks if two hitboxes collide with each other
    pub(crate) fn collides_with(&self, other: &Self) -> Option<CollisionData> {
        // calculate the factor of one colliders translation vector
        let translate_factor = if self.is_dynamic && other.is_dynamic {
            0.5
        } else if self.is_dynamic {
            1.0
        } else {
            0.0
        };
        // assume the normal vector is form the point of view of 1
        // the normal vector in the collision data should be normalized
        if let Some(spec1) = self.mesh_spec() {
            if let Some(spec2) = other.mesh_spec() {
                gjk(spec1, spec2, translate_factor)
            } else {
                let spec2 = other.sphere_spec().unwrap();
                gjk(spec1, spec2, translate_factor)
            }
        } else {
            let spec1 = self.sphere_spec().unwrap();
            if let Some(spec2) = other.mesh_spec() {
                gjk(spec1, spec2, translate_factor)
            } else {
                let spec2 = other.sphere_spec().unwrap();
                gjk(spec1, spec2, translate_factor)
            }
        }
    }
}

/// allows to find the furthest point in a given direction of a collider generially
trait ColliderSpec {
    fn find_furthest_point(&self, direction: &glm::Vec3) -> glm::Vec3;
}

/// specification for a sphere collider used in collision detection
struct SphereColliderSpec {
    scale_dimensions: glm::Vec3,
    collider_pos: glm::Vec3,
    rotation_mat: glm::Mat4,
}

impl ColliderSpec for SphereColliderSpec {
    fn find_furthest_point(&self, direction: &glm::Vec3) -> glm::Vec3 {
        let local_direction = mult_mat4_vec3(&self.rotation_mat.transpose(), direction);
        let stretch_factor = 1.0
            / ((local_direction.x * local_direction.x)
                / (self.scale_dimensions.x * self.scale_dimensions.x)
                + (local_direction.y * local_direction.y)
                    / (self.scale_dimensions.y * self.scale_dimensions.y)
                + (local_direction.z * local_direction.z)
                    / (self.scale_dimensions.z * self.scale_dimensions.z))
                .sqrt();
        let local_point = local_direction * stretch_factor;
        mult_mat4_vec3(&self.rotation_mat, &local_point) + self.collider_pos
    }
}

/// specification for a mesh collider used in collision detection
struct MeshColliderSpec<'a> {
    transform: glm::Mat4,
    points: &'a Vec<glm::Vec3>,
}

impl ColliderSpec for MeshColliderSpec<'_> {
    fn find_furthest_point(&self, direction: &glm::Vec3) -> glm::Vec3 {
        self.points
            .iter()
            .map(|point| mult_mat4_vec3(&self.transform, point))
            .map(|point| (point, point.dot(direction)))
            .max_by(|(_, dot1), (_, dot2)| dot1.partial_cmp(dot2).unwrap())
            .unwrap()
            .0
    }
}

/// holds the data for the current simplex used in the GJK algorithm
#[derive(Debug)]
struct Simplex {
    points: [SupportData; 4],
    size: usize,
}

impl Simplex {
    fn new() -> Self {
        Self {
            points: [
                SupportData::default(),
                SupportData::default(),
                SupportData::default(),
                SupportData::default(),
            ],
            size: 0,
        }
    }

    fn from_points(points: &[SupportData]) -> Self {
        let mut instance = Self::new();
        for point in points {
            instance.push_front(*point);
        }
        instance
    }

    fn push_front(&mut self, element: SupportData) {
        self.points = [element, self.points[0], self.points[1], self.points[2]];
        self.size = 4.min(self.size + 1)
    }
}

/// implementation of the GJK algorithm for detecting collisions
fn gjk(
    collider1: impl ColliderSpec,
    collider2: impl ColliderSpec,
    translate_factor: f32,
) -> Option<CollisionData> {
    let mut supp_data = support(&collider1, &collider2, &X_AXIS);
    let mut simplex = Simplex::from_points(&[supp_data]);
    let mut direction = -supp_data.support;

    loop {
        supp_data = support(&collider1, &collider2, &direction);
        if !same_direction(&direction, &supp_data.support) {
            return None;
        }
        simplex.push_front(supp_data);
        if solve_simplex(&mut simplex, &mut direction) {
            // use the EPA algorithm in case of a collision
            return Some(epa(collider1, collider2, simplex, translate_factor));
        }
    }
}

/// implementation of the EPA algorithm for determining collision data in case of collision
fn epa(
    collider1: impl ColliderSpec,
    collider2: impl ColliderSpec,
    simplex: Simplex,
    translate_factor: f32,
) -> CollisionData {
    let mut polytope = simplex.points.to_vec();
    let mut faces = (0..polytope.len())
        .tuple_combinations::<(usize, usize, usize)>()
        .collect_vec();

    // iteratively add new points to the polytope until the correct normal is found
    loop {
        let (face_idx, min_normal, min_distance) = find_min_data(&polytope, &faces);
        let supp_data = support(&collider1, &collider2, &min_normal);
        let supp_distance = supp_data.support.dot(&min_normal);

        if supp_distance - min_distance <= 1e-6 {
            let face = faces[face_idx];
            let (v1, v2, v3) = (polytope[face.0], polytope[face.1], polytope[face.2]);
            // project origin onto this closest triangle
            let origin_proj = min_normal * min_distance;
            // find the barycentric coordinates for that point
            let (u, v, w) = get_barycentric_coords(
                &vec3_to_vector3(&origin_proj),
                &vec3_to_vector3(&v1.support),
                &vec3_to_vector3(&v2.support),
                &vec3_to_vector3(&v3.support),
            );
            let coll_point = u * v1.fp1 + v * v2.fp1 + w * v3.fp1 - origin_proj * translate_factor;
            let mut translation_vec = -min_normal * supp_distance;
            // seperate the colliders a tiny bit more to avoid collision detection problems
            for coord in translation_vec.iter_mut() {
                if *coord > 0.0 {
                    *coord += 0.0001;
                } else if *coord < 0.0 {
                    *coord -= 0.0001
                }
            }
            // return collision data
            return CollisionData {
                collision_normal: min_normal,
                collision_point: coll_point,
                translation_vec,
            };
        }

        polytope.push(supp_data);
        reconstruct_polytope(&polytope, &mut faces, supp_data);
    }
}

/// reconstructs the polytope after a new point was found in the EPA algorithm
fn reconstruct_polytope(
    polytope: &[SupportData],
    faces: &mut Vec<(usize, usize, usize)>,
    supp_data: SupportData,
) {
    // find faces to remove
    let mut faces_to_remove = faces
        .iter()
        .enumerate()
        .filter_map(|(i, face)| {
            if same_direction(
                &outward_normal(
                    polytope[face.0].support,
                    polytope[face.1].support,
                    polytope[face.2].support,
                    ORIGIN,
                ),
                &(supp_data.support - polytope[face.0].support),
            ) {
                return Some(i);
            }
            None
        })
        .collect_vec();

    // find edges that are not shared between faces
    let mut dangling_edges = HashSet::new();
    for edge in faces_to_remove
        .iter()
        .flat_map(|face_idx| {
            let face = faces[*face_idx];
            [[face.0, face.1], [face.1, face.2], [face.2, face.0]]
        })
        .map(|mut edge| {
            edge.sort_unstable();
            edge
        })
    {
        if dangling_edges.contains(&edge) {
            dangling_edges.remove(&edge);
        } else {
            dangling_edges.insert(edge);
        }
    }

    // remove faces
    faces_to_remove.sort_unstable();
    for face_idx in faces_to_remove.into_iter().rev() {
        faces.remove(face_idx);
    }

    // add new faces
    faces.extend(
        dangling_edges
            .into_iter()
            .map(|edge| (edge[0], edge[1], polytope.len() - 1)),
    );
}

/// finds the currently minimal face data in the EPA algorithm (face_idx, normal, distance)
fn find_min_data(
    polytope: &[SupportData],
    faces: &[(usize, usize, usize)],
) -> (usize, glm::Vec3, f32) {
    faces
        .iter()
        .enumerate()
        .map(|(i, &(node1, node2, node3))| (i, [polytope[node1], polytope[node2], polytope[node3]]))
        .map(|(i, vertices)| {
            let normal = outward_normal(
                vertices[0].support,
                vertices[1].support,
                vertices[2].support,
                ORIGIN,
            )
            .normalize();
            (i, normal, normal.dot(&vertices[0].support))
        })
        .min_by(|(_, _, distance1), (_, _, distance2)| distance1.partial_cmp(distance2).unwrap())
        .unwrap()
}

/// dispatcher of the different simplex cases that modify the current state of the GJK algorithm
fn solve_simplex(simplex: &mut Simplex, direction: &mut glm::Vec3) -> bool {
    match simplex.size {
        2 => solve_line(simplex, direction),
        3 => solve_triangle(simplex, direction),
        4 => solve_tetrahedron(simplex, direction),
        _ => panic!("corrupt simplex size"),
    }
}

/// handles the line simplex case in GJK
fn solve_line(simplex: &mut Simplex, direction: &mut glm::Vec3) -> bool {
    let a = simplex.points[0];
    let b = simplex.points[1];
    let ab = b.support - a.support;
    let ao = -a.support;

    if same_direction(&ab, &ao) {
        *direction = ab.cross(&ao).cross(&ab);
    } else {
        *simplex = Simplex::from_points(&[a]);
        *direction = ao;
    }
    false
}

/// handles the traingle simplex case in GJK
fn solve_triangle(simplex: &mut Simplex, direction: &mut glm::Vec3) -> bool {
    let a = simplex.points[0];
    let b = simplex.points[1];
    let c = simplex.points[2];
    let ab = b.support - a.support;
    let ac = c.support - a.support;
    let ao = -a.support;
    let abc = ab.cross(&ac);

    if same_direction(&abc.cross(&ac), &ao) {
        if same_direction(&ac, &ao) {
            *simplex = Simplex::from_points(&[a, c]);
            *direction = ac.cross(&ao).cross(&ac);
        } else {
            *simplex = Simplex::from_points(&[a, b]);
            return solve_line(simplex, direction);
        }
    } else if same_direction(&ab.cross(&abc), &ao) {
        *simplex = Simplex::from_points(&[a, b]);
        return solve_line(simplex, direction);
    } else if same_direction(&abc, &ao) {
        *direction = abc;
    } else {
        *simplex = Simplex::from_points(&[a, c, b]);
        *direction = -abc;
    }
    false
}

/// handles the tetrahedron simplex case in GJK
fn solve_tetrahedron(simplex: &mut Simplex, direction: &mut glm::Vec3) -> bool {
    let a = simplex.points[0];
    let b = simplex.points[1];
    let c = simplex.points[2];
    let d = simplex.points[3];
    let ao = -a.support;
    let abc = outward_normal(a.support, b.support, c.support, d.support);
    let acd = outward_normal(a.support, c.support, d.support, b.support);
    let adb = outward_normal(a.support, d.support, b.support, c.support);

    if same_direction(&abc, &ao) {
        *simplex = Simplex::from_points(&[a, b, c]);
        return solve_triangle(simplex, direction);
    }
    if same_direction(&acd, &ao) {
        *simplex = Simplex::from_points(&[a, c, d]);
        return solve_triangle(simplex, direction);
    }
    if same_direction(&adb, &ao) {
        *simplex = Simplex::from_points(&[a, d, b]);
        return solve_triangle(simplex, direction);
    }
    true
}

/// computes the support data of the Minkowski difference of the two colliders in a given direction
fn support(
    collider1: &impl ColliderSpec,
    collider2: &impl ColliderSpec,
    direction: &glm::Vec3,
) -> SupportData {
    let fp1 = collider1.find_furthest_point(direction);
    let fp2 = collider2.find_furthest_point(&(-direction));
    SupportData {
        support: fp1 - fp2,
        fp1,
    }
}

/// support data of the Minkowski difference that contains both the support point and the furthest point of collider 1
#[derive(Debug, Default, Copy, Clone)]
struct SupportData {
    support: glm::Vec3,
    fp1: glm::Vec3,
}

/// computes the normal vector of a tetrahedron face in outward direction in the GJK algorithm
fn outward_normal(a: glm::Vec3, b: glm::Vec3, c: glm::Vec3, inward_point: glm::Vec3) -> glm::Vec3 {
    let normal = (b - a).cross(&(c - a));
    if !same_direction(&normal, &(a - inward_point)) {
        return -normal;
    }
    normal
}
