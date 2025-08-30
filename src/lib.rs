pub mod ecs;
pub mod engine;
pub mod engine_builder;
pub mod rendering;
pub mod systems;
pub mod utils;

pub use env_logger;
pub use itertools;
pub use log;
pub use nalgebra_glm as glm;
pub use smallvec;
pub use winit;

/// All features that are very common to use.
pub mod prelude {
    pub use crate::components;
    pub use crate::ecs::component::utils::*;
    pub use crate::ecs::component::*;
    pub use crate::ecs::entity::EntityID;
    pub use crate::ecs::entity_manager::MeshHandle;
    pub use crate::engine::{Engine, EngineMode, LeafyApp};
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
    pub use ahash::{AHashMap, AHashSet};
    pub use bumpalo::boxed::Box as BumpBox;
    pub use bumpalo::collections::Vec as BumpVec;
    pub use bumpalo::Bump as BumpArena;
    pub use gl::types::*;
    pub use smallvec::smallvec;
    pub use smallvec::smallvec_inline;
    pub use smallvec::SmallVec;
}

/// Common internally used names.
pub(crate) mod internal_prelude {
    pub(crate) use crate::prelude::*;
    pub(crate) use crate::utils::file::*;
    pub(crate) use std::any::{type_name, Any, TypeId};
    pub(crate) use std::cell::{Cell, Ref, RefCell, RefMut, UnsafeCell};
    pub(crate) use std::error::Error;
    pub(crate) use std::fmt::Debug;
    pub(crate) use std::marker::PhantomData;
    pub(crate) use std::ops::{Deref, DerefMut};
    pub(crate) use std::path::{Path, PathBuf};
    pub(crate) use std::rc::{Rc, Weak};
    pub(crate) use std::time::{Duration, Instant};
}

#[cfg(test)]
mod tests {
    use crate::ecs::entity_manager::EntityManager;
    use crate::prelude::*;

    #[test]
    fn entity_test() {
        #[allow(dead_code)]
        struct A(u16);
        impl Component for A {}

        #[allow(dead_code)]
        struct B(u16);
        impl Component for B {}

        #[allow(dead_code)]
        struct C(u16);
        impl Component for C {}

        #[allow(dead_code)]
        struct D(u16);
        impl Component for D {}

        let mut ecs = EntityManager::new();
        let a = ecs.create_entity(components!(A(42), B(42)));
        let x = ecs.create_entity(components!(A(42), B(42)));
        assert!(ecs.delete_entity(a));
        assert!(ecs.add_component(x, C(42)));
        assert!(ecs.add_component(x, D(42)));
        assert!(ecs.has_component::<D>(x));
        ecs.remove_component::<D>(x).unwrap();
        assert!(!ecs.has_component::<D>(x));
        assert!(ecs.has_component::<C>(x));
        assert_eq!(unsafe { ecs.query2::<&A, &B>((None, None)) }.count(), 1);
        assert!(ecs.delete_entity(x));
    }

    #[test]
    fn render_data_test() {
        let mut ecs = EntityManager::new();
        let _ = ecs.create_entity(components!(Position::origin(), DirectionalLight::default()));
        let _ = ecs.create_entity(components!(Position::origin()));
        let _ = ecs.create_entity(components!(Position::origin()));
        assert_eq!(
            unsafe { ecs.query1::<&DirectionalLight>((None, None)) }.count(),
            1
        );
        assert_eq!(unsafe { ecs.query1::<&Position>((None, None)) }.count(), 3);
    }
}
