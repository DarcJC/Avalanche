use std::cell::RefCell;
use std::sync::Arc;
use async_std::task::block_on;
use anyhow::Result;
use crate::core::renderer_trait::{Buffer, Renderer};
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

pub trait MeshBuffers<T: Buffer> {
    fn get_or_create_vertex_buffer(&mut self) -> Arc<RefCell<T>>;
    fn get_or_create_index_buffer(&mut self) -> Arc<RefCell<T>>;
    fn get_or_create_texture_coordinate_buffer(&mut self) -> Arc<RefCell<T>>;
}

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

pub fn get_or_create_buffer<T: Buffer>(buffer: &mut Option<Arc<RefCell<T>>>) -> Result<Arc<RefCell<T>>> {
    if buffer.is_none() {
        let manager = block_on(get_window_manager());
        let mut renderer = block_on(manager.renderer.lock());
        let mut new_buffer = RefCell::new(T::default());
        renderer.create_buffer_resource(new_buffer.get_mut())?;
        *buffer = Some(Arc::new(new_buffer));
    }
    Ok(buffer.clone().unwrap())
}

impl<T: Buffer> MeshBuffers<T> for TObjMeshWrapper<T> {
    fn get_or_create_vertex_buffer(&mut self) -> Arc<RefCell<T>> {
        get_or_create_buffer(&mut self.vertex_buffer).unwrap()
    }

    fn get_or_create_index_buffer(&mut self) -> Arc<RefCell<T>> {
        get_or_create_buffer(&mut self.index_buffer).unwrap()
    }

    fn get_or_create_texture_coordinate_buffer(&mut self) -> Arc<RefCell<T>> {
        get_or_create_buffer(&mut self.texcoord_buffer).unwrap()
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
