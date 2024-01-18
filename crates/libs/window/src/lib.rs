#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(trivial_bounds)]

pub mod event;

use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::{Component, EventReader, EventWriter, IntoSystemConfigs, IntoSystemSetConfigs, Mut, Query, Resource, SystemSet, World};
use raw_window_handle::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowBuilder};
use avalanche_hlvk::{Device, Surface, Swapchain};
use avalanche_utils::ID_GENERATOR_32_STATIC;
use crate::event::{WindowEventLoopClearedEvent, WindowResizedEvent, WinitWindowEvent};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowSystemSet {
    EventLoop,
    Update,
}

pub struct WindowSystemPlugin;

impl Plugin for WindowSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<WindowManager>();
        app.configure_sets(Update, (WindowSystemSet::EventLoop, WindowSystemSet::Update).chain());
        app.add_event::<WinitWindowEvent>();
        app.add_event::<WindowResizedEvent>();
        app.add_event::<WindowEventLoopClearedEvent>();
        app.add_systems(Update, (
            winit_event_poll_worker_system
                .before(window_update_system)
                .in_set(WindowSystemSet::EventLoop)
            ,
            window_update_system
                .in_set(WindowSystemSet::Update),
        ));
    }
}

#[derive(Resource)]
pub struct WindowManager {
    pub event_loop: RefCell<EventLoop<()>>,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self {
            event_loop: RefCell::new(
                EventLoopBuilder::default()
                    .build()
                    .unwrap()
            ),
        }
    }
}

#[derive(Component, Clone)]
pub struct WindowId(u32);

#[derive(Component, Clone)]
pub struct WindowComponent {
    pub id: WindowId,
    pub window: Arc<Window>,
    pub surface: Option<Arc<Surface>>,
    pub swapchain: Option<Arc<Swapchain>>,
    pub render_device: Option<Arc<Device>>,
}

impl WindowComponent {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            id: WindowId(ID_GENERATOR_32_STATIC.next_id()),
            window,
            surface: None,
            swapchain: None,
            render_device: None,
        }
    }
}

pub fn new_window_component(event_loop: &EventLoop<()>) -> anyhow::Result<WindowComponent> {
    let window = WindowBuilder::default()
        .with_title("[Avalanche] Default Title")
        .build(event_loop)?;

    Ok(WindowComponent::new(Arc::new(window)))
}

fn winit_event_poll_worker_system(mut window_manager: Mut<WindowManager>, mut window_event: EventWriter<WinitWindowEvent>) {
    #[cfg(feature = "trace")]
    let _span = bevy_utils::tracing::info_span!("poll winit event loop").entered();

    let mut evt_window = None;
    window_manager
        .event_loop
        .borrow_mut()
        .pump_events(
            Some(Duration::ZERO),
            |event, event_target| {
                    match event {
                        Event::WindowEvent {
                            event: WindowEvent::CloseRequested,
                            window_id: _window_id,
                        } => event_target.exit(),
                        Event::WindowEvent {
                            event: window_event,
                            window_id,
                        } => evt_window = window_event(WinitWindowEvent {  window_event, window_id }),
                        _ => (),
                    }
                }
        );
}

fn window_update_system(mut event_reader: EventReader<WinitWindowEvent>, mut event_writer: EventWriter<WindowResizedEvent>, windows: Query<&WindowComponent>) {
    #[cfg(feature = "trace")]
    let _span = bevy_utils::tracing::info_span!("handle window event").entered();

    event_reader.read().for_each(|evt| {
        if let Some(window) = windows
            .iter()
            .find(|i| i.window.id() == evt.window_id) {
            match evt.window_event {
                // WindowEvent::Resized(extent) => {
                // },
                WindowEvent::RedrawRequested => {
                    let size = window.window.inner_size();
                    event_writer.send(WindowResizedEvent { window_id: evt.window_id.clone(), new_size: (size.width, size.height) });
                },
                _ => ()
            }
        }
    });
}


/// ## SAFETY
/// Use this wrapper in main thread.
/// Or just support PC platform to using multiple thread
pub struct HandleWrapper {
    window_handle: RawWindowHandle,
    display_handle: RawDisplayHandle,
}

impl From<Window> for HandleWrapper {
    fn from(value: Window) -> Self {
        Self {
            window_handle: value.window_handle().unwrap().as_raw(),
            display_handle: value.display_handle().unwrap().as_raw(),
        }
    }
}

impl HasDisplayHandle for HandleWrapper {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe { Ok(DisplayHandle::borrow_raw(self.display_handle)) }
    }
}

impl HasWindowHandle for HandleWrapper {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        unsafe { Ok(WindowHandle::borrow_raw(self.window_handle)) }
    }
}

unsafe impl Sync for HandleWrapper {}
unsafe impl Send for HandleWrapper {}
