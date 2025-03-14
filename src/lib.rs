pub mod ecs;
pub mod engine;
pub mod engine_builder;
pub mod gui;
pub mod rendering;
pub mod systems;
pub mod utils;

pub use env_logger;
pub use itertools;
pub use log;
pub use nalgebra_glm as glm;
pub use petgraph;
pub use winit;

/// all features that are very common to use
pub mod prelude {
    pub use crate::components;
    pub use crate::ecs::component::utils::*;
    pub use crate::ecs::component::*;
    pub use crate::ecs::entity::EntityID;
    pub use crate::engine::{Engine, FallingLeafApp};
    pub use crate::engine_builder::EngineAttributes;
    pub use crate::glm;
    pub use crate::glm::vec2;
    pub use crate::glm::vec3;
    pub use crate::glm::vec4;
    pub use crate::glm::Mat3;
    pub use crate::glm::Mat4;
    pub use crate::glm::Quat;
    pub use crate::glm::Vec2;
    pub use crate::glm::Vec3;
    pub use crate::glm::Vec4;
    pub use crate::itertools::Itertools;
    pub use crate::systems::event_system::events::user_space::*;
    pub use crate::systems::event_system::events::*;
    pub use crate::utils::constants::*;
    pub use crate::utils::tools::*;
}

use bumpalo::boxed::Box as BumpBox;
use bumpalo::collections::Vec as BumpVec;
