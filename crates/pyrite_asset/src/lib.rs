extern crate shaderc;

mod asset;
pub mod loaders;

pub use asset::*;

pub mod prelude {
    pub use crate::{AssetLoader, Assets, Handle};
}
