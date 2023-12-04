use std::sync::Arc;
use std::{hash::Hash, hash::Hasher};

use ash::vk;
use pyrite_util::Dependable;

use crate::{
    Allocation, AllocationInfo, Allocator, SharingMode, Vulkan, VulkanAllocator, VulkanDep,
};

pub struct BufferMapInfo {
    pub offset: u64,
    pub size: u64,
}

impl BufferMapInfo {
    pub fn builder() -> BufferMapInfoBuilder {
        BufferMapInfoBuilder { offset: 0, size: 0 }
    }
}

pub struct BufferMapInfoBuilder {
    offset: u64,
    size: u64,
}

impl BufferMapInfoBuilder {
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    pub fn build(self) -> BufferMapInfo {
        BufferMapInfo {
            offset: self.offset,
            size: self.size,
        }
    }
}

pub trait Buffer {
    fn buffer(&self) -> vk::Buffer;
    fn allocation(&self) -> &Allocation;
    fn size(&self) -> u64;
}

pub struct BufferInfo {
    size: u64,
    usage: vk::BufferUsageFlags,
    memory_flags: vk::MemoryPropertyFlags,
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
    memory_flags: vk::MemoryPropertyFlags,
    sharing_mode: SharingMode,
}

impl Default for BufferInfoBuilder {
    fn default() -> Self {
        Self {
            size: 0,
            usage: vk::BufferUsageFlags::empty(),
            memory_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
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

    pub fn memory_flags(mut self, memory_flags: vk::MemoryPropertyFlags) -> Self {
        self.memory_flags = memory_flags;
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
            memory_flags: self.memory_flags,
            sharing_mode: self.sharing_mode,
        }
    }
}

pub type BufferDep = Arc<UntypedBuffer>;

pub struct UntypedBuffer {
    vulkan_dep: VulkanDep,
    allocation: Allocation,
    buffer: vk::Buffer,
    size: u64,
}

impl Hash for UntypedBuffer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.buffer.hash(state);
    }
}

impl PartialEq for UntypedBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.buffer == other.buffer
    }
}

impl Eq for UntypedBuffer {}

impl UntypedBuffer {
    pub fn new(vulkan: &Vulkan, vulkan_allocator: &mut VulkanAllocator, info: &BufferInfo) -> Self {
        let queue_family_indices = info
            .sharing_mode
            .queue_family_indices(vulkan)
            .unwrap_or(vec![]);
        let buffer_create_info = vk::BufferCreateInfo::default()
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

        let allocation = vulkan_allocator.allocate(
            &AllocationInfo::builder()
                .memory_requirements(requirements)
                .memory_property_flags(info.memory_flags)
                .build(),
        );

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

impl Drop for UntypedBuffer {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep.device().destroy_buffer(self.buffer, None);
        }
    }
}

pub struct TypedBuffer<T> {
    untyped_buffer: UntypedBuffer,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TypedBuffer<T> {
    pub fn new(vulkan: &Vulkan, vulkan_allocator: &mut VulkanAllocator, info: &BufferInfo) -> Self {
        Self {
            untyped_buffer: UntypedBuffer::new(vulkan, vulkan_allocator, info),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.untyped_buffer.buffer()
    }

    pub fn allocation(&self) -> &Allocation {
        self.untyped_buffer.allocation()
    }

    pub fn size(&self) -> u64 {
        self.untyped_buffer.size()
    }
}
