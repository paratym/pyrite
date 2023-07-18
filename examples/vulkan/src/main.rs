use ash::vk;
use pyrite::prelude::*;

fn main() {
    let mut app_builder = AppBuilder::new();

    setup_desktop_preset(
        &mut app_builder,
        DesktopConfig {
            application_name: "Vulkan Example".to_string(),
            window_config: WindowConfig::default(),
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

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available: Vec<vk::Semaphore>,
    blit_finished: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,

    frame_index: usize,
}

impl BlitManager {
    fn new(vulkan: &Vulkan) -> Self {
        let device = vulkan.device();
        let command_pool = unsafe {
            let info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(vulkan.default_queue().queue_family_index())
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

            device.create_command_pool(&info, None).unwrap()
        };

        let command_buffers = unsafe {
            let info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .command_buffer_count(2)
                .level(vk::CommandBufferLevel::PRIMARY);

            device.allocate_command_buffers(&info).unwrap()
        };

        fn create_semaphore(device: &ash::Device) -> vk::Semaphore {
            unsafe {
                let info = vk::SemaphoreCreateInfo::builder();
                device.create_semaphore(&info, None).unwrap()
            }
        }

        let image_available = vec![create_semaphore(&device), create_semaphore(&device)];

        let blit_finished = vec![create_semaphore(&device), create_semaphore(&device)];

        let in_flight_fences = unsafe {
            let info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
            (0..2)
                .map(|_| device.create_fence(&info, None).unwrap())
                .collect()
        };

        Self {
            vulkan_dep: vulkan.create_dep(),
            command_pool,
            command_buffers,
            image_available,
            blit_finished,
            in_flight_fences,
            frame_index: 0,
        }
    }

    fn render_frame(&mut self, swapchain: &mut Swapchain) {
        let device: &ash::Device = self.vulkan_dep.device();
        let in_flight_fence = self.in_flight_fences[self.frame_index];
        let image_available = self.image_available[self.frame_index];
        let blit_finished = self.blit_finished[self.frame_index];

        unsafe {
            device
                .wait_for_fences(&[in_flight_fence], true, u64::MAX)
                .expect("Wait for fences failed");

            device
                .reset_fences(&[in_flight_fence])
                .expect("Reset fences failed");
        }

        let image_index = unsafe {
            swapchain
                .swapchain_loader()
                .acquire_next_image(
                    swapchain.swapchain(),
                    u64::MAX,
                    image_available,
                    vk::Fence::null(),
                )
                .unwrap()
                .0
        };
        let swapchain_image = swapchain.image(image_index);

        let command_buffer = self.command_buffers[self.frame_index];

        // Reset and begin command buffer.
        unsafe {
            device
                .reset_command_buffer(
                    command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .unwrap();

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .unwrap();
        }

        // Transfer swapchain image to transfer destination layout.
        unsafe {
            let image_memory_barriers = [vk::ImageMemoryBarrier::builder()
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
                .build()];

            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_memory_barriers,
            );
        }

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
                command_buffer,
                swapchain_image.image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &clear_color,
                &ranges,
            );
        }

        // Transfer swapchain image to present source layout.
        unsafe {
            let image_memory_barriers = [vk::ImageMemoryBarrier::builder()
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
                .build()];

            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_memory_barriers,
            );
        }

        // End command buffer.
        unsafe {
            device.end_command_buffer(command_buffer).unwrap();
        }

        // Submit command buffer to default queue for rendering (our blit).
        unsafe {
            let command_buffers = [command_buffer];
            let wait_semaphores = [image_available];
            let wait_stages = [vk::PipelineStageFlags::TRANSFER];
            let signal_semaphores = [blit_finished];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            device
                .queue_submit(
                    self.vulkan_dep.default_queue().queue(),
                    &[submit_info.build()],
                    in_flight_fence,
                )
                .unwrap();
        }

        // Present swapchain image to screen.
        unsafe {
            let swapchains = [swapchain.swapchain()];
            let image_indices = [image_index];
            let wait_semaphores = [blit_finished];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            swapchain
                .swapchain_loader()
                .queue_present(self.vulkan_dep.default_queue().queue(), &present_info)
                .unwrap_or_else(|result| {
                    if result == vk::Result::ERROR_OUT_OF_DATE_KHR
                        || result == vk::Result::SUBOPTIMAL_KHR
                    {
                        swapchain.refresh(&*self.vulkan_dep);
                        true
                    } else {
                        panic!("Failed to present swapchain image.");
                    }
                });
        }

        self.frame_index = (self.frame_index + 1) % 2;
    }
}

impl Drop for BlitManager {
    fn drop(&mut self) {
        let device = self.vulkan_dep.device();

        unsafe {
            // Wait for all in-flight operations to finish so not resources are in use.
            device
                .wait_for_fences(&self.in_flight_fences, true, u64::MAX)
                .unwrap();

            device.destroy_command_pool(self.command_pool, None);
            for fence in self.in_flight_fences.drain(..) {
                device.destroy_fence(fence, None);
            }
            for semaphore in self
                .image_available
                .drain(..)
                .chain(self.blit_finished.drain(..))
            {
                device.destroy_semaphore(semaphore, None);
            }
        }
    }
}
