use ash::extensions::ext::DebugUtils;
use ash::{Entry, Instance as AshInstance, vk};
use ash::vk::PhysicalDevice;
use raw_window_handle::HasDisplayHandle;
use avalanche_utils::Version;
use crate::util::IntoAshVersion;

#[derive(Clone)]
pub struct Instance {
    pub(crate) inner: AshInstance,
    debug_utils: DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    physical_devices: Vec<PhysicalDevice>,
}

impl Instance {
    pub(crate) fn new(entry: &Entry, display_handle: &dyn HasDisplayHandle, api_version: Version) {
        let app_info = vk::ApplicationInfo::builder()
            .api_version(api_version.into_version());
    }
}
