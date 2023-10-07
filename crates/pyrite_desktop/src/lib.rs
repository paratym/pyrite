pub use desktop::*;

mod desktop;

mod key;
pub mod window;

pub mod prelude {
    pub use crate::{
        desktop::{
            setup_desktop_preset,
            DesktopConfig,
        },
        window::WindowConfig,
    };
}
