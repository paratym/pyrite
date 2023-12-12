mod app;
pub use app::*;

pub mod executor;
pub mod resource;
pub mod schedule;
pub mod stage;
pub mod system;

pub mod prelude {
    pub use crate::{
        app::{AppBuilder, Application},
        resource::{Res, ResMut, Resource},
        stage::StageBuilder,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_app() {
        let app = AppBuilder::new();
    }
}
