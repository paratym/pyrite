use std::{
    any::Any,
    collections::HashMap,
    error::Error,
    fmt::{Display, Formatter},
    ops::Deref,
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

#[derive(Clone, PartialEq, Debug)]
pub struct AssetLoadError {
    file_path: String,
    kind: AssetLoadErrorKind,
}

impl AssetLoadError {
    pub fn new_invalid_file(file_path: String, message: String) -> Self {
        Self {
            file_path,
            kind: AssetLoadErrorKind::InvalidFile { message },
        }
    }

    pub fn new_file_not_found(file_path: String) -> Self {
        Self {
            file_path,
            kind: AssetLoadErrorKind::FileNotFound,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum AssetLoadErrorKind {
    FileNotFound,
    InvalidFile { message: String },
}

impl Display for AssetLoadErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetLoadErrorKind::FileNotFound => write!(f, "File not found"),
            AssetLoadErrorKind::InvalidFile { message } => {
                write!(f, "Invalid file: {}", message)
            }
        }
    }
}

impl Display for AssetLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error loading asset for file: {}, kind: {}",
            self.file_path, self.kind
        )
    }
}

impl Error for AssetLoadError {}

trait ErasedAssetLoader: Send + Sync {
    fn load(&self, file_path: String) -> Result<Box<dyn Any>, AssetLoadError>;
}

struct AssetLoaderWrapper<T: AssetLoader>(T);

impl<T: AssetLoader> ErasedAssetLoader for AssetLoaderWrapper<T> {
    fn load(&self, file_path: String) -> Result<Box<dyn Any>, AssetLoadError> {
        Ok(Box::new(self.0.load(file_path)?))
    }
}

pub trait AssetLoader: Send + Sync + 'static {
    type Asset;

    fn new() -> Self
    where
        Self: Sized;
    fn load(&self, file_path: String) -> Result<Self::Asset, AssetLoadError>
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

                match loader.load(file_path) {
                    Ok(asset) => handle.update_asset(asset),
                    Err(error) => {
                        handle.update_error(error);
                    }
                }
            });
        });
    }
}

trait ErasedHandle: Send + Sync {
    fn is_loaded(&self) -> bool;
    fn is_error(&self) -> bool;
    fn update_asset(&self, asset: Box<dyn Any>);
    fn update_error(&self, error: AssetLoadError);
}

impl<T: Send + Sync + 'static> ErasedHandle for Arc<HandleInner<T>> {
    fn is_loaded(&self) -> bool {
        HandleInner::<T>::is_loaded(self.deref())
    }

    fn is_error(&self) -> bool {
        HandleInner::<T>::is_error(self.deref())
    }

    fn update_asset(&self, asset: Box<dyn Any>) {
        self.asset.write().replace(
            *asset
                .downcast::<T>()
                .expect("Failed to downcast asset to expected type"),
        );
        self.is_error.swap(false, atomic::Ordering::Relaxed);
        self.is_loaded.swap(true, atomic::Ordering::Relaxed);
    }

    fn update_error(&self, error: AssetLoadError) {
        self.error.write().replace(error);
        self.is_error.swap(true, atomic::Ordering::Relaxed);
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

    pub fn is_error(&self) -> bool {
        self.inner.is_error()
    }

    pub fn get(&self) -> Option<MappedRwLockReadGuard<'_, T>> {
        self.inner.get()
    }

    pub fn get_error(&self) -> Option<AssetLoadError> {
        self.inner.get_error()
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
    error: RwLock<Option<AssetLoadError>>,
    is_loaded: AtomicBool,
    is_error: AtomicBool,
    file_path: String,
}

impl<T: Send + Sync + 'static> HandleInner<T> {
    fn new(file_path: String) -> Self {
        Self {
            asset: RwLock::new(None),
            error: RwLock::new(None),
            is_loaded: AtomicBool::new(false),
            is_error: AtomicBool::new(false),
            file_path,
        }
    }

    fn is_loaded(&self) -> bool {
        self.is_loaded.load(atomic::Ordering::Relaxed)
    }

    fn is_error(&self) -> bool {
        self.is_error.load(atomic::Ordering::Relaxed)
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

    fn get_error(&self) -> Option<AssetLoadError> {
        if self.is_error() {
            Some(self.error.read().as_ref().unwrap().clone())
        } else {
            None
        }
    }
}

pub struct WatchedHandle<T> {
    handle: Handle<T>,
    should_reload: Arc<AtomicBool>,
    wait_on_reload: bool,
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
                    notify::EventKind::Modify(_) => {
                        let regex = regex::Regex::new(r"\\|\\\\").unwrap();

                        if event
                            .paths
                            .iter()
                            .any(|path| regex.replace_all(path.to_str().unwrap(), "/").to_string().ends_with(&regex.replace_all(&watcher_file_path, "/").to_string().as_str()))
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
            wait_on_reload: false,
            _watcher: watcher,
        }
    }

    /// Returns true if the handle reloaded.
    pub fn update(&mut self, assets: &mut Assets) -> bool {
        if self.should_reload.load(atomic::Ordering::Relaxed) {
            self.should_reload.store(false, atomic::Ordering::Relaxed);
            self.wait_on_reload = true;
            self.reload(assets);
        }
        if self.wait_on_reload && self.handle.is_loaded() {
            self.wait_on_reload = false;
            return true;
        }

        return false;
    }

    pub fn get(&self) -> Option<MappedRwLockReadGuard<'_, T>> {
        self.handle.get()
    }

    pub fn get_error(&self) -> Option<AssetLoadError> {
        self.handle.get_error()
    }

    pub fn is_loaded(&self) -> bool {
        self.handle.is_loaded()
    }

    pub fn is_error(&self) -> bool {
        self.handle.is_error()
    }

    pub fn reload(&mut self, assets: &mut Assets) {
        self.handle.reload(assets);
    }
}
