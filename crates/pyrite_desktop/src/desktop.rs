use pyrite_asset::{loaders::gltf::GltfLoader, Assets};
use winit::{
    event::{
        DeviceEvent as WinitDeviceEvent, Event as WinitEvent, WindowEvent as WinitWindowEvent,
    },
    event_loop::EventLoop,
};

use pyrite_app::{resource::ResMut, stage::DEFAULT_STAGE, AppBuilder};
use pyrite_input::{Input, SubmitInput};
use pyrite_vulkan::{
    swapchain::Swapchain, Vulkan, VulkanAllocator, VulkanConfig, VulkanStager, STAGING_QUEUE,
};

use crate::{
    key::to_pyrite_key,
    window::{self, Window, WindowConfig, WindowEvent},
};

#[derive(Clone)]
pub struct DesktopConfig {
    /// The name of the application, used internally for vulkan.
    pub application_name: String,
    pub window_config: WindowConfig,
}

/// Sets up the desktop resources needed for the DesktopEntryPoint using the given config.
/// Must be setup on the same thread as the DesktopEntryPoint will be run on.
///
/// Adds the following resources:
/// - Window: Managing window state and events.
/// - Input: Managing input state and events from the window.
/// - Vulkan: Managing vulkan and gives access to a device.
/// - Swapchain: Managing the swapchain and links to the Window resource.
/// - VulkanAllocator: Managing memory allocations.
/// - Assets: Managing assets.
///
/// Creates the following systems:
/// - window::system_window_hotkeys: Handles window hotkeys such as Window Fullscreen.
/// - Assets::update: Updates the assets for background asynchronous loading.
///
/// Sets the entry point to handle control flow and runs the DEFAULT_STAGE.
pub fn setup_desktop_preset(app_builder: &mut AppBuilder, config: DesktopConfig) {
    let event_loop = EventLoop::new();

    // Setup window.
    app_builder.add_resource(Window::new(&event_loop, config.window_config.clone()));
    app_builder.add_resource(Input::new());

    // Setup default rendering resources and systems.
    {
        let vulkan = {
            Vulkan::new(
                VulkanConfig::from_window(
                    config.application_name.clone(),
                    &*app_builder.get_resource::<Window>(),
                )
                .queue(&STAGING_QUEUE),
            )
        };
        app_builder.add_resource(vulkan);

        let swapchain = Swapchain::new(&*app_builder.get_resource::<Vulkan>());
        app_builder.add_resource(swapchain);

        let allocator = VulkanAllocator::new(&*app_builder.get_resource::<Vulkan>());
        app_builder.add_resource(allocator);

        let stager = VulkanStager::new(
            &*app_builder.get_resource::<Vulkan>(),
            &mut *app_builder.get_resource_mut::<VulkanAllocator>(),
        );
        app_builder.add_resource(stager);
    }

    // Setup Assets
    let mut assets = Assets::new();
    assets.add_loader(GltfLoader::new());
    app_builder.add_resource(assets);
    app_builder.add_system(|mut assets: ResMut<Assets>| {
        assets.update();
    });

    app_builder.add_system(window::system_window_hotkeys);

    app_builder.set_entry_point(|mut application| {
        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();

            // Application will exit when the window is closed, everything should auto cleanup on drop.
            if application.get_resource::<Window>().should_close() {
                *control_flow = winit::event_loop::ControlFlow::Exit;
                return;
            }

            match event {
                WinitEvent::WindowEvent { event, .. } => match event {
                    WinitWindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    WinitWindowEvent::Resized(size) => {
                        application
                            .get_resource_mut::<Window>()
                            .push_event(WindowEvent::Resized(size.width, size.height));
                        let vulkan = application.get_resource::<Vulkan>();
                        application
                            .get_resource_mut::<Swapchain>()
                            .refresh(&*vulkan);
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
                    application.execute_stage(DEFAULT_STAGE);

                    application.get_resource_mut::<Input>().clear_inputs();
                }
                _ => (),
            }
        });
    });
}
