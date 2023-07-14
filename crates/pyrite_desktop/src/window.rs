use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event_loop::EventLoop;
use winit::window::Window as WinitWindow;

use pyrite_app::resource::{Res, ResMut, Resource};
use pyrite_input::keyboard::{Key, Modifier};
use pyrite_input::Input;

#[derive(Resource)]
pub struct Window {
    config: WindowConfig,
    winit_window: Option<WinitWindow>,
    should_close: bool,
}

impl Window {
    pub fn new(config: WindowConfig) -> Self {
        Self {
            config,
            winit_window: None,
            should_close: false,
        }
    }

    pub(crate) fn init_window(&mut self, event_loop: &EventLoop<()>) {
        let primary_monitor = event_loop.primary_monitor().expect("No primary monitor");
        let video_mode = primary_monitor.video_modes().next().unwrap();
        let video_mode_size = video_mode.size();

        let window_size = match self.config.state {
            WindowState::Windowed(width, height) => LogicalSize::new(width, height),
            WindowState::Fullscreen => {
                LogicalSize::new(video_mode_size.width, video_mode_size.height)
            }
        };

        let window_position = match self.config.state {
            WindowState::Windowed(_, _) => {
                let monitor_size = primary_monitor.size();
                LogicalPosition::new(
                    (monitor_size.width - window_size.width) / 2,
                    (monitor_size.height - window_size.height) / 2,
                )
            }
            WindowState::Fullscreen => LogicalPosition::new(0, 0),
        };

        let window_resizable = false;

        let window_fullscreen = match self.config.state {
            WindowState::Windowed(_, _) => None,
            WindowState::Fullscreen => Some(winit::window::Fullscreen::Exclusive(video_mode)),
        };

        let window = winit::window::WindowBuilder::new()
            .with_title(&self.config.title)
            .with_position(window_position)
            .with_resizable(window_resizable)
            .with_inner_size(window_size)
            .with_fullscreen(window_fullscreen)
            .with_visible(false)
            .build(event_loop)
            .unwrap();

        self.winit_window = Some(window);

        self.winit_window_mut().set_visible(true);
    }

    pub fn fullscreen(&self) -> bool {
        self.winit_window().fullscreen().is_some()
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        let primary_monitor = self.winit_window().current_monitor().unwrap();
        let video_mode = primary_monitor.video_modes().next().unwrap();
        let video_mode_size = video_mode.size();

        let window_size = match fullscreen {
            true => LogicalSize::new(video_mode_size.width, video_mode_size.height),
            // TODO: Change hard coded values to a system where the window size is scaled down an increment of the common 16:9 aspect ratios. Example: 1920x1080 -> 1600x900.
            false => LogicalSize::new(1280, 720),
        };

        let window_position = match fullscreen {
            true => LogicalPosition::new(0, 0),
            false => {
                let monitor_size = primary_monitor.size();
                LogicalPosition::new(
                    (monitor_size.width - window_size.width) / 2,
                    (monitor_size.height - window_size.height) / 2,
                )
            }
        };

        let window_fullscreen = match fullscreen {
            true => Some(winit::window::Fullscreen::Exclusive(video_mode)),
            false => None,
        };

        self.winit_window_mut().set_fullscreen(window_fullscreen);
        self.winit_window_mut().set_outer_position(window_position);
        self.winit_window_mut().set_inner_size(window_size);
    }

    pub fn width(&self) -> u32 {
        self.winit_window().inner_size().width
    }

    pub fn height(&self) -> u32 {
        self.winit_window().inner_size().height
    }

    pub fn close(&mut self) {
        self.should_close = true;
    }

    pub(crate) fn should_close(&self) -> bool {
        self.should_close
    }

    fn winit_window(&self) -> &WinitWindow {
        self.winit_window.as_ref().expect("Window not initialized")
    }

    fn winit_window_mut(&mut self) -> &mut WinitWindow {
        self.winit_window.as_mut().expect("Window not initialized")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowState {
    Windowed(u32, u32),
    Fullscreen,
}

#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub title: String,
    pub state: WindowState,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Pyrite Game".to_string(),
            state: WindowState::Windowed(1280, 720),
        }
    }
}

pub fn system_window_hotkeys(mut window: ResMut<Window>, input: Res<Input>) {
    fn toggle_fullscreen(window: &mut ResMut<Window>) {
        let is_fullscreen = window.fullscreen();
        window.set_fullscreen(!is_fullscreen);
    }

    if input.is_key_pressed_with_modifiers(Key::Enter, &[Modifier::Alt])
        || input.is_key_pressed(Key::F11)
    {
        toggle_fullscreen(&mut window);
    }
}
