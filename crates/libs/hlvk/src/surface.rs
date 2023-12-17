
use ash::{vk, extensions::khr::Surface as AshSurface, Entry};
use log::debug;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use crate::Instance;

pub struct Surface {
    pub(crate) inner: AshSurface,
    pub surface_khr: vk::SurfaceKHR,
    pub is_main_surface: bool,
}

impl Surface {
    pub(crate) fn new(
        entry: &Entry,
        instance: &Instance,
        window_handle: &dyn HasWindowHandle,
        display_handle: &dyn HasDisplayHandle,
    ) -> anyhow::Result<Self> {
        let inner = AshSurface::new(entry, &instance.inner);
        let surface_khr = unsafe {
            ash_window::create_surface(
                entry,
                &instance.inner,
                display_handle.display_handle()?.as_raw(),
                window_handle.window_handle()?.as_raw(),
                None,
            )?
        };

        Ok(Self { inner, surface_khr, is_main_surface: false })
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        if self.is_main_surface {
            debug!("[Vulkan] Trying to destroy main surface!");
        }
        unsafe {
            self.inner.destroy_surface(self.surface_khr, None);
        }
    }
}
