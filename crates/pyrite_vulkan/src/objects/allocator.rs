use ash::vk;
use pyrite_app::resource::Resource;

use crate::{Vulkan, VulkanDep};
use std::sync::Arc;

pub type Allocation = Arc<MemoryAllocation>;
pub struct MemoryAllocation {
    allocation: vk::DeviceMemory,
    size: u64,
    offset: u64,
}

impl MemoryAllocation {
    pub fn allocation(&self) -> vk::DeviceMemory {
        self.allocation
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
}

pub trait Allocator: Send + Sync {
    fn allocate(&mut self, info: &AllocationInfo) -> Allocation;
}

#[derive(Resource)]
pub struct VulkanAllocator {
    vulkan_dep: VulkanDep,
}

impl Allocator for VulkanAllocator {
    fn allocate(&mut self, info: &AllocationInfo) -> Allocation {
        let memory_requirements = info.memory_requirements;

        let memory_type_index = self.find_memory_type(
            memory_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
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
            allocation,
            size: memory_requirements.size,
            offset: 0,
        })
    }
}

impl VulkanAllocator {
    pub fn new(vulkan: &Vulkan) -> Self {
        Self {
            vulkan_dep: vulkan.create_dep(),
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
