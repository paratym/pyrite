use core::slice;
use std::{ops::Deref, sync::Arc};

use ash::vk;
use pyrite_util::Dependable;

use crate::{DescriptorSetLayout, DescriptorSetLayoutDep, Shader, Vulkan, VulkanDep};

pub type ComputePipelineDep = Arc<ComputePipelineInner>;
pub struct ComputePipeline {
    inner: Arc<ComputePipelineInner>,
}

impl ComputePipeline {
    pub fn new(vulkan: &Vulkan, info: ComputePipelineInfo) -> Self {
        Self {
            inner: Arc::new(ComputePipelineInner::new(vulkan, info)),
        }
    }

    pub fn create_dep(&self) -> ComputePipelineDep {
        self.inner.clone()
    }
}

impl Deref for ComputePipeline {
    type Target = ComputePipelineInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct ComputePipelineInner {
    vulkan_dep: VulkanDep,
    handle: vk::Pipeline,
    layout: vk::PipelineLayout,
}

impl Drop for ComputePipelineInner {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep.device().destroy_pipeline(self.handle, None);
            self.vulkan_dep
                .device()
                .destroy_pipeline_layout(self.layout, None);
        }
    }
}

pub struct ComputePipelineInfo {
    shader: Shader,
    descriptor_set_layouts: Vec<DescriptorSetLayoutDep>,
    push_constant_ranges: Vec<vk::PushConstantRange>,
}

impl ComputePipelineInfo {
    pub fn builder() -> ComputePipelineInfoBuilder {
        ComputePipelineInfoBuilder::default()
    }
}

pub struct ComputePipelineInfoBuilder {
    shader: Option<Shader>,
    descriptor_set_layouts: Vec<DescriptorSetLayoutDep>,
    push_constant_ranges: Vec<vk::PushConstantRange>,
}

impl Default for ComputePipelineInfoBuilder {
    fn default() -> Self {
        Self {
            shader: None,
            descriptor_set_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
        }
    }
}

impl ComputePipelineInfoBuilder {
    pub fn shader(mut self, shader: Shader) -> Self {
        self.shader = Some(shader);
        self
    }

    pub fn descriptor_set_layouts(
        mut self,
        descriptor_set_layouts: Vec<&DescriptorSetLayout>,
    ) -> Self {
        self.descriptor_set_layouts = descriptor_set_layouts
            .into_iter()
            .map(|layout| layout.create_dep())
            .collect();
        self
    }

    pub fn push_constant_ranges(
        mut self,
        push_constant_ranges: Vec<vk::PushConstantRange>,
    ) -> Self {
        self.push_constant_ranges = push_constant_ranges;
        self
    }

    pub fn build(self) -> ComputePipelineInfo {
        ComputePipelineInfo {
            shader: self.shader.unwrap(),
            descriptor_set_layouts: self.descriptor_set_layouts,
            push_constant_ranges: self.push_constant_ranges,
        }
    }
}

impl ComputePipelineInner {
    pub fn new(vulkan: &Vulkan, info: ComputePipelineInfo) -> Self {
        let shader_main_c_str = std::ffi::CString::new("main").unwrap();

        let descriptor_set_layouts = info
            .descriptor_set_layouts
            .iter()
            .map(|layout| layout.descriptor_set_layout())
            .collect::<Vec<_>>();
        let push_constant_ranges = info.push_constant_ranges;
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        let pipeline_layout = unsafe {
            vulkan
                .device()
                .create_pipeline_layout(&pipeline_layout_create_info, None)
        }
        .expect("Failed to create pipeline layout");

        let info = vk::ComputePipelineCreateInfo::default()
            .stage(
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::COMPUTE)
                    .module(info.shader.module())
                    .name(&shader_main_c_str),
            )
            .layout(pipeline_layout);

        let compute_pipeline = unsafe {
            vulkan.device().create_compute_pipelines(
                vk::PipelineCache::null(),
                slice::from_ref(&info),
                None,
            )
        }
        .expect("Failed to create compute pipeline")[0];

        Self {
            vulkan_dep: vulkan.create_dep(),
            handle: compute_pipeline,
            layout: pipeline_layout,
        }
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.handle
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.layout
    }
}
