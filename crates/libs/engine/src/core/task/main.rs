use std::cell::RefCell;
use std::io::Write;
use std::ops::Deref;
use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder, PostStartup, PreStartup, Update};
use bevy_ecs::prelude::{IntoSystemConfigs, Resource, World};
use chrono::Local;
use ash::vk;
use env_logger::Env;
use avalanche_hlvk::{CommandBuffer, CommandPool, Context, ContextBuilder, DeviceFeatures, Swapchain};
use avalanche_window::get_window_manager;

pub struct WindowSystemTaskPlugin;

#[derive(Resource)]
pub struct RenderingContext {
    pub context: Context,
    pub command_pool: CommandPool,
    pub swapchain_command_buffer: Vec<CommandBuffer>,
}

/// Exclusive system to force schedule in main thread
fn start_rendering_system_with_window(world: &mut World) {
    let binding = get_window_manager();
    let mut window_manager = binding.write().unwrap();
    let handle = window_manager.create_main_window().unwrap();
    let window = window_manager.get_raw_window(handle).unwrap();

    let vulkan_context = ContextBuilder::new(window.deref(), window.deref())
        .required_device_features(DeviceFeatures::full())
        .with_raytracing_context(false)
        .app_name("Avalanche Engine")
        .required_device_extensions(vec!["VK_KHR_swapchain"].deref())
        .vulkan_version(avalanche_utils::VERSION_1_3)
        .build().unwrap();

    window_manager.set_window_surface(handle, vulkan_context.surface.clone()).unwrap();

    drop(window_manager);
    drop(binding);

    let command_pool = vulkan_context.create_command_pool(
        vulkan_context.graphics_queue_family,
        Some(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
    ).unwrap();

    let swapchain = Swapchain::new(
        &vulkan_context,
        window.inner_size().width,
        window.inner_size().height,
    ).unwrap();

    // TODO raytracing

    let command_buffers = command_pool.allocate_command_buffers(vk::CommandBufferLevel::PRIMARY, swapchain.images.len() as _).unwrap();

    get_window_manager().write().unwrap().set_window_swapchain(handle, RefCell::new(swapchain)).expect("Failed to add swapchain to window manager.");

    let context = RenderingContext {
        context: vulkan_context,
        command_pool,
        swapchain_command_buffer: command_buffers,
    };

    world.insert_resource(context);
}

fn poll_window_events(_world: &mut World) {
    get_window_manager().write().unwrap().handle_events();
}

fn window_system_tick(world: &mut World) {
    let context = world.get_resource::<RenderingContext>().unwrap();
    get_window_manager().write().unwrap().update_all_windows(&context.context).unwrap();
}

impl Plugin for WindowSystemTaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, start_rendering_system_with_window.before(poll_window_events));
        app.add_systems(Update, (
            poll_window_events,
            window_system_tick.after(poll_window_events)
        ));
    }
}

pub struct LogSystemPlugin;

fn init_env_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {} - ({}:{})",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args(),
                record.module_path().unwrap_or("unknown"),
                record.line().map_or(-1, |value| value as i32)
            )
        })
        .init();
}

impl Plugin for LogSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_env_logger);
    }
}

pub struct MainTaskPluginGroup;

impl PluginGroup for MainTaskPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(LogSystemPlugin)
            .add(WindowSystemTaskPlugin)
    }
}

pub struct SchedulerMinimalPlugins;

impl PluginGroup for SchedulerMinimalPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bevy_app::MainSchedulePlugin)
            .add(bevy_core::TaskPoolPlugin::default())
            .add(bevy_core::TypeRegistrationPlugin)
            .add(bevy_core::FrameCountPlugin)
            .add(bevy_time::TimePlugin)
            .add(bevy_app::ScheduleRunnerPlugin::default())
    }
}
