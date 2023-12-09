use std::sync::Arc;

use ash::vk;

use crate::{util::VulkanResource, Vulkan, VulkanDep};

pub type FenceDep = Arc<FenceInstance>;

pub struct FenceInstance {
    vulkan_dep: VulkanDep,
    fence: vk::Fence,
}

impl FenceInstance {
    pub fn fence(&self) -> vk::Fence {
        self.fence
    }
}

impl VulkanResource for FenceInstance {}

impl Drop for FenceInstance {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep.device().destroy_fence(self.fence, None);
        }
    }
}

pub struct Fence {
    instance: Arc<FenceInstance>,
}

impl Fence {
    pub fn new(vulkan: &Vulkan, signaled: bool) -> Self {
        let fence_flags = if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        };

        let fence = unsafe {
            vulkan
                .device()
                .create_fence(&vk::FenceCreateInfo::default().flags(fence_flags), None)
                .expect("Failed to create fence")
        };
        Self {
            instance: Arc::new(FenceInstance {
                vulkan_dep: vulkan.create_dep(),
                fence,
            }),
        }
    }

    pub fn wait(&self) {
        unsafe {
            self.instance
                .vulkan_dep
                .device()
                .wait_for_fences(&[self.instance.fence], true, std::u64::MAX)
                .expect("Failed to wait for fence");
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.instance
                .vulkan_dep
                .device()
                .reset_fences(&[self.instance.fence])
                .expect("Failed to reset fence");
        }
    }

    pub fn wait_and_reset(&self) {
        self.wait();
        self.reset();
    }

    pub fn fence(&self) -> vk::Fence {
        self.instance.fence
    }

    pub fn create_dep(&self) -> FenceDep {
        self.instance.clone()
    }
}

pub type SemaphoreDep = Arc<SemaphoreInstance>;

pub struct SemaphoreInstance {
    vulkan_dep: VulkanDep,
    semaphore: vk::Semaphore,
}

impl SemaphoreInstance {
    pub fn semaphore(&self) -> vk::Semaphore {
        self.semaphore
    }
}

impl VulkanResource for SemaphoreInstance {}

impl Drop for SemaphoreInstance {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_semaphore(self.semaphore, None);
        }
    }
}

pub struct Semaphore {
    instance: Arc<SemaphoreInstance>,
}

impl Semaphore {
    pub fn new(vulkan: &Vulkan) -> Self {
        let semaphore = unsafe {
            vulkan
                .device()
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .expect("Failed to create semaphore")
        };
        Self {
            instance: Arc::new(SemaphoreInstance {
                vulkan_dep: vulkan.create_dep(),
                semaphore,
            }),
        }
    }

    pub fn semaphore(&self) -> vk::Semaphore {
        self.instance.semaphore
    }

    pub fn create_dep(&self) -> SemaphoreDep {
        self.instance.clone()
    }
}
