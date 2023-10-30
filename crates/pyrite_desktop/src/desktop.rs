use pyrite_asset::{loaders::gltf::GltfLoader, Assets};
use winit::{
    event::{
        DeviceEvent as WinitDeviceEvent, Event as WinitEvent, WindowEvent as WinitWindowEvent,
    },
    event_loop::EventLoop,
};

use pyrite_app::{resource::ResMut, stage::DEFAULT_STAGE, AppBuilder};
use pyrite_input::{keyboard, mouse, Input};
use pyrite_vulkan::{
    swapchain::Swapchain, Vulkan, VulkanAllocator, VulkanConfig, VulkanStager, STAGING_QUEUE,
};

use crate::{
    input::{to_pyrite_button, to_pyrite_key},
    time::Time,
    window::{self, Window, WindowConfig, WindowEvent},
};

/// The pre-update stage, runs before the update/default stage.
pub const PRE_UPDATE_STAGE: &'static str = "pre_update";

/// The render stage, runs after the update/default stage.
pub const RENDER_STAGE: &'static str = "render";

#[derive(Clone)]
pub struct DesktopConfig {
    /// The name of the application, used internally for vulkan.
    pub application_name: String,
    pub window_config: WindowConfig,
    /// The stages to be ran in order, the desktop preset will create it's own stages which are
    /// added by default.
    pub stages: Vec<String>,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            application_name: "Pyrite Application".to_string(),
            window_config: WindowConfig::default(),
            stages: vec![
                PRE_UPDATE_STAGE.to_string(),
                DEFAULT_STAGE.to_string(),
                RENDER_STAGE.to_string(),
            ],
        }
    }
}

/// Sets up the desktop resources needed for the DesktopEntryPoint using the given config.
/// Must be setup on the same thread as the DesktopEntryPoint will be run on.
///
/// Adds the following resources:
/// - Window: Managing window state and events.
/// - Input: Managing input state and events from the window.e
/// - Vulkan: Managing vulkan and gives access to a device.
/// - Swapchain: Managing the swapchain and links to the Window resource.
/// - VulkanAllocator: Managing memory allocations.
/// - Assets: Managing assets.
/// - Time: Reference for application time.
///
/// Creates the following systems:
/// - window::system_window_hotkeys: Handles window hotkeys such as Window Fullscreen.
/// - Assets::update: Updates the assets for background asynchronous loading.
///
/// Sets the entry point to handle control flow and runs the DEFAULT_STAGE.
pub fn setup_desktop_preset(app_builder: &mut AppBuilder, config: DesktopConfig) {
    let event_loop = EventLoop::new();

    // Setup stages.
    app_builder.create_stage(PRE_UPDATE_STAGE.to_string(), |stage_builder| {});
    app_builder.create_stage(RENDER_STAGE.to_string(), |stage_builder| {});

    // Setup time.
    app_builder.add_resource(Time::new());

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
                ), // .queue(&STAGING_QUEUE),
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
    let assets = Assets::new();
    app_builder.add_resource(assets);
    app_builder.add_system_to_stage(
        |mut assets: ResMut<Assets>| {
            assets.update();
        },
        PRE_UPDATE_STAGE,
    );

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
                    WinitWindowEvent::CursorMoved { position, .. } => {
                        application
                            .get_resource_mut::<Input>()
                            .mouse_mut()
                            .submit_input(mouse::SubmitInput::Position(
                                position.x as f32,
                                position.y as f32,
                            ));
                    }
                    _ => (),
                },
                WinitEvent::DeviceEvent { event, .. } => match event {
                    WinitDeviceEvent::Key(input) => {
                        if let Some(key) = to_pyrite_key(input.physical_key) {
                            match input.state {
                                winit::event::ElementState::Pressed => application
                                    .get_resource_mut::<Input>()
                                    .keyboard_mut()
                                    .submit_input(keyboard::SubmitInput::Pressed(key)),
                                winit::event::ElementState::Released => application
                                    .get_resource_mut::<Input>()
                                    .keyboard_mut()
                                    .submit_input(keyboard::SubmitInput::Released(key)),
                            }
                        }
                    }
                    WinitDeviceEvent::MouseMotion { delta } => {
                        application
                            .get_resource_mut::<Input>()
                            .mouse_mut()
                            .submit_input(mouse::SubmitInput::Delta(
                                delta.0 as f32,
                                delta.1 as f32,
                            ));
                    }
                    WinitDeviceEvent::Button { button, state } => {
                        if let Some(button) = to_pyrite_button(button) {
                            match state {
                                winit::event::ElementState::Pressed => application
                                    .get_resource_mut::<Input>()
                                    .mouse_mut()
                                    .submit_input(mouse::SubmitInput::Pressed(button)),
                                winit::event::ElementState::Released => application
                                    .get_resource_mut::<Input>()
                                    .mouse_mut()
                                    .submit_input(mouse::SubmitInput::Released(button)),
                            }
                        }
                    }
                    _ => (),
                },
                WinitEvent::MainEventsCleared => {
                    // Update desktop specific resources.
                    application.get_resource_mut::<Time>().update();
                    application.get_resource_mut::<VulkanStager>().update();

                    application.execute_stage(PRE_UPDATE_STAGE);
                    application.execute_stage(DEFAULT_STAGE);
                    application.execute_stage(RENDER_STAGE);

                    application.get_resource_mut::<Input>().clear_inputs();
                    application.get_resource_mut::<Window>().clear_events();
                }
                _ => (),
            }
        });
    });
}
