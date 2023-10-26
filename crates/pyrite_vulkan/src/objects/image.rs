use crate::{Allocation, AllocationInfo, Allocator, SharingMode, Vulkan, VulkanDep};
use ash::vk;
use pyrite_util::Dependable;
use std::{ops::Deref, sync::Arc};

pub type ImageDep = Arc<Box<dyn InternalImage>>;

pub struct Image {
    internal: Arc<Box<dyn InternalImage>>,
}

pub struct NewImageInfo {
    pub flags: vk::ImageCreateFlags,
    pub image_type: vk::ImageType,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    pub usage: vk::ImageUsageFlags,
    pub tiling: vk::ImageTiling,
    pub samples: vk::SampleCountFlags,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub initial_layout: vk::ImageLayout,
    pub sharing_mode: SharingMode,
    pub image_view_type: vk::ImageViewType,
    pub view_subresource_range: vk::ImageSubresourceRange,
}

impl Default for NewImageInfo {
    fn default() -> Self {
        Self {
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8G8B8A8_UNORM,
            extent: vk::Extent3D {
                width: 512,
                height: 512,
                depth: 1,
            },
            usage: vk::ImageUsageFlags::SAMPLED,
            tiling: vk::ImageTiling::OPTIMAL,
            samples: vk::SampleCountFlags::TYPE_1,
            mip_levels: 1,
            array_layers: 1,
            initial_layout: vk::ImageLayout::UNDEFINED,
            sharing_mode: SharingMode::Exclusive,
            image_view_type: vk::ImageViewType::TYPE_2D,
            view_subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        }
    }
}

impl Image {
    pub fn new(vulkan: &Vulkan, vulkan_allocator: &mut dyn Allocator, info: &NewImageInfo) -> Self {
        Self {
            internal: Arc::new(Box::new(OwnedImage::new(vulkan, vulkan_allocator, info))),
        }
    }

    pub fn new_borrowed(
        image: vk::Image,
        image_view: vk::ImageView,
        image_format: vk::Format,
        image_extent: vk::Extent3D,
    ) -> Self {
        Self {
            internal: Arc::new(Box::new(BorrowedImage::new(
                image,
                image_view,
                image_format,
                image_extent,
            ))),
        }
    }

    pub fn create_dep(&self) -> ImageDep {
        self.internal.clone()
    }
}

impl Deref for Image {
    type Target = dyn InternalImage;

    fn deref(&self) -> &Self::Target {
        self.internal.deref().deref()
    }
}

pub trait InternalImage: Send + Sync {
    fn default_image_memory_barrier(
        &self,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> vk::ImageMemoryBarrier {
        let src_access_mask = match old_layout {
            vk::ImageLayout::UNDEFINED => vk::AccessFlags::empty(),
            vk::ImageLayout::TRANSFER_DST_OPTIMAL => vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => {
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            }
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL => vk::AccessFlags::TRANSFER_READ,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => vk::AccessFlags::SHADER_READ,
            _ => panic!("Unsupported layout transition"),
        };

        let dst_access_mask = match new_layout {
            vk::ImageLayout::TRANSFER_DST_OPTIMAL => vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => {
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            }
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL => vk::AccessFlags::TRANSFER_READ,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => {
                vk::AccessFlags::SHADER_READ | vk::AccessFlags::INPUT_ATTACHMENT_READ
            }
            vk::ImageLayout::PRESENT_SRC_KHR => vk::AccessFlags::MEMORY_READ,
            _ => panic!("Unsupported layout transition"),
        };

        vk::ImageMemoryBarrier::builder()
            .image(self.image())
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .subresource_range(self.default_subresource_range())
            .build()
    }
    fn default_subresource_range(&self) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .layer_count(1)
            .level_count(1)
            .build()
    }

    fn image(&self) -> vk::Image;
    fn image_view(&self) -> vk::ImageView;
    fn image_format(&self) -> vk::Format;
    fn image_extent(&self) -> vk::Extent3D;
}

pub struct OwnedImage {
    vulkan_dep: VulkanDep,
    image: vk::Image,
    image_view: vk::ImageView,
    image_format: vk::Format,
    image_extent: vk::Extent3D,
    allocation: Allocation,
}

impl OwnedImage {
    pub fn new(vulkan: &Vulkan, vulkan_allocator: &mut dyn Allocator, info: &NewImageInfo) -> Self {
        let queue_family_indices = info.sharing_mode.queue_family_indices_or_default(vulkan);
        let image_create_info = vk::ImageCreateInfo::builder()
            .flags(info.flags)
            .image_type(info.image_type)
            .format(info.format)
            .extent(info.extent)
            .mip_levels(info.mip_levels)
            .array_layers(info.array_layers)
            .samples(info.samples)
            .tiling(info.tiling)
            .usage(info.usage)
            .initial_layout(info.initial_layout)
            .sharing_mode(info.sharing_mode.sharing_mode())
            .queue_family_indices(&queue_family_indices);

        let image = unsafe {
            vulkan
                .device()
                .create_image(&image_create_info, None)
                .expect("Failed to create image")
        };

        let memory_requirements = unsafe { vulkan.device().get_image_memory_requirements(image) };
        let allocation = vulkan_allocator.allocate(
            &AllocationInfo::builder()
                .memory_requirements(memory_requirements)
                .memory_property_flags(vk::MemoryPropertyFlags::DEVICE_LOCAL)
                .build(),
        );

        unsafe {
            vulkan.device().bind_image_memory(
                image,
                allocation.device_memory(),
                allocation.offset() as u64,
            )
        }
        .expect("Failed to bind image memory");

        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(info.image_view_type)
            .format(info.format)
            .subresource_range(info.view_subresource_range);

        let image_view = unsafe {
            vulkan
                .device()
                .create_image_view(&image_view_create_info, None)
        }
        .expect("Failed to create image view");

        Self {
            vulkan_dep: vulkan.create_dep(),
            image,
            image_view,
            image_format: info.format,
            image_extent: info.extent,
            allocation,
        }
    }
}

impl InternalImage for OwnedImage {
    fn image(&self) -> vk::Image {
        self.image
    }

    fn image_view(&self) -> vk::ImageView {
        self.image_view
    }

    fn image_format(&self) -> vk::Format {
        self.image_format
    }

    fn image_extent(&self) -> vk::Extent3D {
        self.image_extent
    }
}

impl Drop for OwnedImage {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_image_view(self.image_view, None);
            self.vulkan_dep.device().destroy_image(self.image, None);
        }
    }
}

/// A borrowed image is an image that is owned by another object, but implements the Image trait.
pub struct BorrowedImage {
    image: vk::Image,
    image_view: vk::ImageView,
    image_format: vk::Format,
    image_extent: vk::Extent3D,
}

impl BorrowedImage {
    pub fn new(
        image: vk::Image,
        image_view: vk::ImageView,
        image_format: vk::Format,
        image_extent: vk::Extent3D,
    ) -> Self {
        Self {
            image,
            image_view,
            image_format,
            image_extent,
        }
    }
}

impl InternalImage for BorrowedImage {
    fn image(&self) -> vk::Image {
        self.image
    }

    fn image_view(&self) -> vk::ImageView {
        self.image_view
    }

    fn image_format(&self) -> vk::Format {
        self.image_format
    }

    fn image_extent(&self) -> vk::Extent3D {
        self.image_extent
    }
}
