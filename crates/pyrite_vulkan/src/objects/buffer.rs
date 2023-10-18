use ash::vk;

use crate::{
    Allocation, AllocationInfo, Allocator, SharingMode, Vulkan, VulkanAllocator, VulkanDep,
    VulkanInstance,
};

pub struct Buffer {
    vulkan_dep: VulkanDep,
    allocation: Allocation,
    buffer: vk::Buffer,
    size: u64,
}

pub struct BufferInfo {
    size: u64,
    usage: vk::BufferUsageFlags,
    sharing_mode: SharingMode,
}

impl BufferInfo {
    pub fn builder() -> BufferInfoBuilder {
        BufferInfoBuilder::default()
    }
}

pub struct BufferInfoBuilder {
    size: u64,
    usage: vk::BufferUsageFlags,
    sharing_mode: SharingMode,
}

impl Default for BufferInfoBuilder {
    fn default() -> Self {
        Self {
            size: 0,
            usage: vk::BufferUsageFlags::empty(),
            sharing_mode: SharingMode::Exclusive,
        }
    }
}

impl BufferInfoBuilder {
    pub fn size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    pub fn usage(mut self, usage: vk::BufferUsageFlags) -> Self {
        self.usage = usage;
        self
    }

    pub fn sharing_mode(mut self, sharing_mode: SharingMode) -> Self {
        self.sharing_mode = sharing_mode;
        self
    }

    pub fn build(self) -> BufferInfo {
        BufferInfo {
            size: self.size,
            usage: self.usage,
            sharing_mode: self.sharing_mode,
        }
    }
}

impl Buffer {
    pub fn new(vulkan: &Vulkan, vulkan_allocator: &mut VulkanAllocator, info: &BufferInfo) -> Self {
        let queue_family_indices = info
            .sharing_mode
            .queue_family_indices(vulkan)
            .unwrap_or(vec![]);
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(info.size)
            .usage(info.usage)
            .sharing_mode(info.sharing_mode.sharing_mode())
            .queue_family_indices(&queue_family_indices);

        let buffer = unsafe {
            vulkan
                .device()
                .create_buffer(&buffer_create_info, None)
                .expect("Failed to create buffer")
        };

        let requirements = unsafe { vulkan.device().get_buffer_memory_requirements(buffer) };

        let allocation = vulkan_allocator.allocate(&AllocationInfo {
            memory_requirements: requirements,
        });

        unsafe {
            vulkan
                .device()
                .bind_buffer_memory(buffer, allocation.device_memory(), allocation.offset())
                .expect("Failed to bind buffer memory");
        }

        Self {
            vulkan_dep: vulkan.create_dep(),
            allocation,
            buffer,
            size: info.size,
        }
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn allocation(&self) -> &Allocation {
        &self.allocation
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep.device().destroy_buffer(self.buffer, None);
        }
    }
}
