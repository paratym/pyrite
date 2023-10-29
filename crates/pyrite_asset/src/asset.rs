use std::{
    any::Any,
    collections::HashMap,
    path::Path,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use notify::Watcher;
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
    fn load(&self, file_path: String) -> Box<dyn Any>;
}

struct AssetLoaderWrapper<T: AssetLoader>(T);

impl<T: AssetLoader> ErasedAssetLoader for AssetLoaderWrapper<T> {
    fn load(&self, file_path: String) -> Box<dyn Any> {
        Box::new(self.0.load(file_path))
    }
}

pub trait AssetLoader: Send + Sync + 'static {
    type Asset;

    fn new() -> Self
    where
        Self: Sized;
    fn load(&self, file_path: String) -> Self::Asset
    where
        Self: Sized;
    fn identifiers() -> &'static [&'static str];
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

    pub fn add_loader<T: AssetLoader>(&mut self) {
        for identifier in T::identifiers() {
            self.loaders.insert(
                identifier.to_string(),
                Box::new(AssetLoaderWrapper(T::new())),
            );
        }
    }

    /// Load an asset from a file using the extension to determine the loader.
    /// Currently, the load is synchronous
    pub fn load<T: Send + Sync + 'static>(&mut self, file_path: impl ToString) -> Handle<T> {
        let handle = Handle::new(file_path.to_string());

        self.queue
            .push((file_path.to_string(), Box::new(handle.inner.clone())));

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

                let asset = loader.load(file_path);

                handle.update_asset(asset);
            });
        });
    }
}

trait ErasedHandle: Send + Sync {
    fn is_loaded(&self) -> bool;
    fn update_asset(&self, asset: Box<dyn Any>);
}

impl<T: Send + Sync + 'static> ErasedHandle for Arc<HandleInner<T>> {
    fn is_loaded(&self) -> bool {
        self.is_loaded.load(atomic::Ordering::Relaxed)
    }

    fn update_asset(&self, asset: Box<dyn Any>) {
        self.asset.write().replace(
            *asset
                .downcast::<T>()
                .expect("Failed to downcast asset to expected type"),
        );
        self.is_loaded.swap(true, atomic::Ordering::Relaxed);
    }
}

pub struct Handle<T> {
    inner: Arc<HandleInner<T>>,
}

impl<T: Send + Sync + 'static> Handle<T> {
    pub fn new(file_path: String) -> Self {
        Self {
            inner: Arc::new(HandleInner::new(file_path)),
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.inner.is_loaded()
    }

    pub fn get(&self) -> Option<MappedRwLockReadGuard<'_, T>> {
        self.inner.get()
    }

    pub fn reload(&mut self, assets: &mut Assets) {
        self.inner.is_loaded.swap(false, atomic::Ordering::Relaxed);
        assets
            .queue
            .push((self.inner.file_path.clone(), Box::new(self.inner.clone())));
    }

    pub fn into_watched(self) -> WatchedHandle<T> {
        WatchedHandle::new_with_handle(self.inner.file_path.clone(), self)
    }
}

pub struct HandleInner<T> {
    asset: RwLock<Option<T>>,
    is_loaded: AtomicBool,
    file_path: String,
}

impl<T: Send + Sync + 'static> HandleInner<T> {
    fn new(file_path: String) -> Self {
        Self {
            asset: RwLock::new(None),
            is_loaded: AtomicBool::new(false),
            file_path,
        }
    }

    fn is_loaded(&self) -> bool {
        self.is_loaded.load(atomic::Ordering::Relaxed)
    }

    fn get(&self) -> Option<MappedRwLockReadGuard<'_, T>> {
        if self.is_loaded() {
            Some(RwLockReadGuard::map(
                self.asset.read(),
                |asset: &Option<T>| asset.as_ref().unwrap(),
            ))
        } else {
            None
        }
    }
}

pub struct WatchedHandle<T> {
    handle: Handle<T>,
    should_reload: Arc<AtomicBool>,
    _watcher: notify::RecommendedWatcher,
}

impl<T: Send + Sync + 'static> WatchedHandle<T> {
    pub fn new(file_path: String) -> Self {
        Self::new_with_handle(file_path.clone(), Handle::new(file_path))
    }

    pub fn new_with_handle(file_path: String, handle: Handle<T>) -> Self {
        let should_reload = Arc::new(AtomicBool::new(false));

        // Setup file watcher, we watch the parent directory of the file,
        // and then check if the file path matches the file we are Watching
        // during events to avoid OS specific issues with watching files directly.
        let watcher_should_reload = should_reload.clone();
        let watcher_file_path = file_path.clone();
        let mut watcher = notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| match res {
                Ok(event) => match event.kind {
                    notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) => {
                        if event
                            .paths
                            .iter()
                            .any(|path| path.to_str().unwrap().ends_with(&watcher_file_path))
                        {
                            watcher_should_reload.store(true, atomic::Ordering::Relaxed);
                        }
                    }
                    _ => {}
                },
                Err(e) => println!("watch error: {:?}", e),
            },
        )
        .expect("Failed to create file watcher");

        let file_dir = Path::new(&file_path)
            .parent()
            .expect(format!("Failed to get parent directory of file: {}", file_path).as_str());
        watcher
            .watch(Path::new(&file_dir), notify::RecursiveMode::NonRecursive)
            .expect(format!("Failed to watch file: {}", file_path).as_str());

        Self {
            handle,
            should_reload,
            _watcher: watcher,
        }
    }

    pub fn update(&mut self, assets: &mut Assets) {
        if self.should_reload.load(atomic::Ordering::Relaxed) {
            self.should_reload.store(false, atomic::Ordering::Relaxed);
            self.reload(assets);
        }
    }

    pub fn get(&self) -> Option<MappedRwLockReadGuard<'_, T>> {
        self.handle.get()
    }

    pub fn is_loaded(&self) -> bool {
        self.handle.is_loaded()
    }

    pub fn reload(&mut self, assets: &mut Assets) {
        self.handle.reload(assets);
    }
}
