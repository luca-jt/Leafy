use crate::ecs::entity::{Archetype, ArchetypeID};
use crate::ecs::entity_manager::ECS;
use crate::utils::tools::SplitMut;
use std::any::{Any, TypeId};
use std::collections::hash_map::Values;
use std::marker::PhantomData;

/// a query filter that requires components to be included in an entity
pub struct IncludeFilter(pub(crate) Vec<TypeId>);

impl IncludeFilter {
    /// checks wether or not the filter accepts an archetype
    pub fn matches(&self, archetype: &Archetype) -> bool {
        self.0
            .iter()
            .all(|&ty| archetype.components.contains_key(&ty))
    }
}

/// easy creation of an include filter from given component types
#[macro_export]
macro_rules! include_filter {
    ($($T:ty),*) => {
        IncludeFilter(vec![$(TypeId::of<$T>(), )*])
    };
}

pub(crate) use include_filter;

/// a query filter that requires components to be excluded from an entity
pub struct ExcludeFilter(pub(crate) Vec<TypeId>);

impl ExcludeFilter {
    /// checks wether or not the filter accepts an archetype
    pub fn matches(&self, archetype: &Archetype) -> bool {
        self.0
            .iter()
            .all(|&ty| !archetype.components.contains_key(&ty))
    }
}

/// easy creation of an exclude filter from given component types
#[macro_export]
macro_rules! exclude_filter {
    ($($T:ty),*) => {
        ExcludeFilter(vec![$(TypeId::of<$T>(), )*])
    };
}

pub(crate) use exclude_filter;

/// immutable query for 1 component
pub struct Query1<'a, T: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom: PhantomData<T>,
}

impl<'a, T: Any> Iterator for Query1<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype.take() {
            if self.component_index < archetype.components[&TypeId::of::<T>()].len() {
                let component = &archetype.components[&TypeId::of::<T>()][self.component_index];
                self.component_index += 1;

                return component.downcast_ref::<T>();
            }
        }

        if let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

/// mutable query for 1 component
pub struct Query1Mut<'a, T: Any> {
    archetype_iter: std::collections::hash_map::ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a mut Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom: PhantomData<T>,
}

impl<'a, T: Any> Iterator for Query1Mut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype.take() {
            if self.component_index < archetype.components[&TypeId::of::<T>()].len() {
                let component = &mut archetype.components.get_mut(&TypeId::of::<T>()).unwrap()
                    [self.component_index];
                self.component_index += 1;

                return component.downcast_mut::<T>();
            }
        }

        if let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

/// immutable query for 2 components
pub struct Query2<'a, A: Any, B: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
}

impl<'a, A: Any, B: Any> Iterator for Query2<'a, A, B> {
    type Item = (&'a A, &'a B);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype.take() {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let component_a = &archetype.components[&TypeId::of::<A>()][self.component_index];
                let component_b = &archetype.components[&TypeId::of::<B>()][self.component_index];
                self.component_index += 1;

                return Some((
                    component_a.downcast_ref::<A>().unwrap(),
                    component_b.downcast_ref::<B>().unwrap(),
                ));
            }
        }

        if let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

/// mutable query for 2 components
pub struct Query2Mut<'a, A: Any, B: Any> {
    archetype_iter: std::collections::hash_map::ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a mut Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
}

impl<'a, A: Any, B: Any> Iterator for Query2Mut<'a, A, B> {
    type Item = (&'a mut A, &'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype.take() {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let (components_a, components_b) = archetype
                    .components
                    .get2_mut(&TypeId::of::<A>(), &TypeId::of::<B>());
                let component_a = &mut components_a.unwrap()[self.component_index];
                let component_b = &mut components_b.unwrap()[self.component_index];
                self.component_index += 1;

                return Some((
                    component_a.downcast_mut::<A>().unwrap(),
                    component_b.downcast_mut::<B>().unwrap(),
                ));
            }
        }

        if let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

/// immutable query for 3 components
pub struct Query3<'a, A: Any, B: Any, C: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
    phantom_c: PhantomData<C>,
}

