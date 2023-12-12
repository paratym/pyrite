use std::sync::Arc;

use ash::vk;
use slotmap::{new_key_type, SlotMap};

use crate::{
    util::{VulkanResource, VulkanResourceDep, WeakGenericResourceDep},
    Vulkan, VulkanDep,
};

use super::{Image, ImageMemoryBarrier};

new_key_type! {
    pub struct CommandBufferHandle;
}

pub struct CommandBuffer {
    vulkan_dep: VulkanDep,
    command_pool: std::sync::Weak<CommandPoolInstance>,
    command_buffer: ash::vk::CommandBuffer,
    recorded_dependencies: Vec<WeakGenericResourceDep>,
}

impl CommandBuffer {
    pub fn begin(&mut self) {
        self.recorded_dependencies
            .push(self.command_pool.into_generic_weak());

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.vulkan_dep
                .device()
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin command buffer");
        }
    }

    pub fn end(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .end_command_buffer(self.command_buffer)
                .expect("Failed to end command buffer");
        }
    }

    pub fn pipeline_barrier(
        &mut self,
        src_stage: vk::PipelineStageFlags,
        dst_stage: vk::PipelineStageFlags,
        image_memory_barriers: Vec<ImageMemoryBarrier>,
    ) {
        self.recorded_dependencies
            .extend(image_memory_barriers.iter().map(|image_memory_barrier| {
                Arc::downgrade(&image_memory_barrier.image.create_generic_dep())
            }));
        let vk_image_memory_barriers = image_memory_barriers
            .into_iter()
            .map(|image_memory_barrier| image_memory_barrier.into())
            .collect::<Vec<_>>();

        unsafe {
            self.vulkan_dep.device().cmd_pipeline_barrier(
                self.command_buffer,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &vk_image_memory_barriers,
            );
        }
    }

    pub fn clear_color_image(
        &mut self,
        image: &dyn Image,
        clear_color: vk::ClearColorValue,
        subresource_range: vk::ImageSubresourceRange,
    ) {
        self.recorded_dependencies
            .push(Arc::downgrade(&image.create_generic_dep()));

        unsafe {
            self.vulkan_dep.device().cmd_clear_color_image(
                self.command_buffer,
                image.instance().image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &clear_color,
                &[subresource_range],
            );
        }
    }

    pub fn take_recorded_dependencies(&mut self) -> Vec<WeakGenericResourceDep> {
        std::mem::take(&mut self.recorded_dependencies)
    }

    pub fn command_buffer(&self) -> ash::vk::CommandBuffer {
        self.command_buffer
    }
}

pub type CommandPoolDep = Arc<CommandPoolInstance>;

pub struct CommandPoolInstance {
    vulkan_dep: VulkanDep,
    command_pool: ash::vk::CommandPool,
}

impl VulkanResource for CommandPoolInstance {}

impl Drop for CommandPoolInstance {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_command_pool(self.command_pool, None);
        }
    }
}

pub struct CommandPool {
    instance: Arc<CommandPoolInstance>,
    command_buffers: SlotMap<CommandBufferHandle, CommandBuffer>,
}

impl CommandPool {
    pub fn new(vulkan: &Vulkan) -> Self {
        let command_pool_create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(vulkan.default_queue().queue_family_index());

        // Safety: The command pool is dropped when the internal command pool is dropped
        let command_pool = unsafe {
            vulkan
                .device()
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create command pool")
        };

        Self {
            instance: Arc::new(CommandPoolInstance {
                vulkan_dep: vulkan.create_dep(),
                command_pool,
            }),
            command_buffers: SlotMap::with_key(),
        }
    }

    pub fn get(&self, handle: CommandBufferHandle) -> Option<&CommandBuffer> {
        self.command_buffers.get(handle)
    }

    pub fn get_multiple(&self, handles: Vec<CommandBufferHandle>) -> Vec<&CommandBuffer> {
        self.command_buffers
            .iter()
            .filter(|(handle, _)| handles.iter().any(|h| h == handle))
            .map(|(_, command_buffer)| command_buffer)
            .collect()
    }

    pub fn get_mut(&mut self, handle: CommandBufferHandle) -> Option<&mut CommandBuffer> {
        self.command_buffers.get_mut(handle)
    }

    pub fn get_multiple_mut(
        &mut self,
        handles: Vec<CommandBufferHandle>,
    ) -> Vec<&mut CommandBuffer> {
        self.command_buffers
            .iter_mut()
            .filter(|(handle, _)| handles.iter().any(|h| h == handle))
            .map(|(_, command_buffer)| command_buffer)
            .collect()
    }

    pub fn reset(&mut self) {
        unsafe {
            self.instance
                .vulkan_dep
                .device()
                .reset_command_pool(
                    self.instance.command_pool,
                    vk::CommandPoolResetFlags::empty(),
                )
                .expect("Failed to reset command pool");
        }
    }

    pub fn allocate<const N: usize>(&mut self) -> [CommandBufferHandle; N] {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.instance.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(N as u32);

        let command_buffers = unsafe {
            self.instance
                .vulkan_dep
                .device()
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffers")
        }
        .into_iter()
        .map(|command_buffer| CommandBuffer {
            vulkan_dep: self.instance.vulkan_dep.clone(),
            command_pool: Arc::downgrade(&self.instance),
            command_buffer,
            recorded_dependencies: Vec::new(),
        })
        .collect::<Vec<_>>();

        let mut handles = Vec::new();
        for command_buffer in command_buffers {
            handles.push(self.command_buffers.insert(command_buffer));
        }

        handles.try_into().unwrap_or_else(|_| {
            panic!(
                "Failed to convert command buffer handles into array of length {}",
                N
            )
        })
    }
}
