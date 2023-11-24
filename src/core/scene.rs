mod obj_mesh;
mod utility;

pub use obj_mesh::*;
pub(crate) use utility::*;

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
