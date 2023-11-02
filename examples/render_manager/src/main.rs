use ash::vk;
use pyrite::{
    desktop::RENDER_STAGE,
    prelude::*,
    render::render_manager::{
        setup_render_manager, FrameConfig, RenderManager, RenderManagerConfig,
    },
};

fn main() {
    let mut app_builder = AppBuilder::new();

    setup_desktop_preset(
        &mut app_builder,
        DesktopConfig {
            application_name: "Desktop Example".to_string(),
            window_config: WindowConfig::default(),
            ..Default::default()
        },
    );

    setup_render_manager(
        &mut app_builder,
        &RenderManagerConfig::builder()
            .frames_in_flight(2)
            .resolution((1280, 720))
            .backbuffer_image_usage(
                // Hacky way to get backbuffer image to be created properly. Since an image view is
                // being made for the backbuffer, it expects that the image has a usage that uses
                // that image view.
                vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            )
            .build(),
    );

    app_builder.add_system_to_stage(
        |mut render_manager: ResMut<RenderManager>, vulkan: Res<Vulkan>| {
            let render_manager = &mut *render_manager;
            let cmd = render_manager.frame().command_buffer();

            cmd.pipeline_barrier(
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[render_manager
                    .backbuffer_image()
                    .default_image_memory_barrier(
                        vk::ImageLayout::UNDEFINED,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    )],
            );

            // Clear color with lavender.
            unsafe {
                vulkan.device().cmd_clear_color_image(
                    cmd.command_buffer(),
                    render_manager.backbuffer_image().image(),
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &vk::ClearColorValue {
                        float32: [0.73, 0.48, 0.79, 1.0],
                    },
                    &[vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    }],
                )
            }

            // Update render manager frame config, this is required atleast once.
            render_manager.set_frame_config(&FrameConfig {
                backbuffer_final_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            });
        },
        RENDER_STAGE,
    );

    app_builder.run();
}
