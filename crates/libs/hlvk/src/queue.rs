use std::sync::Arc;
use ash::vk;
use crate::Device;

#[derive(Debug, Clone, Copy)]
pub struct QueueFamily {
    pub index: u32,
    pub(crate) inner: vk::QueueFamilyProperties,
    support_present: bool,
}

impl QueueFamily {
    pub(crate) fn new(
        index: u32,
        inner: vk::QueueFamilyProperties,
        support_present: bool,
    ) -> Self {
        Self {
            index,
            inner,
            support_present,
        }
    }

    pub fn supports_compute(&self) -> bool {
        self.inner.queue_flags.contains(vk::QueueFlags::COMPUTE)
    }

    pub fn supports_graphics(&self) -> bool {
        self.inner.queue_flags.contains(vk::QueueFlags::GRAPHICS)
    }

    pub fn supports_present(&self) -> bool {
        self.support_present
    }

    pub fn has_queues(&self) -> bool {
        self.inner.queue_count > 0
    }

    pub fn supports_timestamp_queries(&self) -> bool {
        self.inner.timestamp_valid_bits > 0
    }
}

#[derive(Clone)]
pub struct Queue {
    device: Arc<Device>,
    pub inner: vk::Queue,
}

impl Queue {
    pub(crate) fn new(device: Arc<Device>, inner: vk::Queue) -> Self {
        Self { device, inner }
    }

    // TODO submit function
}
