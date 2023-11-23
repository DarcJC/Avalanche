use std::cell::RefCell;
use std::sync::Arc;
use async_trait::async_trait;
use crate::core::renderer_trait::Buffer;
use crate::core::window_manager::get_window_manager;

pub struct RenderWorld {
    pub models: Vec<Box<dyn Mesh<NumericType=f32>>>,
}

pub enum PrimitiveType {
    Point = 1,
    Line = 2,
    Triangle = 3,
    Quadrilateral = 4,
    Polygon = 5,
}

pub trait Mesh {
    type NumericType = f32;

    fn get_primitive_type(&self) -> PrimitiveType;

    fn support_index_buffer(&self) -> bool;

    fn get_vertex_buffer_cpu(&self) -> Vec<Self::NumericType>;

    fn get_texture_coordinate_cpu(&self) -> Vec<Self::NumericType>;

    fn get_index_buffer_cpu(&self) -> Vec<u32>;
}

#[async_trait]
pub trait MeshBuffers {
    async fn get_or_create_vertex_buffer(&mut self) -> Arc<RefCell<dyn Buffer>>;
    async fn get_or_create_index_buffer(&mut self) -> Arc<RefCell<dyn Buffer>>;
    async fn get_or_create_texture_coordinate_buffer(&mut self) -> Arc<RefCell<dyn Buffer>>;
}

pub struct TObjMeshWrapper {
    data: tobj::Mesh,
    vertex_buffer: Option<Arc<RefCell<dyn Buffer>>>,
    index_buffer: Option<Arc<RefCell<dyn Buffer>>>,
    texcoord_buffer: Option<Arc<RefCell<dyn Buffer>>>,
}

unsafe impl Send for TObjMeshWrapper {}
unsafe impl Sync for TObjMeshWrapper {}

impl From<tobj::Mesh> for TObjMeshWrapper {
    fn from(value: tobj::Mesh) -> Self {
        Self {
            data: value,
            vertex_buffer: None,
            index_buffer: None,
            texcoord_buffer: None,
        }
    }
}

impl Mesh for TObjMeshWrapper {
    fn get_primitive_type(&self) -> PrimitiveType {
        // TODO check primitive type
        PrimitiveType::Triangle
    }

    fn support_index_buffer(&self) -> bool {
        true
    }

    fn get_vertex_buffer_cpu(&self) -> Vec<Self::NumericType> {
        self.data.positions.clone()
    }

    fn get_texture_coordinate_cpu(&self) -> Vec<Self::NumericType> {
        self.data.texcoords.clone()
    }

    fn get_index_buffer_cpu(&self) -> Vec<u32> {
        self.data.indices.clone()
    }
}

#[async_trait]
impl MeshBuffers for TObjMeshWrapper {
    async fn get_or_create_vertex_buffer(&mut self) -> Arc<RefCell<dyn Buffer>> {
        if self.vertex_buffer.is_none() {
        }

        todo!()
    }

    async fn get_or_create_index_buffer(&mut self) -> Arc<RefCell<dyn Buffer>> {
        todo!()
    }

    async fn get_or_create_texture_coordinate_buffer(&mut self) -> Arc<RefCell<dyn Buffer>> {
        todo!()
    }
}

impl Drop for TObjMeshWrapper {
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
