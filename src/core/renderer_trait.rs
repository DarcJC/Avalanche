use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::Arc;
use anyhow::Result;
use async_std::task::block_on;
use enumflags2::BitFlags;
use crate::core::event_loop::EventLoopManager;
use crate::core::renderer_types::{BLASBuildData, GraphicsAPIType, GraphicsBufferCreationFlags, GraphicsBufferShareModes, GraphicsBufferUsageFlags};
use crate::core::window_manager::{get_window_manager, RendererType};

/// Trait for graphics API wrapping.
///
/// Provide consistency between graphics APIs.
pub trait Renderer {
    type BufferType: Buffer;

    /// Create a new wrapper.
    ///
    /// Instancing an entry of most the graphics APIs here.
    ///
    /// **Note**:
    /// **initialize()** function must be called before invoking interfaces.
    fn new() -> Self where Self: Sized;
    /// Creating a new window.
    ///
    /// Surface and Swapchain will be created and bind at the same time.
    fn create_window(&mut self, event_loop: &mut EventLoopManager, title: &str, width: u32, height: u32) -> winit::window::Window;
    fn list_physical_devices(&self);
    /// Picking physical device and logical device.
    fn initialize(&mut self);
    /// Check if device is supporting ray tracing.
    fn support_ray_tracing(&self) -> bool;

    /// Call to perform real buffer allocation on GPU.
    ///
    /// **Note**
    ///
    /// This function isn't an equivalent to vkCreateBuffer,
    /// it also allocating and binding the device memory to the created buffer.
    fn create_buffer_resource(&mut self, buffer: &mut impl Buffer) -> Result<()>;

    /// Mapping device memory binding with buffer to host.
    fn map_buffer_memory(&self, buffer: &mut impl Buffer) -> Result<*mut c_void>;

    /// Unmapping device memory binding with buffer to host.
    fn unmap_buffer_memory(&self, buffer: &mut impl Buffer) -> Result<()>;

    /// Create a new create info of current graphics API
    fn get_buffer_creation_info(share_modes: BitFlags<GraphicsBufferShareModes>, usage: BitFlags<GraphicsBufferUsageFlags>, flags: BitFlags<GraphicsBufferCreationFlags>, size: usize) -> Result<Rc<dyn Any>>;
}

/// Graphics API Ray Tracing extension wrapper.
///
/// Providing consistency interface between different graphics APIs.
pub trait RayTracingRenderer where Self: Renderer {
    /// Creating Acceleration Structure build info like `VulkanBottomLevelAccelerationStructureBuildItem`
    fn mesh_buffers_to_geometry(&self, buffer: &mut impl MeshBuffers<Self::BufferType>) -> Result<Box<dyn Any>>;
    fn build_bottom_level_acceleration_structure(&mut self, inputs: &BLASBuildData);
}

/// Providing interface to indicate which graphics API is being using currently.
pub trait GraphicAPIBounds {
    fn get_graphics_api() -> GraphicsAPIType where Self: Sized;
}

/// A full feature graphics API abstraction.
pub trait GraphicsAbstract : GraphicAPIBounds + Renderer + RayTracingRenderer {}

/// Buffer wrapper
pub trait Buffer: Default where Self: GraphicAPIBounds {
    fn get_buffer_name<'a>() -> &'a str where Self: Sized;
    fn release(&mut self);
    fn get_pending_upload_size(&self) -> u64;
    unsafe fn fill_buffer_on_device(&mut self, src: *const c_void, size: usize) -> Result<()>;
    fn set_create_description(&mut self, desc: Rc<dyn Any>) -> Result<()>;
}

pub trait MeshBuffers<T: Buffer> {
    fn get_or_create_vertex_buffer(&mut self) -> Arc<RefCell<T>>;
    fn get_or_create_index_buffer(&mut self) -> Arc<RefCell<T>>;
    fn get_or_create_texture_coordinate_buffer(&mut self) -> Arc<RefCell<T>>;

    fn get_primitive_count(&self) -> usize;
    fn get_vertex_count(&self) -> usize;
}

pub trait MeshRayTracingExtension {
    type APIType: GraphicsAbstract = RendererType;
    fn create_rt_geometry(&mut self) -> Result<Box<dyn Any>>;
}

/// Casting generic buffer to API specified buffer.
///
/// **None** will be returned if API bounds checking isn't passing.
pub fn buffer_cast<'a, TargetType: Buffer + 'a, U: Buffer + 'a>(buffer: &mut U) -> Option<&mut TargetType> {
    if TargetType::get_graphics_api() == U::get_graphics_api() {
        Some(unsafe { std::mem::transmute(buffer) })
    } else {
        None
    }
}

/// Create buffer and binding memory with size in create_info.
pub fn get_or_create_buffer<T: Buffer>(buffer: &mut Option<Arc<RefCell<T>>>, buf_desc: Rc<dyn Any>) -> Result<(Arc<RefCell<T>>, bool)> {
    let created = buffer.is_none();
    if buffer.is_none() {
        let manager = block_on(get_window_manager());
        let mut renderer = block_on(manager.renderer.lock());
        let mut new_buffer = RefCell::new(T::default());
        new_buffer.get_mut().set_create_description(buf_desc)?;
        renderer.create_buffer_resource(new_buffer.get_mut())?;
        *buffer = Some(Arc::new(new_buffer));
    }
    Ok((buffer.clone().unwrap(), created))
}
