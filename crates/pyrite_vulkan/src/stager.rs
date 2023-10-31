use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use crate::{
    BufferInfo, CommandBuffer, Image, ImageDep, QueueConfig, QueueType, SharingMode, UntypedBuffer,
    Vulkan, VulkanAllocator, VulkanDep, DEFAULT_QUEUE,
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
    _vulkan_dep: VulkanDep,
    staging_buffers: HashMap<Uuid, StagingBuffer>,
    gpu_async: bool,
    immediate_tasks: Vec<StagingTask>,

    /// For validation only.
    recorded_immediate_tasks: u32,
}

pub struct StagingBuffer {
    buffer: Arc<UntypedBuffer>,
    current_offset: u64,
}

pub enum StageType {
    Immediate,
    Deferred,
}

struct StagingTask {
    src_buffer: Uuid,
    src_offset: u64,
    dst: StagingTaskDst,
}

enum StagingTaskDst {
    Buffer {
        buffer: Arc<UntypedBuffer>,
        offset: u64,
        size: u64,
    },
    Image {
        image: ImageDep,
        subresource: vk::ImageSubresourceLayers,
        extent: vk::Extent3D,
        final_layout: vk::ImageLayout,
        final_access: vk::AccessFlags,
    },
}

impl StagingTaskDst {
    fn is_image(&self) -> bool {
        match self {
            StagingTaskDst::Image { .. } => true,
            _ => false,
        }
    }

    fn image(&self) -> Option<&ImageDep> {
        match self {
            StagingTaskDst::Image { image, .. } => Some(image),
            _ => None,
        }
    }

    fn final_layout(&self) -> Option<vk::ImageLayout> {
        match self {
            StagingTaskDst::Image { final_layout, .. } => Some(*final_layout),
            _ => None,
        }
    }

    fn is_buffer(&self) -> bool {
        match self {
            StagingTaskDst::Buffer { .. } => true,
            _ => false,
        }
    }
}

impl VulkanStager {
    pub fn new(vulkan: &Vulkan, vulkan_allocator: &mut VulkanAllocator) -> Self {
        // Determines if we have an asynchronous queue for staging, if not only synchronous default
        // queue operations will be used.
        let gpu_async = vulkan.queue(STAGING_QUEUE.queue_name()).is_some();

        Self {
            _vulkan_dep: vulkan.create_dep(),
            staging_buffers: HashMap::new(),
            gpu_async,
            immediate_tasks: vec![],
            recorded_immediate_tasks: 0,
        }
    }

    pub fn update(&mut self) {
        // Clear tasks and reset staging buffer offsets
        if self.recorded_immediate_tasks as usize != self.immediate_tasks.len() {
            dbg!(
                "{} staging tasks not submitted to GPU before next frame!",
                self.immediate_tasks.len() - self.recorded_immediate_tasks as usize
            );
        }
        self.immediate_tasks.clear();
        self.recorded_immediate_tasks = 0;
        for (_, staging_buffer) in &mut self.staging_buffers {
            staging_buffer.current_offset = 0;
        }
    }

    fn get_or_create_staging_buffer(
        &mut self,
        vulkan: &Vulkan,
        vulkan_allocator: &mut VulkanAllocator,
        size: u64,
    ) -> Uuid {
        for (uuid, staging_buffer) in &mut self.staging_buffers {
            if staging_buffer.current_offset + size <= staging_buffer.buffer.size() {
                return *uuid;
            }
        }

        let uuid = Uuid::new_v4();
        let buffer = Arc::new(UntypedBuffer::new(
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
        ));

        // Insert the new buffer into the staging buffers map.
        self.staging_buffers.insert(
            uuid,
            StagingBuffer {
                buffer,
                current_offset: 0,
            },
        );

        uuid
    }

    /// Schedules data to be staged to the specified GPU buffer.
    pub unsafe fn schedule_stage_buffer(
        &mut self,
        vulkan: &Vulkan,
        vulkan_allocator: &mut VulkanAllocator,
        data_ptr: *const u8,
        data_size: u64,
        dst_buffer: &Arc<UntypedBuffer>,
        stage_type: StageType,
    ) {
        match stage_type {
            StageType::Immediate => {
                let staging_buffer_uuid =
                    self.get_or_create_staging_buffer(vulkan, vulkan_allocator, data_size as u64);

                let staging_buffer = self.staging_buffers.get_mut(&staging_buffer_uuid).unwrap();
                let memory = staging_buffer.buffer.allocation().map_memory();
                let dst_ptr = (memory.deref().clone() as *mut u8)
                    .offset(staging_buffer.current_offset as isize);
                unsafe {
                    std::ptr::copy_nonoverlapping(data_ptr, dst_ptr, data_size as usize);
                }

                self.immediate_tasks.push(StagingTask {
                    src_buffer: staging_buffer_uuid,
                    src_offset: staging_buffer.current_offset,
                    dst: StagingTaskDst::Buffer {
                        buffer: dst_buffer.clone(),
                        offset: 0,
                        size: data_size as u64,
                    },
                });
                staging_buffer.current_offset += data_size as u64;
            }
            _ => todo!("Deferred staging not implemented yet"),
        }
    }

