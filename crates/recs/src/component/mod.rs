use std::any::Any;

pub mod sparse_set;

/// A trait for types that can be used as components in the RECS system.
///
/// Components must be:
/// - Send: Can be transferred across thread boundaries
/// - Sync: Can be shared between threads
/// - 'static: Have a static lifetime
///
/// Components are pure data containers that can be attached to entities.
/// They should not contain any behavior - that belongs in systems.
pub trait Component: Send + Sync + 'static {}

/// Internal trait for component storage implementations.
/// Provides a type-erased way to store and remove components.
///
/// This trait is implemented by SparseSet and allows the Registry
/// to manage components without knowing their concrete types.
pub trait ComponentStorage: Any {
    /// Removes a component by its entity ID and returns it boxed as Any
    fn remove_by_id(&mut self, id: usize) -> Option<Box<dyn Any>>;
}
