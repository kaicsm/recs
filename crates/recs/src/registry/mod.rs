use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub mod bundle;

use crate::{
    component::{Component, ComponentStorage, sparse_set::SparseSet},
    entity::{Entity, EntityManager},
    error::RecsError,
    registry::bundle::ComponentBundle,
    system::query::{Query, QueryIter},
};

/// The main registry that manages all entities and their components in the RECS system.
///
/// The Registry is responsible for:
/// - Creating and destroying entities
/// - Adding and removing components from entities
/// - Querying entities with specific component combinations
/// - Managing component storage and lifecycle
pub struct Registry {
    /// Manages entity creation, destruction and validation
    entity_manager: EntityManager,
    /// Stores components for all entities, organized by component type
    pub(crate) components: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl Registry {
    /// Creates a new empty Registry.
    pub fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            components: HashMap::new(),
        }
    }

    /// Registers a new component type in the registry.
    /// This is automatically called when adding components, but can be called
    /// manually to pre-allocate storage for a component type.
    pub fn register_component<C: Component + 'static>(&mut self) {
        let type_id = TypeId::of::<C>();
        if !self.components.contains_key(&type_id) {
            self.components
                .insert(type_id, Box::new(SparseSet::<C>::new()));
        }
    }

    /// Creates a new entity without any components.
    /// Use `spawn()` if you want to create an entity with components.
    pub fn create_entity(&mut self) -> Entity {
        self.entity_manager.create_entity()
    }

    pub fn add_component<C: Component + 'static>(
        &mut self,
        entity: Entity,
        component: C,
    ) -> Result<(), RecsError> {
        if !self.entity_manager.is_valid(entity) {
            return Err(RecsError::InvalidEntity(entity));
        }

        let type_id = TypeId::of::<C>();
        let storage = self
            .components
            .entry(type_id)
            .or_insert_with(|| Box::new(SparseSet::<C>::new()));

        if let Some(ss) = (storage.as_mut() as &mut dyn Any).downcast_mut::<SparseSet<C>>() {
            ss.insert(entity, component);
        }

        Ok(())
    }

    pub fn get_component<C: Component + 'static>(&self, entity: Entity) -> Option<&C> {
        if !self.entity_manager.is_valid(entity) {
            return None;
        }

        let type_id = TypeId::of::<C>();
        if let Some(sparse_set) = self.components.get(&type_id) {
            if let Some(ss) = (sparse_set.as_ref() as &dyn Any).downcast_ref::<SparseSet<C>>() {
                return ss.get(entity.id() as usize);
            }
        }
        None
    }

    pub fn get_component_mut<C: Component + 'static>(&mut self, entity: Entity) -> Option<&mut C> {
        if !self.entity_manager.is_valid(entity) {
            return None;
        }

        let type_id = TypeId::of::<C>();
        if let Some(sparse_set) = self.components.get_mut(&type_id) {
            if let Some(ss) = (sparse_set.as_mut() as &mut dyn Any).downcast_mut::<SparseSet<C>>() {
                return ss.get_mut(entity.id() as usize);
            }
        }
        None
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> Result<(), RecsError> {
        self.entity_manager.destroy_entity(entity)?;

        let id = entity.id() as usize;

        for (_type_id, storage) in self.components.iter_mut() {
            storage.remove_by_id(id);
        }

        Ok(())
    }

    pub fn remove_component<C: Component + 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<C, RecsError> {
        if !self.entity_manager.is_valid(entity) {
            return Err(RecsError::InvalidEntity(entity));
        }

        let type_id = TypeId::of::<C>();
        let storage = self.components.get_mut(&type_id);

        if let Some(storage) = storage {
            if let Some(ss) = (storage.as_mut() as &mut dyn Any).downcast_mut::<SparseSet<C>>() {
                return ss
                    .remove(entity.id() as usize)
                    .ok_or(RecsError::ComponentNotFound(type_id));
            }
        }

        Err(RecsError::ComponentNotFound(type_id))
    }

    pub fn query<'q, Q: Query<'q>>(&'q mut self) -> QueryIter<'q, Q> {
        Q::iter(self)
    }

    pub fn spawn<B: ComponentBundle>(&mut self, bundle: B) -> Entity {
        let entity = self.create_entity();
        bundle.add_to_entity(self, entity).expect(
            "Failed to add bundle to newly created entity. This is a bug in the RECS library.",
        );
        entity
    }
}
