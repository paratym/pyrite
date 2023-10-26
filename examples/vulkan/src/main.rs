use ash::vk;
use pyrite::prelude::*;

fn main() {
    let mut app_builder = AppBuilder::new();

    setup_desktop_preset(
        &mut app_builder,
        DesktopConfig {
            application_name: "Vulkan Example".to_string(),
            window_config: WindowConfig::default(),
            ..Default::default()
        },
    );

    let blit_manager = BlitManager::new(&*app_builder.get_resource::<Vulkan>());
    app_builder.add_resource(blit_manager);

    app_builder.add_system(display_gpu_info);
    app_builder.add_system(render_blit);

    app_builder.run();
}

fn display_gpu_info(input: Res<Input>, vulkan: Res<Vulkan>) {
    if input.is_key_pressed(Key::F2) {
        println!("GPU - {}", vulkan.physical_device().name());
    }
}

fn render_blit(mut blit_manager: ResMut<BlitManager>, mut swapchain: ResMut<Swapchain>) {
    blit_manager.render_frame(&mut *swapchain);
}

#[derive(Resource)]
struct BlitManager {
    vulkan_dep: VulkanDep,

    _command_pool: CommandPool,
    command_buffers: Vec<CommandBuffer>,

    image_available: Vec<Semaphore>,
    blit_finished: Vec<Semaphore>,
    in_flight_fences: Vec<Fence>,

    frame_index: usize,
}

impl BlitManager {
    fn new(vulkan: &Vulkan) -> Self {
        let command_pool = CommandPool::new(vulkan);

        let command_buffers = command_pool.allocate_command_buffers(2);

        let image_available = vec![Semaphore::new(vulkan), Semaphore::new(vulkan)];
        let blit_finished = vec![Semaphore::new(vulkan), Semaphore::new(vulkan)];

        let in_flight_fences = vec![Fence::new(vulkan, true), Fence::new(vulkan, true)];

        Self {
            vulkan_dep: vulkan.create_dep(),
            _command_pool: command_pool,
            command_buffers,
            image_available,
            blit_finished,
            in_flight_fences,
            frame_index: 0,
        }
    }

    fn render_frame(&mut self, swapchain: &mut Swapchain) {
        let device: &ash::Device = self.vulkan_dep.device();
        let in_flight_fence = &self.in_flight_fences[self.frame_index];
        let image_available = &self.image_available[self.frame_index];
        let blit_finished = &self.blit_finished[self.frame_index];

        in_flight_fence.wait();
        in_flight_fence.reset();

        let (image_index, _) = swapchain.acquire_next_image(&image_available);
        let swapchain_image = swapchain.image(image_index);

        let command_buffer = &self.command_buffers[self.frame_index];

        // Reset and begin command buffer.
        command_buffer.reset();
        command_buffer.begin();

        // Transfer swapchain image to transfer destination layout.
        command_buffer.pipeline_barrier(
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(swapchain_image.image())
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .build(),
                )
                .build()],
        );

        // Clear swapchain image with clear color.
        unsafe {
            let ranges = [vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build()];

            let mut clear_color = vk::ClearColorValue::default();
            clear_color.float32 = [186.0 / 255.0, 94.0 / 255.0, 168.0 / 255.0, 1.0];

            device.cmd_clear_color_image(
                command_buffer.command_buffer(),
                swapchain_image.image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &clear_color,
                &ranges,
            );
        }

        // Transfer swapchain image to present source layout.
        command_buffer.pipeline_barrier(
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::empty())
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(swapchain_image.image())
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .build(),
                )
                .build()],
        );

        // End command buffer.
        command_buffer.end();

        // Submit command buffer to default queue for rendering (our blit).
        unsafe {
            let command_buffers = [command_buffer.command_buffer()];
            let wait_semaphores = [image_available.semaphore()];
            let wait_stages = [vk::PipelineStageFlags::TRANSFER];
            let signal_semaphores = [blit_finished.semaphore()];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            device
                .queue_submit(
                    self.vulkan_dep.default_queue().queue(),
                    &[submit_info.build()],
                    in_flight_fence.fence(),
                )
                .unwrap();
        }

        // Present swapchain image to screen.
        let present_result = swapchain.present(image_index, &[&blit_finished]);
        if present_result.is_err() {
            swapchain.refresh(&*self.vulkan_dep);
        }
        self.frame_index = (self.frame_index + 1) % 2;
    }
}

impl Drop for BlitManager {
    fn drop(&mut self) {
        let device = self.vulkan_dep.device();

        unsafe {
            // Wait for all in-flight operations to finish so not resources are in use.
            let fences = self
                .in_flight_fences
                .iter()
                .map(|fence| fence.fence())
                .collect::<Vec<_>>();
            device.wait_for_fences(&fences, true, u64::MAX).unwrap();
        }
    }
}
