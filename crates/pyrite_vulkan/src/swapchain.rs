use crate::{BorrowedImage, Image, Vulkan, VulkanDep, VulkanInstance, VulkanRef};
use ash::{extensions, vk};
use pyrite_app::resource::Resource;

#[derive(Resource)]
pub struct Swapchain {
    vulkan_dep: VulkanDep,
    swapchain_loader: extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
}

struct SwapchainSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl Swapchain {
    pub fn new(vulkan: &Vulkan) -> Self {
        let swapchain_loader = extensions::khr::Swapchain::new(vulkan.instance(), vulkan.device());
        let new_swapchain = Self::new_swapchain(&swapchain_loader, vulkan);

        Self {
            vulkan_dep: vulkan.create_dep(),
            swapchain_loader,
            swapchain: new_swapchain.swapchain,
            swapchain_format: new_swapchain.swapchain_format,
            swapchain_extent: new_swapchain.swapchain_extent,
            images: new_swapchain.images,
            image_views: new_swapchain.image_views,
        }
    }

    fn new_swapchain(
        swapchain_loader: &extensions::khr::Swapchain,
        vulkan: VulkanRef,
    ) -> NewSwapchain {
        let swapchain_support = SwapchainSupport::query_swapchain_support(
            vulkan.surface_loader(),
            vulkan.surface().clone(),
            vulkan.physical_device().physical_device(),
        );

        let surface_format = *swapchain_support
            .formats
            .iter()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_SRGB
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&swapchain_support.formats[0]);

        let present_mode = if swapchain_support
            .present_modes
            .contains(&vk::PresentModeKHR::MAILBOX)
        {
            vk::PresentModeKHR::MAILBOX
        } else if swapchain_support
            .present_modes
            .contains(&vk::PresentModeKHR::FIFO_RELAXED)
        {
            vk::PresentModeKHR::FIFO_RELAXED
        } else {
            vk::PresentModeKHR::FIFO
        };

        // Min image count is 1 greater than the minimum to allow for triple buffering, unless it's not supported.
        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let extent = swapchain_support.capabilities.current_extent;

        let queue_family_indices = [vulkan.default_queue().queue_family_index()];
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(vulkan.surface().clone())
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }
            .expect("Failed to create swapchain.");

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }
            .expect("Failed to get swapchain images.");

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let image_views = images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(vk::ComponentMapping::default())
                    .subresource_range(subresource_range);

                unsafe { vulkan.device().create_image_view(&create_info, None) }
                    .expect("Failed to create image view.")
            })
            .collect::<Vec<_>>();

        NewSwapchain {
            swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            images,
            image_views,
        }
    }

    pub fn refresh(&mut self, vulkan: VulkanRef) {
        self.destroy_old_swapchain();

        let new_swapchain = Self::new_swapchain(&self.swapchain_loader, vulkan);
        self.swapchain = new_swapchain.swapchain;
        self.swapchain_format = new_swapchain.swapchain_format;
        self.swapchain_extent = new_swapchain.swapchain_extent;
        self.images = new_swapchain.images;
        self.image_views = new_swapchain.image_views;
    }

    fn destroy_old_swapchain(&mut self) {
        for &image_view in self.image_views.iter() {
            unsafe {
                self.vulkan_dep
                    .device()
                    .destroy_image_view(image_view, None);
            }
        }

        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }

    pub fn swapchain_loader(&self) -> &extensions::khr::Swapchain {
        &self.swapchain_loader
    }

    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    pub fn swapchain_format(&self) -> vk::Format {
        self.swapchain_format
    }

    pub fn swapchain_extent(&self) -> vk::Extent2D {
        self.swapchain_extent
    }

    pub fn image(&self, index: u32) -> Box<dyn Image> {
        Box::new(BorrowedImage::new(
            self.images[index as usize],
            self.image_views[index as usize],
            self.swapchain_format,
            self.swapchain_extent,
        ))
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        self.destroy_old_swapchain();
    }
}

impl SwapchainSupport {
    fn query_swapchain_support(
        surface_loader: &extensions::khr::Surface,
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
    ) -> Self {
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("Failed to get physical device surface capabilities.")
        };

        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("Failed to get physical device surface formats.")
        };

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("Failed to get physical device surface present modes.")
        };

        Self {
            capabilities,
            formats,
            present_modes,
        }
    }
}

struct NewSwapchain {
    swapchain: vk::SwapchainKHR,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
}
