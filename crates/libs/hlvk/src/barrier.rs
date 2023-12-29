use std::sync::Arc;
use std::time::Duration;
use ash::vk;
use crate::{Context, Device};

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

impl Context {
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
    device: Arc<Device>,
    pub(crate) inner: vk::Fence,
}

impl Fence {
    pub fn new(device: Arc<Device>, flags: Option<vk::FenceCreateFlags>) -> anyhow::Result<Self> {
        let flags = flags.unwrap_or_else(vk::FenceCreateFlags::empty);
        let fence_info = vk::FenceCreateInfo::builder().flags(flags);
        let inner = unsafe { device.inner.create_fence(&fence_info, None)? };

        Ok(Self { device, inner })
    }

    pub fn wait(&self, timeout: Option<Duration>) -> anyhow::Result<()> {
        let timeout = timeout.unwrap_or(Duration::from_nanos(u64::MAX)).as_nanos() as u64;

        unsafe {
            self.device
                .inner
                .wait_for_fences(&[self.inner], true, timeout)?
        }

        Ok(())
    }

    pub fn reset(&self) -> anyhow::Result<()> {
        unsafe { self.device.inner.reset_fences(&[self.inner])? };

        Ok(())
    }
}

impl Context {
    pub fn create_fence(&self, flags: Option<vk::FenceCreateFlags>) -> anyhow::Result<Fence> {
        Fence::new(self.device.clone(), flags)
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.destroy_fence(self.inner, None)
        }
    }
}
