use crate::{
    Vulkan,
    VulkanInstance,
};
use ash::vk;

pub struct Shader {
    module: vk::ShaderModule,
}

impl Shader {
    pub fn new(vulkan: &Vulkan, code: &[u32]) -> Self {
        let module = unsafe {
            vulkan.device().create_shader_module(
                &vk::ShaderModuleCreateInfo::builder().code(code).build(),
                None,
            )
        }
        .expect("Failed to create shader module");

        Self { module }
    }

    pub fn module(&self) -> vk::ShaderModule {
        self.module
    }
}
