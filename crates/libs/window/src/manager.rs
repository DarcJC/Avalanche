use std::collections::HashMap;
use std::sync::{Arc};
use std::time::Duration;
use anyhow::Context;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{Window, WindowBuilder, WindowId};
use avalanche_utils::IdGenerator32;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct WindowHandle(u32);

#[derive(Copy, Clone, Debug)]
pub struct WindowState {
    pub(crate) is_surface_dirty: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            is_surface_dirty: true,
        }
    }
}

impl WindowHandle {
    pub fn none() -> Self {
        Self(IdGenerator32::none())
    }

    pub fn is_none(self) -> bool {
        self == Self::none()
    }
}

pub(crate) type EventLoopType = EventLoop<()>;

pub struct WindowManager {
    pub(crate) event_loop: EventLoopType,
    pub(crate) windows: HashMap<WindowHandle, Arc<Window>>,
    pub(crate) window_states: HashMap<WindowHandle, Arc<WindowState>>,
    pub(crate) id_generator: IdGenerator32,
    pub(crate) main_window_id: WindowHandle,
}

unsafe impl Sync for WindowManager {}
unsafe impl Send for WindowManager {}

fn window_event_handler(event: Event<()>, target: &EventLoopWindowTarget<()>) {
    match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if let Ok(_) = WindowManager::notify_window_to_exit(window_id) => target.exit(),

        Event::WindowEvent {
            event: WindowEvent::Resized(..),
            window_id: _window_id,
        } => {
            // TODO: notify window size changed
        }

        Event::AboutToWait => {
        },

        _ => (),
    }
}

impl WindowManager {
    pub fn create_new_window(&mut self) -> anyhow::Result<WindowHandle> {
        let window = WindowBuilder::new()
            .with_title("Default Window")
            .build(&self.event_loop)?;
        let new_id = WindowHandle(self.id_generator.next_id());
        self.windows.insert(new_id, Arc::new(window));
        self.window_states.insert(new_id, Arc::new(WindowState::default()));
        Ok(new_id)
    }

    pub fn create_main_window(&mut self) -> anyhow::Result<WindowHandle> {
        assert!(self.main_window_id.is_none(), "Trying to create main window in multiple times.");

        let handle = self.create_new_window()?;
        self.main_window_id = handle;
        Ok(handle)
    }

    pub fn handle_events(&mut self) {
        let status = self.event_loop.pump_events(Some(Duration::from_secs_f64(0.33)), window_event_handler);
        if let PumpStatus::Exit(_exit_code) = status {
            panic!("Window event loop exited.");
        }
    }

    pub fn find_window_by_window_id(&self, window_id: WindowId) -> anyhow::Result<WindowHandle> {
        Ok(self.windows.iter().find(|(_handle, window)| window.id() == window_id).context("Failed to find window by id.")?.0.clone())
    }
}
