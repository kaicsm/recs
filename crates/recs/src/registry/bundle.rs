use crate::{component::Component, entity::Entity, error::RecsError, registry::Registry};

/// A trait for types that can be added as a bundle of components to an entity.
///
/// This trait is automatically implemented for tuples of components up to 32 elements.
pub trait ComponentBundle {
    /// Adds all components in the bundle to the given entity.
    fn add_to_entity(self, registry: &mut Registry, entity: Entity) -> Result<(), RecsError>;
}

macro_rules! impl_bundle_for_tuple {
    ($($name:ident),+) => {
        impl<$($name),+> ComponentBundle for ($($name,)+)
        where
            $($name: Component + 'static),+
        {
            #[allow(non_snake_case)]
            fn add_to_entity(self, registry: &mut Registry, entity: Entity) -> Result<(), RecsError> {
                let ($($name,)+) = self;
                $(
                    registry.add_component(entity, $name)?;
                )+
                Ok(())
            }
        }
    };
}

impl_bundle_for_tuple!(C0);
impl_bundle_for_tuple!(C0, C1);
impl_bundle_for_tuple!(C0, C1, C2);
impl_bundle_for_tuple!(C0, C1, C2, C3);
impl_bundle_for_tuple!(C0, C1, C2, C3, C4);
impl_bundle_for_tuple!(C0, C1, C2, C3, C4, C5);
impl_bundle_for_tuple!(C0, C1, C2, C3, C4, C5, C6);
impl_bundle_for_tuple!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_bundle_for_tuple!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_bundle_for_tuple!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_bundle_for_tuple!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_bundle_for_tuple!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_bundle_for_tuple!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_bundle_for_tuple!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22, C23
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22, C23, C24
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22, C23, C24, C25
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22, C23, C24, C25, C26
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22, C23, C24, C25, C26, C27
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22, C23, C24, C25, C26, C27, C28
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22, C23, C24, C25, C26, C27, C28, C29
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22, C23, C24, C25, C26, C27, C28, C29, C30
);
impl_bundle_for_tuple!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20,
    C21, C22, C23, C24, C25, C26, C27, C28, C29, C30, C31
);
