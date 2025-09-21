use crate::{
    query::{Query, QueryParam},
    registry::Registry,
};

/// A trait representing a system that can be executed in the ECS.
pub trait System {
    /// Execute the system logic
    fn run(&mut self, registry: &mut Registry);
}

/// A boxed system that can be stored in the Registry's system list
pub type BoxedSystem = Box<dyn System>;

/// Trait for creating systems from functions
pub trait IntoSystem<Params> {
    type System: System;

    fn into_system(self) -> Self::System;
}

/// A system that wraps a function taking a single query parameter
pub struct QuerySystem<F, Q> {
    func: F,
    _phantom: std::marker::PhantomData<Q>,
}

impl<F, Q> QuerySystem<F, Q> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F, Q> System for QuerySystem<F, Q>
where
    F: FnMut(Query<Q>) + 'static,
    for<'q> Q: QueryParam<'q>,
{
    fn run(&mut self, registry: &mut Registry) {
        // We need to use unsafe to work around lifetime issues
        // This is safe because:
        // 1. The registry reference is valid for the entire function call
        // 2. The QueryWrapper doesn't outlive this function
        // 3. No other code can access registry while this function runs
        unsafe {
            let registry_ptr = registry as *mut Registry;
            let query = Query::new(&mut *registry_ptr);
            (self.func)(query);
        }
    }
}

// Implement IntoSystem for functions that take a single query
impl<F, Q> IntoSystem<Query<'_, Q>> for F
where
    F: FnMut(Query<Q>) + 'static,
    for<'q> Q: QueryParam<'q>,
{
    type System = QuerySystem<F, Q>;

    fn into_system(self) -> Self::System {
        QuerySystem::new(self)
    }
}
