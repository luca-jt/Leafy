use crate::ecs::entity::{Archetype, ArchetypeID};
use crate::ecs::entity_manager::{EntityManager, ECS};
use std::any::{Any, TypeId};
use std::collections::hash_map::{Values, ValuesMut};
use std::iter::Filter;
use std::marker::PhantomData;

/// filter functionality for any struct
pub trait QueryFilter {
    /// checks wether or not the filter accepts an archetype
    fn matches(&self, archetype: &Archetype) -> bool;
}

/// a query filter that requires components to be included in an entity
#[derive(Debug, Clone)]
pub struct IncludeFilter(pub Vec<TypeId>);

impl QueryFilter for IncludeFilter {
    fn matches(&self, archetype: &Archetype) -> bool {
        self.0
            .iter()
            .all(|&ty| archetype.components.contains_key(&ty))
    }
}

/// easy creation of a boxed include filter from given component types
#[macro_export]
macro_rules! include_filter {
    ($($T:ty),+) => {
        Box::new($crate::ecs::query::IncludeFilter(vec![$(std::any::TypeId::of::<$T>()), +]))
    };
}

/// a query filter that requires components to be excluded from an entity
#[derive(Debug, Clone)]
pub struct ExcludeFilter(pub Vec<TypeId>);

impl QueryFilter for ExcludeFilter {
    fn matches(&self, archetype: &Archetype) -> bool {
        self.0
            .iter()
            .all(|&ty| !archetype.components.contains_key(&ty))
    }
}

/// easy creation of a boxed exclude filter from given component types
#[macro_export]
macro_rules! exclude_filter {
    ($($T:ty),+) => {
        Box::new($crate::ecs::query::ExcludeFilter(vec![$(std::any::TypeId::of::<$T>()), +]))
    };
}

/// indexes into the first element of an expression parameter pack
macro_rules! first {
    ($first:expr) => {
        $first
    };

    ($first:expr, $($rest:expr), +) => {
        $first
    };
}

/// makes shure the types of all the query entries are different
macro_rules! verify_types {
    ($t:ident) => { true };

    ($first:ident, $($rest:ident), +) => {
        $(std::any::TypeId::of::<$first>() != std::any::TypeId::of::<$rest>()) && + && verify_types!($($rest), +)
    };
}

macro_rules! impl_ref_query {
    ($sname:ident; $fname:ident; $($ret:ident), +; $($ret_opt:ident), *) => {
        pub struct $sname<'a, $($ret: Any), +, $($ret_opt: Any), *> {
            archetype_iter: Filter<Values<'a, ArchetypeID, Archetype>, fn(&&Archetype) -> bool>,
            current_archetype: Option<&'a Archetype>,
            component_index: usize,
            filter: Vec<Box<dyn QueryFilter>>,
            phantom: PhantomData<($($ret), +, $($ret_opt), *)>,
        }

        impl<'a, $($ret: Any), +, $($ret_opt: Any), *> Iterator for $sname<'a, $($ret), +, $($ret_opt), *> {
            #[allow(unused_parens)]
            type Item = ($(&'a $ret), + $(, Option<&'a $ret_opt>) *);

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(archetype) = self.current_archetype {
                    if self.component_index < archetype.components.get(first!($(&TypeId::of::<$ret>()), +)).unwrap().len() {
                        let ret = (
                            $(
                                archetype.components.get(&TypeId::of::<$ret>()).unwrap()[self.component_index].downcast_ref::<$ret>().unwrap()
                            ),+
                            $(
                                , archetype.component_ref_at::<$ret_opt>(self.component_index)
                            )*
                        );
                        self.component_index += 1;
                        return Some(ret);
                    }
                }
                if let Some(archetype) = self.archetype_iter.next() {
                    if self.filter.iter().all(|filter| filter.matches(archetype)) {
                        self.current_archetype = Some(archetype);
                        self.component_index = 0;
                    }
                    return self.next();
                }
                None
            }
        }

        impl ECS {
            pub(crate) fn $fname<$($ret: Any), +, $($ret_opt: Any), *>(
                &self,
                filter: Vec<Box<dyn QueryFilter>>
            ) -> $sname<'_, $($ret), +, $($ret_opt), *> {
                $sname {
                    archetype_iter: self.archetypes.values().filter(|archetype| {$(archetype.contains::<$ret>()) && +}),
                    current_archetype: None,
                    component_index: 0,
                    filter,
                    phantom: PhantomData,
                }
            }
        }

        impl EntityManager {
            #[doc = "immutable query for n components with m optionals"]
            pub fn $fname<$($ret: Any), +, $($ret_opt: Any), *>(
                &self,
                filter: Vec<Box<dyn QueryFilter>>,
            ) -> $sname<'_, $($ret), +, $($ret_opt), *> {
                self.ecs.$fname::<$($ret), +, $($ret_opt), *>(filter)
            }
        }
    };
}

