use crate::{
    DescriptorSet, GraphicsPipeline, Image, InternalImage, RenderPass, UntypedBuffer, Vulkan,
    VulkanDep,
};
use ash::vk;
use pyrite_util::Dependable;
use std::{any::Any, sync::Arc};

pub struct CommandPool {
    internal: Arc<InternalCommandPool>,
}

impl CommandPool {
    pub fn new(vulkan: &Vulkan) -> Self {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(vulkan.default_queue().queue_family_index())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        // Safety: The command pool is dropped when the internal command pool is dropped
        let command_pool = unsafe {
            vulkan
                .device()
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create command pool")
        };

        Self {
            internal: Arc::new(InternalCommandPool {
                vulkan_dep: vulkan.create_dep(),
                command_pool,
            }),
        }
    }

    pub fn allocate_command_buffers(&self, count: u32) -> Vec<CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.internal.command_pool)
            .command_buffer_count(count)
            .level(vk::CommandBufferLevel::PRIMARY);

        // Safety: The command buffer is only used in this struct and is destroyed when this struct
        // is dropped
        let command_buffers = unsafe {
            self.internal
                .vulkan_dep
                .device()
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffers")
        };

        command_buffers
            .into_iter()
            .map(|command_buffer| CommandBuffer {
                command_pool: self.internal.clone(),
                command_buffer,
                used_objects: Vec::new(),
            })
            .collect()
    }
}

struct InternalCommandPool {
    vulkan_dep: VulkanDep,
    command_pool: vk::CommandPool,
}

impl Drop for InternalCommandPool {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_command_pool(self.command_pool, None);
        }
    }
}

pub struct CommandBuffer {
    command_pool: Arc<InternalCommandPool>,
    command_buffer: vk::CommandBuffer,
    used_objects: Vec<Arc<dyn Any + Send + Sync>>,
}

impl CommandBuffer {
    pub fn reset(&self) {
        let command_buffer_reset_flags = vk::CommandBufferResetFlags::RELEASE_RESOURCES;

        unsafe {
            self.command_pool
                .vulkan_dep
                .device()
                .reset_command_buffer(self.command_buffer, command_buffer_reset_flags)
                .expect("Failed to reset command buffer");
        }
    }

    /// Resets the command buffer and begins the command buffer recording.
    pub fn begin(&self) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

