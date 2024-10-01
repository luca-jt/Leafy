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
pub struct IncludeFilter(pub(crate) Vec<TypeId>);

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
        Box::new($crate::ecs::query::IncludeFilter(vec![$(TypeId::of<$T>()), +]))
    };
}

/// a query filter that requires components to be excluded from an entity
#[derive(Debug, Clone)]
pub struct ExcludeFilter(pub(crate) Vec<TypeId>);

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
        Box::new($crate::ecs::query::ExcludeFilter(vec![$(TypeId::of<$T>()), +]))
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
        $(TypeId::of::<$first>() != TypeId::of::<$rest>()) && + && verify_types!($($rest), +)
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
                debug_assert!(verify_types!($($ret), +));
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
            pub fn $fname<$($ret: Any), +, $($ret_opt: Any), *>(
                &mut self,
                filter: Vec<Box<dyn QueryFilter>>,
            ) -> $sname<'_, $($ret), +, $($ret_opt), *> {
                self.ecs.$fname::<$($ret), +, $($ret_opt), *>(filter)
            }
        }
    };
}

impl_ref_query!(Query1;         query1;             A; );
impl_ref_query!(Query2;         query2;             A, B; );
impl_ref_query!(Query2Opt1;     query2_opt1;        A; B);
impl_ref_query!(Query3;         query3;             A, B, C; );
impl_ref_query!(Query3Opt1;     query3_opt1;        A, B; C);
impl_ref_query!(Query3Opt2;     query3_opt2;        A; B, C);
impl_ref_query!(Query4;         query4;             A, B, C, D; );
impl_ref_query!(Query4Opt1;     query4_opt1;        A, B, C; D);
impl_ref_query!(Query4Opt2;     query4_opt2;        A, B; C, D);
impl_ref_query!(Query4Opt3;     query4_opt3;        A; B, C, D);
impl_ref_query!(Query5;         query5;             A, B, C, D, E; );
impl_ref_query!(Query5Opt1;     query5_opt1;        A, B, C, D; E);
impl_ref_query!(Query5Opt2;     query5_opt2;        A, B, C; D, E);
impl_ref_query!(Query5Opt3;     query5_opt3;        A, B; C, D, E);
impl_ref_query!(Query5Opt4;     query5_opt4;        A; B, C, D, E);
impl_ref_query!(Query6;         query6;             A, B, C, D, E, F; );
impl_ref_query!(Query6Opt1;     query6_opt1;        A, B, C, D, E; F);
impl_ref_query!(Query6Opt2;     query6_opt2;        A, B, C, D; E, F);
impl_ref_query!(Query6Opt3;     query6_opt3;        A, B, C; D, E, F);
impl_ref_query!(Query6Opt4;     query6_opt4;        A, B; C, D, E, F);
impl_ref_query!(Query6Opt5;     query6_opt5;        A; B, C, D, E, F);
impl_ref_query!(Query7;         query7;             A, B, C, D, E, F, G; );
impl_ref_query!(Query7Opt1;     query7_opt1;        A, B, C, D, E, F; G);
impl_ref_query!(Query7Opt2;     query7_opt2;        A, B, C, D, E; F, G);
impl_ref_query!(Query7Opt3;     query7_opt3;        A, B, C, D; E, F, G);
impl_ref_query!(Query7Opt4;     query7_opt4;        A, B, C; D, E, F, G);
impl_ref_query!(Query7Opt5;     query7_opt5;        A, B; C, D, E, F, G);
impl_ref_query!(Query7Opt6;     query7_opt6;        A; B, C, D, E, F, G);
impl_ref_query!(Query8;         query8;             A, B, C, D, E, F, G, H; );
impl_ref_query!(Query8Opt1;     query8_opt1;        A, B, C, D, E, F, G; H);
impl_ref_query!(Query8Opt2;     query8_opt2;        A, B, C, D, E, F, G; H);
impl_ref_query!(Query8Opt3;     query8_opt3;        A, B, C, D, E, F, G; H);
impl_ref_query!(Query8Opt4;     query8_opt4;        A, B, C, D, E, F, G; H);
impl_ref_query!(Query8Opt5;     query8_opt5;        A, B, C, D, E, F, G; H);
impl_ref_query!(Query8Opt6;     query8_opt6;        A, B, C, D, E, F, G; H);
impl_ref_query!(Query8Opt7;     query8_opt7;        A, B, C, D, E, F, G; H);
impl_ref_query!(Query9;         query9;             A, B, C, D, E, F, G, H, I; );
impl_ref_query!(Query9Opt1;     query9_opt1;        A, B, C, D, E, F, G, H; I);
impl_ref_query!(Query9Opt2;     query9_opt2;        A, B, C, D, E, F, G; H, I);
impl_ref_query!(Query9Opt3;     query9_opt3;        A, B, C, D, E, F; G, H, I);
impl_ref_query!(Query9Opt4;     query9_opt4;        A, B, C, D, E; F, G, H, I);
impl_ref_query!(Query9Opt5;     query9_opt5;        A, B, C, D; E, F, G, H, I);
impl_ref_query!(Query9Opt6;     query9_opt6;        A, B, C; D, E, F, G, H, I);
impl_ref_query!(Query9Opt7;     query9_opt7;        A, B; C, D, E, F, G, H, I);
impl_ref_query!(Query9Opt8;     query9_opt8;        A; B, C, D, E, F, G, H, I);

