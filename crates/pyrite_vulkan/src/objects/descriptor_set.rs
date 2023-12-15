use std::sync::Arc;

use ash::vk;
use slotmap::{new_key_type, SlotMap};

use crate::{
    util::{GenericResourceDep, VulkanResource, WeakGenericResourceDep},
    Vulkan, VulkanDep,
};

pub type DescriptorSetLayoutDep = Arc<DescriptorSetLayoutInstance>;

pub struct DescriptorSetLayoutInstance {
    vulkan_dep: VulkanDep,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl DescriptorSetLayoutInstance {
    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
}

impl VulkanResource for DescriptorSetLayoutInstance {}

impl Drop for DescriptorSetLayoutInstance {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

pub struct DescriptorSetLayout {
    instance: Arc<DescriptorSetLayoutInstance>,
}

impl DescriptorSetLayout {
    pub fn builder() -> DescriptorSetLayoutBuilder<'static> {
        DescriptorSetLayoutBuilder::new()
    }

    pub fn instance(&self) -> &DescriptorSetLayoutInstance {
        &self.instance
    }

    pub fn create_dep(&self) -> DescriptorSetLayoutDep {
        self.instance.clone()
    }
}

pub struct DescriptorSetLayoutBuilder<'a> {
    bindings: Vec<vk::DescriptorSetLayoutBinding<'a>>,
}

impl DescriptorSetLayoutBuilder<'_> {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn add_binding(
        &mut self,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        descriptor_count: u32,
        stage_flags: vk::ShaderStageFlags,
    ) -> &mut Self {
        self.bindings.push(
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(descriptor_type)
                .descriptor_count(descriptor_count)
                .stage_flags(stage_flags),
        );
        self
    }

    pub fn build(self, vulkan: &Vulkan) -> DescriptorSetLayout {
        let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&self.bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::empty());

        // Safety: The descriptor set layout is dropped when the internal descriptor set layout is dropped
        let descriptor_set_layout = unsafe {
            vulkan
                .device()
                .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
                .expect("Failed to create descriptor set layout")
        };

        DescriptorSetLayout {
            instance: Arc::new(DescriptorSetLayoutInstance {
                vulkan_dep: vulkan.create_dep(),
                descriptor_set_layout,
            }),
        }
    }
}

new_key_type! { pub struct DescriptorSetHandle; }

pub struct DescriptorSet {
    descriptor_set: vk::DescriptorSet,
    written_dependencies: Vec<WeakGenericResourceDep>,
}

impl DescriptorSet {
    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }

    pub fn written_dependencies(&self) -> &[WeakGenericResourceDep] {
        &self.written_dependencies
    }
}

pub type DescriptorSetPoolDep = Arc<DescriptorSetPoolInstance>;

pub struct DescriptorSetPoolInstance {
    vulkan_dep: VulkanDep,
    descriptor_pool: vk::DescriptorPool,
}

impl VulkanResource for DescriptorSetPoolInstance {}

impl Drop for DescriptorSetPoolInstance {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

pub struct DescriptorSetPool {
    instance: Arc<DescriptorSetPoolInstance>,
    descriptor_sets: SlotMap<DescriptorSetHandle, DescriptorSet>,
}

impl DescriptorSetPool {
    pub fn new(vulkan: &Vulkan) -> Self {
        let descriptor_pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(100),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(100),
        ];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&descriptor_pool_sizes)
            .max_sets(100);

        // Safety: The descriptor pool is dropped when the internal descriptor pool is dropped
        let descriptor_pool = unsafe {
            vulkan
                .device()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create descriptor pool")
        };

        Self {
            instance: Arc::new(DescriptorSetPoolInstance {
                vulkan_dep: vulkan.create_dep(),
                descriptor_pool,
            }),
            descriptor_sets: SlotMap::with_key(),
        }
    }

    pub fn get(&self, handle: DescriptorSetHandle) -> Option<&DescriptorSet> {
        self.descriptor_sets.get(handle)
    }

    pub fn get_multiple(&self, handles: Vec<DescriptorSetHandle>) -> Vec<&DescriptorSet> {
        self.descriptor_sets
            .iter()
            .filter(|(handle, _)| handles.contains(handle))
            .map(|(_, descriptor_set)| descriptor_set)
            .collect()
    }

    pub fn get_mut(&mut self, handle: DescriptorSetHandle) -> Option<&mut DescriptorSet> {
        self.descriptor_sets.get_mut(handle)
    }

    pub fn get_multiple_mut(
        &mut self,
        handles: Vec<DescriptorSetHandle>,
    ) -> Vec<&mut DescriptorSet> {
        self.descriptor_sets
            .iter_mut()
            .filter(|(handle, _)| handles.contains(handle))
            .map(|(_, descriptor_set)| descriptor_set)
            .collect()
    }

    pub fn allocate_descriptor_sets<const N: usize>(
        &mut self,
        layout: &DescriptorSetLayout,
    ) -> [DescriptorSetHandle; N] {
        let descriptor_set_layouts = [layout.instance().layout(); N];

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.instance.descriptor_pool)
            .set_layouts(&descriptor_set_layouts);

        let descriptor_sets = unsafe {
            self.instance
                .vulkan_dep
                .device()
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets")
        }
        .into_iter()
        .map(|descriptor_set| DescriptorSet {
            descriptor_set,
            written_dependencies: Vec::new(),
        })
        .collect::<Vec<_>>();

        let mut handles = [DescriptorSetHandle::default(); N];
        for (i, descriptor_set) in descriptor_sets.into_iter().enumerate() {
            handles[i] = self.descriptor_sets.insert(descriptor_set);
        }

        handles
    }
}
