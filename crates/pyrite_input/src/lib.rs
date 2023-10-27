pub mod input;
pub use input::*;

pub mod keyboard;
pub mod mouse;

pub mod prelude {
    pub use crate::{
        input::Input,
        keyboard::{Key, Keyboard, Modifier},
    };
}
