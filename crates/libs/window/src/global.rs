use std::collections::HashMap;
use std::sync::Arc;
use async_std::sync::RwLock;
use once_cell::sync::Lazy;
use winit::window::{Window, WindowId};
use avalanche_utils::{IdGenerator32, static_accessors};
use crate::{WindowHandle, WindowManager};
use crate::manager::EventLoopType;

fn create_window_manager() -> RwLock<WindowManager> {
    RwLock::new(WindowManager {
        event_loop: EventLoopType::new().unwrap(),
        windows: HashMap::new(),
        window_states: HashMap::new(),
        id_generator: IdGenerator32::new(),
        main_window_id: WindowHandle::none(),
    })
}

static WINDOW_MANAGER: Lazy<RwLock<WindowManager>> = Lazy::new(create_window_manager);

static_accessors! {
    'static pub WINDOW_MANAGER, get_window_manager, get_window_manager_mut, get_window_manager_sync, get_window_manager_mut_sync, WindowManager
}

impl WindowHandle {
    pub fn get_raw_window(&self) -> Arc<Window> {
        get_window_manager_sync().windows.get(self).unwrap().clone()
    }

    pub fn get_raw_window_id(&self) -> WindowId {
        self.get_raw_window().id()
    }
}

impl WindowManager {
    pub fn notify_window_to_exit(window_id: WindowId) -> bool {
        if let Ok(_window_handle) = get_window_manager_sync().find_window_by_window_id(window_id) {
            // TODO: impl WindowCloseEvent
            true
        } else {
            true
        }
    }

    pub fn notify_window_resized(_window_id: WindowId) {
    }
}
