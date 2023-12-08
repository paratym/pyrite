use std::sync::Arc;

use ash::vk;
use pyrite_app::resource::Resource;

use crate::{
    objects::{
        image::{self, util::ImageViewCreateInfo, BorrowedImageCreateInfo},
        BorrowedImage, Semaphore,
    },
    util::VulkanResource,
    Vulkan, VulkanDep,
};

pub type SwapchainDep = Arc<SwapchainInstance>;

struct SwapchainInstanceInternal {
    _vulkan_dep: VulkanDep,
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: ash::vk::SwapchainKHR,
}

impl VulkanResource for SwapchainInstanceInternal {}

pub struct SwapchainInstance {
    swapchain: Arc<SwapchainInstanceInternal>,
    images: Vec<BorrowedImage>,
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
        let surface = vulkan.surface().as_ref().unwrap();

        let supported_surface_formats = unsafe {
            surface
                .loader()
                .get_physical_device_surface_formats(
                    vulkan.physical_device().physical_device(),
                    surface.surface(),
                )
                .expect("Failed to get supported surface formats")
        };

        let swapchain_format = supported_surface_formats
            .first()
            .expect("No supported formats found for the swapchain.");

        let swapchain = {
            let swapchain_loader =
                ash::extensions::khr::Swapchain::new(vulkan.instance(), vulkan.device());
            let swapchain = unsafe {
                swapchain_loader.create_swapchain(
                    &ash::vk::SwapchainCreateInfoKHR::default()
                        .surface(vulkan.surface().as_ref().unwrap().surface())
                        .min_image_count(2)
                        .image_array_layers(1)
                        .image_color_space(swapchain_format.color_space)
                        .image_format(swapchain_format.format)
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

            Arc::new(SwapchainInstanceInternal {
                _vulkan_dep: vulkan.create_dep(),
                swapchain_loader,
                swapchain,
            })
        };

        let images = unsafe {
            swapchain
                .swapchain_loader
                .get_swapchain_images(swapchain.swapchain)
        }
        .expect("Failed to get swapchain images")
        .into_iter()
        .map(|image| {
            let image_view = image::util::create_image_view(
                vulkan,
                image,
                swapchain_format.format,
                ImageViewCreateInfo {
                    view_type: vk::ImageViewType::TYPE_2D,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                },
            );

            BorrowedImage::new(
                &swapchain,
                &BorrowedImageCreateInfo {
                    image,
                    image_view: Some(image_view),
                },
            )
        })
        .collect();

        Self { swapchain, images }
    }

    pub fn swapchain_loader(&self) -> &ash::extensions::khr::Swapchain {
        &self.swapchain.swapchain_loader
    }

    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain.swapchain
    }
}

impl VulkanResource for SwapchainInstance {}

impl Drop for SwapchainInstanceInternal {
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

    pub fn image(&self, index: usize) -> &BorrowedImage {
        self.instance.as_ref().unwrap().images.get(index).unwrap()
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
            old_swapchain.map(|i| i.swapchain.swapchain),
        )));
    }

    pub fn get_next_image_index(&self, signal_semaphore: &Semaphore) -> u32 {
        let swapchain = self.instance.as_ref().unwrap();
        let (index, is_suboptimal) = unsafe {
            swapchain
                .swapchain_loader()
                .acquire_next_image(
                    swapchain.swapchain(),
                    std::u64::MAX,
                    signal_semaphore.semaphore(),
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image")
        };

        index
    }

    pub fn instance(&self) -> &Arc<SwapchainInstance> {
        self.instance.as_ref().unwrap()
    }
}
