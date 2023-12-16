use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{anyhow, Context};
use once_cell::sync::Lazy;
use winit::dpi::PhysicalSize;
use winit::window::{Window, WindowId};
use avalanche_utils::IdGenerator32;
use crate::{WindowHandle, WindowManager};
use crate::manager::EventLoopType;

fn create_window_manager() -> Arc<RwLock<WindowManager>> {
    Arc::new(RwLock::new(WindowManager {
        event_loop: RwLock::new(EventLoopType::new().unwrap()),
        windows: HashMap::new(),
        window_states: HashMap::new(),
        id_generator: IdGenerator32::new(),
        main_window_id: WindowHandle::none(),
    }))
}

static WINDOW_MANAGER: Lazy<Arc<RwLock<WindowManager>>> = Lazy::new(create_window_manager);

pub fn get_window_manager() -> Arc<RwLock<WindowManager>> {
    WINDOW_MANAGER.clone()
}

impl WindowManager {
    pub fn notify_window_to_exit(&self, window_id: WindowId) -> anyhow::Result<()> {
        if let Ok(_window_handle) = self.find_window_by_window_id(window_id) {
            // TODO: Notify to exit
            Ok(())
        } else {
            Err(anyhow!("Failed to find window."))
        }
    }

    pub fn notify_window_resized(&self, window_id: WindowId) -> anyhow::Result<()> {
        if let Ok(window_handle) = self.find_window_by_window_id(window_id) {
            let PhysicalSize { width, height } = self.get_raw_window(window_handle)?.inner_size();
            self.update_window_extent(window_handle, (width, height))?;

            Ok(())
        } else {
            Err(anyhow!("Failed to find window."))
        }
    }

    pub fn notify_window_redraw(&mut self, window_id: WindowId) -> anyhow::Result<()> {
        if let Ok(window_handle) = self.find_window_by_window_id(window_id) {
            self.mark_window_redraw(window_handle)?;

            Ok(())
        } else {
            Err(anyhow!("Failed to find window."))
        }
    }

    pub fn get_raw_window(&self, window_handle: WindowHandle) -> anyhow::Result<Arc<Window>> {
        Ok(self.windows.get(&window_handle).context("Window not found by handle.")?.clone())
    }

    pub fn get_raw_window_id(&self, window_handle: WindowHandle) -> anyhow::Result<WindowId> {
        Ok(self.get_raw_window(window_handle)?.id())
    }
}
