use crate::ecs::component::utils::TimeDuration;
use crate::ecs::component::Acceleration;
use crate::ecs::entity::EntityID;
use crate::glm;

pub(crate) const WIN_TITLE: &str = "Falling Leaf";
pub(crate) const DEFAULT_WIN_WIDTH: u32 = 800;
pub(crate) const DEFAULT_WIN_HEIGHT: u32 = 450;

pub(crate) const MAX_TEXTURE_COUNT: usize = 32;
pub(crate) const MAX_DIR_LIGHT_MAPS: usize = 5;
pub(crate) const MAX_POINT_LIGHT_MAPS: usize = 5;
pub(crate) const MAX_POINT_LIGHT_COUNT: usize = 20; // includes the point lights with shadow maps
pub(crate) const SHADOW_MAP_COUNT: usize = MAX_POINT_LIGHT_MAPS + MAX_DIR_LIGHT_MAPS;
pub(crate) const AVAILABLE_REGULAR_TEXTURE_COUNT: usize = MAX_TEXTURE_COUNT - SHADOW_MAP_COUNT;

pub(crate) const NEAR_PLANE: f32 = 0.1;
pub(crate) const FAR_PLANE: f32 = 100.0;
pub(crate) const NEAR_PLANE_SPRITE: f32 = 1.0;
pub(crate) const FAR_PLANE_SPRITE: f32 = 2.0;

/// a single time step in the animation system
pub(crate) const TIME_STEP: TimeDuration = TimeDuration(0.001);

/// earth's gravity
pub const G: Acceleration = Acceleration::new(0.0, -9.81, 0.0);

pub const ORIGIN: glm::Vec3 = glm::Vec3::new(0.0, 0.0, 0.0);
pub const X_AXIS: glm::Vec3 = glm::Vec3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: glm::Vec3 = glm::Vec3::new(0.0, 1.0, 0.0);
pub const Z_AXIS: glm::Vec3 = glm::Vec3::new(0.0, 0.0, 1.0);

/// placeholder for an empty entity slot (will never point to an entity)
pub const NO_ENTITY: EntityID = 0;

/// the number of the LOD's available
pub const NUM_LODS: i32 = 5;

/// contains all built-in bit flag constants
pub mod bits {
    /// flags that are used for internal processing and should not be set manually, but can be read by anybody
    pub mod internal {
        /// is set when a collision with that entity occurs
        pub const COLLIDED: u64 = 1;
    }
    /// flags that can be set by the user to enable certain behavior in the engine
    pub mod user_level {
        /// makes an entity invisible and skips the rendering process for it
        pub const INVISIBLE: u64 = 0;
        /// inidcates wether or not the entity is effected by gravity
        pub const FLOATING: u64 = 2;
        /// makes an entity with a collider only register collisions, not respond to them physics-whise
        pub const IGNORING_COLLISION: u64 = 3;
        /// seperates entities at collision even whithout velocity component, also ignores dynamic collision responses when used on entities with velocity
        pub const STATIC_COLLISION: u64 = 4;
        /// enables the doppler effect audio pitch for an entity
        pub const DOPPLER_EFFECT: u64 = 5;
    }
}
