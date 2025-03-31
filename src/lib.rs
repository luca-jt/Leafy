pub mod ecs;
pub mod engine;
pub mod engine_builder;
pub mod gui;
pub mod rendering;
pub mod systems;
pub mod utils;

pub use ahash;
pub use env_logger;
pub use itertools;
pub use log;
pub use nalgebra_glm as glm;
pub use petgraph;
pub use stb_image;
pub use winit;

/// All features that are very common to use.
pub mod prelude {
    pub use crate::components;
    pub use crate::ecs::component::utils::*;
    pub use crate::ecs::component::*;
    pub use crate::ecs::entity::EntityID;
    pub use crate::ecs::entity_manager::MeshHandle;
    pub use crate::engine::{Engine, EngineMode, FallingLeafApp};
    pub use crate::engine_builder::EngineAttributes;
    pub use crate::exclude_filter;
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
    pub use crate::include_filter;
    pub use crate::itertools::Itertools;
    pub use crate::log;
    pub use crate::systems::event_system::events::user_space::*;
    pub use crate::systems::event_system::events::*;
    pub use crate::systems::rendering_system::ShadowResolution;
    pub use crate::utils::constants::*;
    pub use crate::utils::tools::*;
    pub use gl::types::*;
    pub use winit::window::Theme;
}

/// Common internally used names.
pub(crate) mod internal_prelude {
    pub(crate) use crate::prelude::*;
    pub(crate) use crate::utils::file::*;
    pub(crate) use ahash::{AHashMap, AHashSet};
    pub(crate) use bumpalo::boxed::Box as BumpBox;
    pub(crate) use bumpalo::collections::Vec as BumpVec;
    pub(crate) use bumpalo::Bump;
    pub(crate) use std::any::{type_name_of_val, Any, TypeId};
    pub(crate) use std::cell::{Cell, Ref, RefCell, RefMut, UnsafeCell};
    pub(crate) use std::error::Error;
    pub(crate) use std::fmt::Debug;
    pub(crate) use std::marker::PhantomData;
    pub(crate) use std::ops::{Deref, DerefMut};
    pub(crate) use std::path::{Path, PathBuf};
    pub(crate) use std::rc::{Rc, Weak};
    pub(crate) use std::sync::{LazyLock, Mutex};
    pub(crate) use std::time::{Duration, Instant};
}
