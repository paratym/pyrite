use crate::{
    Buffer, BufferInfo, QueueConfig, QueueType, SharingMode, Vulkan, VulkanAllocator,
    VulkanInstance, DEFAULT_QUEUE,
};
use ash::vk;
use pyrite_app::resource::Resource;

pub static STAGING_QUEUE: QueueConfig = QueueConfig::new(
    "pyrite_vulkan_stager::staging_queue",
    0.7,
    &[QueueType::Transfer],
);

#[derive(Resource)]
pub struct VulkanStager {
    staging_buffer: Buffer,
    gpu_async: bool,
}

impl VulkanStager {
    pub fn new(vulkan: &Vulkan, vulkan_allocator: &mut VulkanAllocator) -> Self {
        // Determines if we have an asynchronous queue for staging, if not only synchronous default
        // queue operations will be used.
        let gpu_async = vulkan.queue(STAGING_QUEUE.queue_name()).is_some();
        let staging_buffer = Buffer::new(
            vulkan,
            vulkan_allocator,
            &BufferInfo::builder()
                .size(1024 * 1024)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(SharingMode::new(vulkan, vec![DEFAULT_QUEUE.queue_name()]).unwrap())
                .build(),
        );

        Self {
            staging_buffer,
            gpu_async,
        }
    }
}
