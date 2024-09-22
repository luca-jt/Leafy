use crate::ecs::entity::{Archetype, ArchetypeID};
use crate::ecs::entity_manager::ECS;
use crate::utils::tools::{SplitGet, SplitMut};
use std::any::{Any, TypeId};
use std::collections::hash_map::{Values, ValuesMut};
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
        Box::new(crate::ecs::query::IncludeFilter(vec![$(TypeId::of<$T>()), +]))
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
        Box::new(crate::ecs::query::ExcludeFilter(vec![$(TypeId::of<$T>()), +]))
    };
}

macro_rules! helper {
    (@COUNT; $($element:expr), *) => {
        <[()]>::len(&[$($crate::ecs::query::helper!(@SUBST; $element)),*])
    };

    (@SUBST; $_element:expr) => { () };

    (@ERASE; $_element:expr) => {};
}

// use array returns for get methods instead of tuples so they are iterable
// also the splitmut trait should change to return one final result
// maybe even change it to options instead of a result
// -> makes more sense if there is no case differentiation between errors

macro_rules! impl_ref_query {
    ($sname:ident; $fname:ident; $getfunc:ident; $($ret:ident), +; $($ret_opt:ident), *) => {
        pub struct $sname<'a, $($ret: Any), +, $($ret_opt: Any), *> {
            archetype_iter: Values<'a, ArchetypeID, Archetype>,
            current_archetype: Option<&'a Archetype>,
            component_index: usize,
            filter: Vec<Box<dyn QueryFilter>>,
            phantom: PhantomData<($($ret), +, $($ret_opt), *)>,
        }

        impl<'a, $($ret: Any), +, $($ret_opt: Any), *> Iterator for $sname<'a, $($ret), +, $($ret_opt), *> {
            type Item = ($(&'a $ret), +, $(Option<&'a $ret_opt>), *);

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(archetype) = self.current_archetype {
                    if $(archetype.contains::<$ret>()) && + {
                        //...
                    }
                    if let Some(components) = archetype.components.$getfunc($(&TypeId::of::<$ret>()), +) {
                        if self.component_index < components[0].len() {
                            let ret = (
                                $({
                                    [self.component_index];
                                    //...
                                })+,
                                $(
                                    archetype.component_ref_at::<$ret_opt>(self.component_index)
                                )*
                            );
                            self.component_index += 1;
                            return Some(ret);
                        }
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
                    archetype_iter: self.archetypes.values(),
                    current_archetype: None,
                    component_index: 0,
                    filter,
                    phantom: PhantomData,
                }
            }
        }
    };
}

macro_rules! impl_mut_query {
    ($sname:ident; $fname:ident; $getfunc:ident; $($ret:ident), +; $($ret_opt:ident), *) => {
        pub struct $sname<'a, $($ret: Any), +, $($ret_opt: Any), *> {
            archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
            current_archetype: Option<*mut Archetype>,
            component_index: usize,
            filter: Vec<Box<dyn QueryFilter>>,
            phantom: PhantomData<($($ret), +, $($ret_opt), *)>,
        }

        impl<'a, $($ret: Any), +, $($ret_opt: Any), *> Iterator for $sname<'a, $($ret), +, $($ret_opt), *> {
            type Item = ($(&'a mut $ret), +, $(Option<&'a mut $ret_opt>), *);

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(archetype) = self.current_archetype {
                    unsafe {
                        if $((*archetype).contains::<$ret>()) && + {
                            //...
                        }
                        if let Some(components) = (*archetype).components.$getfunc($(&TypeId::of::<$ret>()), +) {
                            if self.component_index < components[0].len() {
                                let ret = (
                                    $({
                                        [self.component_index];
                                        //...
                                    })+,
                                    $(
                                        (*archetype).component_mut_at::<$ret_opt>(self.component_index)
                                    )*
                                );
                                self.component_index += 1;
                                return Some(ret);
                            }
                        }
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
                $sname {
                    archetype_iter: self.archetypes.values_mut(),
                    current_archetype: None,
                    component_index: 0,
                    filter,
                    phantom: PhantomData,
                }
            }
        }
    };
}

