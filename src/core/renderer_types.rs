use std::any::Any;
use enumflags2::bitflags;

/// Bottom Level Acceleration Structure Build Data
pub struct BLASBuildData {
    pub geometries: Vec<Box<dyn Any>>,
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

#[bitflags]
#[repr(u64)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum GraphicsBufferCreationFlags {
    Preserved = 0b0001,
}

#[bitflags]
#[repr(u8)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum GraphicsBufferShareModes {
    Exclusive = 0b0001,
    Concurrent = 0b0010,
}

#[bitflags]
#[repr(u64)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum GraphicsBufferUsageFlags {
    VertexBuffer = 0b0001,
    IndexBuffer,
}
