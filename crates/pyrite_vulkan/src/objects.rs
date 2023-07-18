use ash::vk;

pub type BoxedImage = Box<dyn Image>;

pub trait Image {
    fn image(&self) -> vk::Image;
    fn image_view(&self) -> vk::ImageView;
    fn image_format(&self) -> vk::Format;
    fn image_extent(&self) -> vk::Extent2D;
}

// pub struct OwnedImage {
//     image: vk::Image,
//     image_view: vk::ImageView,
//     image_format: vk::Format,
//     image_extent: vk::Extent2D,
// }
//
// impl OwnedImage {
//     pub fn new(
//         image: vk::Image,
//         image_view: vk::ImageView,
//         image_format: vk::Format,
//         image_extent: vk::Extent2D,
//     ) -> Self {
//         Self {
//             image,
//             image_view,
//             image_format,
//             image_extent,
//         }
//     }
// }

/// A borrowed image is an image that is owned by another object, but implements the Image trait.
pub struct BorrowedImage {
    image: vk::Image,
    image_view: vk::ImageView,
    image_format: vk::Format,
    image_extent: vk::Extent2D,
}

impl BorrowedImage {
    pub fn new(
        image: vk::Image,
        image_view: vk::ImageView,
        image_format: vk::Format,
        image_extent: vk::Extent2D,
    ) -> Self {
        Self {
            image,
            image_view,
            image_format,
            image_extent,
        }
    }
}

impl Image for BorrowedImage {
    fn image(&self) -> vk::Image {
        self.image
    }

    fn image_view(&self) -> vk::ImageView {
        self.image_view
    }

    fn image_format(&self) -> vk::Format {
        self.image_format
    }

    fn image_extent(&self) -> vk::Extent2D {
        self.image_extent
    }
}
