use crate::{
    Buffer, BufferInfo, CommandBuffer, QueueConfig, QueueType, SharingMode, Vulkan,
    VulkanAllocator, VulkanInstance, DEFAULT_QUEUE,
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

    pub fn poll(&self, vulkan: &Vulkan) {}

    /// Schedules a buffer to be staged to the GPU using the best available method.
    pub fn schedule_stage_buffer(&self, vulkan: &Vulkan, buffer: &Buffer) {}

    pub fn schedule_stage_buffer_sync(&self, vulkan: &Vulkan, buffer: &Buffer) {}
    pub fn schedule_stage_buffer_async(&self, vulkan: &Vulkan, buffer: &Buffer) {}

    /// Records any queued up synchronous staging tasks to the command buffer
    ///
    /// These are then expected to be submitted to the default queue right before the GPU executes
    /// the next frame.
    pub fn record_synchronous_staging_commands(
        &self,
        vulkan: &Vulkan,
        command_buffer: &CommandBuffer,
    ) {
    }
}
