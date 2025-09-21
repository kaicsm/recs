pub use recs_macros::Component;
pub use recs_macros::Resource;

pub mod component;
pub mod entity;
pub mod error;
pub mod query;
pub mod registry;
pub mod resource;
pub mod system;

pub mod prelude {
    pub use crate::{
        Component, Resource, query::Query, registry::Registry, resource::OptionalRes,
        resource::OptionalResMut, resource::Res, resource::ResMut,
    };
}
