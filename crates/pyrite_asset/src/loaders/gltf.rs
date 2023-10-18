use crate::AssetLoader;

pub struct Gltf {
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

pub struct GltfLoader {}

impl GltfLoader {
    pub fn new() -> Self {
        Self {}
    }
}

impl AssetLoader for GltfLoader {
    type Asset = Gltf;

    fn load(&self, read: &[u8]) -> Self::Asset
    where
        Self: Sized,
    {
        let (document, buffers, images) =
            gltf::import_slice(read).expect("Failed to parse asset as GLTF");
        Gltf {
            document,
            buffers,
            images,
        }
    }

    fn identifier() -> &'static str {
        "gltf"
    }
}
