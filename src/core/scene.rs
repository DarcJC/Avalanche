
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

    fn get_vertex_buffer(&self) -> Vec<Self::NumericType>;

    fn get_texture_coordinate(&self) -> Vec<Self::NumericType>;

    fn get_index_buffer(&self) -> Vec<u32>;
}

pub trait MeshBuffers {
}

impl Mesh for tobj::Mesh {
    fn get_primitive_type(&self) -> PrimitiveType {
        // TODO check primitive type
        PrimitiveType::Triangle
    }

    fn support_index_buffer(&self) -> bool {
        true
    }

    fn get_vertex_buffer(&self) -> Vec<Self::NumericType> {
        self.positions.clone()
    }

    fn get_texture_coordinate(&self) -> Vec<Self::NumericType> {
        self.texcoords.clone()
    }

    fn get_index_buffer(&self) -> Vec<u32> {
        self.indices.clone()
    }
}