/// immutable query for 1 component
pub struct Query1<'a, T: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<T>,
}

impl<'a, T: Any> Iterator for Query1<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some(components) = archetype.components.get(&TypeId::of::<T>()) {
                if self.component_index < components.len() {
                    let component = &components[self.component_index];
                    self.component_index += 1;

                    return component.downcast_ref::<T>();
                }
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

/// mutable query for 1 component
pub struct Query1Mut<'a, T: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<T>,
}

impl<'a, T: Any> Iterator for Query1Mut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            // SAFETY: only one query can exist at a time and the raw pointer is
            // only used for tracking the current iteration
            unsafe {
                if let Some(components) = (*archetype).components.get_mut(&TypeId::of::<T>()) {
                    if self.component_index < components.len() {
                        let component = &mut components[self.component_index];
                        self.component_index += 1;

                        return component.downcast_mut::<T>();
                    }
                }
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

/// immutable query for 2 components
pub struct Query2<'a, A: Any, B: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B)>,
}

impl<'a, A: Any, B: Any> Iterator for Query2<'a, A, B> {
    type Item = (&'a A, &'a B);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some((components_a, components_b)) = archetype
                .components
                .get2(&TypeId::of::<A>(), &TypeId::of::<B>())
            {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = &components_b[self.component_index];
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b.downcast_ref::<B>().unwrap(),
                    ));
                }
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

/// immutable query for 2 components with 1 optional
pub struct Query2Opt1<'a, A: Any, B: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B)>,
}

impl<'a, A: Any, B: Any> Iterator for Query2Opt1<'a, A, B> {
    type Item = (&'a A, Option<&'a B>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some(components_a) = archetype.components.get(&TypeId::of::<A>()) {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = archetype.component_ref_at::<B>(self.component_index);
                    self.component_index += 1;

                    return Some((component_a.downcast_ref::<A>().unwrap(), component_b));
                }
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

/// mutable query for 2 components
pub struct Query2Mut<'a, A: Any, B: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B)>,
}

impl<'a, A: Any, B: Any> Iterator for Query2Mut<'a, A, B> {
    type Item = (&'a mut A, &'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let (Ok(components_a), Ok(components_b)) = (*archetype)
                    .components
                    .get2_mut(&TypeId::of::<A>(), &TypeId::of::<B>())
                {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = &mut components_b[self.component_index];
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b.downcast_mut::<B>().unwrap(),
                        ));
                    }
                }
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

/// mutable query for 2 components with 1 optional
pub struct Query2MutOpt1<'a, A: Any, B: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B)>,
}

impl<'a, A: Any, B: Any> Iterator for Query2MutOpt1<'a, A, B> {
    type Item = (&'a mut A, Option<&'a mut B>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let Some(components_a) = (*archetype).components.get_mut(&TypeId::of::<A>()) {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = (*archetype).component_mut_at::<B>(self.component_index);
                        self.component_index += 1;

                        return Some((component_a.downcast_mut::<A>().unwrap(), component_b));
                    }
                }
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

/// immutable query for 3 components
pub struct Query3<'a, A: Any, B: Any, C: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C)>,
}

impl<'a, A: Any, B: Any, C: Any> Iterator for Query3<'a, A, B, C> {
    type Item = (&'a A, &'a B, &'a C);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some((components_a, components_b, components_c)) = archetype.components.get3(
                &TypeId::of::<A>(),
                &TypeId::of::<B>(),
                &TypeId::of::<C>(),
            ) {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = &components_b[self.component_index];
                    let component_c = &components_c[self.component_index];
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b.downcast_ref::<B>().unwrap(),
                        component_c.downcast_ref::<C>().unwrap(),
                    ));
                }
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

/// immutable query for 3 components with 1 optional
pub struct Query3Opt1<'a, A: Any, B: Any, C: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C)>,
}

