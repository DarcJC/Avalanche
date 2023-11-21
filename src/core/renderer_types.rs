use crate::core::scene::Mesh;

/// Bottom Level Acceleration Structure Build Data
pub struct BLASBuildData {
    pub geometries: Vec<Box<dyn Mesh<NumericType=f32>>>,
}

impl Default for BLASBuildData {
    fn default() -> Self {
        Self {
            geometries: vec![],
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum GraphicsAPIType {
    Vulkan = 0,
}
