use crate::{AssetLoadError, AssetLoader};

pub struct TxtLoader {}

impl AssetLoader for TxtLoader {
    type Asset = String;

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
        Ok(String::from_utf8(
            std::fs::read(file_path.clone())
                .map_err(|_| AssetLoadError::new_file_not_found(file_path.clone()))?,
        )
        .map_err(|err| AssetLoadError::new_invalid_file(file_path.clone(), err.to_string()))?)
    }

    fn identifiers() -> &'static [&'static str] {
        &["txt"]
    }
}
