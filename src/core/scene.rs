use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::Arc;
use enumflags2::{BitFlags};
use crate::core::renderer_trait::{Buffer, get_or_create_buffer, Renderer};
use crate::core::renderer_types::{GraphicsBufferShareModes, GraphicsBufferUsageFlags};
use crate::core::window_manager::RendererType;

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

macro_rules! get_or_create_buf {
    ($func_name:ident, $get_cpu_func:ident, $buf:ident, $type:ty, $buffer_usage:expr) => {
        fn $func_name(&mut self) -> Arc<RefCell<T>> {
            let (buffer, created) = {
                let data = self.$get_cpu_func();
                let data_size = data.len() * std::mem::size_of::<$type>();
                let create_info = RendererType::get_buffer_creation_info(BitFlags::from(GraphicsBufferShareModes::Exclusive), BitFlags::from($buffer_usage), BitFlags::empty(), data_size).unwrap();
                get_or_create_buffer(&mut self.$buf, create_info).unwrap()
            };
            if created {
                let data = self.$get_cpu_func();
                let data_size = data.len() * std::mem::size_of::<$type>();
                let data_addr = data.as_ptr() as *const c_void;
                unsafe { buffer.borrow_mut().fill_buffer_on_device(data_addr, data_size).expect("Failed to copy data to buffer."); }
            }
            buffer
        }
    };
}

impl<T: Buffer> MeshBuffers<T> for TObjMeshWrapper<T> {
    get_or_create_buf!(get_or_create_vertex_buffer, get_vertex_buffer_cpu, vertex_buffer, f32, GraphicsBufferUsageFlags::VertexBuffer);
    get_or_create_buf!(get_or_create_index_buffer, get_index_buffer_cpu, index_buffer, u32, GraphicsBufferUsageFlags::IndexBuffer);
    get_or_create_buf!(get_or_create_texture_coordinate_buffer, get_texture_coordinate_cpu, texcoord_buffer, f32, GraphicsBufferUsageFlags::VertexBuffer);
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
