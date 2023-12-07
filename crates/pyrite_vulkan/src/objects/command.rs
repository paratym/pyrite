use std::sync::Arc;

use ash::vk;
use slotmap::{new_key_type, SlotMap};

use crate::{util::VulkanResource, Vulkan, VulkanDep};

new_key_type! {
    pub struct CommandBufferHandle;
}

pub struct CommandBuffer {
    vulkan_dep: VulkanDep,
    command_buffer: ash::vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn begin(&mut self) {
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

    pub unsafe fn command_buffer(&self) -> ash::vk::CommandBuffer {
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

    pub fn get(&mut self, handle: CommandBufferHandle) -> Option<&mut CommandBuffer> {
        self.command_buffers.get_mut(handle)
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
            command_buffer,
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
