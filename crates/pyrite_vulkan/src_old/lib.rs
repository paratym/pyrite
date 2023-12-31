pub use allocator::*;
pub use objects::*;
pub use stager::*;
pub use vulkan::*;

mod allocator;
mod objects;
mod stager;
mod vulkan;

pub mod swapchain;

pub mod prelude {
    pub use crate::{
        allocator::VulkanAllocator,
        objects::{
            Attachment, AttachmentInfo, AttachmentReference, BorrowedImage, BufferInfo,
            BufferInfoBuilder, CommandBuffer, CommandPool, ComputePipeline, ComputePipelineDep,
            ComputePipelineInfo, ComputePipelineInfoBuilder, ComputePipelineInner, Fence,
            GraphicsPipeline, GraphicsPipelineInfo, GraphicsPipelineInfoBuilder,
            GraphicsPipelineInner, Image, ImageInfo, OwnedImage, RenderPass, Semaphore, Shader,
            Subpass, UntypedBuffer,
        },
        stager::VulkanStager,
        swapchain::Swapchain,
        vulkan::{Vulkan, VulkanConfig, VulkanDep, VulkanRef},
    };
}
