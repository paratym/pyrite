mod app;
pub use app::*;

pub mod resource;
pub mod scheduler;
pub mod system;

pub mod prelude {
    pub use crate::{
        app::{AppBuilder, Application, EntryPoint},
        resource::{Res, ResMut, Resource},
    };
}
