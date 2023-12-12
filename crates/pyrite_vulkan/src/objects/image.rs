use std::sync::Arc;

use ash::vk;

use crate::{
    allocator::{MemoryAllocation, VulkanAllocationInfo, VulkanMemoryAllocator},
    util::{GenericResourceDep, VulkanResource, VulkanResourceDep},
    Vulkan, VulkanDep,
};
use util::ImageViewCreateInfo;

pub type ImageDep = Arc<dyn ImageInstance>;

pub trait Image {
    fn instance(&self) -> &dyn ImageInstance;
    fn create_dep(&self) -> ImageDep;
    fn create_generic_dep(&self) -> GenericResourceDep;
}

pub trait ImageInstance: VulkanResource + Send + Sync + 'static {
    fn image(&self) -> vk::Image;
    fn image_view(&self) -> Option<vk::ImageView>;
}

pub trait GenericImageDep {
    fn into_generic_image(&self) -> ImageDep;
}

impl<R> GenericImageDep for Arc<R>
where
    R: ImageInstance,
{
    fn into_generic_image(&self) -> ImageDep {
        Arc::clone(self) as Arc<dyn ImageInstance>
    }
}

pub struct OwnedImageInstance {
    vulkan_dep: VulkanDep,
    image: vk::Image,
    image_view: Option<vk::ImageView>,
    allocation: MemoryAllocation,
}

impl OwnedImageInstance {
    pub fn allocation(&self) -> &MemoryAllocation {
        &self.allocation
    }
}

impl ImageInstance for OwnedImageInstance {
    fn image(&self) -> vk::Image {
        self.image
    }

    fn image_view(&self) -> Option<vk::ImageView> {
        self.image_view
    }
}

impl VulkanResource for OwnedImageInstance {}

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
    pub fn new(
        vulkan: &Vulkan,
        vulkan_allocator: &mut VulkanMemoryAllocator,
        info: &OwnedImageCreateInfo,
    ) -> Self {
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

        let memory_requirements = unsafe { vulkan.device().get_image_memory_requirements(image) };

        let memory_allocation = vulkan_allocator.allocate(&VulkanAllocationInfo {
            size: memory_requirements.size,
            memory_proprties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            memory_type_bits: memory_requirements.memory_type_bits,
        });

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
                allocation: memory_allocation,
            }),
        }
    }
}

impl Image for OwnedImage {
    fn instance(&self) -> &dyn ImageInstance {
        self.instance.as_ref()
    }

    fn create_dep(&self) -> ImageDep {
        self.instance.clone()
    }

    fn create_generic_dep(&self) -> GenericResourceDep {
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

impl VulkanResource for BorrowedImageInstance {}

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
}

impl Image for BorrowedImage {
    fn instance(&self) -> &dyn ImageInstance {
        self.instance.as_ref()
    }

    fn create_dep(&self) -> ImageDep {
        self.instance.clone()
    }

    fn create_generic_dep(&self) -> GenericResourceDep {
        self.instance.borrowed_dep.clone()
    }
}

pub struct ImageMemoryBarrier<'a> {
    pub image: &'a dyn Image,
    pub old_layout: vk::ImageLayout,
    pub new_layout: vk::ImageLayout,
    pub src_access_mask: vk::AccessFlags,
    pub dst_access_mask: vk::AccessFlags,
}

impl<'a> Into<vk::ImageMemoryBarrier<'a>> for ImageMemoryBarrier<'a> {
    fn into(self) -> vk::ImageMemoryBarrier<'a> {
        vk::ImageMemoryBarrier::default()
            .image(self.image.instance().image())
            .old_layout(self.old_layout)
            .new_layout(self.new_layout)
            .src_access_mask(self.src_access_mask)
            .dst_access_mask(self.dst_access_mask)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            )
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
