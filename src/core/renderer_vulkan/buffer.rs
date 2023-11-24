use std::any::Any;
use std::ffi::c_void;
use std::rc::Rc;
use anyhow::{Context, Result};
use ash::vk;
use crate::core::renderer_trait::{Buffer, GraphicAPIBounds, Renderer};
use crate::core::renderer_types::GraphicsAPIType;
use crate::core::renderer_vulkan::VulkanBuffer;
use crate::core::window_manager::get_window_manager;
use crate::get_renderer_as_var;

impl Buffer for VulkanBuffer {
    fn get_buffer_name<'a>() -> &'a str {
        "Vulkan Buffer Temp Name"
    }

    fn release(&mut self) {
        get_renderer_as_var!(window_manager, renderer);

        if let Some(memory) = self.device_memory {
            unsafe {
                renderer.logical_device.as_ref().unwrap().free_memory(memory, None);
            }
        }

        if let Some(handle) = self.resource {
            unsafe {
                renderer.logical_device.as_ref().unwrap().destroy_buffer(handle, None);
            }
        }
    }

    fn get_pending_upload_size(&self) -> u64 {
        self.create_info.size
    }

    unsafe fn fill_buffer_on_device(&mut self, src: *const c_void, size: usize) -> Result<()> {
        get_renderer_as_var!(window_manager, renderer);
        let addr = renderer.map_buffer_memory(self).context("Failed to map buffer memory")?;

        std::intrinsics::copy(src, addr, size);

        Ok(())
    }

    fn set_create_description(&mut self, desc: Rc<dyn Any>) -> Result<()> {
        let desc = desc.downcast_ref::<vk::BufferCreateInfo>().context("Passing bad argument to set_create_description.")?;
        self.create_info = *desc;
        Ok(())
    }
}

impl GraphicAPIBounds for VulkanBuffer {
    fn get_graphics_api() -> GraphicsAPIType {
        GraphicsAPIType::Vulkan
    }
}
