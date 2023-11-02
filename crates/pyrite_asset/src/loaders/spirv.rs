use crate::{AssetLoadError, AssetLoader};

pub struct SpirVLoader {}

impl AssetLoader for SpirVLoader {
    type Asset = Vec<u32>;

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
        let file_extension = file_path.split('.').last().unwrap();

        let shader_kind = match file_extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            _ => panic!("Unknown shader extension: {}", file_extension),
        };

        let compiler = shaderc::Compiler::new().unwrap();

        let source = std::fs::read_to_string(file_path.clone()).unwrap();

        let binary_result = compiler
            .compile_into_spirv(&source, shader_kind, &file_path, "main", None)
            .map_err(|err| AssetLoadError::new_invalid_file(file_path, err.to_string()))?;

        Ok(binary_result.as_binary().to_vec())
    }

    fn identifiers() -> &'static [&'static str] {
        &["glsl", "vert", "frag", "comp"]
    }
}
