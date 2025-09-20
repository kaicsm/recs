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
