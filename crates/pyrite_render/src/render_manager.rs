use std::{any::Any, sync::Arc};

use ash::vk;
use pyrite_app::{
    resource::{ResMut, Resource},
    AppBuilder,
};
use pyrite_desktop::{POST_RENDER_STAGE, PRE_RENDER_STAGE};
use pyrite_util::Dependable;
use pyrite_vulkan::{
    swapchain::{Swapchain, SwapchainDep},
    CommandBuffer, CommandPool, Fence, Image, ImageDep, ImageInfo, Semaphore, Vulkan,
    VulkanAllocator, VulkanDep,
};

pub fn setup_render_manager(app_builder: &mut AppBuilder, config: &RenderManagerConfig) {
    let render_manager = RenderManager::new(
        &*app_builder.get_resource::<Vulkan>(),
        &mut *app_builder.get_resource_mut::<VulkanAllocator>(),
        &*app_builder.get_resource::<Swapchain>(),
        config,
    );
    app_builder.add_resource(render_manager);

    // Add systems.
    app_builder.add_system_to_stage(RenderManager::pre_render_system, PRE_RENDER_STAGE);
    app_builder.add_system_to_stage(RenderManager::post_render_system, POST_RENDER_STAGE);
}

#[derive(Resource)]
pub struct RenderManager {
    vulkan_dep: VulkanDep,
    _swapchain_dep: SwapchainDep,
    command_pool: CommandPool,
    frames: Vec<Frame>,
    frame_config: Option<FrameConfig>,

    frame_index: usize,
    used_objects: Vec<Arc<dyn Any + Send + Sync>>,
}

pub struct Frame {
    fence: Fence,
    image_available_semaphore: Semaphore,
    render_finished_semaphore: Semaphore,
    command_buffer: CommandBuffer,
}

impl Frame {
    pub fn command_buffer(&self) -> &CommandBuffer {
        &self.command_buffer
    }

    pub fn command_buffer_mut(&mut self) -> &mut CommandBuffer {
        &mut self.command_buffer
    }
}

#[derive(Clone)]
pub struct RenderManagerConfig {
    frames_in_flight: u32,
}

impl RenderManagerConfig {
    pub fn builder() -> RenderManagerConfigBuilder {
        RenderManagerConfigBuilder::default()
    }
}

pub struct RenderManagerConfigBuilder {
    frames_in_flight: u32,
}

impl Default for RenderManagerConfigBuilder {
    fn default() -> Self {
        Self {
            frames_in_flight: 2,
        }
    }
}

impl RenderManagerConfigBuilder {
    pub fn frames_in_flight(mut self, frames_in_flight: u32) -> Self {
        self.frames_in_flight = frames_in_flight;
        self
    }

    pub fn build(self) -> RenderManagerConfig {
        RenderManagerConfig {
            frames_in_flight: self.frames_in_flight,
        }
    }
}

#[derive(Clone)]
pub struct FrameConfig {
    backbuffer_image: ImageDep,
    backbuffer_final_layout: vk::ImageLayout,
    backbuffer_final_access: vk::AccessFlags,
    used_objects: Vec<Arc<dyn Any + Send + Sync>>,
}

impl FrameConfig {
    pub fn builder() -> FrameConfigBuilder<'static> {
        FrameConfigBuilder::default()
    }
}

pub struct FrameConfigBuilder<'a> {
    backbuffer_image: Option<&'a Image>,
    backbuffer_final_layout: vk::ImageLayout,
    backbuffer_final_access: vk::AccessFlags,
    used_objects: Vec<Arc<dyn Any + Send + Sync>>,
}

impl Default for FrameConfigBuilder<'_> {
    fn default() -> Self {
        Self {
            backbuffer_image: None,
            backbuffer_final_layout: vk::ImageLayout::UNDEFINED,
            backbuffer_final_access: vk::AccessFlags::empty(),
            used_objects: Vec::new(),
        }
    }
}

