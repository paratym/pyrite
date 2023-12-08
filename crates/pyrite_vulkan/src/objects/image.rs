use std::sync::Arc;

use ash::vk;

use crate::{
    util::{GenericResourceDep, VulkanResource, VulkanResourceDep},
    Vulkan, VulkanDep,
};
use util::ImageViewCreateInfo;

pub type ImageDep = Arc<dyn ImageInstance>;

pub trait GenericImageDep {
    fn into_generic_image(&self) -> ImageDep;
}

pub trait ImageInstance: Send + Sync + 'static {
    fn image(&self) -> vk::Image;
    fn image_view(&self) -> Option<vk::ImageView>;
}

impl<R> GenericImageDep for Arc<R>
where
    R: ImageInstance,
{
    fn into_generic_image(&self) -> ImageDep {
        Arc::clone(self) as Arc<dyn ImageInstance>
    }
}

impl<R> VulkanResource for R where R: ImageInstance {}

pub struct OwnedImageInstance {
    vulkan_dep: VulkanDep,
    image: vk::Image,
    image_view: Option<vk::ImageView>,
}

impl ImageInstance for OwnedImageInstance {
    fn image(&self) -> vk::Image {
        self.image
    }

    fn image_view(&self) -> Option<vk::ImageView> {
        self.image_view
    }
}

impl Drop for OwnedImageInstance {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep.device().destroy_image(self.image, None);
            if let Some(image_view) = self.image_view {
                self.vulkan_dep
                    .device()
                    .destroy_image_view(image_view, None);
            }
        }
    }
}

pub struct OwnedImage {
    instance: Arc<OwnedImageInstance>,
}

pub struct OwnedImageCreateInfo {
    pub image_type: vk::ImageType,
    pub width: u32,
    pub height: u32,
    pub format: vk::Format,
    pub usage: vk::ImageUsageFlags,
    pub samples: vk::SampleCountFlags,
    pub view_create_info: Option<ImageViewCreateInfo>,
}

impl OwnedImage {
    pub fn new(vulkan: &Vulkan, info: &OwnedImageCreateInfo) -> Self {
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(info.image_type)
            .extent(vk::Extent3D {
                width: info.width,
                height: info.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(info.format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(info.usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(info.samples);

        let image = unsafe {
            vulkan
                .device()
                .create_image(&image_create_info, None)
                .expect("Failed to create image")
        };

        todo!("Allocate memory.");

        let image_view = match &info.view_create_info {
            Some(view_create_info) => {
                let image_view_create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(view_create_info.view_type)
                    .format(info.format)
                    .subresource_range(view_create_info.subresource_range);

                // Safety: The image view is dropped when the internal image view is dropped
                let image_view = unsafe {
                    vulkan
                        .device()
                        .create_image_view(&image_view_create_info, None)
                        .expect("Failed to create image view")
                };

                Some(image_view)
            }
            None => None,
        };

        Self {
            instance: Arc::new(OwnedImageInstance {
                vulkan_dep: vulkan.create_dep(),
                image,
                image_view,
            }),
        }
    }

    pub fn create_dep(&self) -> ImageDep {
        self.instance.clone()
    }
}

pub struct BorrowedImageInstance {
    borrowed_dep: GenericResourceDep,
    image: vk::Image,
    image_view: Option<vk::ImageView>,
}

impl ImageInstance for BorrowedImageInstance {
    fn image(&self) -> vk::Image {
        self.image
    }

    fn image_view(&self) -> Option<vk::ImageView> {
        self.image_view
    }
}

pub struct BorrowedImage {
    instance: Arc<BorrowedImageInstance>,
}

pub struct BorrowedImageCreateInfo {
    pub image: vk::Image,
    pub image_view: Option<vk::ImageView>,
}

impl BorrowedImage {
    pub fn new(borrowed_from: &impl VulkanResourceDep, info: &BorrowedImageCreateInfo) -> Self {
        Self {
            instance: Arc::new(BorrowedImageInstance {
                borrowed_dep: borrowed_from.into_generic(),
                image: info.image,
                image_view: info.image_view,
            }),
        }
    }

    pub fn create_dep(&self) -> ImageDep {
        self.instance.clone()
    }
}

pub mod util {
    pub use super::*;

    use crate::Vulkan;

    pub struct ImageViewCreateInfo {
        pub view_type: vk::ImageViewType,
        pub subresource_range: vk::ImageSubresourceRange,
    }

    pub fn create_image_view(
        vulkan: &Vulkan,
        image: vk::Image,
        format: vk::Format,
        info: ImageViewCreateInfo,
    ) -> vk::ImageView {
        let vk_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(info.view_type)
            .format(format)
            .subresource_range(info.subresource_range);

        unsafe {
            vulkan
                .device()
                .create_image_view(&vk_info, None)
                .expect("Failed to create image view")
        }
    }
}
