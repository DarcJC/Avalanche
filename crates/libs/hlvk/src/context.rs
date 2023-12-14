use std::sync::{Arc, Mutex};
use ash::Entry;
use gpu_allocator::AllocatorDebugSettings;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use log::info;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use avalanche_utils::{Version, VERSION_1_0};
use crate::{Device, DeviceFeatures, Instance, PhysicalDevice, Queue, QueueFamily, Surface};

pub struct Context {
    pub allocator: Arc<Mutex<Allocator>>,
    pub instance: Instance,
    pub physical_device: PhysicalDevice,
    pub device: Arc<Device>,
    pub graphics_queue: Queue,
    pub graphics_queue_family: QueueFamily,
    pub present_queue: Queue,
    pub present_queue_family: QueueFamily,
    /// main surface, other surface is keeping by [avalanche-window] crate
    pub surface: Surface,
    // TODO raytracing
    _entry: Entry,
}

pub struct ContextBuilder<'a> {
    window_handle: &'a dyn HasWindowHandle,
    display_handle: &'a dyn HasDisplayHandle,
    vulkan_version: Version,
    app_name: &'a str,
    required_device_extensions: &'a [&'a str],
    required_device_features: DeviceFeatures,
    /// Should we create raytracing context
    with_raytracing_context: bool,
}

impl<'a> ContextBuilder<'a> {
    pub fn new(
        window_handle: &'a dyn HasWindowHandle,
        display_handle: &'a dyn HasDisplayHandle,
    ) -> Self {
        Self {
            window_handle,
            display_handle,
            vulkan_version: VERSION_1_0,
            app_name: "",
            required_device_extensions: &[],
            required_device_features: Default::default(),
            with_raytracing_context: false,
        }
    }

    pub fn vulkan_version(self, vulkan_version: Version) -> Self {
        Self {
            vulkan_version,
            ..self
        }
    }

    pub fn app_name(self, app_name: &'a str) -> Self {
        Self {
            app_name,
            ..self
        }
    }

    pub fn required_device_extensions(self, required_extensions: &'a [&str]) -> Self {
        Self {
            required_device_extensions: required_extensions,
            ..self
        }
    }

    pub fn required_device_features(self, required_features: DeviceFeatures) -> Self {
        Self {
            required_device_features: required_features,
            ..self
        }
    }

    pub fn with_raytracing_context(self, with_raytracing_context: bool) -> Self {
        Self {
            with_raytracing_context,
            ..self
        }
    }

    pub fn build(self) -> anyhow::Result<Context> {
        Context::new(self)
    }
}

impl Context {
    fn new(
        ContextBuilder {
            window_handle,
            display_handle,
            vulkan_version,
            app_name,
            required_device_extensions,
            required_device_features,
            with_raytracing_context,
        }: ContextBuilder,
    ) -> anyhow::Result<Self> {
        let entry = unsafe { Entry::load()? };
        let mut instance = Instance::new(&entry, display_handle, vulkan_version, app_name)?;

        let surface = Surface::new(&entry, &instance, window_handle, display_handle)?;

        let physical_devices = instance.enumerate_physical_devices(&surface)?;
        let (physical_device, graphics_queue_family, present_queue_family) =
            select_suitable_physical_device(
                physical_devices,
                required_device_extensions,
                &required_device_features)?;
        info!("[Vulkan] Selected physical device: {:?}", physical_device.name);

        let queue_families = [graphics_queue_family, present_queue_family];
        let device = Arc::new(Device::new(
            &instance,
            &physical_device,
            &queue_families,
            required_device_extensions,
            &required_device_features,
        )?);
        let graphics_queue = device.get_queue(graphics_queue_family, 0);
        let present_queue = device.get_queue(present_queue_family, 0);

        let ray_tracing = with_raytracing_context.then(|| {
            // TODO raytracing
        });

        // TODO Command Pool

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.inner.clone(),
            device: device.inner.clone(),
            physical_device: physical_device.inner,
            debug_settings: AllocatorDebugSettings {
                log_allocations : true,
                log_frees: true,
                log_leaks_on_shutdown: true,
                log_memory_information: true,
                log_stack_traces: true,
                ..Default::default()
            },
            buffer_device_address: required_device_features.buffer_device_address,
            allocation_sizes: Default::default(),
        })?;

        Ok(Self {
            allocator: Arc::new(Mutex::new(allocator)),
            instance,
            physical_device,
            device,
            graphics_queue,
            graphics_queue_family,
            present_queue,
            present_queue_family,
            surface,
            _entry: entry,
        })
    }

}

fn select_suitable_physical_device(
    devices: &[PhysicalDevice],
    required_extensions: &[&str],
    required_device_features: &DeviceFeatures,
) -> anyhow::Result<(PhysicalDevice, QueueFamily, QueueFamily)> {
    let mut graphics = None;
    let mut present = None;

    let device = devices
        .iter()
        .find(|device| {
            for family in device.queue_families.iter().filter(|f| f.has_queues()) {
                if family.supports_graphics()
                    && family.supports_compute()
                    && family.supports_timestamp_queries()
                    && graphics.is_none() {
                    graphics = Some(*family);
                }

                if family.supports_present() && present.is_none() {
                    present = Some(*family);
                }

                if graphics.is_some() && present.is_some() {
                    break;
                }
            }

            let extension_support = device.supports_extensions(required_extensions);

            graphics.is_some()
                && present.is_some()
                && extension_support
                && !device.supported_surface_formats.is_empty()
                && !device.supported_present_modes.is_empty()
                && device
                .supported_device_features
                .is_compatible_with(required_device_features)
        })
        .ok_or_else(|| anyhow::anyhow!("Could not find a suitable device"))?;

    Ok((device.clone(), graphics.unwrap(), present.unwrap()))
}

impl Context {
    pub fn device_wait_idle(&self) -> anyhow::Result<()> {
        unsafe { self.device.inner.device_wait_idle()? };

        Ok(())
    }
}