impl<'a> FrameConfigBuilder<'a> {
    pub fn backbuffer(
        mut self,
        image: &'a Image,
        layout: vk::ImageLayout,
        access: vk::AccessFlags,
    ) -> Self {
        self.backbuffer_image = Some(image);
        self.backbuffer_final_layout = layout;
        self.backbuffer_final_access = access;
        self
    }

    pub fn used_objects(mut self, used_objects: Vec<Arc<dyn Any + Send + Sync>>) -> Self {
        self.used_objects = used_objects;
        self
    }

    pub fn build(mut self) -> FrameConfig {
        if self.backbuffer_image.is_none() {
            panic!("Backbuffer image not set.");
        }

        self.used_objects
            .push(self.backbuffer_image.unwrap().create_dep() as Arc<dyn Any + Send + Sync>);
        FrameConfig {
            backbuffer_image: self.backbuffer_image.unwrap().create_dep(),
            backbuffer_final_layout: self.backbuffer_final_layout,
            backbuffer_final_access: self.backbuffer_final_access,
            used_objects: self.used_objects,
        }
    }
}

impl Drop for RenderManager {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep.device().device_wait_idle().unwrap();
        }
    }
}

impl RenderManager {
    fn new(
        vulkan: &Vulkan,
        vulkan_allocator: &mut VulkanAllocator,
        swapchain: &Swapchain,
        config: &RenderManagerConfig,
    ) -> Self {
        let command_pool = CommandPool::new(vulkan);
        let command_buffers = command_pool.allocate_command_buffers(config.frames_in_flight as u32);

        let frames = command_buffers
            .into_iter()
            .map(|command_buffer| Frame {
                command_buffer,
                fence: Fence::new(vulkan, true),
                image_available_semaphore: Semaphore::new(vulkan),
                render_finished_semaphore: Semaphore::new(vulkan),
            })
            .collect();

        Self {
            vulkan_dep: vulkan.create_dep(),
            _swapchain_dep: swapchain.create_dep(),
            command_pool,
            frames,
            frame_config: None,
            frame_index: 0,
            used_objects: Vec::new(),
        }
    }

    pub fn frame(&self) -> &Frame {
        &self.frames[self.frame_index]
    }

    pub fn frame_mut(&mut self) -> &mut Frame {
        &mut self.frames[self.frame_index]
    }

    pub fn frames_in_flight(&self) -> u32 {
        self.frames.len() as u32
    }

    pub fn frame_index(&self) -> usize {
        self.frame_index
    }

    pub fn pre_render_system(
        mut render_manager: ResMut<RenderManager>,
        mut swapchain: ResMut<Swapchain>,
        mut vulkan_stager: ResMut<pyrite_vulkan::VulkanStager>,
    ) {
        // Helps the borrow checker.
        let render_manager = &mut *render_manager;

        // Wait for the previous frame to finish.
        {
            let frame = render_manager
                .frames
                .get_mut(render_manager.frame_index)
                .unwrap();

            // Wait for the fence to be signalled.
            frame.fence.wait();
            frame.fence.reset();

            // Release last frame's used objects.
            render_manager.used_objects.clear();

            // Begin recording the command buffer.
            let command_buffer = &mut frame.command_buffer;
            command_buffer.begin();

            // Record vulkan stager immediate tasks.
            render_manager.used_objects.extend(
                vulkan_stager
                    .record_immediate_tasks(
                        command_buffer,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::ALL_COMMANDS,
                    )
                    .into_iter()
                    // Rust want's me to dynamically cast this to Any + Send + Sync cause it's
                    // dumb.
                    .map(|x| x as Arc<dyn Any + Send + Sync>),
            );
        }
    }

    pub fn set_frame_config(&mut self, frame_config: &FrameConfig) {
        self.frame_config = Some(frame_config.clone());
    }

