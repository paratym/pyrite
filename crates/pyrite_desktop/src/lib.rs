mod desktop;
pub use desktop::*;
mod key;
pub mod window;

pub mod prelude {
    pub use crate::{
        desktop::{setup_desktop_preset, DesktopConfig, DesktopEntryPoint},
        window::WindowConfig,
    };
}
