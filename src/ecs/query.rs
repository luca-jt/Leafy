use crate::ecs::component::Component;
use crate::ecs::entity::{Archetype, ArchetypeID};
use crate::ecs::entity_manager::{EntityManager, ECS};
use crate::BumpBox;
use std::any::{Any, TypeId};
use std::collections::hash_map::ValuesMut;
use std::iter::Filter;
use std::marker::PhantomData;

/// used internally for query macros (should not be implemented elsewhere)
pub trait QueryType<'a>: 'static {
    /// indicates wether the type is optional for the filtering
    const IS_OPTIONAL: bool;
    /// base component type
    type BaseType: Component;
    /// downcast return type
    type ReturnType;
    /// general downcast function
    fn downcast(any: Option<&'a mut BumpBox<dyn Component>>) -> Self::ReturnType;
}

impl<'a, T> QueryType<'a> for &'static T
where
    T: Component,
{
    const IS_OPTIONAL: bool = false;
    type BaseType = T;
    type ReturnType = &'a T;

    fn downcast(any: Option<&'a mut BumpBox<dyn Component>>) -> Self::ReturnType {
        (&**any.unwrap() as &dyn Any).downcast_ref::<T>().unwrap()
    }
}

impl<'a, T> QueryType<'a> for &'static mut T
where
    T: Component,
{
    const IS_OPTIONAL: bool = false;
    type BaseType = T;
    type ReturnType = &'a mut T;

    fn downcast(any: Option<&'a mut BumpBox<dyn Component>>) -> Self::ReturnType {
        (&mut **any.unwrap() as &mut dyn Any)
            .downcast_mut::<T>()
            .unwrap()
    }
}

impl<'a, T> QueryType<'a> for Option<&'static T>
where
    T: Component,
{
    const IS_OPTIONAL: bool = true;
    type BaseType = T;
    type ReturnType = Option<&'a T>;

    fn downcast(any: Option<&'a mut BumpBox<dyn Component>>) -> Self::ReturnType {
        Some((&**any? as &dyn Any).downcast_ref::<T>().unwrap())
    }
}

impl<'a, T> QueryType<'a> for Option<&'static mut T>
where
    T: Component,
{
    const IS_OPTIONAL: bool = true;
    type BaseType = T;
    type ReturnType = Option<&'a mut T>;

    fn downcast(any: Option<&'a mut BumpBox<dyn Component>>) -> Self::ReturnType {
        Some((&mut **any? as &mut dyn Any).downcast_mut::<T>().unwrap())
    }
}

/// a query filter that requires components to be included in an entity
#[derive(Debug, Clone)]
pub struct IncludeFilter(pub Vec<TypeId>);

impl IncludeFilter {
    fn matches(&self, archetype: &Archetype) -> bool {
        self.0
            .iter()
            .all(|&ty| archetype.components.contains_key(&ty))
    }
}

/// easy creation of a boxed include filter from given component types
#[macro_export]
macro_rules! include_filter {
    ($($T:ty),*) => {
        $crate::ecs::query::IncludeFilter(vec![$(std::any::TypeId::of::<$T>()), *])
    };
}

/// a query filter that requires components to be excluded from an entity
#[derive(Debug, Clone)]
pub struct ExcludeFilter(pub Vec<TypeId>);

impl ExcludeFilter {
    fn matches(&self, archetype: &Archetype) -> bool {
        self.0
            .iter()
            .all(|&ty| !archetype.components.contains_key(&ty))
    }
}

/// easy creation of a boxed exclude filter from given component types
#[macro_export]
macro_rules! exclude_filter {
    ($($T:ty),*) => {
        $crate::ecs::query::ExcludeFilter(vec![$(std::any::TypeId::of::<$T>()), *])
    };
}

/// makes shure the types of all the query entries are different
macro_rules! verify_types {
    ($t:ty) => { true };

    ($first:ty, $($rest:ty), +) => {
        $(std::any::TypeId::of::<$first>() != std::any::TypeId::of::<$rest>()) && + && verify_types!($($rest), +)
    };
}

macro_rules! impl_query {
    ($sname:ident; $fname:ident; $($ret:ident), +) => {
        pub struct $sname<'a, $($ret: QueryType<'a>), +> {
            archetype_iter: Filter<ValuesMut<'a, ArchetypeID, Archetype>, fn(&&mut Archetype) -> bool>,
            current_archetype: Option<*mut Archetype>,
            component_index: usize,
            include_filter: IncludeFilter,
            exclude_filter: ExcludeFilter,
            #[allow(unused_parens)]
            phantom: PhantomData<($($ret), +)>,
        }

        impl<'a, $($ret: QueryType<'a>), +> Iterator for $sname<'a, $($ret), +> {
            #[allow(unused_parens)]
            type Item = ($(<$ret as QueryType<'a>>::ReturnType), +);

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(archetype) = self.current_archetype {
                    // SAFETY: only one query can exist at a time and the raw pointer is
                    // only used for tracking the current iteration
                    // debug assert in ECS function prohibits multiple mutable references to the same component
                    if self.component_index < unsafe { (*archetype).components.values().next().unwrap().len() } {
                        let ret = (
                            $(
                                $ret::downcast(unsafe {
                                    (*archetype).components.get_mut(&TypeId::of::<<$ret as QueryType>::BaseType>()).map(|cs| &mut cs[self.component_index])
                                })
                            ),+
                        );
                        self.component_index += 1;
                        return Some(ret);
                    }
                }
                if let Some(archetype) = self.archetype_iter.next() {
                    if self.include_filter.matches(archetype) && self.exclude_filter.matches(archetype) {
                        self.current_archetype = Some(archetype);
                        self.component_index = 0;
                    }
                    return self.next();
                }
                None
            }
        }

