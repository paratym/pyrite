use std::collections::HashSet;

use ash::vk;

use crate::{QueueName, VulkanRef, DEFAULT_QUEUE};
use anyhow::anyhow;

pub enum SharingMode {
    Exclusive,
    Concurrent(Vec<QueueName>),
}

impl SharingMode {
    pub fn new(vulkan: VulkanRef, queues: Vec<QueueName>) -> anyhow::Result<Self> {
        let queue_name_set = queues
            .into_iter()
            .filter(|queue| vulkan.queue(queue).is_some())
            .collect::<HashSet<_>>();

        if queue_name_set.len() == 1 {
            return Ok(Self::Exclusive);
        }
        if queue_name_set.len() == 0 {
            return Err(anyhow!(
                "No queues specified for sharing mode are available."
            ));
        }
        Ok(Self::Concurrent(
            queue_name_set.into_iter().collect::<Vec<_>>(),
        ))
    }

    pub fn new_with_default(vulkan: VulkanRef) -> Self {
        Self::new(vulkan, vec![DEFAULT_QUEUE.queue_name()])
            .expect("Failed to create default sharing mode.")
    }

    pub fn queue_family_indices(&self, vulkan: VulkanRef) -> Option<Vec<u32>> {
        match self {
            Self::Exclusive => None,
            Self::Concurrent(queue_names) => Some(
                queue_names
                    .iter()
                    .map(|queue_name| {
                        vulkan
                            .queue(queue_name)
                            .expect("Failed to get queue for sharing mode, requested queue is not available.")
                            .queue_family_index()
                    })
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
            ),
        }
    }

    pub fn queue_family_indices_or_default(&self, vulkan: VulkanRef) -> Vec<u32> {
        self.queue_family_indices(vulkan).unwrap_or_else(|| vec![])
    }

    pub fn sharing_mode(&self) -> vk::SharingMode {
        match self {
            Self::Exclusive => vk::SharingMode::EXCLUSIVE,
            Self::Concurrent(_) => vk::SharingMode::CONCURRENT,
        }
    }
}
