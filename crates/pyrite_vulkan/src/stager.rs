use std::{cmp::max, collections::HashMap, ops::Deref, sync::Arc};

use crate::{
    Buffer, BufferInfo, CommandBuffer, QueueConfig, QueueType, SharingMode, UntypedBuffer, Vulkan,
    VulkanAllocator, VulkanAllocatorDep, VulkanDep, DEFAULT_QUEUE,
};
use ash::vk;
use pyrite_app::resource::Resource;
use pyrite_util::Dependable;
use uuid::Uuid;

pub static STAGING_QUEUE: QueueConfig = QueueConfig::new(
    "pyrite_vulkan_stager::staging_queue",
    0.7,
    &[QueueType::Transfer],
);

/// The default size of staging buffers in bytes.
static STAGING_BUFFER_DEFAULT_SIZE: u64 = 4000000;

#[derive(Resource)]
pub struct VulkanStager {
    vulkan_dep: VulkanDep,
    staging_buffers: HashMap<Uuid, StagingBuffer>,
    gpu_async: bool,
    immediate_tasks: Vec<StagingTask>,
}

pub struct StagingBuffer {
    buffer: UntypedBuffer,
    current_offset: u64,
}

pub enum StageType {
    Immediate,
    Deferred,
}

struct StagingTask {
    src_buffer: Uuid,
    src_offset: u64,
    dst_buffer: Arc<UntypedBuffer>,
    dst_offset: u64,
    size: u64,
}

impl VulkanStager {
    pub fn new(vulkan: &Vulkan, vulkan_allocator: &mut VulkanAllocator) -> Self {
        // Determines if we have an asynchronous queue for staging, if not only synchronous default
        // queue operations will be used.
        let gpu_async = vulkan.queue(STAGING_QUEUE.queue_name()).is_some();

        Self {
            vulkan_dep: vulkan.create_dep(),
            staging_buffers: HashMap::new(),
            gpu_async,
            immediate_tasks: vec![],
        }
    }

    pub fn update(&mut self) {
        self.immediate_tasks.clear();
    }

    fn get_or_create_staging_buffer(
        &mut self,
        vulkan: &Vulkan,
        vulkan_allocator: &mut VulkanAllocator,
        size: u64,
    ) -> Uuid {
        for (uuid, staging_buffer) in &mut self.staging_buffers {
            if staging_buffer.current_offset + size <= staging_buffer.buffer.size() {
                staging_buffer.current_offset += size;
                return *uuid;
            }
        }

        let uuid = Uuid::new_v4();
        let buffer = UntypedBuffer::new(
            vulkan,
            vulkan_allocator,
            &BufferInfo::builder()
                .size(max(size, STAGING_BUFFER_DEFAULT_SIZE))
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .memory_flags(
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
                .sharing_mode(
                    SharingMode::new(
                        vulkan,
                        vec![DEFAULT_QUEUE.queue_name(), STAGING_QUEUE.queue_name()],
                    )
                    .unwrap(),
                )
                .build(),
        );

        // Insert the new buffer into the staging buffers map.
        self.staging_buffers.insert(
            uuid,
            StagingBuffer {
                buffer,
                current_offset: size,
            },
        );

        uuid
    }

    /// Schedules a buffer to be staged to the GPU using the best available method.
    // pub fn schedule_stage_buffer<T>(&mut self, src_data: &T, dst_buffer: &Buffer) {}

    pub fn schedule_stage_buffer<T: Sized>(
        &mut self,
        vulkan: &Vulkan,
        vulkan_allocator: &mut VulkanAllocator,
        src_data: &T,
        dst_buffer: &Arc<UntypedBuffer>,
        stage_type: StageType,
    ) {
        match stage_type {
            StageType::Immediate => {
                let data_ptr = src_data as *const T as *const u8;
                let data_size = std::mem::size_of::<T>();

                let staging_buffer_uuid =
                    self.get_or_create_staging_buffer(vulkan, vulkan_allocator, data_size as u64);

                let staging_buffer = self.staging_buffers.get_mut(&staging_buffer_uuid).unwrap();
                let memory = staging_buffer.buffer.allocation().map_memory();
                unsafe {
                    std::ptr::copy_nonoverlapping(data_ptr, *memory.deref() as *mut u8, data_size);
                }

                self.immediate_tasks.push(StagingTask {
                    src_buffer: staging_buffer_uuid,
                    src_offset: staging_buffer.current_offset,
                    dst_buffer: dst_buffer.clone(),
                    dst_offset: 0,
                    size: data_size as u64,
                });
                staging_buffer.current_offset += data_size as u64;
            }
            _ => todo!("Deferred staging not implemented yet"),
        }
    }
    // pub fn schedule_stage_buffer_async<T>(&mut self, src_data: &T, dst_buffer: &Buffer) {}

    /// Records any queued up immediate staging tasks to the command buffer
    ///
    /// These are then expected to be submitted to the default queue right before the GPU executes
    /// the next frame.
    pub fn record_immediate_tasks(&self, command_buffer: &CommandBuffer) {
        for task in &self.immediate_tasks {
            let src_buffer = self.staging_buffers.get(&task.src_buffer).unwrap();
            command_buffer.copy_buffer(
                &src_buffer.buffer,
                task.src_offset,
                task.size,
                &*task.dst_buffer,
                task.dst_offset,
            );
        }
    }
}
