use std::collections::HashSet;

use crate::{
    QueueName,
    VulkanRef,
};

pub enum SharingMode {
    Exclusive,
    Concurrent(Vec<QueueName>),
}

impl SharingMode {
    pub fn queue_family_indices(&self, vulkan: VulkanRef) -> Option<Vec<u32>> {
        match self {
            Self::Exclusive => None,
            Self::Concurrent(queue_names) => Some(
                queue_names
                    .iter()
                    .map(|queue_name| vulkan.get_queue(queue_name).queue_family_index())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
            ),
        }
    }
}
