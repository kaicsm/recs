use std::{any::TypeId, fmt};

use crate::entity::Entity;

/// Represents possible errors that can occur in the RECS system
#[derive(Debug)]
pub enum RecsError {
    /// The entity is no longer valid (was destroyed or never existed)
    InvalidEntity(Entity),
    /// The requested component type was not found on the entity
    ComponentNotFound(TypeId),
}

impl fmt::Display for RecsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecsError::InvalidEntity(entity) => {
                write!(
                    f,
                    "Operation on invalid entity: id={}, generation={}",
                    entity.id(),
                    entity.generation()
                )
            }
            RecsError::ComponentNotFound(type_id) => {
                write!(
                    f,
                    "Entity does not have component with TypeId {:?}",
                    type_id
                )
            }
        }
    }
}

impl std::error::Error for RecsError {}
