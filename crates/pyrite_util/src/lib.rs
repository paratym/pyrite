use std::sync::Arc;

pub use pyrite_util_macros::dependable;

pub mod prelude {
    pub use crate::Dependable;
}

/// A trait for allowing dependency creation.
///
pub trait Dependable {
    type Dep;

    fn create_dep(&self) -> Arc<Self::Dep>;
}