        impl ECS {
            pub(crate) fn $fname<'a, $($ret: QueryType<'a>), +>(
                &'a mut self,
                filter: (Option<IncludeFilter>, Option<ExcludeFilter>)
            ) -> $sname<'a, $($ret), +> {
                debug_assert!(verify_types!($(<$ret as QueryType>::BaseType), +));
                $sname {
                    archetype_iter: self.archetypes.values_mut().filter(|archetype| {
                        $((archetype.contains::<<$ret as QueryType>::BaseType>() || <$ret as QueryType>::IS_OPTIONAL)) && +
                    }),
                    current_archetype: None,
                    component_index: 0,
                    include_filter: filter.0.unwrap_or(include_filter!()),
                    exclude_filter: filter.1.unwrap_or(exclude_filter!()),
                    phantom: PhantomData,
                }
            }
        }

        impl EntityManager {
            #[doc = "Query function for n components. You can use ``&T``, ``&mut T``, or ``Option<&T>``/``Option<&mut T>`` as query types."]
            #[doc = "If you use optional type arguments, the query will include entities that don't have that component and yield ``None`` in that case."]
            #[doc = "### Warning"]
            #[doc = "If the query modifies components that influence what asset data is loaded"]
            #[doc = "(e.g ``Scale``, ``Renderable`` ``RigidBody``, ``Collider``),"]
            #[doc = "the asset data is not updated automatically."]
            #[doc = "Recomputes in every query are expensive and components in queries are not necessarily modified."]
            #[doc = "If you still do that you will end up with crashes or undefined behavior."]
            #[doc = "To avoid this, you can manually trigger recomputes of desired entities using the entity manager's methods."]
            #[doc = "This should not be a real problem most of the time, as all components that influence what asset data is loaded are typically quite static."]
            #[doc = "E.g. ``Scale`` only influences inertia values of ``RigidBody``'s, which means modifying a scale in a query alone doesn't require a recompute."]
            #[doc = "### Safety"]
            #[doc = "If you use the same component type twice in the same query, it is possible to aquire two mutable references to the same component,"]
            #[doc = "which harms Rust's borrowing rules. In debug builds the query panics if you do that."]
            #[doc = "In release builds these checks are disabled for performance reasons."]
            #[doc = "Still, queries, as they are currently implemented, allow you to aquire more than one mutable reference of a component."]
            #[doc = "There is a trade-off between ease of use and making shure that Rusts borrowing rules are always followed."]
            #[doc = "For that reason, at the moment, the queries are ``unsafe``."]
            pub unsafe fn $fname<'a, $($ret: QueryType<'a>), +>(
                &'a self,
                filter: (Option<IncludeFilter>, Option<ExcludeFilter>)
            ) -> $sname<'a, $($ret), +> {
                (&mut *self.ecs.get()).$fname::<$($ret), +>(filter)
            }
        }
    };
}

impl_query!(Query1;  query1;  A);
impl_query!(Query2;  query2;  A, B);
impl_query!(Query3;  query3;  A, B, C);
impl_query!(Query4;  query4;  A, B, C, D);
impl_query!(Query5;  query5;  A, B, C, D, E);
impl_query!(Query6;  query6;  A, B, C, D, E, F);
impl_query!(Query7;  query7;  A, B, C, D, E, F, G);
impl_query!(Query8;  query8;  A, B, C, D, E, F, G, H);
impl_query!(Query9;  query9;  A, B, C, D, E, F, G, H, I);
impl_query!(Query10; query10; A, B, C, D, E, F, G, H, I, J);
impl_query!(Query11; query11; A, B, C, D, E, F, G, H, I, J, K);
impl_query!(Query12; query12; A, B, C, D, E, F, G, H, I, J, K, L);
impl_query!(Query13; query13; A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_query!(Query14; query14; A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_query!(Query15; query15; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_query!(Query16; query16; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_query!(Query17; query17; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_query!(Query18; query18; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_query!(Query19; query19; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_query!(Query20; query20; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
