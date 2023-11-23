use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr::addr_of;
use std::sync::Arc;
use crate::core::renderer_trait::{Buffer, get_or_create_buffer};

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

    fn get_vertex_buffer_cpu(&self) -> &Vec<Self::NumericType>;

    fn get_texture_coordinate_cpu(&self) -> &Vec<Self::NumericType>;

    fn get_index_buffer_cpu(&self) -> &Vec<u32>;
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
    fn get_or_create_vertex_buffer(&mut self) -> Arc<RefCell<T>> {
        let (buffer, created) = get_or_create_buffer(&mut self.vertex_buffer).unwrap();
        if created {
            let data = self.get_vertex_buffer_cpu();
            let data_addr = addr_of!(data) as *const c_void;
            let data_size = data.len() * std::mem::size_of::<f32>();
            unsafe { buffer.borrow_mut().fill_buffer_on_device(data_addr, data_size).expect("Failed to copy data to buffer."); }
        }
        buffer
    }

    fn get_or_create_index_buffer(&mut self) -> Arc<RefCell<T>> {
        let (buffer, created) = get_or_create_buffer(&mut self.index_buffer).unwrap();
        if created {
            let data = self.get_index_buffer_cpu();
            let data_addr = addr_of!(data) as *const c_void;
            let data_size = data.len() * std::mem::size_of::<u32>();
            unsafe { buffer.borrow_mut().fill_buffer_on_device(data_addr, data_size).expect("Failed to copy data to buffer."); }
        }
        buffer
    }

    fn get_or_create_texture_coordinate_buffer(&mut self) -> Arc<RefCell<T>> {
        let (buffer, created) = get_or_create_buffer(&mut self.texcoord_buffer).unwrap();
        if created {
            let data = self.get_texture_coordinate_cpu();
            let data_addr = addr_of!(data) as *const c_void;
            let data_size = data.len() * std::mem::size_of::<f32>();
            unsafe { buffer.borrow_mut().fill_buffer_on_device(data_addr, data_size).expect("Failed to copy data to buffer."); }
        }
        buffer
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
