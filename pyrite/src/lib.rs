pub mod app {
    pub use pyrite_app::*;
}

pub mod asset {
    pub use pyrite_asset::*;
}

pub mod desktop {
    pub use pyrite_desktop::*;
}

pub mod vulkan {
    pub use pyrite_vulkan::*;
}

pub mod input {
    pub use pyrite_input::*;
}

pub mod prelude {
    pub use pyrite_app::prelude::*;
    pub use pyrite_asset::prelude::*;
    pub use pyrite_desktop::prelude::*;
    pub use pyrite_input::prelude::*;
    pub use pyrite_vulkan::prelude::*;
}