impl<'a, A: Any, B: Any, C: Any> Iterator for Query3<'a, A, B, C> {
    type Item = (&'a A, &'a B, &'a C);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype.take() {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let component_a = &archetype.components[&TypeId::of::<A>()][self.component_index];
                let component_b = &archetype.components[&TypeId::of::<B>()][self.component_index];
                let component_c = &archetype.components[&TypeId::of::<C>()][self.component_index];
                self.component_index += 1;

                return Some((
                    component_a.downcast_ref::<A>().unwrap(),
                    component_b.downcast_ref::<B>().unwrap(),
                    component_c.downcast_ref::<C>().unwrap(),
                ));
            }
        }

        if let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

/// mutable query for 3 components
pub struct Query3Mut<'a, A: Any, B: Any, C: Any> {
    archetype_iter: std::collections::hash_map::ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a mut Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
    phantom_c: PhantomData<C>,
}

impl<'a, A: Any, B: Any, C: Any> Iterator for Query3Mut<'a, A, B, C> {
    type Item = (&'a mut A, &'a mut B, &'a mut C);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype.take() {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let (components_a, components_b, components_c) = archetype.components.get3_mut(
                    &TypeId::of::<A>(),
                    &TypeId::of::<B>(),
                    &TypeId::of::<C>(),
                );
                let component_a = &mut components_a.unwrap()[self.component_index];
                let component_b = &mut components_b.unwrap()[self.component_index];
                let component_c = &mut components_c.unwrap()[self.component_index];
                self.component_index += 1;

                return Some((
                    component_a.downcast_mut::<A>().unwrap(),
                    component_b.downcast_mut::<B>().unwrap(),
                    component_c.downcast_mut::<C>().unwrap(),
                ));
            }
        }

        if let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

/// immutable query for 4 components
pub struct Query4<'a, A: Any, B: Any, C: Any, D: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
    phantom_c: PhantomData<C>,
    phantom_d: PhantomData<D>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any> Iterator for Query4<'a, A, B, C, D> {
    type Item = (&'a A, &'a B, &'a C, &'a D);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype.take() {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let component_a = &archetype.components[&TypeId::of::<A>()][self.component_index];
                let component_b = &archetype.components[&TypeId::of::<B>()][self.component_index];
                let component_c = &archetype.components[&TypeId::of::<C>()][self.component_index];
                let component_d = &archetype.components[&TypeId::of::<D>()][self.component_index];
                self.component_index += 1;

                return Some((
                    component_a.downcast_ref::<A>().unwrap(),
                    component_b.downcast_ref::<B>().unwrap(),
                    component_c.downcast_ref::<C>().unwrap(),
                    component_d.downcast_ref::<D>().unwrap(),
                ));
            }
        }

        if let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

/// mutable query for 4 components
pub struct Query4Mut<'a, A: Any, B: Any, C: Any, D: Any> {
    archetype_iter: std::collections::hash_map::ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a mut Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
    phantom_c: PhantomData<C>,
    phantom_d: PhantomData<D>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any> Iterator for Query4Mut<'a, A, B, C, D> {
    type Item = (&'a mut A, &'a mut B, &'a mut C, &'a mut D);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype.take() {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let (components_a, components_b, components_c, components_d) =
                    archetype.components.get4_mut(
                        &TypeId::of::<A>(),
                        &TypeId::of::<B>(),
                        &TypeId::of::<C>(),
                        &TypeId::of::<D>(),
                    );
                let component_a = &mut components_a.unwrap()[self.component_index];
                let component_b = &mut components_b.unwrap()[self.component_index];
                let component_c = &mut components_c.unwrap()[self.component_index];
                let component_d = &mut components_d.unwrap()[self.component_index];
                self.component_index += 1;

                return Some((
                    component_a.downcast_mut::<A>().unwrap(),
                    component_b.downcast_mut::<B>().unwrap(),
                    component_c.downcast_mut::<C>().unwrap(),
                    component_d.downcast_mut::<D>().unwrap(),
                ));
            }
        }

        if let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

/// immutable query for 5 components
pub struct Query5<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
    phantom_c: PhantomData<C>,
    phantom_d: PhantomData<D>,
    phantom_e: PhantomData<E>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5<'a, A, B, C, D, E> {
    type Item = (&'a A, &'a B, &'a C, &'a D, &'a E);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype.take() {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let component_a = &archetype.components[&TypeId::of::<A>()][self.component_index];
                let component_b = &archetype.components[&TypeId::of::<B>()][self.component_index];
                let component_c = &archetype.components[&TypeId::of::<C>()][self.component_index];
                let component_d = &archetype.components[&TypeId::of::<D>()][self.component_index];
                let component_e = &archetype.components[&TypeId::of::<E>()][self.component_index];
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

        if let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

/// mutable query for 5 components
pub struct Query5Mut<'a, A: Any, B: Any, C: Any, D: Any, E: Any> {
    archetype_iter: std::collections::hash_map::ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a mut Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
    phantom_c: PhantomData<C>,
    phantom_d: PhantomData<D>,
    phantom_e: PhantomData<E>,
}

