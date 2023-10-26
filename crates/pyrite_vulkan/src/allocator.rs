use ash::vk;
use pyrite_app::resource::Resource;
use pyrite_util::Dependable;

use crate::{Vulkan, VulkanDep};
use std::{ops::Deref, sync::Arc};

pub type Allocation = Arc<MemoryAllocation>;
pub struct MemoryAllocation {
    allocator_dep: VulkanAllocatorDep,
    device_memory: vk::DeviceMemory,
    size: u64,
    offset: u64,
}

impl Drop for MemoryAllocation {
    fn drop(&mut self) {
        unsafe {
            self.allocator_dep
                .vulkan_dep
                .device()
                .free_memory(self.device_memory, None);
        }
    }
}

pub struct MemoryMapHandle<'a> {
    allocation: &'a MemoryAllocation,
    mapped_memory: *mut std::ffi::c_void,
}

impl Deref for MemoryMapHandle<'_> {
    type Target = *mut std::ffi::c_void;

    fn deref(&self) -> &Self::Target {
        &self.mapped_memory
    }
}

impl Drop for MemoryMapHandle<'_> {
    fn drop(&mut self) {
        self.allocation.unmap_memory();
    }
}

impl MemoryAllocation {
    pub fn map_memory(&self) -> MemoryMapHandle<'_> {
        let ptr = unsafe {
            self.allocator_dep.vulkan_dep.device().map_memory(
                self.device_memory,
                self.offset,
                self.size,
                vk::MemoryMapFlags::empty(),
            )
        }
        .unwrap();

        MemoryMapHandle {
            allocation: self,
            mapped_memory: ptr,
        }
    }

    fn unmap_memory(&self) {
        unsafe {
            self.allocator_dep
                .vulkan_dep
                .device()
                .unmap_memory(self.device_memory);
        }
    }

    pub fn device_memory(&self) -> vk::DeviceMemory {
        self.device_memory
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }
}

pub struct AllocationInfo {
    pub memory_requirements: vk::MemoryRequirements,
    pub memory_property_flags: vk::MemoryPropertyFlags,
}

impl AllocationInfo {
    pub fn builder() -> AllocationInfoBuilder {
        AllocationInfoBuilder::new()
    }
}

pub struct AllocationInfoBuilder {
    pub memory_requirements: vk::MemoryRequirements,
    pub memory_property_flags: vk::MemoryPropertyFlags,
}

impl AllocationInfoBuilder {
    fn new() -> Self {
        Self {
            memory_requirements: vk::MemoryRequirements::default(),
            memory_property_flags: vk::MemoryPropertyFlags::empty(),
        }
    }

    pub fn memory_requirements(mut self, memory_requirements: vk::MemoryRequirements) -> Self {
        self.memory_requirements = memory_requirements;
        self
    }

    pub fn memory_property_flags(mut self, memory_property_flags: vk::MemoryPropertyFlags) -> Self {
        self.memory_property_flags = memory_property_flags;
        self
    }

    pub fn build(self) -> AllocationInfo {
        AllocationInfo {
            memory_requirements: self.memory_requirements,
            memory_property_flags: self.memory_property_flags,
        }
    }
}

pub trait Allocator: Send + Sync {
    fn allocate(&mut self, info: &AllocationInfo) -> Allocation;
}

pub type VulkanAllocatorDep = Arc<InternalVulkanAllocator>;

#[derive(Resource)]
pub struct VulkanAllocator {
    internal: Arc<InternalVulkanAllocator>,
}

impl Deref for VulkanAllocator {
    type Target = InternalVulkanAllocator;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl VulkanAllocator {
    pub fn new(vulkan: &Vulkan) -> Self {
        Self {
            internal: Arc::new(InternalVulkanAllocator {
                vulkan_dep: vulkan.create_dep(),
            }),
        }
    }

    fn find_memory_type(&self, memory_type_bits: u32, properties: vk::MemoryPropertyFlags) -> u32 {
        self.vulkan_dep
            .physical_device()
            .memory_properties()
            .memory_types
            .iter()
            .enumerate()
            .find(|(index, memory_type)| {
                (memory_type_bits & (1 << index)) != 0
                    && memory_type.property_flags.contains(properties)
            })
            .map(|(index, _)| index as u32)
            .unwrap()
    }
}

pub struct InternalVulkanAllocator {
    vulkan_dep: VulkanDep,
}

impl Allocator for VulkanAllocator {
    fn allocate(&mut self, info: &AllocationInfo) -> Allocation {
        let memory_requirements = info.memory_requirements;

        let memory_type_index = self.find_memory_type(
            memory_requirements.memory_type_bits,
            info.memory_property_flags,
        );

        let allocation_create_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);

        let allocation = unsafe {
            self.vulkan_dep
                .device()
                .allocate_memory(&allocation_create_info, None)
        }
        .unwrap();

        Arc::new(MemoryAllocation {
            allocator_dep: self.internal.clone(),
            device_memory: allocation,
            size: memory_requirements.size,
            offset: 0,
        })
    }
}
