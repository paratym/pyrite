use crate::{
    Vulkan,
    VulkanDep,
    VulkanInstance,
};
use ash::vk;
use std::sync::Arc;

pub struct DescriptorSetPool {
    internal: Arc<InternalDescriptorSetPool>,
}

pub struct InternalDescriptorSetPool {
    vulkan_dep: VulkanDep,
    descriptor_pool: vk::DescriptorPool,
}

impl DescriptorSetPool {
    pub fn new(vulkan: &Vulkan) -> Self {
        let descriptor_pool_sizes = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(100)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(100)
                .build(),
        ];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&descriptor_pool_sizes)
            .max_sets(25);

        // Safety: The descriptor pool is dropped when the internal descriptor pool is dropped
        let descriptor_pool = unsafe {
            vulkan
                .device()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create descriptor pool")
        };

        Self {
            internal: Arc::new(InternalDescriptorSetPool {
                vulkan_dep: vulkan.create_dep(),
                descriptor_pool,
            }),
        }
    }

    pub fn allocate_descriptor_sets(
        &self,
        descriptor_set_layout: &DescriptorSetLayout,
        count: u32,
    ) -> Vec<DescriptorSet> {
        let descriptor_set_layouts = (0..count)
            .into_iter()
            .map(|_| descriptor_set_layout.descriptor_set_layout)
            .collect::<Vec<_>>();

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.internal.descriptor_pool)
            .set_layouts(&descriptor_set_layouts);

        // Safety: The descriptor set is only used in this struct and is destroyed when this struct
        // is dropped
        let descriptor_sets = unsafe {
            self.internal
                .vulkan_dep
                .device()
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets")
        };

        descriptor_sets
            .into_iter()
            .map(|descriptor_set| DescriptorSet {
                descriptor_pool: self.internal.clone(),
                descriptor_set,
            })
            .collect()
    }

    pub fn descriptor_pool(&self) -> &vk::DescriptorPool {
        &self.internal.descriptor_pool
    }
}

impl Drop for InternalDescriptorSetPool {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

pub struct DescriptorSetLayout {
    vulkan_dep: VulkanDep,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(vulkan: &Vulkan, bindings: &[vk::DescriptorSetLayoutBinding]) -> Self {
        let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();

        // Safety: The descriptor set layout is dropped when the internal descriptor set layout is
        // dropped
        let descriptor_set_layout = unsafe {
            vulkan
                .device()
                .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
                .expect("Failed to create descriptor set layout")
        };

        Self {
            vulkan_dep: vulkan.create_dep(),
            descriptor_set_layout,
        }
    }

    pub fn descriptor_set_layout(&self) -> &vk::DescriptorSetLayout {
        &self.descriptor_set_layout
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

pub struct DescriptorSet {
    descriptor_pool: Arc<InternalDescriptorSetPool>,
    descriptor_set: vk::DescriptorSet,
}

impl DescriptorSet {
    pub fn update_descriptor_sets(&self, descriptor_writes: &[vk::WriteDescriptorSet]) {
        unsafe {
            self.descriptor_pool
                .vulkan_dep
                .device()
                .update_descriptor_sets(descriptor_writes, &[]);
        }
    }

    pub fn descriptor_set(&self) -> &vk::DescriptorSet {
        &self.descriptor_set
    }
}
