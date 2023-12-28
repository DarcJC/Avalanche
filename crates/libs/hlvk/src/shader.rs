use std::ffi::CString;
use std::sync::Arc;
use ash::vk;
use anyhow::Result;
use crate::{Context, Device};

pub struct ShaderModule {
    device: Arc<Device>,
    pub inner: vk::ShaderModule,
}

trait IntoStaged {
    fn into_staged(self, entry_point_name: String, stage: vk::ShaderStageFlags) -> StagedShader;
}

impl ShaderModule {
    pub fn from_spv_bytes(device: Arc<Device>, source: &[u8]) -> Result<Self> {
        let source = read_shader_from_spv_bytes(source)?;

        let create_info = vk::ShaderModuleCreateInfo::builder().code(&source);
        let inner = unsafe { device.inner.create_shader_module(&create_info, None)? };

        Ok(Self { device, inner })
    }
}

impl IntoStaged for ShaderModule {
    fn into_staged(self, entry_point_name: String, stage: vk::ShaderStageFlags) -> StagedShader {
        let module = Arc::new(self);
        StagedShader {
            entry_point_name: CString::new(entry_point_name).unwrap(),
            stage,
            module,
        }
    }
}

impl IntoStaged for Arc<ShaderModule> {
    fn into_staged(self, entry_point_name: String, stage: vk::ShaderStageFlags) -> StagedShader {
        StagedShader {
            module: self,
            entry_point_name: CString::new(entry_point_name).unwrap(),
            stage,
        }
    }
}

pub fn read_shader_from_spv_bytes(bytes: &[u8]) -> Result<Vec<u32>> {
    let mut cursor = std::io::Cursor::new(bytes);
    Ok(ash::util::read_spv(&mut cursor)?)
}

impl Context {
    pub fn create_shader_module(&self, source: &[u8]) -> Result<ShaderModule> {
        ShaderModule::from_spv_bytes(self.device.clone(), source)
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.destroy_shader_module(self.inner, None);
        }
    }
}

pub struct StagedShader {
    pub entry_point_name: CString,
    pub stage: vk::ShaderStageFlags,
    pub module: Arc<ShaderModule>,
}