impl<'a, A: Any, B: Any, C: Any> Iterator for Query3Opt1<'a, A, B, C> {
    type Item = (&'a A, &'a B, Option<&'a C>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some((components_a, components_b)) = archetype
                .components
                .get2(&TypeId::of::<A>(), &TypeId::of::<B>())
            {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = &components_b[self.component_index];
                    let component_c = archetype.component_ref_at::<C>(self.component_index);
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b.downcast_ref::<B>().unwrap(),
                        component_c,
                    ));
                }
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

/// immutable query for 3 components with 2 optionals
pub struct Query3Opt2<'a, A: Any, B: Any, C: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C)>,
}

impl<'a, A: Any, B: Any, C: Any> Iterator for Query3Opt2<'a, A, B, C> {
    type Item = (&'a A, Option<&'a B>, Option<&'a C>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some(components_a) = archetype.components.get(&TypeId::of::<A>()) {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = archetype.component_ref_at::<B>(self.component_index);
                    let component_c = archetype.component_ref_at::<C>(self.component_index);
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b,
                        component_c,
                    ));
                }
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

/// mutable query for 3 components
pub struct Query3Mut<'a, A: Any, B: Any, C: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C)>,
}

impl<'a, A: Any, B: Any, C: Any> Iterator for Query3Mut<'a, A, B, C> {
    type Item = (&'a mut A, &'a mut B, &'a mut C);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let (Ok(components_a), Ok(components_b), Ok(components_c)) = (*archetype)
                    .components
                    .get3_mut(&TypeId::of::<A>(), &TypeId::of::<B>(), &TypeId::of::<C>())
                {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = &mut components_b[self.component_index];
                        let component_c = &mut components_c[self.component_index];
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b.downcast_mut::<B>().unwrap(),
                            component_c.downcast_mut::<C>().unwrap(),
                        ));
                    }
                }
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

/// mutable query for 3 components with 1 optional
pub struct Query3MutOpt1<'a, A: Any, B: Any, C: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C)>,
}

impl<'a, A: Any, B: Any, C: Any> Iterator for Query3MutOpt1<'a, A, B, C> {
    type Item = (&'a mut A, &'a mut B, Option<&'a mut C>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let (Ok(components_a), Ok(components_b)) = (*archetype)
                    .components
                    .get2_mut(&TypeId::of::<A>(), &TypeId::of::<B>())
                {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = &mut components_b[self.component_index];
                        let component_c = (*archetype).component_mut_at::<C>(self.component_index);
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b.downcast_mut::<B>().unwrap(),
                            component_c,
                        ));
                    }
                }
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

/// mutable query for 3 components with 2 optionals
pub struct Query3MutOpt2<'a, A: Any, B: Any, C: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C)>,
}

impl<'a, A: Any, B: Any, C: Any> Iterator for Query3MutOpt2<'a, A, B, C> {
    type Item = (&'a mut A, Option<&'a mut B>, Option<&'a mut C>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let Some(components_a) = (*archetype).components.get_mut(&TypeId::of::<A>()) {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = (*archetype).component_mut_at::<B>(self.component_index);
                        let component_c = (*archetype).component_mut_at::<C>(self.component_index);
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b,
                            component_c,
                        ));
                    }
                }
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

/// immutable query for 4 components
pub struct Query4<'a, A: Any, B: Any, C: Any, D: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any> Iterator for Query4<'a, A, B, C, D> {
    type Item = (&'a A, &'a B, &'a C, &'a D);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some((components_a, components_b, components_c, components_d)) =
                archetype.components.get4(
                    &TypeId::of::<A>(),
                    &TypeId::of::<B>(),
                    &TypeId::of::<C>(),
                    &TypeId::of::<D>(),
                )
            {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = &components_b[self.component_index];
                    let component_c = &components_c[self.component_index];
                    let component_d = &components_d[self.component_index];
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b.downcast_ref::<B>().unwrap(),
                        component_c.downcast_ref::<C>().unwrap(),
                        component_d.downcast_ref::<D>().unwrap(),
                    ));
                }
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

