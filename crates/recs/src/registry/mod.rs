use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub mod bundle;

use crate::{
    component::{Component, ComponentStorage, sparse_set::SparseSet},
    entity::{Entity, EntityManager},
    error::RecsError,
    query::{QueryIter, QueryParam},
    registry::bundle::ComponentBundle,
    resource::{Resource, ResourceStorage},
    system::{BoxedSystem, IntoSystem},
};

/// The main registry that manages all entities and their components in the RECS system.
///
/// The Registry is responsible for:
/// - Creating and destroying entities
/// - Adding and removing components from entities
/// - Querying entities with specific component combinations
/// - Managing component storage and lifecycle
/// - Running systems that operate on entities
pub struct Registry {
    /// Manages entity creation, destruction and validation
    entity_manager: EntityManager,
    /// Stores components for all entities, organized by component type
    pub(crate) components: HashMap<TypeId, Box<dyn ComponentStorage>>,
    /// Stores resources (singleton data) accessible by systems
    pub(crate) resources: ResourceStorage,
    /// List of systems to be executed
    systems: Vec<BoxedSystem>,
}

impl Registry {
    /// Creates a new empty Registry.
    pub fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            components: HashMap::new(),
            resources: ResourceStorage::new(),
            systems: Vec::new(),
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

    pub fn query<'q, Q: QueryParam<'q>>(&'q mut self) -> QueryIter<'q, Q> {
        Q::iter(self)
    }

    pub fn spawn<B: ComponentBundle>(&mut self, bundle: B) -> Entity {
        let entity = self.create_entity();
        bundle.add_to_entity(self, entity).expect(
            "Failed to add bundle to newly created entity. This is a bug in the RECS library.",
        );
        entity
    }

    /// Adds a system to the registry
    pub fn add_system<S, Params>(&mut self, system: S)
    where
        S: IntoSystem<Params>,
        S::System: 'static,
    {
        self.systems.push(Box::new(system.into_system()));
    }

    /// Runs all registered systems in order
    pub fn run_systems(&mut self) {
        // We need to be careful here because we're borrowing self mutably
        // We'll use raw pointers to work around the borrow checker
        let registry_ptr = self as *mut Registry;

        for system in &mut self.systems {
            // Safety: We know the registry is valid for the duration of this call
            // and we're not storing the reference anywhere
            unsafe {
                system.run(&mut *registry_ptr);
            }
        }
    }

    /// Clears all systems from the registry
    pub fn clear_systems(&mut self) {
        self.systems.clear();
    }

    /// Returns the number of registered systems
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Inserts a resource into the registry.
    /// If a resource of the same type already exists, it will be replaced.
    ///
    /// # Example
    /// ```rust
    /// # use recs::prelude::{Registry, Resource};
    /// #[derive(Resource, Debug, Clone)]
    /// struct GameSettings {
    ///     volume: f32,
    ///     difficulty: u8,
    /// }
    ///
    /// let mut registry = Registry::new();
    /// registry.insert_resource(GameSettings { volume: 0.8, difficulty: 2 });
    /// # assert!(registry.has_resource::<GameSettings>());
    /// ```
    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.resources.insert(resource);
    }

