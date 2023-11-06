use crate::{Image, ImageDep, InternalImage, Sampler, UntypedBuffer, Vulkan, VulkanDep};
use ash::vk;
use pyrite_util::Dependable;
use std::{
    any::Any,
    ops::Deref,
    sync::{Arc, RwLock, Weak},
};

type DescriptorSetPoolDep = Arc<InternalDescriptorSetPool>;
pub struct DescriptorSetPool {
    internal: Arc<InternalDescriptorSetPool>,
}

impl Deref for DescriptorSetPool {
    type Target = InternalDescriptorSetPool;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

pub struct InternalDescriptorSetPool {
    vulkan_dep: VulkanDep,
    descriptor_pool: vk::DescriptorPool,
}

impl DescriptorSetPool {
    pub fn new(vulkan: &Vulkan) -> Self {
        Self {
            internal: Arc::new(InternalDescriptorSetPool::new(vulkan)),
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
            .map(|descriptor_set| DescriptorSet::new(self, descriptor_set_layout, descriptor_set))
            .collect()
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

impl InternalDescriptorSetPool {
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
            .max_sets(25)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);

        // Safety: The descriptor pool is dropped when the internal descriptor pool is dropped
        let descriptor_pool = unsafe {
            vulkan
                .device()
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create descriptor pool")
        };

        Self {
            vulkan_dep: vulkan.create_dep(),
            descriptor_pool,
        }
    }
    pub fn descriptor_pool(&self) -> &vk::DescriptorPool {
        &self.descriptor_pool
    }
}

pub type DescriptorSetLayoutDep = Arc<InternalDescriptorSetLayout>;
pub struct DescriptorSetLayout {
    internal: Arc<InternalDescriptorSetLayout>,
}

impl Deref for DescriptorSetLayout {
    type Target = InternalDescriptorSetLayout;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl DescriptorSetLayout {
    pub fn new(vulkan: &Vulkan, bindings: &[vk::DescriptorSetLayoutBinding]) -> Self {
        Self {
            internal: Arc::new(InternalDescriptorSetLayout::new(vulkan, bindings)),
        }
    }

    pub fn create_dep(&self) -> DescriptorSetLayoutDep {
        self.internal.clone()
    }
}

pub struct InternalDescriptorSetLayout {
    vulkan_dep: VulkanDep,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl InternalDescriptorSetLayout {
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

    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
}

impl Drop for InternalDescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

pub type DescriptorSetDep = Arc<DescriptorSetInner>;
pub struct DescriptorSet {
    inner: Arc<DescriptorSetInner>,
}

impl DescriptorSet {
    fn new(
        descriptor_pool: &DescriptorSetPool,
        descriptor_set_layout: &DescriptorSetLayout,
        descriptor_set: vk::DescriptorSet,
    ) -> Self {
        Self {
            inner: Arc::new(DescriptorSetInner::new(
                descriptor_pool,
                descriptor_set_layout,
                descriptor_set,
            )),
        }
    }

    pub fn create_dep(&self) -> DescriptorSetDep {
        self.inner.clone()
    }
}

impl Deref for DescriptorSet {
    type Target = DescriptorSetInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct DescriptorSetInner {
    descriptor_pool_dep: DescriptorSetPoolDep,
    _descriptor_layout_dep: DescriptorSetLayoutDep,
    descriptor_set: vk::DescriptorSet,

    used_objects: RwLock<Vec<Weak<dyn Any + Send + Sync>>>,
}

impl Drop for DescriptorSetInner {
    fn drop(&mut self) {
        unsafe {
            self.descriptor_pool_dep
                .vulkan_dep
                .device()
                .free_descriptor_sets(
                    *self.descriptor_pool_dep.descriptor_pool(),
                    &[self.descriptor_set],
                )
                .expect("Failed to free descriptor sets");
        }
    }
}

impl DescriptorSetInner {
    fn new(
        descriptor_pool: &DescriptorSetPool,
        descriptor_set_layout: &DescriptorSetLayout,
        descriptor_set: vk::DescriptorSet,
    ) -> Self {
        Self {
            descriptor_pool_dep: descriptor_pool.internal.clone(),
            _descriptor_layout_dep: descriptor_set_layout.create_dep(),
            descriptor_set,
            used_objects: RwLock::new(Vec::new()),
        }
    }