macro_rules! impl_mut_query {
    ($sname:ident; $fname:ident; $($ret:ident), +; $($ret_opt:ident), *) => {
        pub struct $sname<'a, $($ret: Any), +, $($ret_opt: Any), *> {
            archetype_iter: Filter<ValuesMut<'a, ArchetypeID, Archetype>, fn(&&mut Archetype) -> bool>,
            current_archetype: Option<*mut Archetype>,
            component_index: usize,
            filter: Vec<Box<dyn QueryFilter>>,
            phantom: PhantomData<($($ret), +, $($ret_opt), *)>,
        }

        impl<'a, $($ret: Any), +, $($ret_opt: Any), *> Iterator for $sname<'a, $($ret), +, $($ret_opt), *> {
            #[allow(unused_parens)]
            type Item = ($(&'a mut $ret), + $(, Option<&'a mut $ret_opt>) *);

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(archetype) = self.current_archetype {
                    // SAFETY: only one query can exist at a time and the raw pointer is
                    // only used for tracking the current iteration
                    // debug assert in ECS function prohibits multiple mutable references to the same component
                    if self.component_index < unsafe { (*archetype).components.get(first!($(&TypeId::of::<$ret>()), +)).unwrap().len() } {
                        let ret = (
                            $(
                                unsafe { (*archetype).components.get_mut(&TypeId::of::<$ret>()).unwrap()[self.component_index].downcast_mut::<$ret>().unwrap() }
                            ),+
                            $(
                                , unsafe { (*archetype).component_mut_at::<$ret_opt>(self.component_index) }
                            )*
                        );
                        self.component_index += 1;
                        return Some(ret);
                    }
                }
                if let Some(archetype) = self.archetype_iter.next() {
                    if self.filter.iter().all(|filter| filter.matches(archetype)) {
                        self.current_archetype = Some(archetype);
                        self.component_index = 0;
                    }
                    return self.next();
                }
                None
            }
        }

        impl ECS {
            pub(crate) fn $fname<$($ret: Any), +, $($ret_opt: Any), *>(
                &mut self,
                filter: Vec<Box<dyn QueryFilter>>,
            ) -> $sname<'_, $($ret), +, $($ret_opt), *> {
                debug_assert!(verify_types!($($ret), + $(, $ret_opt) *));
                $sname {
                    archetype_iter: self.archetypes.values_mut().filter(|archetype| {$(archetype.contains::<$ret>()) && +}),
                    current_archetype: None,
                    component_index: 0,
                    filter,
                    phantom: PhantomData,
                }
            }
        }

        impl EntityManager {
            #[doc = "mutable query for n components with m optionals"]
            #[doc = "### Warning"]
            #[doc = "If the query modifies components that influence what asset data is loaded"]
            #[doc = "(e.g ``Scale``, ``MeshType``, ``MeshAttribute``, ``RigidBody``, ``HitboxType``),"]
            #[doc = "the asset data is not updated automatically."]
            #[doc = "Recomputes in every query are expensive and components in mutable queries are not necessarily modified."]
            #[doc = "If you still do that you will end up with crashes or undefined behavior."]
            #[doc = "To avoid this, you can manually trigger a full recompute of all entities after you executed the query using ``full_recompute()``."]
            #[doc = "Currently, this is the only way to handle this. Keep in mind that this is a big performance hit."]
            #[doc = "This should not be a real problem most of the time, as all components that influence what asset data is loaded are typically quite static."]
            #[doc = "E.g. ``Scale`` only influences inertia values of ``RigidBody``'s, which means modifying a scale in a query alone doesn't require a recompute"]
            #[doc = "and modifying a ``MeshAttribute`` in a query only matters if there is texture data at play."]
            #[doc = "### Unsafe"]
            #[doc = "If you use the same component type twice in the same query, it is possible to aquire two mutable references to the same component,"]
            #[doc = "which harms Rust's borrowing rules. In debug builds the query panics if you do that."]
            #[doc = "In release builds these checks are disabled for performance reasons."]
            #[doc = "At the moment all queries borrow the entire entity manager and it is common to use other entity manager functions inside of queries."]
            #[doc = "For this reason this function is marked as ``unsafe`` until this is changed in the future, as there would be borrowing issues otherwhise."]
            pub unsafe fn $fname<$($ret: Any), +, $($ret_opt: Any), *>(
                &self,
                filter: Vec<Box<dyn QueryFilter>>,
            ) -> $sname<'_, $($ret), +, $($ret_opt), *> {
                (*(&self.ecs as *const ECS as *mut ECS)).$fname::<$($ret), +, $($ret_opt), *>(filter)
            }
        }
    };
}

