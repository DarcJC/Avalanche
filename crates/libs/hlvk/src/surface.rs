
use ash::{vk, extensions::khr::Surface as AshSurface, Entry};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use crate::Instance;

#[derive(Clone)]
pub struct Surface {
    pub(crate) inner: AshSurface,
    pub surface_khr: vk::SurfaceKHR,
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

        Ok(Self { inner, surface_khr })
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.inner.destroy_surface(self.surface_khr, None);
        }
    }
}