impl<'a, A: Any, B: Any, C: Any, D: Any, E: Any> Iterator for Query5Mut<'a, A, B, C, D, E> {
    type Item = (&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(archetype) = self.current_archetype.take() {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let (components_a, components_b, components_c, components_d, components_e) =
                    archetype.components.get5_mut(
                        &TypeId::of::<A>(),
                        &TypeId::of::<B>(),
                        &TypeId::of::<C>(),
                        &TypeId::of::<D>(),
                        &TypeId::of::<E>(),
                    );
                let component_a = &mut components_a.unwrap()[self.component_index];
                let component_b = &mut components_b.unwrap()[self.component_index];
                let component_c = &mut components_c.unwrap()[self.component_index];
                let component_d = &mut components_d.unwrap()[self.component_index];
                let component_e = &mut components_e.unwrap()[self.component_index];
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

        if let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

impl ECS {
    /// create immutable query for 1 component with given filters, iterable
    pub fn query1<T: Any>(&self, include: IncludeFilter, exclude: ExcludeFilter) -> Query1<'_, T> {
        Query1 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom: PhantomData,
        }
    }

    /// create mutable query for 1 component with given filters, iterable
    pub fn query1_mut<T: Any>(
        &mut self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> Query1Mut<'_, T> {
        Query1Mut {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom: PhantomData,
        }
    }

    /// create immutable query for 2 components with given filters, iterable
    pub fn query2<A: Any, B: Any>(
        &self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> Query2<'_, A, B> {
        Query2 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom_a: PhantomData,
            phantom_b: PhantomData,
        }
    }

    /// create mutable query for 2 components with given filters, iterable
    pub fn query2_mut<A: Any, B: Any>(
        &mut self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> Query2Mut<'_, A, B> {
        Query2Mut {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom_a: PhantomData,
            phantom_b: PhantomData,
        }
    }

    /// create immutable query for 3 components with given filters, iterable
    pub fn query3<A: Any, B: Any, C: Any>(
        &self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> Query3<'_, A, B, C> {
        Query3 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom_a: PhantomData,
            phantom_b: PhantomData,
            phantom_c: PhantomData,
        }
    }

    /// create mutable query for 3 components with given filters, iterable
    pub fn query3_mut<A: Any, B: Any, C: Any>(
        &mut self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> Query3Mut<'_, A, B, C> {
        Query3Mut {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom_a: PhantomData,
            phantom_b: PhantomData,
            phantom_c: PhantomData,
        }
    }

    /// create immutable query for 4 components with given filters, iterable
    pub fn query4<A: Any, B: Any, C: Any, D: Any>(
        &self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> Query4<'_, A, B, C, D> {
        Query4 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom_a: PhantomData,
            phantom_b: PhantomData,
            phantom_c: PhantomData,
            phantom_d: PhantomData,
        }
    }

    /// create mutable query for 4 components with given filters, iterable
    pub fn query4_mut<A: Any, B: Any, C: Any, D: Any>(
        &mut self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> Query4Mut<'_, A, B, C, D> {
        Query4Mut {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom_a: PhantomData,
            phantom_b: PhantomData,
            phantom_c: PhantomData,
            phantom_d: PhantomData,
        }
    }

    /// create immutable query for 5 components with given filters, iterable
    pub fn query5<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> Query5<'_, A, B, C, D, E> {
        Query5 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom_a: PhantomData,
            phantom_b: PhantomData,
            phantom_c: PhantomData,
            phantom_d: PhantomData,
            phantom_e: PhantomData,
        }
    }

    /// create mutable query for 5 components with given filters, iterable
    pub fn query5_mut<A: Any, B: Any, C: Any, D: Any, E: Any>(
        &mut self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> Query5Mut<'_, A, B, C, D, E> {
        Query5Mut {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom_a: PhantomData,
            phantom_b: PhantomData,
            phantom_c: PhantomData,
            phantom_d: PhantomData,
            phantom_e: PhantomData,
        }
    }
}
