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

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn load(&self, file_path: String) -> Self::Asset
    where
        Self: Sized,
    {
        let data = std::fs::read(file_path).expect("Failed to read asset");

        let (document, buffers, images) =
            gltf::import_slice(data).expect("Failed to parse asset as GLTF");
        Gltf {
            document,
            buffers,
            images,
        }
    }

    fn identifiers() -> &'static [&'static str] {
        &["gltf"]
    }
}
