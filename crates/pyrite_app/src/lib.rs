mod app;
pub use app::*;

pub mod executor;
pub mod resource;
pub mod schedule;
pub mod system;

pub mod prelude {
    pub use crate::{
        app::{AppBuilder, Application},
        resource::{Res, ResMut, Resource},
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn async_app() {}
}
