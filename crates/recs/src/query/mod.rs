use std::{any::TypeId, marker::PhantomData};

use crate::{
    component::{Component, sparse_set::SparseSet},
    registry::Registry,
};

/// A trait for querying entities with specific component combinations.
pub trait QueryParam<'q> {
    /// The type returned by the query iterator
    type Item;

    /// Creates a new iterator over entities that match this query
    fn iter(registry: &'q mut Registry) -> QueryIter<'q, Self>
    where
        Self: Sized;
}

/// A standalone query that can be passed to systems
pub struct Query<'q, Q> {
    registry: &'q mut Registry,
    _phantom: PhantomData<Q>,
}

impl<'q, Q> Query<'q, Q> {
    pub fn new(registry: &'q mut Registry) -> Self {
        Self {
            registry,
            _phantom: PhantomData,
        }
    }
}

impl<'q, Q: QueryParam<'q>> IntoIterator for Query<'q, Q>
where
    QueryIter<'q, Q>: Iterator<Item = Q::Item>,
{
    type Item = Q::Item;
    type IntoIter = QueryIter<'q, Q>;

    fn into_iter(self) -> Self::IntoIter {
        Q::iter(self.registry)
    }
}

/// A helper trait for query items.
pub trait QueryItem<'q> {
    type Component: Component;
    type Item;
    fn get_storage(
        components: &mut std::collections::HashMap<
            TypeId,
            Box<dyn crate::component::ComponentStorage>,
        >,
    ) -> Option<*mut SparseSet<Self::Component>>;
    unsafe fn get_from_storage(
        storage: *mut SparseSet<Self::Component>,
        entity_id: u32,
    ) -> Option<Self::Item>;
}

impl<'q, C: Component + 'static> QueryItem<'q> for &C {
    type Component = C;
    type Item = &'q C;

    fn get_storage(
        components: &mut std::collections::HashMap<
            TypeId,
            Box<dyn crate::component::ComponentStorage>,
        >,
    ) -> Option<*mut SparseSet<Self::Component>> {
        let type_id = TypeId::of::<C>();
        components
            .get_mut(&type_id)
            .and_then(|storage| {
                (storage.as_mut() as &mut dyn std::any::Any).downcast_mut::<SparseSet<C>>()
            })
            .map(|ss| ss as *mut SparseSet<C>)
    }

    unsafe fn get_from_storage(storage: *mut SparseSet<C>, entity_id: u32) -> Option<Self::Item> {
        unsafe { (*storage).get(entity_id as usize) }
    }
}

impl<'q, C: Component + 'static> QueryItem<'q> for &mut C {
    type Component = C;
    type Item = &'q mut C;

    fn get_storage(
        components: &mut std::collections::HashMap<
            TypeId,
            Box<dyn crate::component::ComponentStorage>,
        >,
    ) -> Option<*mut SparseSet<Self::Component>> {
        let type_id = TypeId::of::<C>();
        components
            .get_mut(&type_id)
            .and_then(|storage| {
                (storage.as_mut() as &mut dyn std::any::Any).downcast_mut::<SparseSet<C>>()
            })
            .map(|ss| ss as *mut SparseSet<C>)
    }

    unsafe fn get_from_storage(storage: *mut SparseSet<C>, entity_id: u32) -> Option<Self::Item> {
        unsafe { (*storage).get_mut(entity_id as usize) }
    }
}

pub struct QueryIter<'q, Q: QueryParam<'q>> {
    registry: &'q mut Registry,
    entity_index: usize,
    _phantom: PhantomData<Q>,
}

macro_rules! impl_query_for_tuple {
    ($($name:ident),+) => {
        impl<'q, $($name: QueryItem<'q>),+> QueryParam<'q> for ($($name,)+) {
            type Item = ($($name::Item,)+);

            fn iter(registry: &'q mut Registry) -> QueryIter<'q, Self> {
                QueryIter {
                    registry,
                    entity_index: 0,
                    _phantom: PhantomData,
                }
            }
        }

        impl<'q, $($name: QueryItem<'q>),+> Iterator for QueryIter<'q, ($($name,)+)> {
            type Item = ($($name::Item,)+);

            #[allow(non_snake_case)]
            fn next(&mut self) -> Option<Self::Item> {
                $(
                    let $name = $name::get_storage(&mut self.registry.components)?;
                )+

                // SAFETY: Raw pointers are safe because lifetimes are managed by 'q
                // and QueryIter structure, preventing deallocation while iterator exists
                unsafe {
                    let mut smallest_slice: Option<&[crate::entity::Entity]> = None;
                    $(
                        let current_slice = &(*$name).entities;
                        match smallest_slice {
                            None => smallest_slice = Some(current_slice),
                            Some(s) if current_slice.len() < s.len() => smallest_slice = Some(current_slice),
                            _ => (),
                        }
                    )+

                    let entities_to_iterate = smallest_slice.unwrap();

                    while self.entity_index < entities_to_iterate.len() {
                        let entity = entities_to_iterate[self.entity_index];
                        self.entity_index += 1;
                        let id = entity.id();

                        if let ($(Some($name),)+) = (
                            $(
                                $name::get_from_storage($name, id),
                            )+
                        ) {
                            return Some(($($name,)+));
                        }
                    }
                }

                None
            }
        }
    };
}

