use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use anyhow::{anyhow, Context};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{Window, WindowBuilder, WindowId};
use avalanche_hlvk::{Semaphore, Surface, Swapchain};
use avalanche_utils::IdGenerator32;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct WindowHandle(u32);

#[derive(Copy, Clone, Debug)]
pub struct WindowState {
    pub(crate) is_surface_dirty: bool,
    pub(crate) is_pending_redraw: bool,
    pub(crate) extent: (u32, u32),
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            is_surface_dirty: true,
            is_pending_redraw: false,
            extent: (0, 0),
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
    pub(crate) event_loop: RwLock<EventLoopType>,
    pub(crate) windows: HashMap<WindowHandle, Arc<Window>>,
    pub(crate) window_states: HashMap<WindowHandle, RwLock<WindowState>>,
    pub(crate) window_surfaces: RwLock<HashMap<WindowHandle, Arc<Surface>>>,
    pub(crate) window_swapchains: RwLock<HashMap<WindowHandle, Arc<Swapchain>>>,
    pub(crate) id_generator: IdGenerator32,
    pub(crate) main_window_id: WindowHandle,
}

unsafe impl Sync for WindowManager {}
unsafe impl Send for WindowManager {}

// fn window_event_handler(event: Event<()>, target: &EventLoopWindowTarget<()>) {
// }

impl WindowManager {
    pub fn create_new_window(&mut self) -> anyhow::Result<WindowHandle> {
        let window = WindowBuilder::new()
            .with_title("Default Window")
            .build(self.event_loop.get_mut().unwrap())?;
        let new_id = WindowHandle(self.id_generator.next_id());
        let PhysicalSize { width, height } = window.inner_size();
        self.windows.insert(new_id, Arc::new(window));
        let mut state = WindowState::default();
        state.extent = (width, height);
        self.window_states.insert(new_id, RwLock::new(state));
        Ok(new_id)
    }

    pub fn create_main_window(&mut self) -> anyhow::Result<WindowHandle> {
        assert!(self.main_window_id.is_none(), "Trying to create main window in multiple times.");

        let handle = self.create_new_window()?;
        self.main_window_id = handle;
        Ok(handle)
    }

    pub fn handle_events(&self) {
        let status = self.event_loop.write().unwrap().pump_events(Some(Duration::from_secs_f64(0.33)), |event: Event<()>, target: &EventLoopWindowTarget<()>| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if let Ok(_) = self.notify_window_to_exit(window_id) => target.exit(),

                Event::WindowEvent {
                    event: WindowEvent::Resized(..),
                    window_id,
                } => {
                    let _ = self.notify_window_resized(window_id);
                },

                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    window_id,
                } => {
                    let _ = self.notify_window_redraw(window_id);
                },

                Event::AboutToWait => {
                },

                _ => (),
            }
        });
        if let PumpStatus::Exit(_exit_code) = status {
            panic!("Window event loop exited.");
        }
    }

    pub fn find_window_by_window_id(&self, window_id: WindowId) -> anyhow::Result<WindowHandle> {
        Ok(self.windows.iter().find(|(_handle, window)| window.id() == window_id).context("Failed to find window by id.")?.0.clone())
    }

    pub fn mark_surface_dirty(&mut self, window_handle: WindowHandle) -> anyhow::Result<()> {
        self.window_states.get_mut(&window_handle).context("Failed to find window with handle")?.write().unwrap().is_surface_dirty = true;

        Ok(())
    }

    pub fn mark_window_redraw(&self, window_handle: WindowHandle) -> anyhow::Result<()> {
        self.window_states.get(&window_handle).context("Failed to find window with handle")?.write().unwrap().is_pending_redraw = true;

        Ok(())
    }

    pub fn update_window_extent(&self, window_handle: WindowHandle, extent: (u32, u32)) -> anyhow::Result<()> {
        let mut proxy = self.window_states.get(&window_handle).context("Failed to find window with handle")?.write().unwrap();
        if proxy.extent != extent {
            proxy.is_surface_dirty = true;
            proxy.extent = extent;
        }

        Ok(())
    }

    pub fn set_window_surface(&self, window_handle: WindowHandle, surface: Arc<Surface>) -> anyhow::Result<()> {
        if !self.windows.contains_key(&window_handle) {
            return Err(anyhow!("Window handle is not maintained by window manager."));
        }

        self.window_surfaces.write().unwrap().insert(window_handle, surface);

        Ok(())
    }

    pub fn set_window_swapchain(&self, window_handle: WindowHandle, swapchain: Arc<Swapchain>) -> anyhow::Result<()> {
        if !self.windows.contains_key(&window_handle) {
            return Err(anyhow!("Window handle is not maintained by window manager."));
        }

        self.window_swapchains.write().unwrap().insert(window_handle, swapchain);

        Ok(())
    }

    pub fn update_window(&self, window_handle: WindowHandle) -> anyhow::Result<()> {
        if !self.windows.contains_key(&window_handle)
        {
            return Err(anyhow!("Window handle is not maintained by window manager."));
        }

        let mut recreate_surface = false;
        let mut execute_present = false;
        if let Some(state) = self.window_states.get(&window_handle) {
            let state = state.read().unwrap();
            execute_present = state.is_pending_redraw;
            recreate_surface = state.is_surface_dirty;
        }

        if !recreate_surface && !execute_present {
            // Nothing to be done
            return Ok(())
        }

        if let Some(_surface) = self.window_surfaces.read().unwrap().get(&window_handle)
            && let Some(swapchain) = self.window_swapchains.read().unwrap().get(&window_handle)
        {
            let next_image_info = swapchain.acquire_next_image_v2(Duration::new(1, 0))?;
            swapchain.queue_present(next_image_info.index, &[], );
            Ok(())
        }
        else {
            Err(anyhow!("Failed to find correct context of handle {window_handle}!"))
        }

    }

    pub fn update_all_windows(&self) {}
}
