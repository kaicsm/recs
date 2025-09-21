use std::{
    any::Any,
    mem::replace,
    slice::{Iter, IterMut},
};

use crate::{
    component::{Component, ComponentStorage},
    entity::Entity,
};

/// A sparse set implementation for efficiently storing and accessing components.
///
/// This data structure uses two arrays:
/// - A sparse array mapping entity IDs to dense array indices
/// - A dense array containing the actual components and corresponding entities
///
/// This allows for:
/// - O(1) component access by entity ID
/// - Cache-friendly iteration over components
/// - Memory efficient storage for sparse data
#[derive(Debug)]
pub struct SparseSet<C> {
    /// Dense array of components, tightly packed with no gaps
    dense: Vec<C>,
    /// Parallel array of entities corresponding to components in the dense array
    pub(crate) entities: Vec<Entity>,
    /// Sparse array mapping entity IDs to indices in the dense array
    sparse: Vec<Option<usize>>,
}

impl<C> SparseSet<C>
where
    C: Component,
{
    /// Creates a new empty SparseSet
    pub fn new() -> Self {
        Self {
            dense: Vec::new(),
            entities: Vec::new(),
            sparse: Vec::new(),
        }
    }

    /// Inserts or updates a component for an entity
    ///
    /// If the entity already has this component type, it will be updated.
    /// Otherwise, the component will be added to the end of the dense array.
    pub fn insert(&mut self, entity: Entity, component: C) {
        let id = entity.id() as usize;
        if id >= self.sparse.len() {
            self.sparse.resize(id + 1, None);
        }

        if let Some(&dense_index) = self.sparse.get(id).and_then(|x| x.as_ref()) {
            if let Some(c) = self.dense.get_mut(dense_index) {
                *c = component;
            }
            self.entities[dense_index] = entity;
            return;
        }

        let new_index = self.dense.len();
        self.dense.push(component);
        self.sparse[id] = Some(new_index);
        self.entities.push(entity);
    }

    /// Removes a component by entity ID
    ///
    /// If the entity had this component type, returns Some(component).
    /// Otherwise returns None.
    ///
    /// When a component is removed, the last component in the dense array
    /// is moved to fill its place, maintaining packed storage.
    pub fn remove(&mut self, id: usize) -> Option<C> {
        let dense_index = match self.sparse.get(id) {
            Some(Some(idx)) => *idx,
            _ => return None,
        };

        let last_index = self.dense.len() - 1;
        let last_item = self.dense.pop().unwrap();
        let last_entity = self.entities.pop().unwrap();

        let removed = if dense_index != last_index {
            let replaced = replace(&mut self.dense[dense_index], last_item);
            self.entities[dense_index] = last_entity;
            self.sparse[last_entity.id() as usize] = Some(dense_index);
            replaced
        } else {
            last_item
        };

        self.sparse[id] = None;

        Some(removed)
    }

    /// Gets a reference to an entity's component if it exists
    pub fn get(&self, id: usize) -> Option<&C> {
        if id >= self.sparse.len() {
            return None;
        }

        if let Some(index) = self.sparse[id] {
            self.dense.get(index)
        } else {
            None
        }
    }

    /// Gets a mutable reference to an entity's component if it exists
    pub fn get_mut(&mut self, id: usize) -> Option<&mut C> {
        if id >= self.sparse.len() {
            return None;
        }

        if let Some(index) = self.sparse[id] {
            self.dense.get_mut(index)
        } else {
            None
        }
    }

    /// Returns an iterator over references to all components
    pub fn iter(&self) -> Iter<'_, C> {
        self.dense.iter()
    }

    /// Returns an iterator over mutable references to all components
    pub fn iter_mut(&mut self) -> IterMut<'_, C> {
        self.dense.iter_mut()
    }

    /// Returns an iterator over all (entity, component) pairs
    pub fn iter_with_entities(&self) -> impl Iterator<Item = (Entity, &C)> {
        self.entities.iter().copied().zip(self.dense.iter())
    }

    /// Returns the number of components stored in this set
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    /// Returns true if this set contains no components
    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }
}

impl<C: Component + 'static> ComponentStorage for SparseSet<C> {
    fn remove_by_id(&mut self, id: usize) -> Option<Box<dyn std::any::Any>> {
        self.remove(id).map(|c| Box::new(c) as Box<dyn Any>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Entity;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: i32,
        y: i32,
    }
    impl Component for Position {}

    fn create_entity(id: u32) -> Entity {
        Entity::new(id, 1)
    }

    #[test]
    fn test_insert_and_get() {
        let mut ss = SparseSet::<Position>::new();
        let entity = create_entity(5);

        ss.insert(entity, Position { x: 10, y: 20 });

        let component = ss.get(5).unwrap();
        assert_eq!(component, &Position { x: 10, y: 20 });

        let component_mut = ss.get_mut(5).unwrap();
        component_mut.x = 99;

        assert_eq!(ss.get(5).unwrap(), &Position { x: 99, y: 20 });
    }

    #[test]
    fn test_remove_component_swap_back() {
        let mut ss = SparseSet::<Position>::new();
        let entity0 = create_entity(0);
        let entity1 = create_entity(1);
        let entity2 = create_entity(2);

        ss.insert(entity0, Position { x: 0, y: 0 });
        ss.insert(entity1, Position { x: 1, y: 1 });
        ss.insert(entity2, Position { x: 2, y: 2 });

        assert_eq!(ss.len(), 3);

        let removed = ss.remove(entity1.id() as usize);
        assert_eq!(removed, Some(Position { x: 1, y: 1 }));

        assert_eq!(ss.len(), 2);
        assert!(ss.get(entity1.id() as usize).is_none());
        assert_eq!(
            ss.get(entity2.id() as usize),
            Some(&Position { x: 2, y: 2 })
        );
        assert_eq!(
            ss.get(entity0.id() as usize),
            Some(&Position { x: 0, y: 0 })
        );
    }
}
