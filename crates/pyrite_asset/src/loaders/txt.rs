use crate::AssetLoader;

pub struct TxtLoader {}

impl AssetLoader for TxtLoader {
    type Asset = String;

    fn load(&self, read: &[u8]) -> Self::Asset
    where
        Self: Sized,
    {
        String::from_utf8(read.to_vec()).expect("Failed to parse asset as UTF-8")
    }

    fn identifier() -> &'static str {
        "txt"
    }
}
