use crate::ecs::component::utils::TimeDuration;
use crate::ecs::component::Acceleration;
use crate::internal_prelude::*;

pub(crate) const WIN_TITLE: &str = "Falling Leaf";
pub(crate) const DEFAULT_WIN_WIDTH: u32 = 800;
pub(crate) const DEFAULT_WIN_HEIGHT: u32 = 450;

// rendering constants
pub(crate) const MAX_TEXTURE_COUNT: usize = 32;
pub(crate) const MAX_DIR_LIGHT_MAPS: usize = 5;
pub(crate) const MAX_POINT_LIGHT_MAPS: usize = 5;
pub(crate) const MAX_POINT_LIGHT_COUNT: usize = 20; // includes the point lights with shadow maps
pub(crate) const NEAR_PLANE: f32 = 0.1;
pub(crate) const FAR_PLANE: f32 = 100.0;
pub(crate) const FAR_PLANE_SPRITE: f32 = 2.0;
pub(crate) const SPRITE_LAYER_COUNT: usize = 10;

pub(crate) const COMPONENT_COLUMN_INIT_SIZE: usize = 100;
pub(crate) const ENTITY_TYPE_STACK_ALLOCATION: usize = 16;
pub(crate) const COMPONENT_STACK_ALLOCATION_BYTES: usize = 64;

/// a single time step in the animation system
pub(crate) const TIME_STEP: TimeDuration = TimeDuration(0.002);

/// Earth's gravity.
pub const G: Acceleration = Acceleration::new(0.0, -9.81, 0.0);

pub const ORIGIN: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const X_AXIS: Vec3 = Vec3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const Z_AXIS: Vec3 = Vec3::new(0.0, 0.0, 1.0);

/// Placeholder for an empty entity slot (will never point to an entity).
pub const NO_ENTITY: EntityID = 0;

/// The number of mesh LOD's available (includes the base mesh aside from simplified versions).
pub const NUM_LODS: i32 = 5;

/// Contains all built-in bit flag constants to be used with the ``EntityFlags`` component.
pub mod bits {
    /// Flags that are used for internal processing and should not be set manually, but can be read by anybody.
    pub mod internal {
        /// is set when a collision with that entity occurs
        pub const COLLIDED: u64 = 1;
    }
    /// Flags that can be set by the user to enable certain behavior in the engine.
    pub mod user_level {
        /// Makes an entity invisible and skips the rendering process for it.
        pub const INVISIBLE: u64 = 0;
        /// Inidcates wether or not the entity is effected by gravity.
        pub const FLOATING: u64 = 2;
        /// Makes an entity with a ``Collider`` component only register collisions, not respond to them physics-whise.
        pub const IGNORING_COLLISION: u64 = 3;
        /// Enables seperating entities at collision even without ``Velocity`` component. Also ignores dynamic collision responses when used on entities with velocity.
        pub const STATIC_COLLISION: u64 = 4;
        /// Enables the doppler effect audio pitch for an entity.
        pub const DOPPLER_EFFECT: u64 = 5;
        /// Enables object outlining in rendering with the respective color.
        pub const STENCIL_OUTLINE: u64 = 6;
        /// Makes an entity invisible and skips the rendering process for it, but keeps the renderer data.
        pub const INVISIBLE_CACHED: u64 = 7;
    }
}