impl_ref_query!(Query1;          query1;            A; );
impl_ref_query!(Query2;          query2;            A, B; );
impl_ref_query!(Query2Opt1;      query2_opt1;       A; B);
impl_ref_query!(Query3;          query3;            A, B, C; );
impl_ref_query!(Query3Opt1;      query3_opt1;       A, B; C);
impl_ref_query!(Query3Opt2;      query3_opt2;       A; B, C);
impl_ref_query!(Query4;          query4;            A, B, C, D; );
impl_ref_query!(Query4Opt1;      query4_opt1;       A, B, C; D);
impl_ref_query!(Query4Opt2;      query4_opt2;       A, B; C, D);
impl_ref_query!(Query4Opt3;      query4_opt3;       A; B, C, D);
impl_ref_query!(Query5;          query5;            A, B, C, D, E; );
impl_ref_query!(Query5Opt1;      query5_opt1;       A, B, C, D; E);
impl_ref_query!(Query5Opt2;      query5_opt2;       A, B, C; D, E);
impl_ref_query!(Query5Opt3;      query5_opt3;       A, B; C, D, E);
impl_ref_query!(Query5Opt4;      query5_opt4;       A; B, C, D, E);
impl_ref_query!(Query6;          query6;            A, B, C, D, E, F; );
impl_ref_query!(Query6Opt1;      query6_opt1;       A, B, C, D, E; F);
impl_ref_query!(Query6Opt2;      query6_opt2;       A, B, C, D; E, F);
impl_ref_query!(Query6Opt3;      query6_opt3;       A, B, C; D, E, F);
impl_ref_query!(Query6Opt4;      query6_opt4;       A, B; C, D, E, F);
impl_ref_query!(Query6Opt5;      query6_opt5;       A; B, C, D, E, F);
impl_ref_query!(Query7;          query7;            A, B, C, D, E, F, G; );
impl_ref_query!(Query7Opt1;      query7_opt1;       A, B, C, D, E, F; G);
impl_ref_query!(Query7Opt2;      query7_opt2;       A, B, C, D, E; F, G);
impl_ref_query!(Query7Opt3;      query7_opt3;       A, B, C, D; E, F, G);
impl_ref_query!(Query7Opt4;      query7_opt4;       A, B, C; D, E, F, G);
impl_ref_query!(Query7Opt5;      query7_opt5;       A, B; C, D, E, F, G);
impl_ref_query!(Query7Opt6;      query7_opt6;       A; B, C, D, E, F, G);
impl_ref_query!(Query8;          query8;            A, B, C, D, E, F, G, H; );
impl_ref_query!(Query8Opt1;      query8_opt1;       A, B, C, D, E, F, G; H);
impl_ref_query!(Query8Opt2;      query8_opt2;       A, B, C, D, E, F; G, H);
impl_ref_query!(Query8Opt3;      query8_opt3;       A, B, C, D, E; F, G, H);
impl_ref_query!(Query8Opt4;      query8_opt4;       A, B, C, D; E, F, G, H);
impl_ref_query!(Query8Opt5;      query8_opt5;       A, B, C; D, E, F, G, H);
impl_ref_query!(Query8Opt6;      query8_opt6;       A, B; C, D, E, F, G, H);
impl_ref_query!(Query8Opt7;      query8_opt7;       A; B, C, D, E, F, G, H);
impl_ref_query!(Query9;          query9;            A, B, C, D, E, F, G, H, I; );
impl_ref_query!(Query9Opt1;      query9_opt1;       A, B, C, D, E, F, G, H; I);
impl_ref_query!(Query9Opt2;      query9_opt2;       A, B, C, D, E, F, G; H, I);
impl_ref_query!(Query9Opt3;      query9_opt3;       A, B, C, D, E, F; G, H, I);
impl_ref_query!(Query9Opt4;      query9_opt4;       A, B, C, D, E; F, G, H, I);
impl_ref_query!(Query9Opt5;      query9_opt5;       A, B, C, D; E, F, G, H, I);
impl_ref_query!(Query9Opt6;      query9_opt6;       A, B, C; D, E, F, G, H, I);
impl_ref_query!(Query9Opt7;      query9_opt7;       A, B; C, D, E, F, G, H, I);
impl_ref_query!(Query9Opt8;      query9_opt8;       A; B, C, D, E, F, G, H, I);
impl_ref_query!(Query10;         query10;           A, B, C, D, E, F, G, H, I, J; );
impl_ref_query!(Query10Opt1;     query10_opt1;      A, B, C, D, E, F, G, H, I; J);
impl_ref_query!(Query10Opt2;     query10_opt2;      A, B, C, D, E, F, G, H; I, J);
impl_ref_query!(Query10Opt3;     query10_opt3;      A, B, C, D, E, F, G; H, I, J);
impl_ref_query!(Query10Opt4;     query10_opt4;      A, B, C, D, E, F; G, H, I, J);
impl_ref_query!(Query10Opt5;     query10_opt5;      A, B, C, D, E; F, G, H, I, J);
impl_ref_query!(Query10Opt6;     query10_opt6;      A, B, C, D; E, F, G, H, I, J);
impl_ref_query!(Query10Opt7;     query10_opt7;      A, B, C; D, E, F, G, H, I, J);
impl_ref_query!(Query10Opt8;     query10_opt8;      A, B; C, D, E, F, G, H, I, J);
impl_ref_query!(Query10Opt9;     query10_opt9;      A; B, C, D, E, F, G, H, I, J);
impl_ref_query!(Query11;         query11;           A, B, C, D, E, F, G, H, I, J, K; );
impl_ref_query!(Query11Opt1;     query11_opt1;      A, B, C, D, E, F, G, H, I, J; K);
impl_ref_query!(Query11Opt2;     query11_opt2;      A, B, C, D, E, F, G, H, I; J, K);
impl_ref_query!(Query11Opt3;     query11_opt3;      A, B, C, D, E, F, G, H; I, J, K);
impl_ref_query!(Query11Opt4;     query11_opt4;      A, B, C, D, E, F, G; H, I, J, K);
impl_ref_query!(Query11Opt5;     query11_opt5;      A, B, C, D, E, F; G, H, I, J, K);
impl_ref_query!(Query11Opt6;     query11_opt6;      A, B, C, D, E; F, G, H, I, J, K);
impl_ref_query!(Query11Opt7;     query11_opt7;      A, B, C, D; E, F, G, H, I, J, K);
impl_ref_query!(Query11Opt8;     query11_opt8;      A, B, C; D, E, F, G, H, I, J, K);
impl_ref_query!(Query11Opt9;     query11_opt9;      A, B; C, D, E, F, G, H, I, J, K);
impl_ref_query!(Query11Opt10;    query11_opt10;     A; B, C, D, E, F, G, H, I, J, K);
impl_ref_query!(Query12;         query12;           A, B, C, D, E, F, G, H, I, J, K, L; );
impl_ref_query!(Query12Opt1;     query12_opt1;      A, B, C, D, E, F, G, H, I, J, K; L);
impl_ref_query!(Query12Opt2;     query12_opt2;      A, B, C, D, E, F, G, H, I, J; K, L);
impl_ref_query!(Query12Opt3;     query12_opt3;      A, B, C, D, E, F, G, H, I; J, K, L);
impl_ref_query!(Query12Opt4;     query12_opt4;      A, B, C, D, E, F, G, H; I, J, K, L);
impl_ref_query!(Query12Opt5;     query12_opt5;      A, B, C, D, E, F, G; H, I, J, K, L);
impl_ref_query!(Query12Opt6;     query12_opt6;      A, B, C, D, E, F; G, H, I, J, K, L);
impl_ref_query!(Query12Opt7;     query12_opt7;      A, B, C, D, E; F, G, H, I, J, K, L);
impl_ref_query!(Query12Opt8;     query12_opt8;      A, B, C, D; E, F, G, H, I, J, K, L);
impl_ref_query!(Query12Opt9;     query12_opt9;      A, B, C; D, E, F, G, H, I, J, K, L);
impl_ref_query!(Query12Opt10;    query12_opt10;     A, B; C, D, E, F, G, H, I, J, K, L);
impl_ref_query!(Query12Opt11;    query12_opt11;     A; B, C, D, E, F, G, H, I, J, K, L);
impl_ref_query!(Query13;         query13;           A, B, C, D, E, F, G, H, I, J, K, L, M; );
impl_ref_query!(Query13Opt1;     query13_opt1;      A, B, C, D, E, F, G, H, I, J, K, L; M);
impl_ref_query!(Query13Opt2;     query13_opt2;      A, B, C, D, E, F, G, H, I, J, K; L, M);
impl_ref_query!(Query13Opt3;     query13_opt3;      A, B, C, D, E, F, G, H, I, J; K, L, M);
impl_ref_query!(Query13Opt4;     query13_opt4;      A, B, C, D, E, F, G, H, I; J, K, L, M);
impl_ref_query!(Query13Opt5;     query13_opt5;      A, B, C, D, E, F, G, H; I, J, K, L, M);
impl_ref_query!(Query13Opt6;     query13_opt6;      A, B, C, D, E, F, G; H, I, J, K, L, M);
impl_ref_query!(Query13Opt7;     query13_opt7;      A, B, C, D, E, F; G, H, I, J, K, L, M);
impl_ref_query!(Query13Opt8;     query13_opt8;      A, B, C, D, E; F, G, H, I, J, K, L, M);
impl_ref_query!(Query13Opt9;     query13_opt9;      A, B, C, D; E, F, G, H, I, J, K, L, M);
impl_ref_query!(Query13Opt10;    query13_opt10;     A, B, C; D, E, F, G, H, I, J, K, L, M);
impl_ref_query!(Query13Opt11;    query13_opt11;     A, B; C, D, E, F, G, H, I, J, K, L, M);
impl_ref_query!(Query13Opt12;    query13_opt12;     A; B, C, D, E, F, G, H, I, J, K, L, M);
impl_ref_query!(Query14;         query14;           A, B, C, D, E, F, G, H, I, J, K, L, M, N; );
impl_ref_query!(Query14Opt1;     query14_opt1;      A, B, C, D, E, F, G, H, I, J, K, L, M; N);
impl_ref_query!(Query14Opt2;     query14_opt2;      A, B, C, D, E, F, G, H, I, J, K, L; M, N);
impl_ref_query!(Query14Opt3;     query14_opt3;      A, B, C, D, E, F, G, H, I, J, K; L, M, N);
impl_ref_query!(Query14Opt4;     query14_opt4;      A, B, C, D, E, F, G, H, I, J; K, L, M, N);
impl_ref_query!(Query14Opt5;     query14_opt5;      A, B, C, D, E, F, G, H, I; J, K, L, M, N);
impl_ref_query!(Query14Opt6;     query14_opt6;      A, B, C, D, E, F, G, H; I, J, K, L, M, N);
impl_ref_query!(Query14Opt7;     query14_opt7;      A, B, C, D, E, F, G; H, I, J, K, L, M, N);
impl_ref_query!(Query14Opt8;     query14_opt8;      A, B, C, D, E, F; G, H, I, J, K, L, M, N);
impl_ref_query!(Query14Opt9;     query14_opt9;      A, B, C, D, E; F, G, H, I, J, K, L, M, N);
impl_ref_query!(Query14Opt10;    query14_opt10;     A, B, C, D; E, F, G, H, I, J, K, L, M, N);
impl_ref_query!(Query14Opt11;    query14_opt11;     A, B, C; D, E, F, G, H, I, J, K, L, M, N);
impl_ref_query!(Query14Opt12;    query14_opt12;     A, B; C, D, E, F, G, H, I, J, K, L, M, N);
impl_ref_query!(Query14Opt13;    query14_opt13;     A; B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_ref_query!(Query15;         query15;           A, B, C, D, E, F, G, H, I, J, K, L, M, N, O; );
impl_ref_query!(Query15Opt1;     query15_opt1;      A, B, C, D, E, F, G, H, I, J, K, L, M, N; O);
impl_ref_query!(Query15Opt2;     query15_opt2;      A, B, C, D, E, F, G, H, I, J, K, L, M; N, O);
impl_ref_query!(Query15Opt3;     query15_opt3;      A, B, C, D, E, F, G, H, I, J, K, L; M, N, O);
impl_ref_query!(Query15Opt4;     query15_opt4;      A, B, C, D, E, F, G, H, I, J, K; L, M, N, O);
impl_ref_query!(Query15Opt5;     query15_opt5;      A, B, C, D, E, F, G, H, I, J; K, L, M, N, O);
impl_ref_query!(Query15Opt6;     query15_opt6;      A, B, C, D, E, F, G, H, I; J, K, L, M, N, O);
impl_ref_query!(Query15Opt7;     query15_opt7;      A, B, C, D, E, F, G, H; I, J, K, L, M, N, O);
impl_ref_query!(Query15Opt8;     query15_opt8;      A, B, C, D, E, F, G; H, I, J, K, L, M, N, O);
impl_ref_query!(Query15Opt9;     query15_opt9;      A, B, C, D, E, F; G, H, I, J, K, L, M, N, O);
impl_ref_query!(Query15Opt10;    query15_opt10;     A, B, C, D, E; F, G, H, I, J, K, L, M, N, O);
impl_ref_query!(Query15Opt11;    query15_opt11;     A, B, C, D; E, F, G, H, I, J, K, L, M, N, O);
impl_ref_query!(Query15Opt12;    query15_opt12;     A, B, C; D, E, F, G, H, I, J, K, L, M, N, O);
impl_ref_query!(Query15Opt13;    query15_opt13;     A, B; C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_ref_query!(Query15Opt14;    query15_opt14;     A; B, C, D, E, F, G, H, I, J, K, L, M, N, O);

impl_mut_query!(Query1Mut;       query1_mut;        A; );
impl_mut_query!(Query2Mut;       query2_mut;        A, B; );
impl_mut_query!(Query2MutOpt1;   query2_mut_opt1;   A; B);
impl_mut_query!(Query3Mut;       query3_mut;        A, B, C; );
impl_mut_query!(Query3MutOpt1;   query3_mut_opt1;   A, B; C);
impl_mut_query!(Query3MutOpt2;   query3_mut_opt2;   A; B, C);
impl_mut_query!(Query4Mut;       query4_mut;        A, B, C, D; );
impl_mut_query!(Query4MutOpt1;   query4_mut_opt1;   A, B, C; D);
impl_mut_query!(Query4MutOpt2;   query4_mut_opt2;   A, B; C, D);
impl_mut_query!(Query4MutOpt3;   query4_mut_opt3;   A; B, C, D);
impl_mut_query!(Query5Mut;       query5_mut;        A, B, C, D, E; );
impl_mut_query!(Query5MutOpt1;   query5_mut_opt1;   A, B, C, D; E);
impl_mut_query!(Query5MutOpt2;   query5_mut_opt2;   A, B, C; D, E);
impl_mut_query!(Query5MutOpt3;   query5_mut_opt3;   A, B; C, D, E);
impl_mut_query!(Query5MutOpt4;   query5_mut_opt4;   A; B, C, D, E);
impl_mut_query!(Query6Mut;       query6_mut;        A, B, C, D, E, F; );
impl_mut_query!(Query6MutOpt1;   query6_mut_opt1;   A, B, C, D, E; F);
impl_mut_query!(Query6MutOpt2;   query6_mut_opt2;   A, B, C, D; E, F);
impl_mut_query!(Query6MutOpt3;   query6_mut_opt3;   A, B, C; D, E, F);
impl_mut_query!(Query6MutOpt4;   query6_mut_opt4;   A, B; C, D, E, F);
impl_mut_query!(Query6MutOpt5;   query6_mut_opt5;   A; B, C, D, E, F);
impl_mut_query!(Query7Mut;       query7_mut;        A, B, C, D, E, F, G; );
impl_mut_query!(Query7MutOpt1;   query7_mut_opt1;   A, B, C, D, E, F; G);
impl_mut_query!(Query7MutOpt2;   query7_mut_opt2;   A, B, C, D, E; F, G);
impl_mut_query!(Query7MutOpt3;   query7_mut_opt3;   A, B, C, D; E, F, G);
impl_mut_query!(Query7MutOpt4;   query7_mut_opt4;   A, B, C; D, E, F, G);
impl_mut_query!(Query7MutOpt5;   query7_mut_opt5;   A, B; C, D, E, F, G);
impl_mut_query!(Query7MutOpt6;   query7_mut_opt6;   A; B, C, D, E, F, G);
impl_mut_query!(Query8Mut;       query8_mut;        A, B, C, D, E, F, G, H; );
impl_mut_query!(Query8MutOpt1;   query8_mut_opt1;   A, B, C, D, E, F, G; H);
impl_mut_query!(Query8MutOpt2;   query8_mut_opt2;   A, B, C, D, E, F; G, H);
impl_mut_query!(Query8MutOpt3;   query8_mut_opt3;   A, B, C, D, E; F, G, H);
impl_mut_query!(Query8MutOpt4;   query8_mut_opt4;   A, B, C, D; E, F, G, H);
impl_mut_query!(Query8MutOpt5;   query8_mut_opt5;   A, B, C; D, E, F, G, H);
impl_mut_query!(Query8MutOpt6;   query8_mut_opt6;   A, B; C, D, E, F, G, H);
impl_mut_query!(Query8MutOpt7;   query8_mut_opt7;   A; B, C, D, E, F, G, H);
impl_mut_query!(Query9Mut;       query9_mut;        A, B, C, D, E, F, G, H, I; );
impl_mut_query!(Query9MutOpt1;   query9_mut_opt1;   A, B, C, D, E, F, G, H; I);
impl_mut_query!(Query9MutOpt2;   query9_mut_opt2;   A, B, C, D, E, F, G; H, I);
impl_mut_query!(Query9MutOpt3;   query9_mut_opt3;   A, B, C, D, E, F; G, H, I);
impl_mut_query!(Query9MutOpt4;   query9_mut_opt4;   A, B, C, D, E; F, G, H, I);
impl_mut_query!(Query9MutOpt5;   query9_mut_opt5;   A, B, C, D; E, F, G, H, I);
impl_mut_query!(Query9MutOpt6;   query9_mut_opt6;   A, B, C; D, E, F, G, H, I);
impl_mut_query!(Query9MutOpt7;   query9_mut_opt7;   A, B; C, D, E, F, G, H, I);
impl_mut_query!(Query9MutOpt8;   query9_mut_opt8;   A; B, C, D, E, F, G, H, I);
impl_mut_query!(Query10Mut;      query10_mut;       A, B, C, D, E, F, G, H, I, J; );
impl_mut_query!(Query10MutOpt1;  query10_mut_opt1;  A, B, C, D, E, F, G, H, I; J);
impl_mut_query!(Query10MutOpt2;  query10_mut_opt2;  A, B, C, D, E, F, G, H; I, J);
impl_mut_query!(Query10MutOpt3;  query10_mut_opt3;  A, B, C, D, E, F, G; H, I, J);
impl_mut_query!(Query10MutOpt4;  query10_mut_opt4;  A, B, C, D, E, F; G, H, I, J);
impl_mut_query!(Query10MutOpt5;  query10_mut_opt5;  A, B, C, D, E; F, G, H, I, J);
impl_mut_query!(Query10MutOpt6;  query10_mut_opt6;  A, B, C, D; E, F, G, H, I, J);
impl_mut_query!(Query10MutOpt7;  query10_mut_opt7;  A, B, C; D, E, F, G, H, I, J);
impl_mut_query!(Query10MutOpt8;  query10_mut_opt8;  A, B; C, D, E, F, G, H, I, J);
impl_mut_query!(Query10MutOpt9;  query10_mut_opt9;  A; B, C, D, E, F, G, H, I, J);
impl_mut_query!(QueryMut11;      query11_mut;       A, B, C, D, E, F, G, H, I, J, K; );
impl_mut_query!(QueryMut11Opt1;  query11_mut_opt1;  A, B, C, D, E, F, G, H, I, J; K);
impl_mut_query!(QueryMut11Opt2;  query11_mut_opt2;  A, B, C, D, E, F, G, H, I; J, K);
impl_mut_query!(QueryMut11Opt3;  query11_mut_opt3;  A, B, C, D, E, F, G, H; I, J, K);
impl_mut_query!(QueryMut11Opt4;  query11_mut_opt4;  A, B, C, D, E, F, G; H, I, J, K);
impl_mut_query!(QueryMut11Opt5;  query11_mut_opt5;  A, B, C, D, E, F; G, H, I, J, K);
impl_mut_query!(QueryMut11Opt6;  query11_mut_opt6;  A, B, C, D, E; F, G, H, I, J, K);
impl_mut_query!(QueryMut11Opt7;  query11_mut_opt7;  A, B, C, D; E, F, G, H, I, J, K);
impl_mut_query!(QueryMut11Opt8;  query11_mut_opt8;  A, B, C; D, E, F, G, H, I, J, K);
impl_mut_query!(QueryMut11Opt9;  query11_mut_opt9;  A, B; C, D, E, F, G, H, I, J, K);
impl_mut_query!(QueryMut11Opt10; query11_mut_opt10; A; B, C, D, E, F, G, H, I, J, K);
impl_mut_query!(QueryMut12;      query12_mut;       A, B, C, D, E, F, G, H, I, J, K, L; );
impl_mut_query!(QueryMut12Opt1;  query12_mut_opt1;  A, B, C, D, E, F, G, H, I, J, K; L);
impl_mut_query!(QueryMut12Opt2;  query12_mut_opt2;  A, B, C, D, E, F, G, H, I, J; K, L);
impl_mut_query!(QueryMut12Opt3;  query12_mut_opt3;  A, B, C, D, E, F, G, H, I; J, K, L);
impl_mut_query!(QueryMut12Opt4;  query12_mut_opt4;  A, B, C, D, E, F, G, H; I, J, K, L);
impl_mut_query!(QueryMut12Opt5;  query12_mut_opt5;  A, B, C, D, E, F, G; H, I, J, K, L);
impl_mut_query!(QueryMut12Opt6;  query12_mut_opt6;  A, B, C, D, E, F; G, H, I, J, K, L);
impl_mut_query!(QueryMut12Opt7;  query12_mut_opt7;  A, B, C, D, E; F, G, H, I, J, K, L);
impl_mut_query!(QueryMut12Opt8;  query12_mut_opt8;  A, B, C, D; E, F, G, H, I, J, K, L);
impl_mut_query!(QueryMut12Opt9;  query12_mut_opt9;  A, B, C; D, E, F, G, H, I, J, K, L);
impl_mut_query!(QueryMut12Opt10; query12_mut_opt10; A, B; C, D, E, F, G, H, I, J, K, L);
impl_mut_query!(QueryMut12Opt11; query12_mut_opt11; A; B, C, D, E, F, G, H, I, J, K, L);
impl_mut_query!(QueryMut13;      query13_mut;       A, B, C, D, E, F, G, H, I, J, K, L, M; );
impl_mut_query!(QueryMut13Opt1;  query13_mut_opt1;  A, B, C, D, E, F, G, H, I, J, K, L; M);
impl_mut_query!(QueryMut13Opt2;  query13_mut_opt2;  A, B, C, D, E, F, G, H, I, J, K; L, M);
impl_mut_query!(QueryMut13Opt3;  query13_mut_opt3;  A, B, C, D, E, F, G, H, I, J; K, L, M);
impl_mut_query!(QueryMut13Opt4;  query13_mut_opt4;  A, B, C, D, E, F, G, H, I; J, K, L, M);
impl_mut_query!(QueryMut13Opt5;  query13_mut_opt5;  A, B, C, D, E, F, G, H; I, J, K, L, M);
impl_mut_query!(QueryMut13Opt6;  query13_mut_opt6;  A, B, C, D, E, F, G; H, I, J, K, L, M);
impl_mut_query!(QueryMut13Opt7;  query13_mut_opt7;  A, B, C, D, E, F; G, H, I, J, K, L, M);
impl_mut_query!(QueryMut13Opt8;  query13_mut_opt8;  A, B, C, D, E; F, G, H, I, J, K, L, M);
impl_mut_query!(QueryMut13Opt9;  query13_mut_opt9;  A, B, C, D; E, F, G, H, I, J, K, L, M);
impl_mut_query!(QueryMut13Opt10; query13_mut_opt10; A, B, C; D, E, F, G, H, I, J, K, L, M);
impl_mut_query!(QueryMut13Opt11; query13_mut_opt11; A, B; C, D, E, F, G, H, I, J, K, L, M);
impl_mut_query!(QueryMut13Opt12; query13_mut_opt12; A; B, C, D, E, F, G, H, I, J, K, L, M);
impl_mut_query!(QueryMut14;      query14_mut;       A, B, C, D, E, F, G, H, I, J, K, L, M, N; );
impl_mut_query!(QueryMut14Opt1;  query14_mut_opt1;  A, B, C, D, E, F, G, H, I, J, K, L, M; N);
impl_mut_query!(QueryMut14Opt2;  query14_mut_opt2;  A, B, C, D, E, F, G, H, I, J, K, L; M, N);
impl_mut_query!(QueryMut14Opt3;  query14_mut_opt3;  A, B, C, D, E, F, G, H, I, J, K; L, M, N);
impl_mut_query!(QueryMut14Opt4;  query14_mut_opt4;  A, B, C, D, E, F, G, H, I, J; K, L, M, N);
impl_mut_query!(QueryMut14Opt5;  query14_mut_opt5;  A, B, C, D, E, F, G, H, I; J, K, L, M, N);
impl_mut_query!(QueryMut14Opt6;  query14_mut_opt6;  A, B, C, D, E, F, G, H; I, J, K, L, M, N);
impl_mut_query!(QueryMut14Opt7;  query14_mut_opt7;  A, B, C, D, E, F, G; H, I, J, K, L, M, N);
impl_mut_query!(QueryMut14Opt8;  query14_mut_opt8;  A, B, C, D, E, F; G, H, I, J, K, L, M, N);
impl_mut_query!(QueryMut14Opt9;  query14_mut_opt9;  A, B, C, D, E; F, G, H, I, J, K, L, M, N);
impl_mut_query!(QueryMut14Opt10; query14_mut_opt10; A, B, C, D; E, F, G, H, I, J, K, L, M, N);
impl_mut_query!(QueryMut14Opt11; query14_mut_opt11; A, B, C; D, E, F, G, H, I, J, K, L, M, N);
impl_mut_query!(QueryMut14Opt12; query14_mut_opt12; A, B; C, D, E, F, G, H, I, J, K, L, M, N);
impl_mut_query!(QueryMut14Opt13; query14_mut_opt13; A; B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_mut_query!(QueryMut15;      query15_mut;       A, B, C, D, E, F, G, H, I, J, K, L, M, N, O; );
impl_mut_query!(QueryMut15Opt1;  query15_mut_opt1;  A, B, C, D, E, F, G, H, I, J, K, L, M, N; O);
impl_mut_query!(QueryMut15Opt2;  query15_mut_opt2;  A, B, C, D, E, F, G, H, I, J, K, L, M; N, O);
impl_mut_query!(QueryMut15Opt3;  query15_mut_opt3;  A, B, C, D, E, F, G, H, I, J, K, L; M, N, O);
impl_mut_query!(QueryMut15Opt4;  query15_mut_opt4;  A, B, C, D, E, F, G, H, I, J, K; L, M, N, O);
impl_mut_query!(QueryMut15Opt5;  query15_mut_opt5;  A, B, C, D, E, F, G, H, I, J; K, L, M, N, O);
impl_mut_query!(QueryMut15Opt6;  query15_mut_opt6;  A, B, C, D, E, F, G, H, I; J, K, L, M, N, O);
impl_mut_query!(QueryMut15Opt7;  query15_mut_opt7;  A, B, C, D, E, F, G, H; I, J, K, L, M, N, O);
impl_mut_query!(QueryMut15Opt8;  query15_mut_opt8;  A, B, C, D, E, F, G; H, I, J, K, L, M, N, O);
impl_mut_query!(QueryMut15Opt9;  query15_mut_opt9;  A, B, C, D, E, F; G, H, I, J, K, L, M, N, O);
impl_mut_query!(QueryMut15Opt10; query15_mut_opt10; A, B, C, D, E; F, G, H, I, J, K, L, M, N, O);
impl_mut_query!(QueryMut15Opt11; query15_mut_opt11; A, B, C, D; E, F, G, H, I, J, K, L, M, N, O);
impl_mut_query!(QueryMut15Opt12; query15_mut_opt12; A, B, C; D, E, F, G, H, I, J, K, L, M, N, O);
impl_mut_query!(QueryMut15Opt13; query15_mut_opt13; A, B; C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_mut_query!(QueryMut15Opt14; query15_mut_opt14; A; B, C, D, E, F, G, H, I, J, K, L, M, N, O);