/// immutable query for 4 components with 1 optional
pub struct Query4Opt1<'a, A: Any, B: Any, C: Any, D: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any> Iterator for Query4Opt1<'a, A, B, C, D> {
    type Item = (&'a A, &'a B, &'a C, Option<&'a D>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some((components_a, components_b, components_c)) = archetype.components.get3(
                &TypeId::of::<A>(),
                &TypeId::of::<B>(),
                &TypeId::of::<C>(),
            ) {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = &components_b[self.component_index];
                    let component_c = &components_c[self.component_index];
                    let component_d = archetype.component_ref_at::<D>(self.component_index);
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b.downcast_ref::<B>().unwrap(),
                        component_c.downcast_ref::<C>().unwrap(),
                        component_d,
                    ));
                }
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

/// immutable query for 4 components with 2 optionals
pub struct Query4Opt2<'a, A: Any, B: Any, C: Any, D: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any> Iterator for Query4Opt2<'a, A, B, C, D> {
    type Item = (&'a A, &'a B, Option<&'a C>, Option<&'a D>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some((components_a, components_b)) = archetype
                .components
                .get2(&TypeId::of::<A>(), &TypeId::of::<B>())
            {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = &components_b[self.component_index];
                    let component_c = archetype.component_ref_at::<C>(self.component_index);
                    let component_d = archetype.component_ref_at::<D>(self.component_index);
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b.downcast_ref::<B>().unwrap(),
                        component_c,
                        component_d,
                    ));
                }
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

/// immutable query for 4 components with 3 optionals
pub struct Query4Opt3<'a, A: Any, B: Any, C: Any, D: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any> Iterator for Query4Opt3<'a, A, B, C, D> {
    type Item = (&'a A, Option<&'a B>, Option<&'a C>, Option<&'a D>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some(components_a) = archetype.components.get(&TypeId::of::<A>()) {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = archetype.component_ref_at::<B>(self.component_index);
                    let component_c = archetype.component_ref_at::<C>(self.component_index);
                    let component_d = archetype.component_ref_at::<D>(self.component_index);
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b,
                        component_c,
                        component_d,
                    ));
                }
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

/// mutable query for 4 components
pub struct Query4Mut<'a, A: Any, B: Any, C: Any, D: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any> Iterator for Query4Mut<'a, A, B, C, D> {
    type Item = (&'a mut A, &'a mut B, &'a mut C, &'a mut D);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let (Ok(components_a), Ok(components_b), Ok(components_c), Ok(components_d)) =
                    (*archetype).components.get4_mut(
                        &TypeId::of::<A>(),
                        &TypeId::of::<B>(),
                        &TypeId::of::<C>(),
                        &TypeId::of::<D>(),
                    )
                {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = &mut components_b[self.component_index];
                        let component_c = &mut components_c[self.component_index];
                        let component_d = &mut components_d[self.component_index];
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b.downcast_mut::<B>().unwrap(),
                            component_c.downcast_mut::<C>().unwrap(),
                            component_d.downcast_mut::<D>().unwrap(),
                        ));
                    }
                }
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

/// mutable query for 4 components with 1 optional
pub struct Query4MutOpt1<'a, A: Any, B: Any, C: Any, D: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any> Iterator for Query4MutOpt1<'a, A, B, C, D> {
    type Item = (&'a mut A, &'a mut B, &'a mut C, Option<&'a mut D>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let (Ok(components_a), Ok(components_b), Ok(components_c)) = (*archetype)
                    .components
                    .get3_mut(&TypeId::of::<A>(), &TypeId::of::<B>(), &TypeId::of::<C>())
                {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = &mut components_b[self.component_index];
                        let component_c = &mut components_c[self.component_index];
                        let component_d = (*archetype).component_mut_at::<D>(self.component_index);
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b.downcast_mut::<B>().unwrap(),
                            component_c.downcast_mut::<C>().unwrap(),
                            component_d,
                        ));
                    }
                }
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

