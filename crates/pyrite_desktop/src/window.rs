use std::collections::HashSet;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::EventLoop,
    window::Window as WinitWindow,
};

use pyrite_app::resource::{Res, ResMut, Resource};
use pyrite_input::{
    keyboard::{Key, Modifier},
    Input,
};
use pyrite_vulkan::SurfaceWindow;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum WindowEvent {
    Resized(u32, u32),
}

#[derive(Resource)]
pub struct Window {
    winit_window: WinitWindow,
    should_close: bool,
    events: HashSet<WindowEvent>,
}

impl Window {
    pub fn new(event_loop: &EventLoop<()>, config: WindowConfig) -> Self {
        let primary_monitor = event_loop.primary_monitor().expect("No primary monitor");
        let video_mode = primary_monitor.video_modes().next().unwrap();
        let video_mode_size = video_mode.size();

        let window_size = match config.state {
            WindowState::Windowed(width, height) => LogicalSize::new(width, height),
            WindowState::Fullscreen => {
                LogicalSize::new(video_mode_size.width, video_mode_size.height)
            }
        };

        let window_position = match config.state {
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

        let window_fullscreen = match config.state {
            WindowState::Windowed(_, _) => None,
            WindowState::Fullscreen => Some(winit::window::Fullscreen::Exclusive(video_mode)),
        };

        let window = winit::window::WindowBuilder::new()
            .with_title(config.title)
            .with_position(window_position)
            .with_resizable(window_resizable)
            .with_inner_size(window_size)
            .with_fullscreen(window_fullscreen)
            .with_visible(false)
            .build(event_loop)
            .unwrap();

        window.set_visible(true);

        Self {
            winit_window: window,
            should_close: false,
            events: HashSet::new(),
        }
    }

    pub(crate) fn push_event(&mut self, event: WindowEvent) {
        self.events.insert(event);
    }

    pub fn resized(&self) -> Option<(u32, u32)> {
        self.events.iter().find_map(|event| match event {
            WindowEvent::Resized(width, height) => Some((*width, *height)),
        })
    }

    pub fn fullscreen(&self) -> bool {
        self.winit_window.fullscreen().is_some()
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        let primary_monitor = self.winit_window.current_monitor().unwrap();
        let video_mode = primary_monitor.video_modes().next().unwrap();
        let video_mode_size = video_mode.size();

        let window_size = match fullscreen {
            true => LogicalSize::new(video_mode_size.width, video_mode_size.height),
            // TODO: Change hard coded values to a system where the window size is scaled down an
            // increment of the common 16:9 aspect ratios. Example: 1920x1080 -> 1600x900.
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

        self.winit_window.set_fullscreen(window_fullscreen);
        self.winit_window.set_outer_position(window_position);
        self.winit_window.set_inner_size(window_size);
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.winit_window.set_cursor_visible(visible);
    }

    pub fn set_cursor_grab_mode(&mut self, grab_mode: CursorGrabMode) {
        self.winit_window
            .set_cursor_grab(match grab_mode {
                CursorGrabMode::None => winit::window::CursorGrabMode::None,
                CursorGrabMode::Confined => winit::window::CursorGrabMode::Confined,
                CursorGrabMode::Locked => winit::window::CursorGrabMode::Locked,
            })
            .unwrap();
    }

    pub fn width(&self) -> u32 {
        self.winit_window.inner_size().width
    }

    pub fn height(&self) -> u32 {
        self.winit_window.inner_size().height
    }

    pub fn close(&mut self) {
        self.should_close = true;
    }

    pub(crate) fn should_close(&self) -> bool {
        self.should_close
    }
}

pub enum CursorGrabMode {
    None,
    Confined,
    Locked,
}

impl SurfaceWindow for Window {}

unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.winit_window.raw_display_handle()
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.winit_window.raw_window_handle()
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

    if input.is_key_pressed_with_modifiers(Key::Enter, &[Modifier::Alt]) {
        toggle_fullscreen(&mut window);
    }

    if input.is_key_pressed_with_modifiers(Key::F4, &[Modifier::Alt]) {
        window.close();
    }
}
