#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(trivial_bounds)]

use std::cell::RefCell;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::{Component, Resource, World};
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowBuilder};
use avalanche_hlvk::{Device, Surface, Swapchain};
use avalanche_utils::ID_GENERATOR_32_STATIC;


pub struct WindowSystemPlugin;

impl Plugin for WindowSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<WindowManager>();
        app.add_systems(Update, (
            window_update_system,
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
    let window_manager = world.get_non_send_resource_mut::<WindowManager>().unwrap();
    window_manager
        .event_loop
        .borrow_mut()
        .pump_events(
            Some(Duration::from_secs_f64(0.33)),
            |event, event_target| {
                }
        );
}

fn window_update_system(world: &mut World) {
}
