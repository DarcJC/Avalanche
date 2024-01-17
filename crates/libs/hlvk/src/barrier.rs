use std::sync::Arc;
use std::time::Duration;
use anyhow::Context;
use ash::vk;
use crate::{Context as CContext, Device};

/// Semaphore is for ordering gpu tasks
pub struct Semaphore {
    device: Arc<Device>,
    pub inner: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: Arc<Device>) -> anyhow::Result<Self> {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let inner = unsafe { device.inner.create_semaphore(&semaphore_info, None)? };

        Ok(Self { device, inner })
    }
}

impl CContext {
    pub fn create_semaphore(&self) -> anyhow::Result<Semaphore> {
        Semaphore::new(self.device.clone())
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.destroy_semaphore(self.inner, None);
        }
    }
}

/// Fence is for host-gpu sync
pub struct Fence {
    device: Option<Arc<Device>>,
    pub(crate) inner: vk::Fence,
}

impl Fence {
    pub fn new(device: Arc<Device>, flags: Option<vk::FenceCreateFlags>) -> anyhow::Result<Self> {
        let flags = flags.unwrap_or_else(vk::FenceCreateFlags::empty);
        let fence_info = vk::FenceCreateInfo::builder().flags(flags);
        let inner = unsafe { device.inner.create_fence(&fence_info, None)? };

        Ok(Self { device: Some(device), inner })
    }

    pub fn null() -> Self {
        Self {
            device: None,
            inner: vk::Fence::null(),
        }
    }

    pub fn wait(&self, timeout: Option<Duration>) -> anyhow::Result<()> {
        let timeout = timeout.unwrap_or(Duration::from_nanos(u64::MAX)).as_nanos() as u64;

        unsafe {
            self.device
                .as_ref()
                .context("Could not wait null fence.")?
                .inner
                .wait_for_fences(&[self.inner], true, timeout)?
        }

        Ok(())
    }

    pub fn reset(&self) -> anyhow::Result<()> {
        unsafe { self.device.as_ref().context("Could not wait null fence.")?.inner.reset_fences(&[self.inner])? };

        Ok(())
    }

    pub fn is_null(&self) -> bool {
        self.device.is_none()
    }
}

impl CContext {
    pub fn create_fence(&self, flags: Option<vk::FenceCreateFlags>) -> anyhow::Result<Fence> {
        Fence::new(self.device.clone(), flags)
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            if let Some(device) = &self.device {
                device.inner.destroy_fence(self.inner, None)
            }
        }
    }
}
