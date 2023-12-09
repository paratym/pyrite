use ash::vk;

use crate::objects::{CommandBuffer, Fence, Semaphore};
use crate::swapchain::Swapchain;
use crate::util::{GenericResourceDep, VulkanResourceDep};
use crate::VulkanQueue;

/// A queue exectutor keeps track of in flight frame resources.
pub struct QueueExecutor<const N: usize> {
    vulkan_dep: crate::VulkanDep,
    queue_name: String,
    in_flight_dependencies: [Vec<GenericResourceDep>; N],
}

pub struct QueueExecutorSubmitInfo<'a> {
    pub command_buffers: Vec<&'a mut CommandBuffer>,
    pub frame_index: usize,
    pub wait_semaphores: Vec<(&'a Semaphore, vk::PipelineStageFlags)>,
    pub signal_semaphores: Vec<&'a Semaphore>,
    pub fence: Option<&'a Fence>,
}

impl<const N: usize> QueueExecutor<N> {
    pub fn new(vulkan: &crate::Vulkan, queue_name: impl Into<String>) -> Self {
        let in_flight_dependencies = (0..N)
            .map(|_| Vec::new())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| panic!("Failed to create frames in flight."));

        Self {
            vulkan_dep: vulkan.create_dep(),
            queue_name: queue_name.into(),
            in_flight_dependencies,
        }
    }

    /// Block until the in flight frame resources are ready to be used.
    /// Releases all the previously in flight resources.
    pub fn release_frame_resources(&mut self, frame_index: usize) {
        self.in_flight_dependencies[frame_index].clear();
    }

    pub fn submit(&mut self, mut info: QueueExecutorSubmitInfo) {
        let in_flight_dependencies = &mut self.in_flight_dependencies[info.frame_index as usize];
        in_flight_dependencies.extend(
            info.command_buffers
                .iter_mut()
                .flat_map(|command_buffer| command_buffer.take_recorded_dependencies()
                    .into_iter()
                    .map(|weak_dep| weak_dep.upgrade().expect("Tried to submit a command buffer with a dependency that was already dropped."))),
        );
        in_flight_dependencies.extend(
            info.wait_semaphores
                .iter()
                .map(|(semaphore, _)| semaphore.create_dep().into_generic()),
        );
        in_flight_dependencies.extend(
            info.signal_semaphores
                .iter()
                .map(|semaphore| semaphore.create_dep().into_generic()),
        );
        if let Some(fence) = info.fence {
            in_flight_dependencies.push(fence.create_dep().into_generic());
        }

        let vk_command_buffers = info
            .command_buffers
            .iter()
            .map(|command_buffer| command_buffer.command_buffer())
            .collect::<Vec<_>>();
        let vk_wait_semaphores = info
            .wait_semaphores
            .iter()
            .map(|semaphore| semaphore.0.semaphore())
            .collect::<Vec<_>>();
        let vk_wait_stages = info
            .wait_semaphores
            .iter()
            .map(|semaphore| semaphore.1)
            .collect::<Vec<_>>();
        let vk_signal_semaphores = info
            .signal_semaphores
            .iter()
            .map(|semaphore| semaphore.semaphore())
            .collect::<Vec<_>>();
        let vk_submit_infos = [vk::SubmitInfo::default()
            .command_buffers(&vk_command_buffers)
            .wait_semaphores(&vk_wait_semaphores)
            .wait_dst_stage_mask(&vk_wait_stages)
            .signal_semaphores(&vk_signal_semaphores)];
        let vk_fence = match info.fence {
            Some(fence) => fence.fence(),
            None => vk::Fence::null(),
        };
        unsafe {
            self.vulkan_dep
                .device()
                .queue_submit(self.queue().queue(), &vk_submit_infos, vk_fence)
                .expect("Failed to submit queue")
        };
    }

    pub fn present(
        &mut self,
        swapchain: &Swapchain,
        image_index: u32,
        wait_semaphores: Vec<&Semaphore>,
    ) {
        let image_indices = [image_index];
        let wait_semaphores = wait_semaphores
            .iter()
            .map(|semaphore| semaphore.semaphore())
            .collect::<Vec<_>>();
        let swapchains = [swapchain.instance().swapchain()];
        let present_info = vk::PresentInfoKHR::default()
            .image_indices(&image_indices)
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains);

        let present_result = unsafe {
            swapchain
                .instance()
                .swapchain_loader()
                .queue_present(self.queue().queue(), &present_info)
        };
    }

    pub fn wait_idle(&self) {
        unsafe {
            self.vulkan_dep
                .device()
                .queue_wait_idle(self.queue().queue())
                .expect("Failed to wait for queue to become idle.");
        }
    }

    fn queue(&self) -> &VulkanQueue {
        self.vulkan_dep.queue(&self.queue_name).unwrap()
    }
}
