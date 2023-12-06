use std::sync::Arc;

use pyrite_app::resource::Resource;

use crate::{Vulkan, VulkanDep};

pub type SwapchainDep = Arc<SwapchainInstance>;

pub struct SwapchainInstance {
    _vulkan_dep: VulkanDep,
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: ash::vk::SwapchainKHR,
}

pub struct SwapchainCreateInfo {
    pub width: u32,
    pub height: u32,
    pub present_mode: ash::vk::PresentModeKHR,
}

impl SwapchainInstance {
    pub fn new(
        vulkan: &Vulkan,
        info: &SwapchainCreateInfo,
        old_swapchain: Option<ash::vk::SwapchainKHR>,
    ) -> Self {
        if vulkan.surface().is_none() {
            panic!("Cannot create swapchain without a surface");
        }

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(vulkan.instance(), vulkan.device());
        let swapchain = unsafe {
            swapchain_loader.create_swapchain(
                &ash::vk::SwapchainCreateInfoKHR::default()
                    .surface(vulkan.surface().as_ref().unwrap().surface())
                    .min_image_count(2)
                    .image_array_layers(1)
                    .image_color_space(ash::vk::ColorSpaceKHR::SRGB_NONLINEAR)
                    .image_format(ash::vk::Format::B8G8R8A8_SRGB)
                    .image_extent(ash::vk::Extent2D {
                        width: info.width,
                        height: info.height,
                    })
                    .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
                    .image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
                    .pre_transform(ash::vk::SurfaceTransformFlagsKHR::IDENTITY)
                    .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
                    .present_mode(info.present_mode)
                    .clipped(true)
                    .old_swapchain(old_swapchain.unwrap_or(ash::vk::SwapchainKHR::null())),
                None,
            )
        }
        .expect("Failed to create swapchain");

        Self {
            _vulkan_dep: vulkan.create_dep(),
            swapchain_loader,
            swapchain,
        }
    }
}

impl Drop for SwapchainInstance {
    fn drop(&mut self) {
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

#[derive(Resource)]
pub struct Swapchain {
    instance: Option<Arc<SwapchainInstance>>,
}

impl Swapchain {
    pub fn new() -> Self {
        Self { instance: None }
    }

    /// Constructs the swapchain and replaces the old one.
    pub fn refresh(&mut self, vulkan: &Vulkan, info: &SwapchainCreateInfo) {
        let old_swapchain = self.instance.take();

        // If the swapchain is still in use, that's ok since Vulkan will allow for replacing a
        // swapchain while in flight, the old swapchain will be destroyed once it's no longer
        // reference counted.
        self.instance = Some(Arc::new(SwapchainInstance::new(
            vulkan,
            info,
            old_swapchain.map(|i| i.swapchain),
        )));
    }
}
