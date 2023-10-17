use std::{
    any::Any,
    collections::HashMap,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use pyrite_app::resource::Resource;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

#[derive(Resource)]
pub struct Assets {
    loaders: HashMap<String, Box<dyn ErasedAssetLoader>>,
    queue: Vec<(String, Box<dyn ErasedHandle>)>,
    pool: rayon::ThreadPool,
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
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();

        Self {
            loaders: HashMap::new(),
            queue: Vec::new(),
            pool,
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
    pub fn load<T: Send + Sync + 'static>(&mut self, file_path: impl ToString) -> Handle<T> {
        let handle = Handle::new();

        self.queue
            .push((file_path.to_string(), handle.create_wrapper()));

        handle
    }

    pub fn update(&mut self) {
        let queue = std::mem::take(&mut self.queue);

        let loaders = &self.loaders;

        let pool = &self.pool;

        pool.install(|| {
            queue.into_par_iter().for_each(|(file_path, handle)| {
                let extension = file_path
                    .split('.')
                    .last()
                    .expect("Asset file path has no extension");

                let loader = loaders
                    .get(extension)
                    .expect("No loader for asset extension");

                let data = std::fs::read(file_path).expect("Failed to read asset file");

                let asset = loader.load(&data);

                handle.update_asset(asset);
            });
        });
    }
}

trait ErasedHandle: Send + Sync {
    fn is_loaded(&self) -> bool;
    fn update_asset(&self, asset: Box<dyn Any>);
}

struct HandleWrapper<T> {
    inner: Handle<T>,
}

impl<T: Send + Sync + 'static> ErasedHandle for HandleWrapper<T> {
    fn is_loaded(&self) -> bool {
        self.inner.is_loaded()
    }

    fn update_asset(&self, asset: Box<dyn Any>) {
        self.inner.inner.asset.write().replace(
            *asset
                .downcast::<T>()
                .expect("Failed to downcast asset to expected type"),
        );
        self.inner
            .inner
            .is_loaded
            .swap(true, atomic::Ordering::Relaxed);
    }
}

pub struct Handle<T> {
    inner: Arc<HandleInner<T>>,
}

pub struct HandleInner<T> {
    asset: RwLock<Option<T>>,
    is_loaded: AtomicBool,
}

impl<T: Send + Sync + 'static> Handle<T> {
    fn new() -> Self {
        Self {
            inner: Arc::new(HandleInner {
                asset: RwLock::new(None),
                is_loaded: AtomicBool::new(false),
            }),
        }
    }

    fn create_wrapper(&self) -> Box<dyn ErasedHandle> {
        Box::new(HandleWrapper {
            inner: Handle {
                inner: self.inner.clone(),
            },
        })
    }

    pub fn is_loaded(&self) -> bool {
        self.inner.is_loaded.load(atomic::Ordering::Relaxed)
    }

    pub fn get(&self) -> Option<MappedRwLockReadGuard<'_, T>> {
        if self.is_loaded() {
            Some(RwLockReadGuard::map(
                self.inner.asset.read(),
                |asset: &Option<T>| asset.as_ref().unwrap(),
            ))
        } else {
            None
        }
    }
}