/// mutable query for 4 components with 2 optionals
pub struct Query4MutOpt2<'a, A: Any, B: Any, C: Any, D: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any> Iterator for Query4MutOpt2<'a, A, B, C, D> {
    type Item = (&'a mut A, &'a mut B, Option<&'a mut C>, Option<&'a mut D>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let (Ok(components_a), Ok(components_b)) = (*archetype)
                    .components
                    .get2_mut(&TypeId::of::<A>(), &TypeId::of::<B>())
                {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = &mut components_b[self.component_index];
                        let component_c = (*archetype).component_mut_at::<C>(self.component_index);
                        let component_d = (*archetype).component_mut_at::<D>(self.component_index);
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b.downcast_mut::<B>().unwrap(),
                            component_c,
                            component_d,
                        ));
                    }
                }
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

/// mutable query for 4 components with 3 optionals
pub struct Query4MutOpt3<'a, A: Any, B: Any, C: Any, D: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any> Iterator for Query4MutOpt3<'a, A, B, C, D> {
    type Item = (
        &'a mut A,
        Option<&'a mut B>,
        Option<&'a mut C>,
        Option<&'a mut D>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let Some(components_a) = (*archetype).components.get_mut(&TypeId::of::<A>()) {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = (*archetype).component_mut_at::<B>(self.component_index);
                        let component_c = (*archetype).component_mut_at::<C>(self.component_index);
                        let component_d = (*archetype).component_mut_at::<D>(self.component_index);
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b,
                            component_c,
                            component_d,
                        ));
                    }
                }
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

/// immutable query for 5 components
pub struct Query5<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D, E)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5<'a, A, B, C, D, E> {
    type Item = (&'a A, &'a B, &'a C, &'a D, &'a E);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some((components_a, components_b, components_c, components_d, components_e)) =
                archetype.components.get5(
                    &TypeId::of::<A>(),
                    &TypeId::of::<B>(),
                    &TypeId::of::<C>(),
                    &TypeId::of::<D>(),
                    &TypeId::of::<E>(),
                )
            {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = &components_b[self.component_index];
                    let component_c = &components_c[self.component_index];
                    let component_d = &components_d[self.component_index];
                    let component_e = &components_e[self.component_index];
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b.downcast_ref::<B>().unwrap(),
                        component_c.downcast_ref::<C>().unwrap(),
                        component_d.downcast_ref::<D>().unwrap(),
                        component_e.downcast_ref::<E>().unwrap(),
                    ));
                }
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

/// immutable query for 5 components with 1 optional
pub struct Query5Opt1<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D, E)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5Opt1<'a, A, B, C, D, E> {
    type Item = (&'a A, &'a B, &'a C, &'a D, Option<&'a E>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some((components_a, components_b, components_c, components_d)) =
                archetype.components.get4(
                    &TypeId::of::<A>(),
                    &TypeId::of::<B>(),
                    &TypeId::of::<C>(),
                    &TypeId::of::<D>(),
                )
            {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = &components_b[self.component_index];
                    let component_c = &components_c[self.component_index];
                    let component_d = &components_d[self.component_index];
                    let component_e = archetype.component_ref_at::<E>(self.component_index);
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b.downcast_ref::<B>().unwrap(),
                        component_c.downcast_ref::<C>().unwrap(),
                        component_d.downcast_ref::<D>().unwrap(),
                        component_e,
                    ));
                }
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

/// immutable query for 5 components with 2 optionals
pub struct Query5Opt2<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D, E)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5Opt2<'a, A, B, C, D, E> {
    type Item = (&'a A, &'a B, &'a C, Option<&'a D>, Option<&'a E>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some((components_a, components_b, components_c)) = archetype.components.get3(
                &TypeId::of::<A>(),
                &TypeId::of::<B>(),
                &TypeId::of::<C>(),
            ) {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = &components_b[self.component_index];
                    let component_c = &components_c[self.component_index];
                    let component_d = archetype.component_ref_at::<D>(self.component_index);
                    let component_e = archetype.component_ref_at::<E>(self.component_index);
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b.downcast_ref::<B>().unwrap(),
                        component_c.downcast_ref::<C>().unwrap(),
                        component_d,
                        component_e,
                    ));
                }
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

