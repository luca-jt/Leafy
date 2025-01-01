use crate::ecs::component::utils::*;
use crate::ecs::component::*;
use crate::ecs::entity_manager::EntityManager;
use crate::engine::{Engine, EngineMode, FallingLeafApp};
use crate::glm;
use crate::rendering::data::calc_model_matrix;
use crate::rendering::mesh::Hitbox;
use crate::systems::event_system::events::*;
use crate::systems::event_system::EventObserver;
use crate::utils::constants::bits::internal::*;
use crate::utils::constants::bits::user_level::*;
use crate::utils::constants::{G, ORIGIN, X_AXIS, Y_AXIS};
use crate::utils::tools::{copied_or_default, normalize_non_zero, same_direction, to_vec4};
use itertools::Itertools;
use std::collections::HashSet;
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
        let dt = self.time_of_last_sim.delta_time();
        let transformed_dt = dt * self.animation_speed;
        self.time_accumulated += transformed_dt;
        while self.time_accumulated >= self.time_step_size {
            self.apply_physics(engine.entity_manager_mut().deref_mut(), self.time_step_size);
            self.time_accumulated -= self.time_step_size;
        }
        self.update_cam(engine, dt);
        self.handle_collisions(engine.entity_manager_mut().deref_mut());
        self.time_of_last_sim.reset();
    }

    /// checks for collision between entities with hitboxes and resolves them
    fn handle_collisions(&self, entity_manager: &mut EntityManager) {
        let mut entity_data = unsafe {
            entity_manager
                .query9_mut_opt6::<Position, Collider, MeshType, Velocity, AngularMomentum, Scale, RigidBody, EntityFlags, Orientation>(vec![])
                .map(|(p, hb, mt, v, am, s, rb, f, o)| {
                    (
                        p,
                        entity_manager.hitbox_from_data(mt, &hb.hitbox_type).unwrap(),
                        entity_manager.asset_from_type(mt, LOD::None).unwrap(),
                        hb,
                        v,
                        am,
                        s,
                        rb,
                        f.map(|flags| {
                            flags.set_bit(COLLISION, false);
                            flags
                        }),
                        o
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
                .collect_vec();

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

                    // rotations
                    let rot1 = copied_or_default(&entity_data[i].9);
                    let rot2 = copied_or_default(&entity_data[j].9);

                    //  translate the colliders to the positional offset of the hitbox
                    let collider_1 = ColliderData {
                        position: *entity_data[i].0,
                        scale: copied_or_default(&entity_data[i].6),
                        orientation: rot1,
                        center_of_mass: rb_1.center_of_mass,
                        hitbox: entity_data[i].1,
                        collider: entity_data[i].3,
                    };
                    let collider_2 = ColliderData {
                        position: *entity_data[j].0,
                        scale: copied_or_default(&entity_data[j].6),
                        orientation: rot2,
                        center_of_mass: rb_2.center_of_mass,
                        hitbox: entity_data[j].1,
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

                        // rotation matrices + local inertia matrices
                        let rotation_mat1 = glm::mat4_to_mat3(&rot1.rotation_matrix());
                        let rotation_mat2 = glm::mat4_to_mat3(&rot2.rotation_matrix());
                        let local_inertia_inv_1 =
                            rotation_mat1 * rb_1.inv_inertia_tensor * rotation_mat1.transpose();
                        let local_inertia_inv_2 =
                            rotation_mat2 * rb_2.inv_inertia_tensor * rotation_mat2.transpose();

                        // angular velocities
                        let av1 = local_inertia_inv_1 * copied_or_default(&entity_data[i].5).data();
                        let av2 = local_inertia_inv_2 * copied_or_default(&entity_data[j].5).data();

                        // relative velocity
                        let v_rel = copied_or_default(&entity_data[i].4).data()
                            + av1.cross(&mass_center_coll_point_1)
                            - copied_or_default(&entity_data[j].4).data()
                            - av2.cross(&mass_center_coll_point_2);

                        // components of the relative velocity
                        let normal_component = collison_data
                            .collision_normal
                            .dot(&normalize_non_zero(v_rel).unwrap_or(Y_AXIS))
                            * v_rel;
                        let tangential_component = v_rel - normal_component;

                        // other data
                        let coll_normal = collison_data.collision_normal;
                        let coll_tangent =
                            normalize_non_zero(tangential_component).unwrap_or(ORIGIN);

                        let restitution_coefficient = 0.0; // TODO: change that in the future with component data
                        let total_friction = rb_1.friction.min(rb_2.friction).clamp(0.0, 1.0);

                        // effective mass calculations
                        let eff_mass_n_component =
                            coll_normal.cross(&mass_center_coll_point_1).dot(
                                &(local_inertia_inv_1
                                    * coll_normal.cross(&mass_center_coll_point_1)),
                            ) + coll_normal.cross(&mass_center_coll_point_2).dot(
                                &(local_inertia_inv_2
                                    * coll_normal.cross(&mass_center_coll_point_2)),
                            );

                        let eff_mass_t_component =
                            coll_tangent.cross(&mass_center_coll_point_1).dot(
                                &(local_inertia_inv_1
                                    * coll_tangent.cross(&mass_center_coll_point_1)),
                            ) + coll_tangent.cross(&mass_center_coll_point_2).dot(
                                &(local_inertia_inv_2
                                    * coll_tangent.cross(&mass_center_coll_point_2)),
                            );

                        // resolve the collision depending on what enities are movable
                        // if only one is movable, treat the mass of the immovable entity as infinite
                        if is_dynamic_1 && is_dynamic_2 {
                            // both have velocity and therefore are movable

                            // seperate the two objects
                            *entity_data[i].0.data_mut() += -0.5 * collison_data.translation_vec;
                            *entity_data[j].0.data_mut() += 0.5 * collison_data.translation_vec;

                            // normal impulse
                            let effective_mass_n =
                                1.0 / rb_1.mass + 1.0 / rb_2.mass + eff_mass_n_component;
                            let normal_impulse = -((1.0 + restitution_coefficient)
                                * normal_component)
                                / effective_mass_n;

                            // tangential impulse
                            let effective_mass_t =
                                1.0 / rb_1.mass + 1.0 / rb_2.mass + eff_mass_t_component;
                            let tang_impulse = -((1.0 + restitution_coefficient)
                                * tangential_component)
                                / effective_mass_t
                                * total_friction;

                            // apply impulses
                            *entity_data[i].4.as_mut().unwrap().data_mut() +=
                                normal_impulse / rb_1.mass;
                            *entity_data[j].4.as_mut().unwrap().data_mut() +=
                                -normal_impulse / rb_2.mass;

                            if let Some(angular_mom) = entity_data[i].5.as_mut() {
                                *angular_mom.data_mut() += local_inertia_inv_1
                                    * mass_center_coll_point_1.cross(&tang_impulse)
                                    + local_inertia_inv_1
                                        * mass_center_coll_point_1.cross(&normal_impulse);
                            }
                            if let Some(angular_mom) = entity_data[j].5.as_mut() {
                                *angular_mom.data_mut() += local_inertia_inv_2
                                    * mass_center_coll_point_2.cross(&(-tang_impulse))
                                    + local_inertia_inv_2
                                        * mass_center_coll_point_2.cross(&(-normal_impulse));
                            }
                        } else if is_dynamic_1 {
                            // only 1 is movable
                            *entity_data[i].0.data_mut() += -collison_data.translation_vec;

                            // normal impulse
                            let effective_mass_n = 1.0 / rb_1.mass + eff_mass_n_component;
                            let normal_impulse = -((1.0 + restitution_coefficient)
                                * normal_component)
                                / effective_mass_n;

                            // tangential impulse
                            let effective_mass_t = 1.0 / rb_1.mass + eff_mass_t_component;
                            let tang_impulse = -((1.0 + restitution_coefficient)
                                * tangential_component)
                                / effective_mass_t
                                * total_friction;

                            // apply impulses
                            *entity_data[i].4.as_mut().unwrap().data_mut() +=
                                normal_impulse / rb_1.mass;

                            if let Some(angular_mom) = entity_data[i].5.as_mut() {
                                *angular_mom.data_mut() += local_inertia_inv_1
                                    * mass_center_coll_point_1.cross(&tang_impulse)
                                    + local_inertia_inv_1
                                        * mass_center_coll_point_1.cross(&normal_impulse);
                            }
                        } else if is_dynamic_2 {
                            // only 2 is movable
                            *entity_data[j].0.data_mut() += collison_data.translation_vec;

                            // normal impulse
                            let effective_mass_n = 1.0 / rb_2.mass + eff_mass_n_component;
                            let normal_impulse = -((1.0 + restitution_coefficient)
                                * normal_component)
                                / effective_mass_n;

                            // tangential impulse
                            let effective_mass_t = 1.0 / rb_2.mass + eff_mass_t_component;
                            let tang_impulse = -((1.0 + restitution_coefficient)
                                * tangential_component)
                                / effective_mass_t
                                * total_friction;

                            // apply impulses
                            *entity_data[j].4.as_mut().unwrap().data_mut() +=
                                -normal_impulse / rb_2.mass;

                            if let Some(angular_mom) = entity_data[j].5.as_mut() {
                                *angular_mom.data_mut() += local_inertia_inv_2
                                    * mass_center_coll_point_2.cross(&(-tang_impulse))
                                    + local_inertia_inv_2
                                        * mass_center_coll_point_2.cross(&(-normal_impulse));
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
        for (p, v, a_opt, rb_opt, o_opt, am_opt, flags) in unsafe {
            entity_manager
                .query7_mut_opt5::<Position, Velocity, Acceleration, RigidBody, Orientation, AngularMomentum, EntityFlags>(vec![])
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

            if let (Some(am), Some(o)) = (am_opt, o_opt) {
                let inv_inertia_mat = rb_opt.copied().unwrap_or_default().inv_inertia_tensor;
                let rot_mat = glm::mat4_to_mat3(&o.rotation_matrix());
                let local_inertia_mat = rot_mat * inv_inertia_mat * rot_mat.transpose();
                let ang_vel = local_inertia_mat * am.data();
                o.0 += 0.5 * o.0 * glm::quat(ang_vel.x, ang_vel.y, ang_vel.z, 0.0) * time_step.0;
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
    position: Position,
    scale: Scale,
    orientation: Orientation,
    center_of_mass: glm::Vec3,
    hitbox: &'a Hitbox,
    collider: &'a Collider,
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
        // assume the normal vector is form the point of view of 1
        // the normal vector in the collision data should be normalized
        if let Some(spec1) = self.mesh_spec() {
            if let Some(spec2) = other.mesh_spec() {
                gjk(spec1, spec2)
            } else {
                let spec2 = other.sphere_spec().unwrap();
                gjk(spec1, spec2)
            }
        } else {
            let spec1 = self.sphere_spec().unwrap();
            if let Some(spec2) = other.mesh_spec() {
                gjk(spec1, spec2)
            } else {
                let spec2 = other.sphere_spec().unwrap();
                gjk(spec1, spec2)
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
        let local_direction = (self.rotation_mat.transpose() * to_vec4(direction)).xyz();
        let stretch_factor = 1.0
            / ((local_direction.x * local_direction.x)
                / (self.scale_dimensions.x * self.scale_dimensions.x)
                + (local_direction.y * local_direction.y)
                    / (self.scale_dimensions.y * self.scale_dimensions.y)
                + (local_direction.z * local_direction.z)
                    / (self.scale_dimensions.z * self.scale_dimensions.z))
                .sqrt();
        let local_point = local_direction * stretch_factor;
        (self.rotation_mat * to_vec4(&local_point) + to_vec4(&self.collider_pos)).xyz()
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
            .map(|point| (self.transform * to_vec4(point)).xyz())
            .map(|point| (point, point.dot(direction)))
            .max_by(|(_, dot1), (_, dot2)| dot1.partial_cmp(dot2).unwrap())
            .unwrap()
            .0
    }
}

/// holds the data for the current simplex used in the GJK algorithm
#[derive(Debug)]
struct Simplex {
    points: [glm::Vec3; 4],
    size: usize,
}

impl Simplex {
    fn new() -> Self {
        Self {
            points: [ORIGIN, ORIGIN, ORIGIN, ORIGIN],
            size: 0,
        }
    }

    fn from_points(points: &[glm::Vec3]) -> Self {
        let mut instance = Self::new();
        for point in points {
            instance.push_front(*point);
        }
        instance
    }

    fn push_front(&mut self, element: glm::Vec3) {
        self.points = [element, self.points[0], self.points[1], self.points[2]];
        self.size = 4.min(self.size + 1)
    }
}

/// implementation of the GJK algorithm for detecting collisions
fn gjk(collider1: impl ColliderSpec, collider2: impl ColliderSpec) -> Option<CollisionData> {
    let mut supp = support(&collider1, &collider2, &X_AXIS);
    let mut simplex = Simplex::from_points(&[supp]);
    let mut direction = -supp;

    loop {
        supp = support(&collider1, &collider2, &direction);
        if !same_direction(&direction, &supp) {
            return None;
        }
        simplex.push_front(supp);
        if solve_simplex(&mut simplex, &mut direction) {
            // use the EPA algorithm in case of a collision
            return Some(epa(collider1, collider2, simplex));
        }
    }
}

/// implementation of the EPA algorithm for determining collision data in case of collision
fn epa(
    collider1: impl ColliderSpec,
    collider2: impl ColliderSpec,
    simplex: Simplex,
) -> CollisionData {
    let mut polytope = simplex.points.to_vec();
    let mut faces = (0..polytope.len())
        .tuple_combinations::<(usize, usize, usize)>()
        .collect_vec();

    // iteratively add new points to the polytope until the correct normal is found
    loop {
        let (face_idx, min_normal, min_distance) = find_min_data(&polytope, &faces);
        let supp = support(&collider1, &collider2, &min_normal);
        let supp_distance = supp.dot(&min_normal);

        if supp_distance - min_distance <= 0.001 {
            let face = faces[face_idx];
            let triangle = [polytope[face.0], polytope[face.1], polytope[face.2]];
            // return collision data
            return CollisionData {
                collision_normal: min_normal,
                collision_point: find_collision_point(triangle, min_normal),
                translation_vec: min_normal * (supp_distance + 0.001),
            };
        }

        polytope.push(supp);
        reconstruct_polytope(&polytope, &mut faces, supp);
    }
}

/// finds the collision point from the terminating config of the EPA algorithm
fn find_collision_point(triangle: [glm::Vec3; 3], normal: glm::Vec3) -> glm::Vec3 {
    let origin_proj = normal.dot(&triangle[0]) * normal;
    // check if the point is inside the triangle using barycentric coordinates
    let u = (triangle[1] - triangle[2])
        .cross(&(origin_proj - triangle[2]))
        .dot(&normal);
    let v = (triangle[2] - triangle[0])
        .cross(&(origin_proj - triangle[0]))
        .dot(&normal);
    let w = 1.0 - u - v;

    if u >= 0.0 && v >= 0.0 && w >= 0.0 {
        return origin_proj;
    }
    // else, find the closest point on the edges
    let p_edge1 = closest_point_on_segment_from_origin(triangle[0], triangle[1]);
    let p_edge2 = closest_point_on_segment_from_origin(triangle[0], triangle[2]);
    let p_edge3 = closest_point_on_segment_from_origin(triangle[1], triangle[2]);
    // return the closest point
    [p_edge1, p_edge2, p_edge3]
        .into_iter()
        .min_by(|p1, p2| p1.norm_squared().partial_cmp(&p2.norm_squared()).unwrap())
        .unwrap()
}

/// finds the closest point from the origin on a segment
fn closest_point_on_segment_from_origin(p1: glm::Vec3, p2: glm::Vec3) -> glm::Vec3 {
    let segment = p2 - p1;
    let t = (-p1).dot(&segment) / segment.norm_squared();
    let t_clamped = t.clamp(0.0, 1.0);
    p1 + segment * t_clamped
}

/// reconstructs the polytope after a new point was found in the EPA algorithm
fn reconstruct_polytope(
    polytope: &[glm::Vec3],
    faces: &mut Vec<(usize, usize, usize)>,
    supp: glm::Vec3,
) {
    // find faces to remove
    let mut faces_to_remove = faces
        .iter()
        .enumerate()
        .filter_map(|(i, face)| {
            if same_direction(
                &outward_normal(polytope[face.0], polytope[face.1], polytope[face.2], ORIGIN),
                &(supp - polytope[face.0]),
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
    polytope: &[glm::Vec3],
    faces: &[(usize, usize, usize)],
) -> (usize, glm::Vec3, f32) {
    faces
        .iter()
        .enumerate()
        .map(|(i, &(node1, node2, node3))| (i, [polytope[node1], polytope[node2], polytope[node3]]))
        .map(|(idx, vertices)| {
            let normal = outward_normal(vertices[0], vertices[1], vertices[2], ORIGIN).normalize();
            (idx, normal, normal.dot(&vertices[0]))
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
    let ab = b - a;
    let ao = -a;

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
    let ab = b - a;
    let ac = c - a;
    let ao = -a;
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
    let ao = -a;
    let abc = outward_normal(a, b, c, d);
    let acd = outward_normal(a, c, d, b);
    let adb = outward_normal(a, d, b, c);

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

/// computes the support point of the Minkowski difference of the two colliders in a given direction
fn support(
    collider1: &impl ColliderSpec,
    collider2: &impl ColliderSpec,
    direction: &glm::Vec3,
) -> glm::Vec3 {
    collider1.find_furthest_point(direction) - collider2.find_furthest_point(&(-direction))
}

/// computes the normal vector of a tetrahedron face in outward direction in the GJK algorithm
fn outward_normal(a: glm::Vec3, b: glm::Vec3, c: glm::Vec3, inward_point: glm::Vec3) -> glm::Vec3 {
    let normal = (b - a).cross(&(c - a));
    if !same_direction(&normal, &(a - inward_point)) {
        return -normal;
    }
    normal
}
