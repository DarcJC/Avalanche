#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(trivial_bounds)]

pub mod event;

use std::cell::RefCell;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::{Component, EventReader, EventWriter, IntoSystemConfigs, IntoSystemSetConfigs, Query, Resource, SystemSet, World};
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

#[derive(Component)]
pub struct WindowComponent {
    pub id: WindowId,
    pub window: RwLock<Window>,
    pub surface: Option<Arc<Surface>>,
    pub swapchain: Option<RwLock<Swapchain>>,
    pub render_device: Option<Arc<Device>>,
}

impl WindowComponent {
    pub fn new(window: RwLock<Window>) -> Self {
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

    Ok(WindowComponent::new(RwLock::new(window)))
}

fn winit_event_poll_worker_system(world: &mut World) {
    #[cfg(feature = "trace")]
    let _span = bevy_utils::tracing::info_span!("poll winit event loop").entered();

    let window_manager = world.get_non_send_resource_mut::<WindowManager>().unwrap();
    let mut evt_window = None;
    let mut evt_wait = None;
    window_manager
        .event_loop
        .borrow_mut()
        .pump_events(
            Some(Duration::from_secs_f64(0.33)),
            |event, event_target| {
                    match event {
                        Event::WindowEvent {
                            event: WindowEvent::CloseRequested,
                            window_id: _window_id,
                        } => event_target.exit(),
                        Event::WindowEvent {
                            event: window_event,
                            window_id,
                        } => evt_window = Some(WinitWindowEvent {  window_event, window_id }),
                        Event::AboutToWait => evt_wait = Some(WindowEventLoopClearedEvent()),
                        _ => (),
                    }
                }
        );
    if let Some(evt) = evt_window {
        world.send_event(evt);
    }
    if let Some(evt) = evt_wait {
        world.send_event(evt);
    }
}

fn window_update_system(mut event_reader: EventReader<WinitWindowEvent>, mut event_writer: EventWriter<WindowResizedEvent>, windows: Query<&WindowComponent>) {
    #[cfg(feature = "trace")]
    let _span = bevy_utils::tracing::info_span!("handle window event").entered();

    event_reader.read().for_each(|evt| {
        if let Some(window) = windows
            .iter()
            .find(|i| i.window.read().unwrap().id() == evt.window_id) {
            match evt.window_event {
                // WindowEvent::Resized(extent) => {
                // },
                WindowEvent::RedrawRequested => {
                    let size = window.window.read().unwrap().inner_size();
                    event_writer.send(WindowResizedEvent { window_id: evt.window_id.clone(), new_size: (size.width, size.height) });
                },
                _ => ()
            }
        }
    });
}
