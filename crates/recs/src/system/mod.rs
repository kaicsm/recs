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
