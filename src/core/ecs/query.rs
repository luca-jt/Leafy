use crate::ecs::component::Component;
use crate::ecs::{Archetype, ArchetypeID, ECS};
use std::any::TypeId;
use std::marker::PhantomData;

/// a query filter that requires components to be included in an entity
pub struct IncludeFilter(Vec<TypeId>);

impl IncludeFilter {
    /// checks wether or not the filter accepts an archetype
    pub fn matches(&self, archetype: &Archetype) -> bool {
        self.0
            .iter()
            .all(|&ty| archetype.components.contains_key(&ty))
    }
}

/// easy creation of an include filter from given component types
macro_rules! include_filter {
    ($($T:ty),*) => {
        IncludeFilter(vec![$(TypeId::of<$T>(), )*])
    };
}

/// a query filter that requires components to be excluded from an entity
pub struct ExcludeFilter(Vec<TypeId>);

impl ExcludeFilter {
    /// checks wether or not the filter accepts an archetype
    pub fn matches(&self, archetype: &Archetype) -> bool {
        self.0
            .iter()
            .all(|&ty| !archetype.components.contains_key(&ty))
    }
}

/// easy creation of an exclude filter from given component types
macro_rules! exclude_filter {
    ($($T:ty),*) => {
        ExcludeFilter(vec![$(TypeId::of<$T>(), )*])
    };
}

pub struct Query1<'a, T: Component> {
    archetype_iter: std::collections::hash_map::Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom: PhantomData<T>,
}

impl<'a, T: Component> Iterator for Query1<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(archetype) = self.current_archetype {
            if self.component_index < archetype.components[&TypeId::of::<T>()].len() {
                let component = &archetype.components[&TypeId::of::<T>()][self.component_index];
                self.component_index += 1;
                return component.downcast_ref::<T>();
            } else {
                self.current_archetype = None;
            }
        }

        while let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

pub struct QueryMut1<'a, T: Component> {
    archetype_iter: std::collections::hash_map::ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a mut Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom: PhantomData<T>,
}

impl<'a, T: Component> Iterator for QueryMut1<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(archetype) = self.current_archetype {
            if self.component_index < archetype.components[&TypeId::of::<T>()].len() {
                let component = &mut archetype.components.get_mut(&TypeId::of::<T>()).unwrap()
                    [self.component_index];
                self.component_index += 1;
                return component.downcast_mut::<T>();
            } else {
                self.current_archetype = None;
            }
        }

        while let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

pub struct Query2<'a, A: Component, B: Component> {
    archetype_iter: std::collections::hash_map::Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
}

impl<'a, A: Component, B: Component> Iterator for Query2<'a, A, B> {
    type Item = (&'a A, &'a B);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(archetype) = self.current_archetype {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let component_a = &archetype.components[&TypeId::of::<A>()][self.component_index];
                let component_b = &archetype.components[&TypeId::of::<B>()][self.component_index];
                self.component_index += 1;
                return Some((
                    component_a.downcast_ref::<A>().unwrap(),
                    component_b.downcast_ref::<B>().unwrap(),
                ));
            } else {
                self.current_archetype = None;
            }
        }

        while let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

pub struct Query2Mut<'a, A: Component, B: Component> {
    archetype_iter: std::collections::hash_map::ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a mut Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
}

impl<'a, A: Component, B: Component> Iterator for Query2Mut<'a, A, B> {
    type Item = (&'a mut A, &'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(archetype) = self.current_archetype {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let component_a = &mut archetype.components.get_mut(&TypeId::of::<A>()).unwrap()
                    [self.component_index];
                let component_b = &mut archetype.components.get_mut(&TypeId::of::<B>()).unwrap()
                    [self.component_index];
                self.component_index += 1;
                return Some((
                    component_a.downcast_mut::<A>().unwrap(), // todo: use unsafe for mutliple &mut (archetype ensures the validity)
                    component_b.downcast_mut::<B>().unwrap(),
                ));
            } else {
                self.current_archetype = None;
            }
        }

        while let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

pub struct Query3<'a, A: Component, B: Component, C: Component> {
    archetype_iter: std::collections::hash_map::Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
    phantom_c: PhantomData<C>,
}

impl<'a, A: Component, B: Component, C: Component> Iterator for Query3<'a, A, B, C> {
    type Item = (&'a A, &'a B, &'a C);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(archetype) = self.current_archetype {
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
            } else {
                self.current_archetype = None;
            }
        }

        while let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

pub struct Query3Mut<'a, A: Component, B: Component, C: Component> {
    archetype_iter: std::collections::hash_map::ValuesMut<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a mut Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
    phantom_c: PhantomData<C>,
}

impl<'a, A: Component, B: Component, C: Component> Iterator for Query3Mut<'a, A, B, C> {
    type Item = (&'a mut A, &'a mut B, &'a mut C);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(archetype) = self.current_archetype {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let component_a = &mut archetype.components.get_mut(&TypeId::of::<A>()).unwrap()
                    [self.component_index];
                let component_b = &mut archetype.components.get_mut(&TypeId::of::<B>()).unwrap()
                    [self.component_index];
                let component_c = &mut archetype.components.get_mut(&TypeId::of::<C>()).unwrap()
                    [self.component_index];
                self.component_index += 1;
                return Some((
                    component_a.downcast_mut::<A>().unwrap(),
                    component_b.downcast_mut::<B>().unwrap(),
                    component_c.downcast_mut::<C>().unwrap(),
                ));
            } else {
                self.current_archetype = None;
            }
        }

        while let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