/// immutable query for 5 components with 3 optionals
pub struct Query5Opt3<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D, E)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5Opt3<'a, A, B, C, D, E> {
    type Item = (&'a A, &'a B, Option<&'a C>, Option<&'a D>, Option<&'a E>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some((components_a, components_b)) = archetype
                .components
                .get2(&TypeId::of::<A>(), &TypeId::of::<B>())
            {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = &components_b[self.component_index];
                    let component_c = archetype.component_ref_at::<C>(self.component_index);
                    let component_d = archetype.component_ref_at::<D>(self.component_index);
                    let component_e = archetype.component_ref_at::<E>(self.component_index);
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b.downcast_ref::<B>().unwrap(),
                        component_c,
                        component_d,
                        component_e,
                    ));
                }
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

/// immutable query for 5 components with 4 optionals
pub struct Query5Opt4<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D, E)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5Opt4<'a, A, B, C, D, E> {
    type Item = (
        &'a A,
        Option<&'a B>,
        Option<&'a C>,
        Option<&'a D>,
        Option<&'a E>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            if let Some(components_a) = archetype.components.get(&TypeId::of::<A>()) {
                if self.component_index < components_a.len() {
                    let component_a = &components_a[self.component_index];
                    let component_b = archetype.component_ref_at::<B>(self.component_index);
                    let component_c = archetype.component_ref_at::<C>(self.component_index);
                    let component_d = archetype.component_ref_at::<D>(self.component_index);
                    let component_e = archetype.component_ref_at::<E>(self.component_index);
                    self.component_index += 1;

                    return Some((
                        component_a.downcast_ref::<A>().unwrap(),
                        component_b,
                        component_c,
                        component_d,
                        component_e,
                    ));
                }
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

/// mutable query for 5 components
pub struct Query5Mut<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D, E)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5Mut<'a, A, B, C, D, E> {
    type Item = (&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let (
                    Ok(components_a),
                    Ok(components_b),
                    Ok(components_c),
                    Ok(components_d),
                    Ok(components_e),
                ) = (*archetype).components.get5_mut(
                    &TypeId::of::<A>(),
                    &TypeId::of::<B>(),
                    &TypeId::of::<C>(),
                    &TypeId::of::<D>(),
                    &TypeId::of::<E>(),
                ) {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = &mut components_b[self.component_index];
                        let component_c = &mut components_c[self.component_index];
                        let component_d = &mut components_d[self.component_index];
                        let component_e = &mut components_e[self.component_index];
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b.downcast_mut::<B>().unwrap(),
                            component_c.downcast_mut::<C>().unwrap(),
                            component_d.downcast_mut::<D>().unwrap(),
                            component_e.downcast_mut::<E>().unwrap(),
                        ));
                    }
                }
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

/// mutable query for 5 components with 1 optional
pub struct Query5MutOpt1<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D, E)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5MutOpt1<'a, A, B, C, D, E> {
    type Item = (
        &'a mut A,
        &'a mut B,
        &'a mut C,
        &'a mut D,
        Option<&'a mut E>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let (Ok(components_a), Ok(components_b), Ok(components_c), Ok(components_d)) =
                    (*archetype).components.get4_mut(
                        &TypeId::of::<A>(),
                        &TypeId::of::<B>(),
                        &TypeId::of::<C>(),
                        &TypeId::of::<D>(),
                    )
                {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = &mut components_b[self.component_index];
                        let component_c = &mut components_c[self.component_index];
                        let component_d = &mut components_d[self.component_index];
                        let component_e = (*archetype).component_mut_at::<E>(self.component_index);
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b.downcast_mut::<B>().unwrap(),
                            component_c.downcast_mut::<C>().unwrap(),
                            component_d.downcast_mut::<D>().unwrap(),
                            component_e,
                        ));
                    }
                }
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

/// mutable query for 5 components with 2 optionals
pub struct Query5MutOpt2<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D, E)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5MutOpt2<'a, A, B, C, D, E> {
    type Item = (
        &'a mut A,
        &'a mut B,
        &'a mut C,
        Option<&'a mut D>,
        Option<&'a mut E>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let (Ok(components_a), Ok(components_b), Ok(components_c)) = (*archetype)
                    .components
                    .get3_mut(&TypeId::of::<A>(), &TypeId::of::<B>(), &TypeId::of::<C>())
                {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = &mut components_b[self.component_index];
                        let component_c = &mut components_c[self.component_index];
                        let component_d = (*archetype).component_mut_at::<D>(self.component_index);
                        let component_e = (*archetype).component_mut_at::<E>(self.component_index);
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b.downcast_mut::<B>().unwrap(),
                            component_c.downcast_mut::<C>().unwrap(),
                            component_d,
                            component_e,
                        ));
                    }
                }
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

