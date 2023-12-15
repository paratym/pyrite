use std::sync::Arc;

use ash::vk;

use crate::{util::VulkanResource, Vulkan, VulkanDep};

pub type ShaderDep = Arc<ShaderInstance>;

pub struct ShaderInstance {
    vulkan_dep: VulkanDep,
    module: vk::ShaderModule,
}

impl ShaderInstance {
    pub fn module(&self) -> vk::ShaderModule {
        self.module
    }
}

impl VulkanResource for ShaderInstance {}

impl Drop for ShaderInstance {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_shader_module(self.module, None);
        }
    }
}

pub struct Shader {
    instance: Arc<ShaderInstance>,
}

impl Shader {
    pub fn new(vulkan: &Vulkan, code: &[u32]) -> Self {
        let module = unsafe {
            vulkan
                .device()
                .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(code), None)
                .unwrap()
        };

        Self {
            instance: Arc::new(ShaderInstance {
                vulkan_dep: vulkan.create_dep(),
                module,
            }),
        }
    }

    pub fn module(&self) -> vk::ShaderModule {
        self.instance.module()
    }

    pub fn create_dep(&self) -> ShaderDep {
        self.instance.clone()
    }
}
