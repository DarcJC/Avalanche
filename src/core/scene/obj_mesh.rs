use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::Arc;
use enumflags2::{BitFlags};
use crate::core::renderer_trait::{Buffer, get_or_create_buffer, GraphicsAbstract, MeshBuffers, MeshRayTracingExtension, Renderer};
use crate::core::renderer_types::{GraphicsBufferShareModes, GraphicsBufferUsageFlags};
use crate::core::scene::{get_or_create_buf, Mesh, PrimitiveType};
use crate::core::window_manager::{get_window_manager, RendererType};

pub struct TObjMeshWrapper<T: Buffer> {
    data: tobj::Mesh,
    vertex_buffer: Option<Arc<RefCell<T>>>,
    index_buffer: Option<Arc<RefCell<T>>>,
    texcoord_buffer: Option<Arc<RefCell<T>>>,
}

unsafe impl<T: Buffer> Send for TObjMeshWrapper<T> {}
unsafe impl<T: Buffer> Sync for TObjMeshWrapper<T> {}

impl<T: Buffer> From<tobj::Mesh> for TObjMeshWrapper<T> {
    fn from(value: tobj::Mesh) -> Self {
        Self {
            data: value,
            vertex_buffer: None,
            index_buffer: None,
            texcoord_buffer: None,
        }
    }
}

impl<T: Buffer> Mesh for TObjMeshWrapper<T> {
    fn get_primitive_type(&self) -> PrimitiveType {
        // TODO check primitive type
        PrimitiveType::Triangle
    }

    fn support_index_buffer(&self) -> bool {
        true
    }

    fn get_vertex_buffer_cpu(&self) -> &Vec<Self::NumericType> {
        &self.data.positions
    }

    fn get_texture_coordinate_cpu(&self) -> &Vec<Self::NumericType> {
        &self.data.texcoords
    }

    fn get_index_buffer_cpu(&self) -> &Vec<u32> {
        &self.data.indices
    }
}

impl<T: Buffer> MeshBuffers<T> for TObjMeshWrapper<T> {
    get_or_create_buf!(get_or_create_vertex_buffer, get_vertex_buffer_cpu, vertex_buffer, f32, GraphicsBufferUsageFlags::VertexBuffer);
    get_or_create_buf!(get_or_create_index_buffer, get_index_buffer_cpu, index_buffer, u32, GraphicsBufferUsageFlags::IndexBuffer);
    get_or_create_buf!(get_or_create_texture_coordinate_buffer, get_texture_coordinate_cpu, texcoord_buffer, f32, GraphicsBufferUsageFlags::VertexBuffer);

    fn get_primitive_count(&self) -> usize {
        self.data.indices.len() / 3
    }

    fn get_vertex_count(&self) -> usize {
        self.data.positions.len()
    }
}

impl<T: Buffer> Drop for TObjMeshWrapper<T> {
    fn drop(&mut self) {
        if let Some(buffer) = &mut self.vertex_buffer {
            buffer.borrow_mut().release()
        }
        if let Some(buffer) = &mut self.index_buffer {
            buffer.borrow_mut().release()
        }
        if let Some(buffer) = &mut self.texcoord_buffer {
            buffer.borrow_mut().release()
        }
    }
}
