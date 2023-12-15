use ash::vk;

use crate::{Vulkan, VulkanDep};

use super::{DescriptorSetLayout, DescriptorSetLayoutDep, PushConstantRange};

pub struct PipelineLayoutInstance {
    vulkan_dep: VulkanDep,
    descriptor_set_layout_dependencies: Vec<DescriptorSetLayoutDep>,
    pipeline_layout: vk::PipelineLayout,
}

impl PipelineLayoutInstance {
    pub fn new(vulkan: &Vulkan, create_info: PipelineLayoutCreateInfo<'_>) -> Self {
        let descriptor_set_layout_dependencies = create_info
            .descriptor_set_layouts
            .iter()
            .map(|layout| layout.create_dep())
            .collect::<Vec<_>>();

        let vk_descriptor_set_layouts = descriptor_set_layout_dependencies
            .iter()
            .map(|layout| layout.layout())
            .collect::<Vec<_>>();

        let vk_push_constant_ranges = create_info
            .push_constant_ranges
            .into_iter()
            .map(|range| range.into())
            .collect::<Vec<_>>();

        let vk_create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&vk_descriptor_set_layouts)
            .push_constant_ranges(&vk_push_constant_ranges);
        let pipeline_layout = unsafe {
            vulkan
                .device()
                .create_pipeline_layout(&vk_create_info, None)
                .unwrap()
        };

        Self {
            vulkan_dep: vulkan.create_dep(),
            descriptor_set_layout_dependencies,
            pipeline_layout,
        }
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
}

impl Drop for PipelineLayoutInstance {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

pub struct PipelineLayoutCreateInfo<'a> {
    pub descriptor_set_layouts: Vec<&'a DescriptorSetLayout>,
    pub push_constant_ranges: Vec<PushConstantRange>,
}

impl Default for PipelineLayoutCreateInfo<'_> {
    fn default() -> Self {
        Self {
            descriptor_set_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
        }
    }
}

impl<'a> PipelineLayoutCreateInfo<'a> {
    pub fn add_descriptor_set_layout(mut self, layout: &'a DescriptorSetLayout) -> Self {
        self.descriptor_set_layouts.push(layout);
        self
    }

    pub fn add_push_constant_range(mut self, range: PushConstantRange) -> Self {
        self.push_constant_ranges.push(range);
        self
    }
}
