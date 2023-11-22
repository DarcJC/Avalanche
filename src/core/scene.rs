use crate::core::renderer_trait::Buffer;

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

pub trait MeshBuffers {
    fn get_or_create_vertex_buffer(&mut self) -> &mut Box<dyn Buffer>;
    fn get_or_create_index_buffer(&mut self) -> &mut Box<dyn Buffer>;
    fn get_or_create_texture_coordinate_buffer(&mut self) -> &mut Box<dyn Buffer>;
}

pub struct TObjMeshWrapper {
    data: tobj::Mesh,
    vertex_buffer: Option<Box<dyn Buffer>>,
    index_buffer: Option<Box<dyn Buffer>>,
    texcoord_buffer: Option<Box<dyn Buffer>>,
}

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

impl MeshBuffers for TObjMeshWrapper {
    fn get_or_create_vertex_buffer(&mut self) -> &mut Box<dyn Buffer> {
        if self.vertex_buffer.is_none() {

        }

        self.vertex_buffer.as_mut().unwrap()
    }

    fn get_or_create_index_buffer(&mut self) -> &mut Box<dyn Buffer> {
        todo!()
    }

    fn get_or_create_texture_coordinate_buffer(&mut self) -> &mut Box<dyn Buffer> {
        todo!()
    }
}

impl Drop for TObjMeshWrapper {
    fn drop(&mut self) {
        if let Some(buffer) = &mut self.vertex_buffer {
            buffer.release();
        }
        if let Some(buffer) = &mut self.index_buffer {
            buffer.release();
        }
        if let Some(buffer) = &mut self.texcoord_buffer {
            buffer.release();
        }
    }
}
