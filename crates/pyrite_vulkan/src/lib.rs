mod vulkan;
pub use vulkan::*;

mod objects;
pub use objects::*;

pub mod swapchain;

pub mod prelude {
    pub use crate::objects::Image;
    pub use crate::swapchain::Swapchain;
    pub use crate::vulkan::{Vulkan, VulkanConfig, VulkanDep, VulkanInstance, VulkanRef};
}
