pub use objects::*;
pub use vulkan::*;

mod objects;
mod vulkan;

pub mod swapchain;

pub mod prelude {
    pub use crate::{
        objects::{
            CommandBuffer,
            CommandPool,
            
            // GraphicsPipeline
            GraphicsPipelineInfo,
            GraphicsPipelineInfoBuilder,
            GraphicsPipeline,
            RenderPass,
            Subpass,
            Attachment,
            AttachmentInfo,
            AttachmentReference,
            
            Shader,
            
            // Images
            NewImageInfo,
            Image,
            OwnedImage,
            BorrowedImage,
            
            // Sync
            Semaphore,
            Fence,
        },
        swapchain::Swapchain,
        vulkan::{
            Vulkan,
            VulkanConfig,
            VulkanDep,
            VulkanInstance,
            VulkanRef,
        },
    };
}
