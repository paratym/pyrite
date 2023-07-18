use winit::event::{
    DeviceEvent as WinitDeviceEvent, Event as WinitEvent, WindowEvent as WinitWindowEvent,
};
use winit::event_loop::EventLoop;

use pyrite_app::AppBuilder;
use pyrite_input::{Input, SubmitInput};
use pyrite_vulkan::swapchain::Swapchain;
use pyrite_vulkan::{Vulkan, VulkanConfig};

use crate::key::to_pyrite_key;
use crate::window::{self, Window, WindowConfig};

#[derive(Clone)]
pub struct DesktopConfig {
    /// The name of the application, used internally for vulkan.
    pub application_name: String,
    pub window_config: WindowConfig,
}

/// Sets up the desktop resources needed for the DesktopEntryPoint using the given config.
/// Must be setup on the same thread as the DesktopEntryPoint will be run on.
pub fn setup_desktop_preset(app_builder: &mut AppBuilder, config: DesktopConfig) {
    let event_loop = EventLoop::new();

    // Setup window
    app_builder.add_resource(Window::new(&event_loop, config.window_config.clone()));
    app_builder.add_resource(Input::new());

    // Setup rendering
    {
        let vulkan = {
            Vulkan::new(VulkanConfig::from_window(
                config.application_name.clone(),
                &*app_builder.get_resource::<Window>(),
            ))
        };
        app_builder.add_resource(vulkan);

        let swapchain = Swapchain::new(&*app_builder.get_resource::<Vulkan>());
        app_builder.add_resource(swapchain);
    }

    app_builder.add_system(window::system_window_hotkeys);

    app_builder.set_entry_point(|mut application| {
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
                    WinitWindowEvent::Resized(_size) => {
                        let vulkan = application.get_resource::<Vulkan>();
                        application
                            .get_resource_mut::<Swapchain>()
                            .refresh(&*vulkan);
                    },
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
    });
}
