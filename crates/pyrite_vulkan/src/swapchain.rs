use std::{
    cmp::{max, min},
    sync::Arc,
};

use ash::vk;
use pyrite_app::resource::Resource;

use crate::{
    objects::{
        image::{self, util::ImageViewCreateInfo, BorrowedImageCreateInfo},
        BorrowedImage, Semaphore,
    },
    util::{Extent2D, VulkanResource},
    Vulkan, VulkanDep,
};

pub type SwapchainDep = Arc<SwapchainInstance>;

struct SwapchainInstanceInternal {
    _vulkan_dep: VulkanDep,
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: ash::vk::SwapchainKHR,
}

impl VulkanResource for SwapchainInstanceInternal {}

pub struct SwapchainInfo {
    extent: Extent2D,
    format: vk::Format,
}

impl SwapchainInfo {
    pub fn extent(&self) -> &Extent2D {
        &self.extent
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }
}

pub struct SwapchainInstance {
    info: SwapchainInfo,
    swapchain: Arc<SwapchainInstanceInternal>,
    images: Vec<BorrowedImage>,
}

pub struct SwapchainCreateInfo {
    pub width: u32,
    pub height: u32,
    pub preferred_present_mode: ash::vk::PresentModeKHR,
    pub preferred_image_count: u32,
    pub image_usage: ash::vk::ImageUsageFlags,
    pub create_image_views: bool,
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
        let supported_present_modes = unsafe {
            surface
                .loader()
                .get_physical_device_surface_present_modes(
                    vulkan.physical_device().physical_device(),
                    surface.surface(),
                )
                .expect("Failed to get supported surface present modes")
        };
        let surface_capabilities = unsafe {
            surface
                .loader()
                .get_physical_device_surface_capabilities(
                    vulkan.physical_device().physical_device(),
                    surface.surface(),
                )
                .expect("Failed to get supported surface capabilities")
        };

        let format = supported_surface_formats
            .first()
            .expect("No supported formats found for the swapchain.");

        let image_count = min(
            max(
                surface_capabilities.min_image_count,
                info.preferred_image_count,
            ),
            surface_capabilities.max_image_count,
        );

        let present_mode = if supported_present_modes.contains(&info.preferred_present_mode) {
            info.preferred_present_mode
        } else {
            ash::vk::PresentModeKHR::FIFO
        };

        let swapchain = {
            let swapchain_loader =
                ash::extensions::khr::Swapchain::new(vulkan.instance(), vulkan.device());
            let swapchain = unsafe {
                swapchain_loader.create_swapchain(
                    &ash::vk::SwapchainCreateInfoKHR::default()
                        .surface(vulkan.surface().as_ref().unwrap().surface())
                        .min_image_count(image_count)
                        .image_array_layers(1)
                        .image_color_space(format.color_space)
                        .image_format(format.format)
                        .image_extent(ash::vk::Extent2D {
                            width: info.width,
                            height: info.height,
                        })
                        .image_usage(info.image_usage)
                        .image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
                        .pre_transform(ash::vk::SurfaceTransformFlagsKHR::IDENTITY)
                        .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
                        .present_mode(present_mode)
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
            let image_view = if info.create_image_views {
                Some(image::util::create_image_view(
                    vulkan,
                    image,
                    format.format,
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
                ))
            } else {
                None
            };

            BorrowedImage::new(&swapchain, &BorrowedImageCreateInfo { image, image_view })
        })
        .collect();

        Self {
            info: SwapchainInfo {
                extent: Extent2D {
                    width: info.width,
                    height: info.height,
                },
                format: format.format,
            },
            swapchain,
            images,
        }
    }

    pub fn info(&self) -> &SwapchainInfo {
        &self.info
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

    pub fn get_next_image_index(
        &self,
        signal_semaphore: &Semaphore,
    ) -> Result<u32, SwapchainError> {
        let swapchain = self.instance.as_ref().unwrap();
        let result = unsafe {
            swapchain.swapchain_loader().acquire_next_image(
                swapchain.swapchain(),
                std::u64::MAX,
                signal_semaphore.semaphore(),
                vk::Fence::null(),
            )
        };

        match result {
            Ok((image_index, _)) => Ok(image_index),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err(SwapchainError::OutOfDate),
            Err(vk::Result::SUBOPTIMAL_KHR) => Err(SwapchainError::SubOptimal),
            Err(_) => Err(SwapchainError::Unknown),
        }
    }

    pub fn instance(&self) -> &Arc<SwapchainInstance> {
        self.instance.as_ref().unwrap()
    }
}

#[derive(Debug)]
pub enum SwapchainError {
    OutOfDate,
    SubOptimal,
    Unknown,
}