    pub fn post_render_system(
        mut render_manager: ResMut<RenderManager>,
        mut swapchain: ResMut<Swapchain>,
        mut vulkan_stager: ResMut<pyrite_vulkan::VulkanStager>,
    ) {
        // Helps the borrow checker.
        let render_manager = &mut *render_manager;
        let frame_config = render_manager
            .frame_config
            .take()
            .expect("Frame config not set.");

        for obj in frame_config.used_objects {
            render_manager.used_objects.push(obj);
        }
        render_manager.used_objects.push(frame_config.backbuffer_image.clone());

        // Process the current frame..
        {
            let frame = render_manager
                .frames
                .get_mut(render_manager.frame_index)
                .unwrap();

            let (image_index, is_outdated) =
                swapchain.acquire_next_image(&frame.image_available_semaphore);
            if is_outdated {
                swapchain.refresh();
                return;
            }

            let swapchain_image = swapchain.image(image_index);

            let command_buffer = &mut frame.command_buffer;

            // Transition the backbuffer image to transfer source and swapchain image to transfer
            // destination.
            command_buffer.pipeline_barrier(
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[
                    frame_config.backbuffer_image.image_memory_barrier(
                        frame_config.backbuffer_final_layout,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        frame_config.backbuffer_final_access,
                        vk::AccessFlags::TRANSFER_READ,
                    ),
                    swapchain_image.default_image_memory_barrier(
                        vk::ImageLayout::UNDEFINED,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    ),
                ],
            );

            // Blit the backbuffer image to the swapchain image.
            let blit_info = vk::ImageBlit::builder()
                .src_subresource(
                    vk::ImageSubresourceLayers::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .mip_level(0)
                        .base_array_layer(0)
                        .layer_count(1)
                        .build(),
                )
                .src_offsets([
                    vk::Offset3D::builder().x(0).y(0).z(0).build(),
                    vk::Offset3D::builder()
                        .x(frame_config.backbuffer_image.image_extent().width as i32)
                        .y(frame_config.backbuffer_image.image_extent().height as i32)
                        .z(1)
                        .build(),
                ])
                .dst_subresource(
                    vk::ImageSubresourceLayers::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .mip_level(0)
                        .base_array_layer(0)
                        .layer_count(1)
                        .build(),
                )
                .dst_offsets([
                    vk::Offset3D::builder().x(0).y(0).z(0).build(),
                    vk::Offset3D::builder()
                        .x(swapchain_image.image_extent().width as i32)
                        .y(swapchain_image.image_extent().height as i32)
                        .z(1)
                        .build(),
                ])
                .build();
            unsafe {
                render_manager.vulkan_dep.device().cmd_blit_image(
                    command_buffer.command_buffer(),
                    frame_config.backbuffer_image.image(),
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    swapchain_image.image(),
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[blit_info],
                    vk::Filter::NEAREST,
                );
            }

            // Transfer the previous swapchain image to present source.
            command_buffer.pipeline_barrier(
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[swapchain_image.default_image_memory_barrier(
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                )],
            );

            println!("finished rendering frame {}", render_manager.frame_index);

            // Finish recording the command buffer.
            command_buffer.end();

            unsafe {
                render_manager
                    .vulkan_dep
                    .device()
                    .queue_submit(
                        render_manager.vulkan_dep.default_queue().queue(),
                        &[vk::SubmitInfo::builder()
                            .command_buffers(&[command_buffer.command_buffer()])
                            .wait_semaphores(&[frame.image_available_semaphore.semaphore()])
                            .wait_dst_stage_mask(&[vk::PipelineStageFlags::BOTTOM_OF_PIPE])
                            .signal_semaphores(&[frame.render_finished_semaphore.semaphore()])
                            .build()],
                        frame.fence.fence(),
                    )
                    .expect("Failed to submit queue");
            }

            let present_result =
                swapchain.present(image_index, &[&frame.render_finished_semaphore]);
            if present_result.is_err() {
                println!("Suboptimal khr");
                swapchain.refresh();
            }
        }

        // Update frame index.
        render_manager.frame_index =
            (render_manager.frame_index + 1) % render_manager.frames_in_flight() as usize;
    }
}