    /// Schedules data to be staged to a GPU image using the best available method.
    /// Data pointer must be the same size as the image
    pub unsafe fn schedule_stage_image(
        &mut self,
        vulkan: &Vulkan,
        vulkan_allocator: &mut VulkanAllocator,
        data_ptr: *const u8,
        dst_image: &Image,
        dst_channels: u32,
        dst_subresource: vk::ImageSubresourceLayers,
        dst_extent: vk::Extent3D,
        dst_final_layout: vk::ImageLayout,
        dst_final_access: vk::AccessFlags,
        stage_type: StageType,
    ) {
        match stage_type {
            StageType::Immediate => {
                let data_size =
                    (dst_extent.width * dst_extent.height * dst_extent.depth) * dst_channels;
                let staging_buffer_uuid =
                    self.get_or_create_staging_buffer(vulkan, vulkan_allocator, data_size as u64);

                let staging_buffer = self.staging_buffers.get_mut(&staging_buffer_uuid).unwrap();
                let memory = staging_buffer.buffer.allocation().map_memory();
                let dst_ptr = (memory.deref().clone() as *mut u8)
                    .offset(staging_buffer.current_offset as isize);
                unsafe {
                    std::ptr::copy_nonoverlapping(data_ptr, dst_ptr, data_size as usize);
                }

                self.immediate_tasks.push(StagingTask {
                    src_buffer: staging_buffer_uuid,
                    src_offset: staging_buffer.current_offset,
                    dst: StagingTaskDst::Image {
                        image: dst_image.create_dep(),
                        subresource: dst_subresource,
                        extent: dst_extent,
                        final_layout: dst_final_layout,
                        final_access: dst_final_access,
                    },
                });
                staging_buffer.current_offset += data_size as u64;
            }
            _ => todo!("Deferred staging not implemented yet"),
        }
    }

    /// Records any queued up immediate staging tasks to the command buffer
    /// These are then expected to be submitted to the default queue right before the GPU executes
    /// the next frame.
    pub fn record_immediate_tasks(
        &mut self,
        command_buffer: &CommandBuffer,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
    ) -> Vec<Arc<UntypedBuffer>> {
        let mut used_staging_buffers = HashSet::new();

        let dst_image_tasks = self
            .immediate_tasks
            .iter()
            .filter(|task| task.dst.is_image())
            .collect::<Vec<_>>();

        // Transition destination images to transfer destination layout.
        let image_memory_barriers = dst_image_tasks
            .iter()
            .map(|task| {
                let dst_image = task.dst.image().unwrap();
                dst_image.default_image_memory_barrier(
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                )
            })
            .collect::<Vec<_>>();
        command_buffer.pipeline_barrier(
            src_stage_mask,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &image_memory_barriers,
        );

        // Record copies.
        for task in &self.immediate_tasks {
            match task.dst {
                StagingTaskDst::Buffer {
                    ref buffer,
                    offset,
                    size,
                } => {
                    let src_buffer = self.staging_buffers.get(&task.src_buffer).unwrap();
                    command_buffer.copy_buffer_to_buffer(
                        &src_buffer.buffer,
                        task.src_offset,
                        size,
                        &*buffer,
                        offset,
                    );
                }
                StagingTaskDst::Image {
                    ref image,
                    subresource,
                    extent,
                    final_layout,
                    final_access,
                } => {
                    let src_buffer = self.staging_buffers.get(&task.src_buffer).unwrap();
                    command_buffer.copy_buffer_to_image(
                        &src_buffer.buffer,
                        task.src_offset,
                        image.deref().as_ref(),
                        subresource,
                        extent,
                    );
                }
            }
            used_staging_buffers.insert(task.src_buffer);
            self.recorded_immediate_tasks += 1;
        }

        let image_memory_barriers = dst_image_tasks
            .iter()
            .map(|task| {
                let dst_image = task.dst.image().unwrap();
                dst_image.default_image_memory_barrier(
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    task.dst.final_layout().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        command_buffer.pipeline_barrier(
            vk::PipelineStageFlags::TRANSFER,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &image_memory_barriers,
        );

        used_staging_buffers
            .into_iter()
            .map(|uuid| self.staging_buffers.get(&uuid).unwrap().buffer.clone())
            .collect()
    }
}
