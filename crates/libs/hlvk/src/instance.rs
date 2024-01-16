use std::ffi::{c_void, CStr, CString};
use ash::extensions::ext::DebugUtils;
use ash::{Entry, Instance as AshInstance, vk};
use log::{debug, error, info, warn};
use raw_window_handle::HasDisplayHandle;
use avalanche_utils::{CURRENT_APPLICATION_NAME, CURRENT_APPLICATION_VERSION, Version};
use crate::{PhysicalDevice, Surface};
use crate::util::IntoAshVersion;

pub struct Instance {
    pub(crate) inner: AshInstance,
    debug_utils: Option<DebugUtils>,
    debug_utils_messenger: Option<vk::DebugUtilsMessengerEXT>,
    physical_devices: Vec<PhysicalDevice>,
}

impl Instance {
    pub(crate) fn new(entry: &Entry, display_handle: &dyn HasDisplayHandle, api_version: Version, app_name: &str) -> anyhow::Result<Self> {
        let engine_name = CString::new(CURRENT_APPLICATION_NAME)?;
        let app_name = CString::new(app_name)?;

        let app_info = vk::ApplicationInfo::builder()
            .api_version(api_version.into_version())
            .engine_version(CURRENT_APPLICATION_VERSION.into_version())
            .engine_name(engine_name.as_c_str())
            .application_name(app_name.as_c_str())
            .build();

        let is_debug = if cfg!(debug_assertions){
            true
        }
        else {
            std::env::var("PROFILE").unwrap_or(String::new()).eq("debug")
        };

        let mut extension_names = ash_window::enumerate_required_extensions(display_handle.display_handle()?.as_raw())?.to_vec();
        if is_debug {
            extension_names.push(DebugUtils::name().as_ptr());
        }

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .build();

        let inner = unsafe { entry.create_instance(&instance_create_info, None)? };

        // Enable debug layer
        Ok(if cfg!(feature = "validation") {
            let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .flags(vk::DebugUtilsMessengerCreateFlagsEXT::empty())
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                )
                .pfn_user_callback(Some(vulkan_debug_callback))
                .build();

            let debug_utils = DebugUtils::new(entry, &inner);
            let debug_utils_messenger = unsafe { debug_utils.create_debug_utils_messenger(&create_info, None)? };

            Self {
                inner,
                debug_utils: Some(debug_utils),
                debug_utils_messenger: Some(debug_utils_messenger),
                physical_devices: vec![],
            }
        }
        else {
            Self {
                inner,
                debug_utils: None,
                debug_utils_messenger: None,
                physical_devices: vec![],
            }
        })
    }

    pub(crate) fn enumerate_physical_devices(
        &mut self,
        surface: &Surface,
    ) -> anyhow::Result<&[PhysicalDevice]> {
        if self.physical_devices.is_empty() {
            let physical_devices = unsafe { self.inner.enumerate_physical_devices()? };

            let mut physical_devices = physical_devices
                .into_iter()
                .map(|physical_device| PhysicalDevice::new(&self.inner, surface, physical_device))
                .collect::<anyhow::Result<Vec<_>>>()?;

            physical_devices.sort_by_key(|physical_device| match physical_device.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                _ => 2,
            });

            self.physical_devices = physical_devices;
        }

        Ok(&self.physical_devices)
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    severity_flag: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_flag: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    use vk::DebugUtilsMessageSeverityFlagsEXT as SeverityFlag;

    let message = CStr::from_ptr((*p_callback_data).p_message);
    match severity_flag {
        SeverityFlag::VERBOSE => debug!("[Vulkan][{:?}] {:?}", type_flag, message),
        SeverityFlag::INFO => info!("[Vulkan][{:?}] {:?}", type_flag, message),
        SeverityFlag::WARNING => warn!("[Vulkan][{:?}] {:?}", type_flag, message),
        _ => error!("[Vulkan][{:?}] {:?}", type_flag, message),
    }

    vk::FALSE
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            if let Some(debug_utils) = self.debug_utils.take() && let Some(debug_utils_messenger) = self.debug_utils_messenger.take() {
                debug_utils.destroy_debug_utils_messenger(debug_utils_messenger, None);
            }
            self.inner.destroy_instance(None);
        }
    }
}