pub struct Query4<'a, A: Component, B: Component, C: Component, D: Component> {
    archetype_iter: std::collections::hash_map::Values<'a, ArchetypeID, Archetype>,
    current_archetype: Option<&'a Archetype>,
    component_index: usize,
    include: IncludeFilter,
    exclude: ExcludeFilter,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
    phantom_c: PhantomData<C>,
    phantom_d: PhantomData<D>,
}

impl<'a, A: Component, B: Component, C: Component, D: Component> Iterator
    for Query4<'a, A, B, C, D>
{
    type Item = (&'a A, &'a B, &'a C, &'a D);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(archetype) = self.current_archetype {
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
            } else {
                self.current_archetype = None;
            }
        }

        while let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

pub struct Query4Mut<'a, A: Component, B: Component, C: Component, D: Component> {
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

impl<'a, A: Component, B: Component, C: Component, D: Component> Iterator
    for Query4Mut<'a, A, B, C, D>
{
    type Item = (&'a mut A, &'a mut B, &'a mut C, &'a mut D);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(archetype) = self.current_archetype {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let component_a = &mut archetype.components.get_mut(&TypeId::of::<A>()).unwrap()
                    [self.component_index];
                let component_b = &mut archetype.components.get_mut(&TypeId::of::<B>()).unwrap()
                    [self.component_index];
                let component_c = &mut archetype.components.get_mut(&TypeId::of::<C>()).unwrap()
                    [self.component_index];
                let component_d = &mut archetype.components.get_mut(&TypeId::of::<D>()).unwrap()
                    [self.component_index];
                self.component_index += 1;
                return Some((
                    component_a.downcast_mut::<A>().unwrap(),
                    component_b.downcast_mut::<B>().unwrap(),
                    component_c.downcast_mut::<C>().unwrap(),
                    component_d.downcast_mut::<D>().unwrap(),
                ));
            } else {
                self.current_archetype = None;
            }
        }

        while let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

pub struct Query5<'a, A: Component, B: Component, C: Component, D: Component, E: Component> {
    archetype_iter: std::collections::hash_map::Values<'a, ArchetypeID, Archetype>,
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

impl<'a, A: Component, B: Component, C: Component, D: Component, E: Component> Iterator
    for Query5<'a, A, B, C, D, E>
{
    type Item = (&'a A, &'a B, &'a C, &'a D, &'a E);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(archetype) = self.current_archetype {
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
            } else {
                self.current_archetype = None;
            }
        }

        while let Some(archetype) = self.archetype_iter.next() {
            if self.include.matches(archetype) && self.exclude.matches(archetype) {
                self.current_archetype = Some(archetype);
                self.component_index = 0;
                return self.next();
            }
        }

        None
    }
}

pub struct Query5Mut<'a, A: Component, B: Component, C: Component, D: Component, E: Component> {
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

impl<'a, A: Component, B: Component, C: Component, D: Component, E: Component> Iterator
    for Query5Mut<'a, A, B, C, D, E>
{
    type Item = (&'a mut A, &'a mut B, &'a mut C, &'a mut D, &'a mut E);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(archetype) = self.current_archetype {
            if self.component_index < archetype.components[&TypeId::of::<A>()].len() {
                let component_a = &mut archetype.components.get_mut(&TypeId::of::<A>()).unwrap()
                    [self.component_index];
                let component_b = &mut archetype.components.get_mut(&TypeId::of::<B>()).unwrap()
                    [self.component_index];
                let component_c = &mut archetype.components.get_mut(&TypeId::of::<C>()).unwrap()
                    [self.component_index];
                let component_d = &mut archetype.components.get_mut(&TypeId::of::<D>()).unwrap()
                    [self.component_index];
                let component_e = &mut archetype.components.get_mut(&TypeId::of::<E>()).unwrap()
                    [self.component_index];
                self.component_index += 1;
                return Some((
                    component_a.downcast_mut::<A>().unwrap(),
                    component_b.downcast_mut::<B>().unwrap(),
                    component_c.downcast_mut::<C>().unwrap(),
                    component_d.downcast_mut::<D>().unwrap(),
                    component_e.downcast_mut::<E>().unwrap(),
                ));
            } else {
                self.current_archetype = None;
            }
        }

        while let Some(archetype) = self.archetype_iter.next() {
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
    pub fn query1<T: Component>(
        &self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> Query1<'_, T> {
        Query1 {
            archetype_iter: self.archetypes.values(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom: PhantomData,
        }
    }

    pub fn query1_mut<T: Component>(
        &mut self,
        include: IncludeFilter,
        exclude: ExcludeFilter,
    ) -> QueryMut1<'_, T> {
        QueryMut1 {
            archetype_iter: self.archetypes.values_mut(),
            current_archetype: None,
            component_index: 0,
            include,
            exclude,
            phantom: PhantomData,
        }
    }

    pub fn query2<A: Component, B: Component>(
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

    pub fn query2_mut<A: Component, B: Component>(
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

    pub fn query3<A: Component, B: Component, C: Component>(
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

    pub fn query3_mut<A: Component, B: Component, C: Component>(
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

    pub fn query4<A: Component, B: Component, C: Component, D: Component>(
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

    pub fn query4_mut<A: Component, B: Component, C: Component, D: Component>(
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

    pub fn query5<A: Component, B: Component, C: Component, D: Component, E: Component>(
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

    pub fn query5_mut<A: Component, B: Component, C: Component, D: Component, E: Component>(
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
