use crate::ecs::component::Component;
use crate::ecs::{Archetype, ArchetypeID, ECS};
use std::any::TypeId;
use std::marker::PhantomData;

pub struct IncludeFilter(Vec<TypeId>);

impl IncludeFilter {
    pub fn matches(&self, archetype: &Archetype) -> bool {
        self.0
            .iter()
            .all(|&ty| archetype.components.contains_key(&ty))
    }
}

macro_rules! include_filter {
    ($($T:ty),*) => {
        IncludeFilter(vec![$(TypeId::of<$T>(), )*])
    };
}

pub struct ExcludeFilter(Vec<TypeId>);

impl ExcludeFilter {
    pub fn matches(&self, archetype: &Archetype) -> bool {
        self.0
            .iter()
            .all(|&ty| !archetype.components.contains_key(&ty))
    }
}

macro_rules! exclude_filter {
    ($($T:ty),*) => {
        ExcludeFilter(vec![$(TypeId::of<$T>(), )*])
    };
}

struct Query1<'a, T: Component> {
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

struct QueryMut1<'a, T: Component> {
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
}