/// mutable query for 5 components with 3 optionals
pub struct Query5MutOpt3<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D, E)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5MutOpt3<'a, A, B, C, D, E> {
    type Item = (
        &'a mut A,
        &'a mut B,
        Option<&'a mut C>,
        Option<&'a mut D>,
        Option<&'a mut E>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let (Ok(components_a), Ok(components_b)) = (*archetype)
                    .components
                    .get2_mut(&TypeId::of::<A>(), &TypeId::of::<B>())
                {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = &mut components_b[self.component_index];
                        let component_c = (*archetype).component_mut_at::<C>(self.component_index);
                        let component_d = (*archetype).component_mut_at::<D>(self.component_index);
                        let component_e = (*archetype).component_mut_at::<E>(self.component_index);
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b.downcast_mut::<B>().unwrap(),
                            component_c,
                            component_d,
                            component_e,
                        ));
                    }
                }
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

/// mutable query for 5 components with 4 optionals
pub struct Query5MutOpt4<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<*mut Archetype>,
    component_index: usize,
    filter: Vec<Box<dyn QueryFilter>>,
    phantom: PhantomData<(A, B, C, D, E)>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5MutOpt4<'a, A, B, C, D, E> {
    type Item = (
        &'a mut A,
        Option<&'a mut B>,
        Option<&'a mut C>,
        Option<&'a mut D>,
        Option<&'a mut E>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype {
            unsafe {
                if let Some(components_a) = (*archetype).components.get_mut(&TypeId::of::<A>()) {
                    if self.component_index < components_a.len() {
                        let component_a = &mut components_a[self.component_index];
                        let component_b = (*archetype).component_mut_at::<B>(self.component_index);
                        let component_c = (*archetype).component_mut_at::<C>(self.component_index);
                        let component_d = (*archetype).component_mut_at::<D>(self.component_index);
                        let component_e = (*archetype).component_mut_at::<E>(self.component_index);
                        self.component_index += 1;

                        return Some((
                            component_a.downcast_mut::<A>().unwrap(),
                            component_b,
                            component_c,
                            component_d,
                            component_e,
                        ));
                    }
                }
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
    /// create immutable query for 1 component with given filters, iterable
    pub(crate) fn query1<T: Any>(&self, filter: Vec<Box<dyn QueryFilter>>) -> Query1<'_, T> {
        Query1 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 1 component with given filters, iterable
    pub(crate) fn query1_mut<T: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query1Mut<'_, T> {
        Query1Mut {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 2 components with given filters, iterable
    pub(crate) fn query2<A: Any, B: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query2<'_, A, B> {
        Query2 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 2 components, 1 optional, with given filters, iterable
    pub(crate) fn query2_opt1<A: Any, B: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query2Opt1<'_, A, B> {
        Query2Opt1 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 2 components with given filters, iterable
    pub(crate) fn query2_mut<A: Any, B: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query2Mut<'_, A, B> {
        Query2Mut {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 2 components, 1 optional, with given filters, iterable
    pub(crate) fn query2_mut_opt1<A: Any, B: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query2MutOpt1<'_, A, B> {
        Query2MutOpt1 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 3 components with given filters, iterable
    pub(crate) fn query3<A: Any, B: Any, C: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3<'_, A, B, C> {
        Query3 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 3 components, 1 optional, with given filters, iterable
    pub(crate) fn query3_opt1<A: Any, B: Any, C: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3Opt1<'_, A, B, C> {
        Query3Opt1 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 3 components, 2 optional, with given filters, iterable
    pub(crate) fn query3_opt2<A: Any, B: Any, C: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3Opt2<'_, A, B, C> {
        Query3Opt2 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 3 components with given filters, iterable
    pub(crate) fn query3_mut<A: Any, B: Any, C: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3Mut<'_, A, B, C> {
        Query3Mut {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 3 components, 1 optional, with given filters, iterable
    pub(crate) fn query3_mut_opt1<A: Any, B: Any, C: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3MutOpt1<'_, A, B, C> {
        Query3MutOpt1 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 3 components, 2 optional, with given filters, iterable
    pub(crate) fn query3_mut_opt2<A: Any, B: Any, C: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query3MutOpt2<'_, A, B, C> {
        Query3MutOpt2 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 4 components with given filters, iterable
    pub(crate) fn query4<A: Any, B: Any, C: Any, D: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4<'_, A, B, C, D> {
        Query4 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 4 components, 1 optional, with given filters, iterable
    pub(crate) fn query4_opt1<A: Any, B: Any, C: Any, D: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4Opt1<'_, A, B, C, D> {
        Query4Opt1 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 4 components, 2 optional, with given filters, iterable
    pub(crate) fn query4_opt2<A: Any, B: Any, C: Any, D: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4Opt2<'_, A, B, C, D> {
        Query4Opt2 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 4 components, 3 optional, with given filters, iterable
    pub(crate) fn query4_opt3<A: Any, B: Any, C: Any, D: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4Opt3<'_, A, B, C, D> {
        Query4Opt3 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 4 components with given filters, iterable
    pub(crate) fn query4_mut<A: Any, B: Any, C: Any, D: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4Mut<'_, A, B, C, D> {
        Query4Mut {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 4 components, 1 optional, with given filters, iterable
    pub(crate) fn query4_mut_opt1<A: Any, B: Any, C: Any, D: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4MutOpt1<'_, A, B, C, D> {
        Query4MutOpt1 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 4 components, 2 optional, with given filters, iterable
    pub(crate) fn query4_mut_opt2<A: Any, B: Any, C: Any, D: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4MutOpt2<'_, A, B, C, D> {
        Query4MutOpt2 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 4 components, 3 optional, with given filters, iterable
    pub(crate) fn query4_mut_opt3<A: Any, B: Any, C: Any, D: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query4MutOpt3<'_, A, B, C, D> {
        Query4MutOpt3 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 5 components with given filters, iterable
    pub(crate) fn query5<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5<'_, A, B, C, D, E> {
        Query5 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 5 components, 1 optional, with given filters, iterable
    pub(crate) fn query5_opt1<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5Opt1<'_, A, B, C, D, E> {
        Query5Opt1 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 5 components, 2 optional, with given filters, iterable
    pub(crate) fn query5_opt2<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5Opt2<'_, A, B, C, D, E> {
        Query5Opt2 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 5 components, 3 optional, with given filters, iterable
    pub(crate) fn query5_opt3<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5Opt3<'_, A, B, C, D, E> {
        Query5Opt3 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 5 components, 4 optional, with given filters, iterable
    pub(crate) fn query5_opt4<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5Opt4<'_, A, B, C, D, E> {
        Query5Opt4 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 5 components with given filters, iterable
    pub(crate) fn query5_mut<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5Mut<'_, A, B, C, D, E> {
        Query5Mut {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 5 components, 1 optional, with given filters, iterable
    pub(crate) fn query5_mut_opt1<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5MutOpt1<'_, A, B, C, D, E> {
        Query5MutOpt1 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 5 components, 2 optional, with given filters, iterable
    pub(crate) fn query5_mut_opt2<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5MutOpt2<'_, A, B, C, D, E> {
        Query5MutOpt2 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 5 components, 3 optional, with given filters, iterable
    pub(crate) fn query5_mut_opt3<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5MutOpt3<'_, A, B, C, D, E> {
        Query5MutOpt3 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 5 components, 4 optional, with given filters, iterable
    pub(crate) fn query5_mut_opt4<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        filter: Vec<Box<dyn QueryFilter>>,
    ) -> Query5MutOpt4<'_, A, B, C, D, E> {
        Query5MutOpt4 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            filter,
            phantom: PhantomData,
        }
    }
}
