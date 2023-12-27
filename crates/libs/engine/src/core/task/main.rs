use std::io::Write;
use std::ops::Deref;
use std::sync::RwLock;
use std::time::Duration;
use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder, PostStartup, PreStartup, Update};
use bevy_ecs::prelude::{EventReader, IntoSystemConfigs, Query, Res, Resource, World};
use chrono::Local;
use ash::vk;
use env_logger::Env;
use log::warn;
use avalanche_hlvk::{CommandBuffer, CommandPool, Context, ContextBuilder, DeviceFeatures, Swapchain};
use avalanche_window::{new_window_component, WindowComponent, WindowManager, WindowSystemPlugin, WindowSystemSet};
use avalanche_window::event::{WindowEventLoopClearedEvent, WindowResizedEvent};

pub struct EngineContextSetupPlugin;

#[derive(Resource)]
pub struct RenderingContext {
    pub context: Context,
    pub command_pool: CommandPool,
    pub swapchain_command_buffer: Vec<CommandBuffer>,
}

/// Exclusive system to force schedule in main thread
fn start_rendering_system_with_window(world: &mut World) {
    let window_manager = world.get_non_send_resource::<WindowManager>().unwrap();
    let mut first_window_component = new_window_component(window_manager.event_loop.borrow_mut().deref()).unwrap();
    let window_ref = first_window_component.window.write().unwrap();

    let vulkan_context = ContextBuilder::new(window_ref.deref(), window_ref.deref())
        .required_device_features(DeviceFeatures::full())
        .with_raytracing_context(false)
        .app_name("Avalanche Engine")
        .required_device_extensions(vec!["VK_KHR_swapchain"].deref())
        .vulkan_version(avalanche_utils::VERSION_1_3)
        .build().unwrap();

    let command_pool = vulkan_context.create_command_pool(
        vulkan_context.graphics_queue_family,
        Some(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
    ).unwrap();

    let swapchain = Swapchain::new(
        &vulkan_context,
        window_ref.inner_size().width,
        window_ref.inner_size().height,
    ).unwrap();

    // TODO raytracing

    drop(window_ref);

    let command_buffers = command_pool.allocate_command_buffers(vk::CommandBufferLevel::PRIMARY, swapchain.images.len() as _).unwrap();

    first_window_component.render_device = Some(vulkan_context.device.clone());
    first_window_component.surface = Some(vulkan_context.surface.clone());
    first_window_component.swapchain = Some(RwLock::new(swapchain));

    let context = RenderingContext {
        context: vulkan_context,
        command_pool,
        swapchain_command_buffer: command_buffers,
    };

    world.insert_resource(context);
    world.spawn(first_window_component);
}

fn window_resize_handler(mut event_reader: EventReader<WindowResizedEvent>, windows: Query<&WindowComponent>, rendering_context: Res<RenderingContext>) {
    event_reader.read().for_each(|evt|  {
        if let Some(window) = windows
            .iter()
            .find(|i| i.window.read().unwrap().id() == evt.window_id)  {
            let res = window.swapchain.as_ref().unwrap().write().unwrap().resize(&rendering_context.context, evt.new_size.0, evt.new_size.1);
            if let Err(err) = res {
                warn!("[Window] Failed to recreate swapchain for window: {err}");
            }
        }
    })
}

fn window_event_loop_cleared(mut event_reader: EventReader<WindowEventLoopClearedEvent>, windows: Query<&WindowComponent>, rendering_context: Res<RenderingContext>) {
    if event_reader.read().is_empty() {
        return;
    }
    windows
        .iter()
        .for_each(|window| {
            let mut swapchain = window.swapchain.as_ref().unwrap().write().unwrap();
            // TODO: catch exception
            let next_image = swapchain.acquire_next_image(Duration::from_secs_f64(0.33), None).unwrap();
            swapchain.queue_present(next_image.index, &[], &rendering_context.context.present_queue).unwrap();
        });
}

impl Plugin for EngineContextSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, start_rendering_system_with_window);
        app.add_systems(Update, (
            window_resize_handler.after(WindowSystemSet::Update),
            window_event_loop_cleared.after(window_resize_handler).after(WindowSystemSet::Update),
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
            .add(WindowSystemPlugin)
            .add(EngineContextSetupPlugin)
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
