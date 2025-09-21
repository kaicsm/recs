use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// A trait for types that can be used as resources in the RECS system.
///
/// Resources are singleton data that can be accessed by systems.
/// They are useful for global state, configuration, and services.
///
/// Resources must be:
/// - Send: Can be transferred across thread boundaries
/// - Sync: Can be shared between threads
/// - 'static: Have a static lifetime
pub trait Resource: Send + Sync + 'static {}

/// Storage for resources in the ECS system.
///
/// Resources are stored in a type-erased HashMap and can be accessed
/// by their TypeId. Only one instance of each resource type can exist.
#[derive(Default)]
pub struct ResourceStorage {
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ResourceStorage {
    /// Creates a new empty ResourceStorage
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Inserts a resource into the storage.
    /// If a resource of the same type already exists, it will be replaced.
    pub fn insert<R: Resource>(&mut self, resource: R) {
        let type_id = TypeId::of::<R>();
        self.resources.insert(type_id, Box::new(resource));
    }

    /// Gets a reference to a resource if it exists
    pub fn get<R: Resource>(&self) -> Option<&R> {
        let type_id = TypeId::of::<R>();
        self.resources
            .get(&type_id)
            .and_then(|resource| resource.downcast_ref::<R>())
    }

    /// Gets a mutable reference to a resource if it exists
    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        let type_id = TypeId::of::<R>();
        self.resources
            .get_mut(&type_id)
            .and_then(|resource| resource.downcast_mut::<R>())
    }

    /// Removes a resource from storage and returns it
    pub fn remove<R: Resource>(&mut self) -> Option<R> {
        let type_id = TypeId::of::<R>();
        self.resources
            .remove(&type_id)
            .and_then(|resource| resource.downcast::<R>().ok())
            .map(|boxed| *boxed)
    }

    /// Checks if a resource of the given type exists
    pub fn contains<R: Resource>(&self) -> bool {
        let type_id = TypeId::of::<R>();
        self.resources.contains_key(&type_id)
    }

    /// Returns the number of resources stored
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Returns true if no resources are stored
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Clears all resources from storage
    pub fn clear(&mut self) {
        self.resources.clear();
    }
}

/// A system parameter that provides read-only access to a resource
pub struct Res<'a, R: Resource> {
    resource: &'a R,
}

impl<'a, R: Resource> Res<'a, R> {
    pub fn new(resource: &'a R) -> Self {
        Self { resource }
    }
}

impl<'a, R: Resource> std::ops::Deref for Res<'a, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

/// A system parameter that provides mutable access to a resource
pub struct ResMut<'a, R: Resource> {
    resource: &'a mut R,
}

impl<'a, R: Resource> ResMut<'a, R> {
    pub fn new(resource: &'a mut R) -> Self {
        Self { resource }
    }
}

impl<'a, R: Resource> std::ops::Deref for ResMut<'a, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<'a, R: Resource> std::ops::DerefMut for ResMut<'a, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource
    }
}

/// A system parameter that provides optional read-only access to a resource
pub struct OptionalRes<'a, R: Resource> {
    resource: Option<&'a R>,
}

impl<'a, R: Resource> OptionalRes<'a, R> {
    pub fn new(resource: Option<&'a R>) -> Self {
        Self { resource }
    }

    pub fn is_some(&self) -> bool {
        self.resource.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.resource.is_none()
    }

    pub fn as_ref(&self) -> Option<&'a R> {
        self.resource
    }
}

impl<'a, R: Resource> std::ops::Deref for OptionalRes<'a, R> {
    type Target = Option<&'a R>;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

/// A system parameter that provides optional mutable access to a resource
pub struct OptionalResMut<'a, R: Resource> {
    resource: Option<&'a mut R>,
}

impl<'a, R: Resource> OptionalResMut<'a, R> {
    pub fn new(resource: Option<&'a mut R>) -> Self {
        Self { resource }
    }

    pub fn is_some(&self) -> bool {
        self.resource.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.resource.is_none()
    }

    pub fn as_mut(&mut self) -> Option<&mut R> {
        match &mut self.resource {
            Some(r) => Some(*r),
            None => None,
        }
    }
}

impl<'a, R: Resource> std::ops::Deref for OptionalResMut<'a, R> {
    type Target = Option<&'a mut R>;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

impl<'a, R: Resource> std::ops::DerefMut for OptionalResMut<'a, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}
