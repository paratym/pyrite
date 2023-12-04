use crate::{Vulkan, VulkanDep};
use ash::vk;
use pyrite_util::Dependable;

pub struct Shader {
    vulkan_dep: VulkanDep,
    module: vk::ShaderModule,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_shader_module(self.module, None);
        }
    }
}

impl Shader {
    pub fn new(vulkan: &Vulkan, code: &[u32]) -> Self {
        let module = unsafe {
            vulkan
                .device()
                .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(code), None)
        }
        .expect("Failed to create shader module");

        Self {
            vulkan_dep: vulkan.create_dep(),
            module,
        }
    }

    pub fn module(&self) -> vk::ShaderModule {
        self.module
    }
}
