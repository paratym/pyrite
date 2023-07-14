pub mod app {
    pub use pyrite_app::*;
}

pub mod desktop {
    pub use pyrite_desktop::*;
}

pub mod prelude {
    pub use pyrite_app::prelude::*;
    pub use pyrite_desktop::prelude::*;
    pub use pyrite_input::prelude::*;
}
