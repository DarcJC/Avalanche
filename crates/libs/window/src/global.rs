use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{anyhow, Context};
use once_cell::sync::Lazy;
use winit::window::{Window, WindowId};
use avalanche_utils::IdGenerator32;
use crate::{WindowHandle, WindowManager};
use crate::manager::EventLoopType;

fn create_window_manager() -> Arc<Mutex<WindowManager>> {
    Arc::new(Mutex::new(WindowManager {
        event_loop: EventLoopType::new().unwrap(),
        windows: HashMap::new(),
        window_states: HashMap::new(),
        id_generator: IdGenerator32::new(),
        main_window_id: WindowHandle::none(),
    }))
}

static WINDOW_MANAGER: Lazy<Arc<Mutex<WindowManager>>> = Lazy::new(create_window_manager);

pub fn get_window_manager() -> Arc<Mutex<WindowManager>> {
    WINDOW_MANAGER.clone()
}

impl WindowHandle {
    pub fn get_raw_window(&self) -> anyhow::Result<Arc<Window>> {
        Ok(get_window_manager().lock().unwrap().windows.get(self).context("Window not found by handle.")?.clone())
    }

    pub fn get_raw_window_id(&self) -> anyhow::Result<WindowId> {
        Ok(self.get_raw_window()?.id())
    }
}

impl WindowManager {
    pub fn notify_window_to_exit(window_id: WindowId) -> anyhow::Result<()> {
        if let Ok(_window_handle) = get_window_manager().lock().unwrap().find_window_by_window_id(window_id) {
            // TODO: impl WindowCloseEvent
            Ok(())
        } else {
            Err(anyhow!("Failed to find window."))
        }
    }

    pub fn notify_window_resized(_window_id: WindowId) {
    }
}
