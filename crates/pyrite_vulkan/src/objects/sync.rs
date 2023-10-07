use crate::{
    Vulkan,
    VulkanDep,
    VulkanInstance,
};
use ash::vk;

pub struct Semaphore {
    vulkan_dep: VulkanDep,
    semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(vulkan: &Vulkan) -> Self {
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();

        // Safety: The semaphore is only used in this struct and is destroyed when this struct is
        // dropped
        let semaphore = unsafe {
            vulkan
                .device()
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create semaphore")
        };

        Self {
            vulkan_dep: vulkan.create_dep(),
            semaphore,
        }
    }

    pub fn semaphore(&self) -> vk::Semaphore {
        self.semaphore
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep
                .device()
                .destroy_semaphore(self.semaphore, None);
        }
    }
}

pub struct Fence {
    vulkan_dep: VulkanDep,
    fence: vk::Fence,
}

impl Fence {
    pub fn new(vulkan: &Vulkan, is_signaled: bool) -> Self {
        let fence_create_info = vk::FenceCreateInfo::builder().flags(if is_signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        });

        // Safety: The fence is only used in this struct and is destroyed when this struct is
        // dropped
        let fence = unsafe {
            vulkan
                .device()
                .create_fence(&fence_create_info, None)
                .expect("Failed to create fence")
        };

        Self {
            vulkan_dep: vulkan.create_dep(),
            fence,
        }
    }

    pub fn wait(&self) {
        unsafe {
            self.vulkan_dep
                .device()
                .wait_for_fences(&[self.fence], true, std::u64::MAX)
                .expect("Failed to wait for fence");
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.vulkan_dep
                .device()
                .reset_fences(&[self.fence])
                .expect("Failed to reset fence");
        }
    }

    pub fn fence(&self) -> vk::Fence {
        self.fence
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_dep.device().destroy_fence(self.fence, None);
        }
    }
}
