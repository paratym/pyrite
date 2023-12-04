use pyrite_app::resource::Resource;

#[derive(Resource)]
pub struct VulkanDescriptorSetAllocator {}

impl VulkanDescriptorSetAllocator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn allocate_descriptor_set(&mut self) {}
}

#[derive(Resource)]
pub struct VulkanBufferAllocator {}

impl VulkanBufferAllocator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn allocate_buffer(&mut self) {}
}

#[derive(Resource)]
pub struct VulkanMemoryAllocator {}

impl VulkanMemoryAllocator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn allocate_memory(&mut self) {}
}