impl_mut_query!(Query1Mut;      query1_mut;         A; );
impl_mut_query!(Query2Mut;      query2_mut;         A, B; );
impl_mut_query!(Query2MutOpt1;  query2_mut_opt1;    A; B);
impl_mut_query!(Query3Mut;      query3_mut;         A, B, C; );
impl_mut_query!(Query3MutOpt1;  query3_mut_opt1;    A, B; C);
impl_mut_query!(Query3MutOpt2;  query3_mut_opt2;    A; B, C);
impl_mut_query!(Query4Mut;      query4_mut;         A, B, C, D; );
impl_mut_query!(Query4MutOpt1;  query4_mut_opt1;    A, B, C; D);
impl_mut_query!(Query4MutOpt2;  query4_mut_opt2;    A, B; C, D);
impl_mut_query!(Query4MutOpt3;  query4_mut_opt3;    A; B, C, D);
impl_mut_query!(Query5Mut;      query5_mut;         A, B, C, D, E; );
impl_mut_query!(Query5MutOpt1;  query5_mut_opt1;    A, B, C, D; E);
impl_mut_query!(Query5MutOpt2;  query5_mut_opt2;    A, B, C; D, E);
impl_mut_query!(Query5MutOpt3;  query5_mut_opt3;    A, B; C, D, E);
impl_mut_query!(Query5MutOpt4;  query5_mut_opt4;    A; B, C, D, E);
impl_mut_query!(Query6Mut;      query6_mut;         A, B, C, D, E, F; );
impl_mut_query!(Query6MutOpt1;  query6_mut_opt1;    A, B, C, D, E; F);
impl_mut_query!(Query6MutOpt2;  query6_mut_opt2;    A, B, C, D; E, F);
impl_mut_query!(Query6MutOpt3;  query6_mut_opt3;    A, B, C; D, E, F);
impl_mut_query!(Query6MutOpt4;  query6_mut_opt4;    A, B; C, D, E, F);
impl_mut_query!(Query6MutOpt5;  query6_mut_opt5;    A; B, C, D, E, F);
impl_mut_query!(Query7Mut;      query7_mut;         A, B, C, D, E, F, G; );
impl_mut_query!(Query7MutOpt1;  query7_mut_opt1;    A, B, C, D, E, F; G);
impl_mut_query!(Query7MutOpt2;  query7_mut_opt2;    A, B, C, D, E; F, G);
impl_mut_query!(Query7MutOpt3;  query7_mut_opt3;    A, B, C, D; E, F, G);
impl_mut_query!(Query7MutOpt4;  query7_mut_opt4;    A, B, C; D, E, F, G);
impl_mut_query!(Query7MutOpt5;  query7_mut_opt5;    A, B; C, D, E, F, G);
impl_mut_query!(Query7MutOpt6;  query7_mut_opt6;    A; B, C, D, E, F, G);
impl_mut_query!(Query8Mut;      query8_mut;         A, B, C, D, E, F, G, H; );
impl_mut_query!(Query8MutOpt1;  query8_mut_opt1;    A, B, C, D, E, F, G; H);
impl_mut_query!(Query8MutOpt2;  query8_mut_opt2;    A, B, C, D, E, F, G; H);
impl_mut_query!(Query8MutOpt3;  query8_mut_opt3;    A, B, C, D, E, F, G; H);
impl_mut_query!(Query8MutOpt4;  query8_mut_opt4;    A, B, C, D, E, F, G; H);
impl_mut_query!(Query8MutOpt5;  query8_mut_opt5;    A, B, C, D, E, F, G; H);
impl_mut_query!(Query8MutOpt6;  query8_mut_opt6;    A, B, C, D, E, F, G; H);
impl_mut_query!(Query8MutOpt7;  query8_mut_opt7;    A, B, C, D, E, F, G; H);
impl_mut_query!(Query9Mut;      query9_mut;         A, B, C, D, E, F, G, H, I; );
impl_mut_query!(Query9MutOpt1;  query9_mut_opt1;    A, B, C, D, E, F, G, H; I);
impl_mut_query!(Query9MutOpt2;  query9_mut_opt2;    A, B, C, D, E, F, G; H, I);
impl_mut_query!(Query9MutOpt3;  query9_mut_opt3;    A, B, C, D, E, F; G, H, I);
impl_mut_query!(Query9MutOpt4;  query9_mut_opt4;    A, B, C, D, E; F, G, H, I);
impl_mut_query!(Query9MutOpt5;  query9_mut_opt5;    A, B, C, D; E, F, G, H, I);
impl_mut_query!(Query9MutOpt6;  query9_mut_opt6;    A, B, C; D, E, F, G, H, I);
impl_mut_query!(Query9MutOpt7;  query9_mut_opt7;    A, B; C, D, E, F, G, H, I);
impl_mut_query!(Query9MutOpt8;  query9_mut_opt8;    A; B, C, D, E, F, G, H, I);
