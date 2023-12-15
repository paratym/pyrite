use std::sync::Arc;

use ash::vk;

use crate::{util::VulkanResource, Vulkan, VulkanDep};

use super::{PipelineLayoutCreateInfo, PipelineLayoutInstance, Shader};

pub type ComputePipelineDep = Arc<ComputePipelineInstance>;

pub struct ComputePipelineInstance {
    vulkan_dep: VulkanDep,
    pipeline_layout: PipelineLayoutInstance,
    pipeline: vk::Pipeline,
}

impl ComputePipelineInstance {
    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn pipeline_layout(&self) -> &PipelineLayoutInstance {
        &self.pipeline_layout
    }
}

impl VulkanResource for ComputePipelineInstance {}

impl Drop for ComputePipelineInstance {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_pipeline(self.pipeline, None);
        }
    }
}

pub struct ComputePipelineCreateInfo<'a> {
    pub shader: &'a Shader,
    pub shader_entry_point: String,
    pub pipeline_layout_info: PipelineLayoutCreateInfo<'a>,
}

pub struct ComputePipeline {
    instance: Arc<ComputePipelineInstance>,
}

impl ComputePipeline {
    pub fn new(vulkan: &Vulkan, create_info: ComputePipelineCreateInfo<'_>) -> Self {
        let pipeline_layout = PipelineLayoutInstance::new(vulkan, create_info.pipeline_layout_info);

        let vk_shader_name = std::ffi::CString::new(create_info.shader_entry_point).unwrap();
        let vk_create_info = vk::ComputePipelineCreateInfo::default()
            .stage(
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::COMPUTE)
                    .module(create_info.shader.module())
                    .name(vk_shader_name.as_c_str()),
            )
            .layout(pipeline_layout.layout());

        let pipeline = unsafe {
            vulkan
                .device()
                .create_compute_pipelines(vk::PipelineCache::null(), &[vk_create_info], None)
                .unwrap()[0]
        };

        Self {
            instance: Arc::new(ComputePipelineInstance {
                vulkan_dep: vulkan.create_dep(),
                pipeline_layout,
                pipeline,
            }),
        }
    }

    pub fn instance(&self) -> &ComputePipelineInstance {
        &self.instance
    }

    pub fn create_dep(&self) -> ComputePipelineDep {
        self.instance.clone()
    }
}