        unsafe {
            self.command_pool
                .vulkan_dep
                .device()
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin command buffer");
        }
    }

    pub fn end(&self) {
        unsafe {
            self.command_pool
                .vulkan_dep
                .device()
                .end_command_buffer(self.command_buffer)
                .expect("Failed to end command buffer");
        }
    }

    pub fn dynamic_state_viewport(&self, viewport: vk::Viewport) {
        unsafe {
            self.command_pool.vulkan_dep.device().cmd_set_viewport(
                self.command_buffer,
                0,
                &[viewport],
            );
        }
    }

    pub fn dynamic_state_scissor(&self, scissor: vk::Rect2D) {
        unsafe {
            self.command_pool.vulkan_dep.device().cmd_set_scissor(
                self.command_buffer,
                0,
                &[scissor],
            );
        }
    }

    pub fn bind_graphics_pipeline(&mut self, graphics_pipeline: &GraphicsPipeline) {
        // Safety: Since graphics_pipeline is by reference it is guaranteed to be valid here.
        unsafe {
            self.command_pool.vulkan_dep.device().cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline.pipeline(),
            );
        }
        self.used_objects.push(graphics_pipeline.create_dep());
    }

    pub fn begin_render_pass(
        &self,
        render_pass: &RenderPass,
        render_area: vk::Rect2D,
        clear_values: &[vk::ClearValue],
    ) {
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass.render_pass())
            .framebuffer(render_pass.framebuffer())
            .render_area(render_area)
            .clear_values(clear_values);

        unsafe {
            self.command_pool.vulkan_dep.device().cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub fn end_render_pass(&self) {
        unsafe {
            self.command_pool
                .vulkan_dep
                .device()
                .cmd_end_render_pass(self.command_buffer);
        }
    }

    pub fn pipeline_barrier(
        &self,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
        dependency_flags: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        unsafe {
            self.command_pool.vulkan_dep.device().cmd_pipeline_barrier(
                self.command_buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers,
                buffer_memory_barriers,
                image_memory_barriers,
            );
        }
    }

    pub fn copy_buffer_to_buffer(
        &self,
        src_buffer: &UntypedBuffer,
        src_offset: u64,
        src_size: u64,
        dst_buffer: &UntypedBuffer,
        dst_offset: u64,
    ) {
        let regions = [vk::BufferCopy::builder()
            .src_offset(src_offset)
            .dst_offset(dst_offset)
            .size(src_size)
            .build()];

        unsafe {
            self.command_pool.vulkan_dep.device().cmd_copy_buffer(
                self.command_buffer,
                src_buffer.buffer(),
                dst_buffer.buffer(),
                &regions,
            )
        }
    }

    /// Copies the specified buffer to the specified image. The image must be in the TRANSFER_DST_OPTIMAL layout.
    pub fn copy_buffer_to_image(
        &self,
        src_buffer: &UntypedBuffer,
        src_offset: u64,
        dst_image: &dyn InternalImage,
        dst_subresource: vk::ImageSubresourceLayers,
        dst_extent: vk::Extent3D,
    ) {
        let regions = [vk::BufferImageCopy::builder()
            .buffer_offset(src_offset)
            .image_subresource(dst_subresource)
            .image_extent(dst_extent)
            .build()];

        unsafe {
            self.command_pool
                .vulkan_dep
                .device()
                .cmd_copy_buffer_to_image(
                    self.command_buffer,
                    src_buffer.buffer(),
                    dst_image.image(),
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &regions,
                )
        }
    }

    pub fn bind_vertex_buffer(&mut self, location: u32, buffer: &Arc<UntypedBuffer>) {
        let offsets = [0];

        // Safety: Since buffer is by reference it is guaranteed to be valid here. We are keeping
        // the buffer alive until the command buffer is dropped or reset.
        unsafe {
            self.command_pool
                .vulkan_dep
                .device()
                .cmd_bind_vertex_buffers(
                    self.command_buffer,
                    location,
                    &[buffer.buffer()],
                    &offsets,
                );
        }
        self.used_objects.push(buffer.clone());
    }

    pub fn bind_index_buffer(&mut self, buffer: &Arc<UntypedBuffer>, index_type: vk::IndexType) {
        // Safety: Since buffer is by reference it is guaranteed to be valid here. We are keeping
        // the buffer alive until the command buffer is dropped or reset.
        unsafe {
            self.command_pool.vulkan_dep.device().cmd_bind_index_buffer(
                self.command_buffer,
                buffer.buffer(),
                0,
                index_type,
            );
        }
        self.used_objects.push(buffer.clone());
    }

    pub fn draw_indexed(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            self.command_pool.vulkan_dep.device().cmd_draw_indexed(
                self.command_buffer,
                vertex_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    pub fn bind_descriptor_sets(
        &mut self,
        bind_point: vk::PipelineBindPoint,
        pipeline_layout: vk::PipelineLayout,
        descriptor_sets: &[&DescriptorSet],
    ) {
        for descriptor_set in descriptor_sets {
            self.used_objects.push(descriptor_set.create_dep());
        }

        let descriptor_sets = descriptor_sets
            .iter()
            .map(|descriptor_set| descriptor_set.descriptor_set())
            .collect::<Vec<_>>();
        unsafe {
            self.command_pool
                .vulkan_dep
                .device()
                .cmd_bind_descriptor_sets(
                    self.command_buffer,
                    bind_point,
                    pipeline_layout,
                    0,
                    &descriptor_sets,
                    &[],
                );
        }
    }

    pub fn command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }
}
