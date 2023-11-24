use std::any::Any;
use std::mem::size_of;
use std::ops::Deref;
use ash::vk;
use anyhow::Result;
use ash::vk::DeviceSize;
use crate::core::renderer_trait::{MeshBuffers, RayTracingRenderer};
use crate::core::renderer_types::BLASBuildData;
use crate::core::renderer_vulkan::VulkanRenderer;

pub struct VulkanBottomLevelAccelerationStructureBuildItem {
    pub geometry: vk::AccelerationStructureGeometryKHR,
    pub offset: vk::AccelerationStructureBuildRangeInfoKHR,
    pub flags: vk::BuildAccelerationStructureFlagsKHR,
}

impl VulkanBottomLevelAccelerationStructureBuildItem {
    pub fn new(geometry: vk::AccelerationStructureGeometryKHR, offset: vk::AccelerationStructureBuildRangeInfoKHR) -> Self {
        Self {
            geometry,
            offset,
            flags: vk::BuildAccelerationStructureFlagsKHR::empty(),
        }
    }
}

impl RayTracingRenderer for VulkanRenderer {
    fn mesh_buffers_to_geometry(&self, buffer: &mut impl MeshBuffers<Self::BufferType>) -> Result<Box<dyn Any>> {
        // let device = self.logical_device.as_ref().context("Renderer hasn't initialized.")?;
        let vertex_buffer = buffer.get_or_create_vertex_buffer();
        let vertex_buffer_addr = self.get_buffer_device_address(vertex_buffer.borrow().deref())?;
        let index_buffer = buffer.get_or_create_index_buffer();
        let index_buffer_addr = self.get_buffer_device_address(index_buffer.borrow().deref())?;

        let max_primitive_count = buffer.get_primitive_count();

        let mut triangles = vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
            .vertex_format(vk::Format::R32G32B32_SFLOAT)
            .vertex_data(vk::DeviceOrHostAddressConstKHR::default())
            .vertex_stride((size_of::<f32>() * 3) as DeviceSize)
            .index_type(vk::IndexType::UINT32)
            .index_data(vk::DeviceOrHostAddressConstKHR::default())
            .max_vertex((buffer.get_vertex_count() - 1) as u32)
            .build();

        triangles.vertex_data.device_address = vertex_buffer_addr;
        triangles.index_data.device_address = index_buffer_addr;

        let mut geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
            .flags(vk::GeometryFlagsKHR::OPAQUE)
            .build();
        geometry.geometry.triangles = triangles;

        let offset = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .first_vertex(0)
            .primitive_count(max_primitive_count as u32)
            .primitive_offset(0)
            .transform_offset(0)
            .build();

        Ok(Box::new(VulkanBottomLevelAccelerationStructureBuildItem::new(geometry, offset)))
    }

    fn build_bottom_level_acceleration_structure(&self, _inputs: &BLASBuildData) {
    }
}
