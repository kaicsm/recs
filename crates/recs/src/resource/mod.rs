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
        self.resource.as_mut().map(|r| &mut **r)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct GameConfig {
        speed: f32,
        paused: bool,
    }
    impl Resource for GameConfig {}

    #[derive(Debug, PartialEq, Default)]
    struct Score(u32);
    impl Resource for Score {}

    #[test]
    fn test_storage_new_is_empty() {
        let storage = ResourceStorage::new();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut storage = ResourceStorage::new();
        storage.insert(GameConfig {
            speed: 1.0,
            paused: false,
        });

        assert_eq!(storage.len(), 1);
        assert!(storage.contains::<GameConfig>());
        assert!(!storage.contains::<Score>());

        let config = storage.get::<GameConfig>().unwrap();
        assert_eq!(config.speed, 1.0);
        assert!(!config.paused);

        assert!(storage.get::<Score>().is_none());
    }

    #[test]
    fn test_get_mut_and_modify() {
        let mut storage = ResourceStorage::new();
        storage.insert(Score(100));

        let score = storage.get_mut::<Score>().unwrap();
        score.0 += 50;

        let updated_score = storage.get::<Score>().unwrap();
        assert_eq!(updated_score.0, 150);
    }

    #[test]
    fn test_insert_overwrites_existing_resource() {
        let mut storage = ResourceStorage::new();
        storage.insert(Score(50));
        assert_eq!(storage.get::<Score>().unwrap().0, 50);

        storage.insert(Score(100));
        assert_eq!(
            storage.len(),
            1,
            "Length should not increase when overwriting."
        );
        assert_eq!(storage.get::<Score>().unwrap().0, 100);
    }

    #[test]
    fn test_remove_resource() {
        let mut storage = ResourceStorage::new();
        storage.insert(Score(99));

        assert!(storage.contains::<Score>());

        let removed_score = storage.remove::<Score>();
        assert_eq!(removed_score, Some(Score(99)));

        assert!(!storage.contains::<Score>());
        assert!(storage.get::<Score>().is_none());
        assert_eq!(storage.len(), 0);

        let removed_again = storage.remove::<Score>();
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_clear_removes_all_resources() {
        let mut storage = ResourceStorage::new();
        storage.insert(GameConfig {
            speed: 2.0,
            paused: true,
        });
        storage.insert(Score(1000));

        assert_eq!(storage.len(), 2);
        storage.clear();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
        assert!(!storage.contains::<GameConfig>());
        assert!(!storage.contains::<Score>());
    }

    #[test]
    fn test_res_and_resmut_deref() {
        let mut config = GameConfig {
            speed: 1.0,
            paused: false,
        };

        let res_config = Res::new(&config);
        assert_eq!(res_config.speed, 1.0);
        assert!(!res_config.paused);

        let mut resmut_config = ResMut::new(&mut config);
        resmut_config.paused = true;

        assert!(config.paused);
    }

    #[test]
    fn test_optional_res_wrapper() {
        let config = GameConfig {
            speed: 1.0,
            paused: false,
        };

        let opt_res_some = OptionalRes::new(Some(&config));
        assert!(opt_res_some.is_some());
        assert!(!opt_res_some.is_none());
        assert_eq!(opt_res_some.as_ref().unwrap(), &config);
        assert_eq!(opt_res_some.as_ref().unwrap().speed, 1.0);

        let opt_res_none: OptionalRes<GameConfig> = OptionalRes::new(None);
        assert!(opt_res_none.is_none());
        assert!(!opt_res_none.is_some());
        assert!(opt_res_none.as_ref().is_none());
    }

    #[test]
    fn test_optional_res_mut_wrapper() {
        let mut config = GameConfig {
            speed: 1.0,
            paused: false,
        };

        let mut opt_res_mut_some = OptionalResMut::new(Some(&mut config));
        assert!(opt_res_mut_some.is_some());

        let inner_mut = opt_res_mut_some.as_mut();
        assert!(inner_mut.is_some());
        inner_mut.unwrap().speed = 99.0;

        assert_eq!(config.speed, 99.0);

        let mut opt_res_mut_none: OptionalResMut<GameConfig> = OptionalResMut::new(None);
        assert!(opt_res_mut_none.is_none());
        assert!(opt_res_mut_none.as_mut().is_none());
    }
}
