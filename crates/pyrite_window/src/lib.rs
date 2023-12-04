mod window;

pub use window::*;

pub mod prelude {
    pub use crate::window::{Window, WindowConfig};
}
