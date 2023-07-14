use winit::event::{
    DeviceEvent as WinitDeviceEvent, Event as WinitEvent, WindowEvent as WinitWindowEvent,
};

use pyrite_app::{AppBuilder, Application, EntryPoint};
use pyrite_input::{Input, SubmitInput};

use crate::key::to_pyrite_key;
use crate::window::{self, Window, WindowConfig};

#[derive(Clone)]
pub struct DesktopConfig {
    pub window_config: WindowConfig,
}

/// Sets up the desktop resources needed for the DesktopEntryPoint using the given config.
pub fn setup_desktop_preset(app_builder: &mut AppBuilder, config: DesktopConfig) {
    app_builder.add_resource(Window::new(config.window_config));
    app_builder.add_resource(Input::new());

    app_builder.add_system(window::system_window_hotkeys);
}

pub struct DesktopEntryPoint {}

impl EntryPoint for DesktopEntryPoint {
    fn run(mut application: Application) {
        let event_loop = winit::event_loop::EventLoop::new();
        {
            let mut window = application.get_resource_mut::<Window>();
            window.init_window(&event_loop);
        }
        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();

            // Application will exit when the window is closed, everything should auto complete on Drop.
            if application.get_resource::<Window>().should_close() {
                *control_flow = winit::event_loop::ControlFlow::Exit;
                return;
            }

            match event {
                WinitEvent::WindowEvent { event, .. } => match event {
                    WinitWindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    _ => (),
                },
                WinitEvent::DeviceEvent { event, .. } => match event {
                    WinitDeviceEvent::Key(input) => {
                        if let Some(key) = to_pyrite_key(input.physical_key) {
                            match input.state {
                                winit::event::ElementState::Pressed => application
                                    .get_resource_mut::<Input>()
                                    .submit_input(SubmitInput::Pressed(key)),
                                winit::event::ElementState::Released => application
                                    .get_resource_mut::<Input>()
                                    .submit_input(SubmitInput::Released(key)),
                            }
                        }
                    }
                    _ => (),
                },
                WinitEvent::MainEventsCleared => {
                    application.execute_systems();
                    application.get_resource_mut::<Input>().clear_inputs();
                }
                _ => (),
            }
        });
    }
}
