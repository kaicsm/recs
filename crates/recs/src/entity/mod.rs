use crate::error::RecsError;

/// Represents a unique entity in the RECS system.
///
/// Each entity is identified by two numbers:
/// - An ID that can be reused when entities are destroyed
/// - A generation number that ensures old references to reused IDs are invalid
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Entity(u32, u32);

impl Entity {
    /// Creates a new Entity with the specified ID and generation number
    pub fn new(id: u32, generation: u32) -> Self {
        Self(id, generation)
    }

    /// Returns the entity's ID
    pub fn id(&self) -> u32 {
        self.0
    }

    /// Returns the entity's generation number
    pub fn generation(&self) -> u32 {
        self.1
    }
}

/// Manages entity lifecycle, including creation, destruction, and validation.
///
/// The EntityManager maintains:
/// - A list of generation numbers for each entity ID
/// - A list of freed entity IDs that can be reused
pub struct EntityManager {
    /// Generation numbers for each entity ID
    generations: Vec<u32>,
    /// List of entity IDs that can be reused
    free_list: Vec<usize>,
}

impl EntityManager {
    /// Creates a new empty EntityManager
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Creates a new entity with a unique ID and generation number.
    /// If there are freed IDs available, one will be reused with an incremented generation.
    pub fn create_entity(&mut self) -> Entity {
        if let Some(index) = self.free_list.pop() {
            let generation = self.generations[index];
            Entity(index as u32, generation)
        } else {
            let index = self.generations.len();
            self.generations.push(1);
            Entity(index as u32, 1)
        }
    }

    /// Destroys an entity, making its ID available for reuse.
    /// Returns an error if the entity is invalid.
    pub fn destroy_entity(&mut self, entity: Entity) -> Result<(), RecsError> {
        if !self.is_valid(entity) {
            return Err(RecsError::InvalidEntity(entity));
        }

        let index = entity.id() as usize;
        self.generations[index] += 1;
        self.free_list.push(index);

        Ok(())
    }

    /// Checks if an entity reference is still valid by comparing its generation
    /// number with the current generation for that entity ID.
    pub fn is_valid(&self, entity: Entity) -> bool {
        let index = entity.0 as usize;
        index < self.generations.len() && self.generations[index] == entity.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_first_entity() {
        let mut manager = EntityManager::new();
        let entity = manager.create_entity();

        assert_eq!(entity.id(), 0);
        assert_eq!(entity.generation(), 1);
        assert!(manager.is_valid(entity));
    }

    #[test]
    fn test_create_multiple_entities() {
        let mut manager = EntityManager::new();
        let entity1 = manager.create_entity();
        let entity2 = manager.create_entity();

        assert_eq!(entity1.id(), 0);
        assert_eq!(entity2.id(), 1);
    }

    #[test]
    fn test_destroy_and_reuse_entity_id() {
        let mut manager = EntityManager::new();
        let entity1 = manager.create_entity();

        assert!(manager.destroy_entity(entity1).is_ok());
        assert!(!manager.is_valid(entity1));

        let entity2 = manager.create_entity();

        assert_eq!(entity2.id(), 0);
        assert_eq!(entity2.generation(), 2);
        assert!(manager.is_valid(entity2));

        let old_invalid_entity = Entity::new(0, 1);
        assert!(!manager.is_valid(old_invalid_entity));
    }

    #[test]
    fn test_destroy_invalid_entity_returns_error() {
        let mut manager = EntityManager::new();
        let invalid_entity = Entity::new(10, 1);

        let result = manager.destroy_entity(invalid_entity);
        assert!(result.is_err());
        matches!(result.unwrap_err(), RecsError::InvalidEntity(_));
    }
}
