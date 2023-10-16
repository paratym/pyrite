use std::{any::Any, collections::HashMap, ops::Deref, sync::Arc};

use pyrite_app::resource::Resource;

#[derive(Resource)]
pub struct Assets {
    loaders: HashMap<String, Box<dyn ErasedAssetLoader>>,
}

trait ErasedAssetLoader: Send + Sync {
    fn load(&self, data: &[u8]) -> Box<dyn Any>;
}

struct AssetLoaderWrapper<T: AssetLoader>(T);

impl<T: AssetLoader> ErasedAssetLoader for AssetLoaderWrapper<T> {
    fn load(&self, data: &[u8]) -> Box<dyn Any> {
        Box::new(self.0.load(data))
    }
}

pub trait AssetLoader: Send + Sync + 'static {
    type Asset;

    fn load(&self, data: &[u8]) -> Self::Asset
    where
        Self: Sized;
    fn identifier() -> &'static str;
}

impl Assets {
    pub fn new() -> Self {
        Self {
            loaders: HashMap::new(),
        }
    }

    pub fn add_loader<T: AssetLoader>(&mut self, loader: T) {
        self.loaders.insert(
            T::identifier().to_string(),
            Box::new(AssetLoaderWrapper(loader)),
        );
    }

    /// Load an asset from a file using the extension to determine the loader.
    /// Currently, the load is synchronous
    pub fn load<T: 'static>(&self, file_path: impl ToString) -> Handle<T> {
        let file_path = file_path.to_string();
        let extension = file_path
            .split('.')
            .last()
            .expect("Asset file path has no extension");

        let loader = self
            .loaders
            .get(extension)
            .expect("No loader for asset extension");

        let data = std::fs::read(file_path).expect("Failed to read asset file");

        let asset = loader.load(&data);

        let asset = asset
            .downcast::<T>()
            .expect("Failed to downcast asset to requested type");

        Handle {
            inner: Arc::new(*asset),
        }
    }
}

/// An active asset handle, when all handles to an asset are dropped, the asset is cleaned up.
/// Assets are by nature read-only, if a writable asset is needed, get a `RwHandle`
pub struct Handle<T> {
    inner: Arc<T>,
}

impl<T> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
