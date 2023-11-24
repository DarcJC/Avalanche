use std::collections::HashMap;
use std::default::Default;
use ash::vk;
use crate::core::renderer_trait::{GraphicAPIBounds, GraphicsAbstract};
use crate::core::renderer_types::GraphicsAPIType;

mod base;
mod ray_tracing;
mod utility;
mod buffer;

pub use base::*;
pub use ray_tracing::*;
pub use utility::*;
pub use buffer::*;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct QueueInfo {
    graphics: u32,
    compute: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CompatibilityFlags {
    khr_acceleration_structure: bool,
    khr_ray_tracing_pipeline: bool,
    rt_max_ray_recursion_depth: u32,
}

impl Default for CompatibilityFlags {
    fn default() -> Self {
        Self {
            khr_acceleration_structure: false,
            khr_ray_tracing_pipeline: false,
            rt_max_ray_recursion_depth: 0,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct VulkanAccelerationStructureState {}

pub struct VulkanRenderer {
    entry: ash::Entry,
    instance: ash::Instance,
    physical_device: Option<vk::PhysicalDevice>,
    logical_device: Option<ash::Device>,
    surfaces: HashMap<winit::window::WindowId, vk::SurfaceKHR>,
    queue_info: QueueInfo,
    compatibility_flags: CompatibilityFlags,
    rt_as_state: VulkanAccelerationStructureState,
}

impl GraphicAPIBounds for VulkanRenderer {
    fn get_graphics_api() -> GraphicsAPIType {
        GraphicsAPIType::Vulkan
    }
}

impl GraphicsAbstract for VulkanRenderer {}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        // Cleanup.
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Debug, Default)]
pub struct VulkanBuffer {
    pub create_info: vk::BufferCreateInfo,
    resource: Option<vk::Buffer>,
    device_memory: Option<vk::DeviceMemory>,
}
