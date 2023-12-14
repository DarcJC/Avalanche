use std::sync::{Arc, Mutex};
use anyhow::Result;
use ash::vk;
use gpu_allocator::vulkan::{Allocation, Allocator};
use crate::Device;

pub struct Image {
    device: Arc<Device>,
    allocator: Arc<Mutex<Allocator>>,
    pub(crate) inner: vk::Image,
    allocation: Option<Allocation>,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    /// Preventing internal referenced Image been destroyed.
    is_external_referenced: bool,
}

#[derive(Clone)]
pub struct ImageView {
    device: Arc<Device>,
    pub(crate) inner: vk::ImageView,
}
