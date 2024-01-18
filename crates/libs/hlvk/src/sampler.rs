use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use anyhow::Result;
use ash::vk;
use ash::vk::Handle;
use crate::{Context, Device};

pub struct Sampler {
    device: Arc<Device>,
    pub(crate) inner: vk::Sampler,
}

impl Sampler {
    pub(crate) fn new(device: Arc<Device>, create_info: &vk::SamplerCreateInfo) -> Result<Self> {
        let inner = unsafe { device.inner.create_sampler(create_info, None)? };

        Ok(Self { device, inner })
    }
}

impl Context {
    pub fn create_sampler(&self, create_info: &vk::SamplerCreateInfo) -> Result<Sampler> {
        Sampler::new(self.device.clone(), create_info)
    }
}

impl Debug for Sampler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Sampler({})", self.inner.as_raw())
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.destroy_sampler(self.inner, None)
        }
    }
}
