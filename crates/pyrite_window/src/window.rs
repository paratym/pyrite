use pyrite_app::resource::Resource;
use winit::{self, window::Window as WinitWindow};

pub struct WindowConfig {
    pub title: String,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Pyrite App".to_string(),
        }
    }
}

impl WindowConfig {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
}

#[derive(Resource)]
pub struct Window {
    winit_window: WinitWindow,
}

impl raw_window_handle::HasDisplayHandle for Window {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.winit_window.display_handle()
    }
}

impl raw_window_handle::HasWindowHandle for Window {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.winit_window.window_handle()
    }
}

impl Window {
    pub fn new(config: &WindowConfig, event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let winit_window = winit::window::WindowBuilder::new()
            .with_title(config.title.clone())
            .with_visible(false)
            .build(event_loop)
            .unwrap();

        Self { winit_window }
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.winit_window.set_visible(visible);
    }

    pub fn width(&self) -> u32 {
        self.winit_window.inner_size().width
    }

    pub fn height(&self) -> u32 {
        self.winit_window.inner_size().height
    }
}
