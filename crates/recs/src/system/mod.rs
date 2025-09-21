use crate::{
    query::{Query, QueryParam},
    registry::Registry,
    resource::{OptionalRes, OptionalResMut, Res, ResMut, Resource},
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

/// Trait for system parameters that can be extracted from the Registry
pub trait SystemParam {
    /// Extract this parameter from the registry
    ///
    /// # Safety
    /// This function uses raw pointers to work around lifetime issues.
    /// The caller must ensure that the registry remains valid for the
    /// lifetime of the returned parameter.
    unsafe fn from_registry(registry: *mut Registry) -> Self;
}

impl<'q, Q: QueryParam<'q>> SystemParam for Query<'q, Q> {
    unsafe fn from_registry(registry: *mut Registry) -> Self {
        unsafe { Query::new(&mut *registry) }
    }
}

impl<R: Resource> SystemParam for Res<'_, R> {
    unsafe fn from_registry(registry: *mut Registry) -> Self {
        unsafe {
            let resource = (*registry).resources.get::<R>().expect(&format!(
                "Resource {} not found. Did you forget to insert it?",
                std::any::type_name::<R>()
            ));
            Res::new(resource)
        }
    }
}

impl<R: Resource> SystemParam for ResMut<'_, R> {
    unsafe fn from_registry(registry: *mut Registry) -> Self {
        unsafe {
            let resource = (*registry).resources.get_mut::<R>().expect(&format!(
                "Resource {} not found. Did you forget to insert it?",
                std::any::type_name::<R>()
            ));
            ResMut::new(resource)
        }
    }
}

impl<R: Resource> SystemParam for OptionalRes<'_, R> {
    unsafe fn from_registry(registry: *mut Registry) -> Self {
        unsafe {
            let resource = (*registry).resources.get::<R>();
            OptionalRes::new(resource)
        }
    }
}

impl<R: Resource> SystemParam for OptionalResMut<'_, R> {
    unsafe fn from_registry(registry: *mut Registry) -> Self {
        unsafe {
            let resource = (*registry).resources.get_mut::<R>();
            OptionalResMut::new(resource)
        }
    }
}

/// A system that wraps a function taking system parameters
pub struct FunctionSystem<F, Params> {
    func: F,
    _phantom: std::marker::PhantomData<Params>,
}

impl<F, Params> FunctionSystem<F, Params> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

macro_rules! impl_system {
    ($($param:ident),*) => {
        #[allow(non_snake_case)]
        impl<F, $($param: SystemParam),*> System for FunctionSystem<F, ($($param,)*)>
        where
            F: FnMut($($param),*) + 'static,
        {
            fn run(&mut self, registry: &mut Registry) {
                #[allow(unused_unsafe)]
                unsafe {
                    #[allow(unused_variables)]
                    let registry_ptr = registry as *mut Registry;
                    $(let $param = $param::from_registry(registry_ptr);)*
                    (self.func)($($param),*);
                }
            }
        }

        #[allow(non_snake_case)]
        impl<F, $($param: SystemParam),*> IntoSystem<($($param,)*)> for F
        where
            F: FnMut($($param),*) + 'static,
        {
            type System = FunctionSystem<F, ($($param,)*)>;

            fn into_system(self) -> Self::System {
                FunctionSystem::new(self)
            }
        }
    };
}

impl_system!();
impl_system!(P0);
impl_system!(P0, P1);
impl_system!(P0, P1, P2);
impl_system!(P0, P1, P2, P3);
impl_system!(P0, P1, P2, P3, P4);
impl_system!(P0, P1, P2, P3, P4, P5);
impl_system!(P0, P1, P2, P3, P4, P5, P6);
impl_system!(P0, P1, P2, P3, P4, P5, P6, P7);
impl_system!(P0, P1, P2, P3, P4, P5, P6, P7, P8);
impl_system!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9);
impl_system!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10);
impl_system!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11);
impl_system!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12);
impl_system!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13);
impl_system!(
    P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14
);
impl_system!(
    P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15
);

#[cfg(test)]
mod tests {
    use crate::component::Component;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
    }

    impl Component for Position {}

    #[derive(Debug, PartialEq)]
    struct Velocity {
        dx: f32,
    }

    impl Component for Velocity {}

    #[derive(Debug, PartialEq)]
    struct Time {
        delta: f32,
    }

    impl Resource for Time {}

    #[derive(Default, Debug, PartialEq)]
    struct Counter {
        value: i32,
    }

    impl Resource for Counter {}

    fn movement_system(query: Query<(&mut Position, &Velocity)>) {
        for (pos, vel) in query {
            pos.x += vel.dx;
        }
    }

    fn time_reader_system(time: Res<Time>, mut counter: ResMut<Counter>) {
        if time.delta > 0.0 {
            counter.value += 1;
        }
    }

    fn optional_resource_system(time: OptionalRes<Time>, mut counter: ResMut<Counter>) {
        if time.is_some() {
            counter.value = 10;
        } else {
            counter.value = -10;
        }
    }

    #[test]
    fn test_system_with_query() {
        let mut registry = Registry::new();
        let entity = registry.spawn((Position { x: 10.0 }, Velocity { dx: 5.0 }));

        registry.add_system(movement_system);
        registry.run_systems();

        let pos = registry.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 15.0);
    }

    #[test]
    fn test_system_with_resources() {
        let mut registry = Registry::new();
        registry.insert_resource(Time { delta: 0.1 });
        registry.init_resource::<Counter>();

        registry.add_system(time_reader_system);
        registry.run_systems();

        let counter = registry.get_resource::<Counter>().unwrap();
        assert_eq!(counter.value, 1);
    }

    #[test]
    fn test_system_with_optional_resource_present() {
        let mut registry = Registry::new();
        registry.insert_resource(Time { delta: 0.1 });
        registry.init_resource::<Counter>();

        registry.add_system(optional_resource_system);
        registry.run_systems();

        let counter = registry.get_resource::<Counter>().unwrap();
        assert_eq!(counter.value, 10);
    }

    #[test]
    fn test_system_with_optional_resource_absent() {
        let mut registry = Registry::new();
        registry.init_resource::<Counter>();

        registry.add_system(optional_resource_system);
        registry.run_systems();

        let counter = registry.get_resource::<Counter>().unwrap();
        assert_eq!(counter.value, -10);
    }

    #[test]
    #[should_panic(expected = "Resource recs::system::tests::Time not found")]
    fn test_system_panics_on_missing_required_resource() {
        let mut registry = Registry::new();
        registry.init_resource::<Counter>();

        registry.add_system(time_reader_system);
        registry.run_systems();
    }
}