    /// Gets a reference to a resource if it exists
    ///
    /// # Example
    /// ```rust
    /// # use recs::prelude::{Registry, Resource};
    /// # #[derive(Resource, Debug, Clone)]
    /// # struct GameSettings { volume: f32, difficulty: u8 }
    /// # let mut registry = Registry::new();
    /// # registry.insert_resource(GameSettings { volume: 0.8, difficulty: 2 });
    /// let settings = registry.get_resource::<GameSettings>();
    /// if let Some(settings) = settings {
    ///     println!("Volume: {}", settings.volume);
    /// #   assert_eq!(settings.volume, 0.8);
    /// }
    /// ```
    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.resources.get::<R>()
    }

    /// Gets a mutable reference to a resource if it exists
    ///
    /// # Example
    /// ```rust
    /// # use recs::prelude::{Registry, Resource};
    /// # #[derive(Resource, Debug, Clone)]
    /// # struct GameSettings { volume: f32, difficulty: u8 }
    /// # let mut registry = Registry::new();
    /// # registry.insert_resource(GameSettings { volume: 0.8, difficulty: 2 });
    /// if let Some(mut settings) = registry.get_resource_mut::<GameSettings>() {
    ///     settings.volume = 0.9;
    /// }
    /// # assert_eq!(registry.get_resource::<GameSettings>().unwrap().volume, 0.9);
    /// ```
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resources.get_mut::<R>()
    }

    /// Removes a resource from the registry and returns it
    ///
    /// # Example
    /// ```rust
    /// # use recs::prelude::{Registry, Resource};
    /// # #[derive(Resource, Debug, Clone, PartialEq)]
    /// # struct GameSettings { volume: f32, difficulty: u8 }
    /// # let mut registry = Registry::new();
    /// # registry.insert_resource(GameSettings { volume: 0.8, difficulty: 2 });
    /// let settings = registry.remove_resource::<GameSettings>();
    /// # assert_eq!(settings, Some(GameSettings { volume: 0.8, difficulty: 2 }));
    /// # assert!(!registry.has_resource::<GameSettings>());
    /// ```
    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        self.resources.remove::<R>()
    }

    /// Checks if a resource of the given type exists
    ///
    /// # Example
    /// ```rust
    /// # use recs::prelude::{Registry, Resource};
    /// # #[derive(Resource, Debug, Clone)]
    /// # struct GameSettings { volume: f32, difficulty: u8 }
    /// # let mut registry = Registry::new();
    /// # registry.insert_resource(GameSettings { volume: 0.8, difficulty: 2 });
    /// if registry.has_resource::<GameSettings>() {
    ///     println!("Game settings are configured!");
    /// }
    /// # assert!(registry.has_resource::<GameSettings>());
    /// ```
    pub fn has_resource<R: Resource>(&self) -> bool {
        self.resources.contains::<R>()
    }

    /// Inserts a resource with a default value if it doesn't exist
    ///
    /// # Example
    /// ```rust
    /// # use recs::prelude::{Registry, Resource};
    /// # #[derive(Resource, Debug, Clone, Default)]
    /// # struct GameSettings { volume: f32, difficulty: u8 }
    /// # let mut registry = Registry::new();
    /// registry.init_resource::<GameSettings>();
    /// # assert!(registry.has_resource::<GameSettings>());
    /// ```
    pub fn init_resource<R: Resource + Default>(&mut self) {
        if !self.has_resource::<R>() {
            self.insert_resource(R::default());
        }
    }
}

/// Implementation for spawning single components
impl<C: Component + 'static> ComponentBundle for C {
    fn add_to_entity(self, registry: &mut Registry, entity: Entity) -> Result<(), RecsError> {
        registry.add_component(entity, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: i32,
    }

    impl Component for Position {}

    #[derive(Debug, PartialEq)]
    struct Velocity {
        dx: i32,
    }

    impl Component for Velocity {}

    #[derive(Debug, PartialEq)]
    struct GameTime {
        time: f32,
    }

    impl Resource for GameTime {}

    #[test]
    fn test_spawn_and_get_component() {
        let mut registry = Registry::new();
        let entity = registry.spawn((Position { x: 10 }, Velocity { dx: -1 }));

        let pos = registry.get_component::<Position>(entity).unwrap();
        assert_eq!(pos, &Position { x: 10 });

        let vel = registry.get_component::<Velocity>(entity).unwrap();
        assert_eq!(vel, &Velocity { dx: -1 });
    }

    #[test]
    fn test_destroy_entity_removes_all_components() {
        let mut registry = Registry::new();
        let entity = registry.spawn((Position { x: 10 }, Velocity { dx: -1 }));

        assert!(registry.get_component::<Position>(entity).is_some());

        registry.destroy_entity(entity).unwrap();

        assert!(registry.get_component::<Position>(entity).is_none());
        assert!(registry.get_component::<Velocity>(entity).is_none());
    }

    #[test]
    fn test_simple_query() {
        let mut registry = Registry::new();
        registry.spawn((Position { x: 1 },));
        registry.spawn((Position { x: 2 }, Velocity { dx: 10 }));
        registry.spawn((Velocity { dx: 20 },));

        let mut count = 0;
        for (pos,) in registry.query::<(&Position,)>() {
            assert!(pos.x == 1 || pos.x == 2);
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_resource_management() {
        let mut registry = Registry::new();
        registry.insert_resource(GameTime { time: 0.0 });

        let time_res = registry.get_resource::<GameTime>().unwrap();
        assert_eq!(time_res, &GameTime { time: 0.0 });

        let time_res_mut = registry.get_resource_mut::<GameTime>().unwrap();
        time_res_mut.time = 1.0;

        assert_eq!(registry.get_resource::<GameTime>().unwrap().time, 1.0);
    }
}
