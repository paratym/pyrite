mod app;
pub use app::*;

pub mod resource;
pub mod scheduler;
pub mod stage;
pub mod system;

pub mod prelude {
    pub use crate::{
        app::{AppBuilder, Application},
        resource::{Res, ResMut, Resource},
        stage::StageBuilder,
    };
}
