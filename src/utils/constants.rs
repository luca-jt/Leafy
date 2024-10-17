use crate::ecs::component::Acceleration;
use crate::glm;

pub const WIN_TITLE: &str = "Falling Leaf";
pub const MIN_WIN_WIDTH: u32 = 800;
pub const MIN_WIN_HEIGHT: u32 = 450;

pub const MAX_TEXTURE_COUNT: usize = 32;
pub const MAX_LIGHT_SRC_COUNT: usize = 5;

pub const G: Acceleration = Acceleration::new(0.0, -9.81, 0.0);

pub const ORIGIN: glm::Vec3 = glm::Vec3::new(0.0, 0.0, 0.0);
pub const X_AXIS: glm::Vec3 = glm::Vec3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: glm::Vec3 = glm::Vec3::new(0.0, 1.0, 0.0);
pub const Z_AXIS: glm::Vec3 = glm::Vec3::new(0.0, 0.0, 1.0);
