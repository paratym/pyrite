use crate::AssetLoader;

pub struct TxtLoader {}

impl AssetLoader for TxtLoader {
    type Asset = String;

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
        String::from_utf8(std::fs::read(file_path).expect("Failed to read asset"))
            .expect("Failed to parse asset as UTF-8")
    }

    fn identifiers() -> &'static [&'static str] {
        &["txt"]
    }
}
