use crate::ecs::component::Acceleration;
use crate::ecs::entity::EntityID;
use crate::glm;

pub(crate) const WIN_TITLE: &str = "Falling Leaf";
pub(crate) const MIN_WIN_WIDTH: u32 = 800;
pub(crate) const MIN_WIN_HEIGHT: u32 = 450;

pub(crate) const MAX_TEXTURE_COUNT: usize = 32;
pub(crate) const MAX_LIGHT_SRC_COUNT: usize = 5;

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
        pub const COLLISION: u64 = 1;
    }
    /// flags that can be
    pub mod user_level {
        /// makes an entity invisible and skips the rendering process for it
        pub const INVISIBLE: u64 = 0;
        /// inidcates wether or not
        pub const FLOATING: u64 = 2;
    }
}