    pub unsafe fn update_descriptor_set(&self, descriptor_writes: &[vk::WriteDescriptorSet]) {
        self.used_objects.write().unwrap().clear();
        self.descriptor_pool_dep
            .vulkan_dep
            .device()
            .update_descriptor_sets(descriptor_writes, &[]);
    }

    pub fn write(&self) -> DescriptorSetWriter {
        DescriptorSetWriter::new(self)
    }

    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }

    pub fn used_objects(&self) -> Vec<Weak<dyn Any + Send + Sync>> {
        self.used_objects.read().unwrap().clone()
    }
}

pub struct DescriptorSetWriter<'a> {
    descriptor_set: &'a DescriptorSetInner,
    descriptor_writes: Vec<vk::WriteDescriptorSet>,
    buffer_infos: Vec<vk::DescriptorBufferInfo>,
    image_infos: Vec<vk::DescriptorImageInfo>,
    used_objects: Vec<Weak<dyn Any + Send + Sync>>,
}

impl<'a> DescriptorSetWriter<'a> {
    pub fn new(descriptor_set: &'a DescriptorSetInner) -> Self {
        Self {
            descriptor_set,
            descriptor_writes: Vec::new(),
            buffer_infos: Vec::new(),
            image_infos: Vec::new(),
            used_objects: Vec::new(),
        }
    }

    pub fn set_uniform_buffer(mut self, binding: u32, buffer: &Arc<UntypedBuffer>) -> Self {
        // Used to keep buffer info pointer alive until write call.
        self.buffer_infos.push(
            vk::DescriptorBufferInfo::builder()
                .buffer(buffer.buffer())
                .range(vk::WHOLE_SIZE)
                .build(),
        );
        let buffer_info = self.buffer_infos.get(self.buffer_infos.len() - 1).unwrap();

        self.descriptor_writes.push(
            vk::WriteDescriptorSet::builder()
                .dst_set(self.descriptor_set.descriptor_set())
                .dst_binding(binding)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(buffer_info))
                .build(),
        );
        self.used_objects
            .push(Arc::downgrade(buffer) as Weak<dyn Any + Send + Sync>);
        self
    }

    pub fn set_storage_image(mut self, binding: u32, image: ImageDep) -> Self {
        self.image_infos.push(
            vk::DescriptorImageInfo::builder()
                .image_view(image.image_view())
                .image_layout(vk::ImageLayout::GENERAL)
                .build(),
        );
        let image_info = self.image_infos.get(self.image_infos.len() - 1).unwrap();

        self.descriptor_writes.push(
            vk::WriteDescriptorSet::builder()
                .dst_set(self.descriptor_set.descriptor_set())
                .dst_binding(binding)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(image_info))
                .build(),
        );

        self.used_objects
            .push(Arc::downgrade(&image) as Weak<dyn Any + Send + Sync>);

        self
    }

    pub fn set_combined_image_sampler(
        mut self,
        binding: u32,
        image_layout: vk::ImageLayout,
        image: ImageDep,
        sampler: &Sampler,
    ) -> Self {
        self.image_infos.push(
            vk::DescriptorImageInfo::builder()
                .image_view(image.image_view())
                .image_layout(image_layout)
                .sampler(sampler.sampler())
                .build(),
        );
        let image_info = self.image_infos.get(self.image_infos.len() - 1).unwrap();

        self.descriptor_writes.push(
            vk::WriteDescriptorSet::builder()
                .dst_set(self.descriptor_set.descriptor_set())
                .dst_binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(image_info))
                .build(),
        );

        self.used_objects
            .push(Arc::downgrade(&image) as Weak<dyn Any + Send + Sync>);
        self.used_objects
            .push(Arc::downgrade(&sampler.create_dep()) as Weak<dyn Any + Send + Sync>);

        self
    }

    pub fn submit_writes(self) {
        // Safety: The bound items to the descriptor set are tracked and bound whenever this
        // descriptor set is in use. The descriptor set is also guaranteed to not be null since we
        // have a reference to it.
        unsafe {
            self.descriptor_set
                .update_descriptor_set(&self.descriptor_writes)
        };

        let mut descriptor_set_used_buffers = self.descriptor_set.used_objects.write().unwrap();
        descriptor_set_used_buffers.clear();
        descriptor_set_used_buffers.extend(self.used_objects);
    }
}
