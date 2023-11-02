use crate::{AssetLoadError, AssetLoader};

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub channels: u8,

    /// The image data in RGBA8 format.
    pub data: Vec<u8>,
}

pub struct ImageLoader {}

impl AssetLoader for ImageLoader {
    type Asset = Image;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn load(&self, file_path: String) -> Result<Self::Asset, AssetLoadError>
    where
        Self: Sized,
    {
        let img = image::open(file_path).unwrap();
        let channels = img.color().channel_count();
        let rgba8 = img.into_rgba8();
        Ok(Image {
            width: rgba8.width(),
            height: rgba8.height(),
            channels: channels as u8,
            data: rgba8.into_vec(),
        })
    }

    fn identifiers() -> &'static [&'static str] {
        &["png", "jpg", "jpeg"]
    }
}
