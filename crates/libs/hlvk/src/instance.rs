use std::ffi::{c_void, CStr, CString};
use ash::extensions::ext::DebugUtils;
use ash::{Entry, Instance as AshInstance, vk};
use ash::vk::PhysicalDevice;
use log::{debug, error, info, warn};
use raw_window_handle::HasDisplayHandle;
use avalanche_utils::{CURRENT_APPLICATION_NAME, CURRENT_APPLICATION_VERSION, Version};
use crate::util::IntoAshVersion;

#[derive(Clone)]
pub struct Instance {
    pub(crate) inner: AshInstance,
    debug_utils: Option<DebugUtils>,
    debug_utils_messenger: Option<vk::DebugUtilsMessengerEXT>,
    physical_devices: Vec<PhysicalDevice>,
}

impl Instance {
    pub(crate) fn new(entry: &Entry, display_handle: &dyn HasDisplayHandle, api_version: Version) -> anyhow::Result<Self> {
        let app_name = CString::new(CURRENT_APPLICATION_NAME)?;

        let app_info = vk::ApplicationInfo::builder()
            .api_version(api_version.into_version())
            .application_version(CURRENT_APPLICATION_VERSION.into_version())
            .application_name(app_name.as_c_str())
            .build();

        let is_debug = std::env::var("PROFILE")?.eq("debug");

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
        Ok(if is_debug {
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
