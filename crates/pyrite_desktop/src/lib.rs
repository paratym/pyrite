pub use desktop::*;

mod desktop;

mod input;
pub mod time;
pub mod window;

pub mod prelude {
    pub use crate::{
        desktop::{setup_desktop_preset, DesktopConfig},
        time::Time,
        window::{Window, WindowConfig},
    };
}