impl_query_for_tuple!(Q0);
impl_query_for_tuple!(Q0, Q1);
impl_query_for_tuple!(Q0, Q1, Q2);
impl_query_for_tuple!(Q0, Q1, Q2, Q3);
impl_query_for_tuple!(Q0, Q1, Q2, Q3, Q4);
impl_query_for_tuple!(Q0, Q1, Q2, Q3, Q4, Q5);
impl_query_for_tuple!(Q0, Q1, Q2, Q3, Q4, Q5, Q6);
impl_query_for_tuple!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7);
impl_query_for_tuple!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8);
impl_query_for_tuple!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9);
impl_query_for_tuple!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10);
impl_query_for_tuple!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11);
impl_query_for_tuple!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12);
impl_query_for_tuple!(Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21, Q22
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21, Q22, Q23
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21, Q22, Q23, Q24
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21, Q22, Q23, Q24, Q25
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21, Q22, Q23, Q24, Q25, Q26
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21, Q22, Q23, Q24, Q25, Q26, Q27
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21, Q22, Q23, Q24, Q25, Q26, Q27, Q28
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21, Q22, Q23, Q24, Q25, Q26, Q27, Q28, Q29
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21, Q22, Q23, Q24, Q25, Q26, Q27, Q28, Q29, Q30
);
impl_query_for_tuple!(
    Q0, Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15, Q16, Q17, Q18, Q19, Q20,
    Q21, Q22, Q23, Q24, Q25, Q26, Q27, Q28, Q29, Q30, Q31
);

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {}

    #[derive(Debug, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    impl Component for Velocity {}

    #[derive(Debug, PartialEq)]
    struct PlayerTag;

    impl Component for PlayerTag {}

    #[test]
    fn test_query_single_immutable() {
        let mut registry = Registry::new();
        let _e1 = registry.spawn((Position { x: 1.0, y: 1.0 },));
        let _e2 = registry.spawn((Position { x: 2.0, y: 2.0 }, PlayerTag));
        registry.spawn((PlayerTag,));

        let mut count = 0;
        let mut total_x = 0.0;
        for (pos,) in registry.query::<(&Position,)>() {
            count += 1;
            total_x += pos.x;
        }

        assert_eq!(count, 2);
        assert_eq!(total_x, 3.0);
    }

    #[test]
    fn test_query_single_mutable() {
        let mut registry = Registry::new();
        let entity = registry.spawn((Position { x: 1.0, y: 1.0 },));

        for (pos,) in registry.query::<(&mut Position,)>() {
            pos.x = 100.0;
        }

        let changed_pos = registry.get_component::<Position>(entity).unwrap();
        assert_eq!(changed_pos.x, 100.0);
    }

    #[test]
    fn test_query_multiple_components() {
        let mut registry = Registry::new();
        let _e1 = registry.spawn((Position { x: 1.0, y: 1.0 }, Velocity { dx: 1.0, dy: 0.0 }));
        registry.spawn(Position { x: 2.0, y: 2.0 });
        registry.spawn(Velocity { dx: 2.0, dy: 0.0 });

        let mut count = 0;
        for (pos, vel) in registry.query::<(&Position, &Velocity)>() {
            count += 1;
            assert_eq!(pos.x, 1.0);
            assert_eq!(vel.dx, 1.0);
        }
        assert_eq!(
            count, 1,
            "The query should find only one entity with both components"
        );
    }

    #[test]
    fn test_query_mutable_and_immutable() {
        let mut registry = Registry::new();
        let entity = registry.spawn((Position { x: 1.0, y: 1.0 }, Velocity { dx: 5.0, dy: 0.0 }));

        for (pos, vel) in registry.query::<(&mut Position, &Velocity)>() {
            pos.x += vel.dx;
        }

        let changed_pos = registry.get_component::<Position>(entity).unwrap();
        assert_eq!(changed_pos.x, 6.0);
    }

    #[test]
    fn test_query_iterates_over_smallest_set() {
        let mut registry = Registry::new();
        for i in 0..100 {
            if i < 3 {
                registry.spawn((
                    PlayerTag,
                    Position {
                        x: i as f32,
                        y: 0.0,
                    },
                ));
            } else {
                registry.spawn((PlayerTag,));
            }
        }

        let mut count = 0;
        for (_, _tag) in registry.query::<(&Position, &PlayerTag)>() {
            count += 1;
        }

        assert_eq!(count, 3);
    }

    #[test]
    fn test_empty_query_result() {
        let mut registry = Registry::new();
        registry.spawn((Position { x: 1.0, y: 1.0 },));

        let mut count = 0;
        for (_pos, _vel) in registry.query::<(&Position, &Velocity)>() {
            count += 1;
        }
        assert_eq!(count, 0);
    }
}
