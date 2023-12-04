use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use ash::{extensions, vk};

use pyrite_app::resource::Resource;
use pyrite_util::Dependable;

use crate::{Image, ImageDep, Semaphore, Vulkan, VulkanDep, VulkanRef};

pub type SwapchainDep = Arc<RwLock<SwapchainInner>>;

#[derive(Resource)]
pub struct Swapchain {
    // Synchronization: RwLock should never block since we have our async scheduler following
    // borrow checker rules. Swapchain dependencies should never be shared cross thread manually.
    inner: Arc<RwLock<SwapchainInner>>,
}

pub struct SwapchainInner {
    vulkan_dep: VulkanDep,
    swapchain_loader: extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    borrowed_images: Vec<Image>,
}

/// A new swapchain is created when the swapchain is refreshed. It it primarily a struct used to
/// return swapchain data from a function.
struct NewSwapchain {
    swapchain: vk::SwapchainKHR,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    borrowed_images: Vec<Image>,
}

struct SwapchainSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl Swapchain {
    pub fn new(vulkan: &Vulkan) -> Self {
        Self {
            inner: Arc::new(RwLock::new(SwapchainInner::new(vulkan))),
        }
    }

    /// Will destroy any borrowed images in use and create a new swapchain out of the old one, will
    /// wait for queue idle.
    pub fn refresh(&mut self) {
        self.inner.write().unwrap().refresh();
    }

    pub fn acquire_next_image(&self, semaphore: &Semaphore) -> (u32, bool) {
        let inner = self.inner.read().unwrap();
        unsafe {
            inner
                .swapchain_loader
                .acquire_next_image(
                    inner.swapchain,
                    std::u64::MAX,
                    semaphore.semaphore(),
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image.")
        }
    }

    pub fn present(
        &self,
        image_index: u32,
        wait_semaphores: &[&Semaphore],
    ) -> anyhow::Result<bool> {
        let inner = self.inner.read().unwrap();
        let wait_semaphores = wait_semaphores
            .iter()
            .map(|semaphore| semaphore.semaphore())
            .collect::<Vec<_>>();
        let swapchains = [inner.swapchain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let res = unsafe {
            inner
                .swapchain_loader
                .queue_present(inner.vulkan_dep.default_queue().queue(), &present_info)
        };

        if res.is_err() {
            return Err(anyhow::anyhow!("Swapchain is out of date."));
        }

        Ok(res.unwrap())
    }

    pub fn image(&self, index: u32) -> ImageDep {
        self.inner.read().unwrap().borrowed_images[index as usize].create_dep()
    }

    pub fn create_dep(&self) -> SwapchainDep {
        self.inner.clone()
    }

    pub fn swapchain_loader(&self) -> extensions::khr::Swapchain {
        self.inner.read().unwrap().swapchain_loader.clone()
    }

    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.inner.read().unwrap().swapchain
    }

    pub fn swapchain_format(&self) -> vk::Format {
        self.inner.read().unwrap().swapchain_format
    }

    pub fn swapchain_extent(&self) -> vk::Extent2D {
        self.inner.read().unwrap().swapchain_extent
    }
}

impl SwapchainInner {
    pub fn new(vulkan: &Vulkan) -> Self {
        let swapchain_loader = extensions::khr::Swapchain::new(vulkan.instance(), vulkan.device());
        let new_swapchain = Self::new_swapchain(&swapchain_loader, vulkan, None);

        Self {
            vulkan_dep: vulkan.create_dep(),
            swapchain_loader,
            swapchain: new_swapchain.swapchain,
            swapchain_format: new_swapchain.swapchain_format,
            swapchain_extent: new_swapchain.swapchain_extent,
            images: new_swapchain.images,
            image_views: new_swapchain.image_views,
            borrowed_images: new_swapchain.borrowed_images,
        }
    }

    pub fn refresh(&mut self) {
        unsafe { self.vulkan_dep.device().device_wait_idle().unwrap() };
        self.destroy_old_swapchain(true);

        let new_swapchain = SwapchainInner::new_swapchain(
            &self.swapchain_loader,
            &self.vulkan_dep,
            Some(self.swapchain),
        );

        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None)
        };
        self.swapchain = new_swapchain.swapchain;
        self.swapchain_format = new_swapchain.swapchain_format;
        self.swapchain_extent = new_swapchain.swapchain_extent;
        self.images = new_swapchain.images;
        self.image_views = new_swapchain.image_views;
        self.borrowed_images = new_swapchain.borrowed_images;
    }

    fn new_swapchain(
        swapchain_loader: &extensions::khr::Swapchain,
        vulkan: VulkanRef,
        old_swapchain: Option<vk::SwapchainKHR>,
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

        // Min image count is 1 greater than the minimum to allow for triple buffering, unless it's
        // not supported.
        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let extent = swapchain_support.capabilities.current_extent;
        println!("Swapchain extent: {:?}", extent);
        println!(
            "Swapchain min extent: {:?}",
            swapchain_support.capabilities.min_image_extent
        );
        println!(
            "Swapchain max extent: {:?}",
            swapchain_support.capabilities.max_image_extent
        );

        let queue_family_indices = [vulkan.default_queue().queue_family_index()];
        let mut create_info = vk::SwapchainCreateInfoKHR::default()
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

        if let Some(old_swapchain) = old_swapchain {
            create_info = create_info.old_swapchain(old_swapchain);
        }

        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }
            .expect("Failed to create swapchain.");

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }
            .expect("Failed to get swapchain images.");

        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let image_views = images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(vk::ComponentMapping::default())
                    .subresource_range(subresource_range);

                unsafe { vulkan.device().create_image_view(&create_info, None) }
                    .expect("Failed to create image view.")
            })
            .collect::<Vec<_>>();

        let borrowed_images = images
            .iter()
            .zip(image_views.iter())
            .map(|(&image, &image_view)| {
                Image::new_borrowed(
                    image,
                    image_view,
                    surface_format.format,
                    vk::Extent3D {
                        width: extent.width,
                        height: extent.height,
                        depth: 1,
                    },
                )
            })
            .collect::<Vec<_>>();

        NewSwapchain {
            swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            images,
            image_views,
            borrowed_images,
        }
    }

    fn destroy_old_swapchain(&self, keep_swapchain_handle: bool) {
        for &image_view in self.image_views.iter() {
            unsafe {
                self.vulkan_dep
                    .device()
                    .destroy_image_view(image_view, None);
            }
        }

        if !keep_swapchain_handle {
            unsafe {
                self.swapchain_loader
                    .destroy_swapchain(self.swapchain, None);
            }
        }
    }
}

impl Drop for SwapchainInner {
    fn drop(&mut self) {
        self.destroy_old_swapchain(false);
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
