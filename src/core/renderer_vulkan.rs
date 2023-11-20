use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use ash::vk;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use crate::ash_window;
use crate::core::event_loop::EventLoopManager;
use crate::core::renderer_trait::Renderer;

pub struct VulkanRenderer {
    entry: ash::Entry,
    instance: ash::Instance,
    event_loop: Option<winit::event_loop::EventLoop<()>>,
    physical_device: Option<vk::PhysicalDevice>,
    logical_device: Option<ash::Device>,
    surfaces: HashMap<winit::window::WindowId, vk::SurfaceKHR>,
}

impl VulkanRenderer {
    fn rate_device(&self, device: &vk::PhysicalDevice) -> i8 {
        let properties = unsafe { self.instance.get_physical_device_properties(*device) };

        let queue_families = unsafe { self.instance.get_physical_device_queue_family_properties(*device) };

        let supports_graphics = queue_families.iter().any(|q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS));
        let supports_compute = queue_families.iter().any(|q| q.queue_flags.contains(vk::QueueFlags::COMPUTE));

        // discrete gpu and supports compute&graphics
        let mut result = 0;
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            result += 2;
        }
        if !supports_graphics || !supports_compute{
            result -= 4;
        }
        result
    }

    fn select_physical_device(&self) -> vk::PhysicalDevice {
        let devices = unsafe { self.instance.enumerate_physical_devices().expect("Failed to enumerate Physical Device.") };

        let mut max_score = 0;
        let mut best_device = None;

        for device in devices {
            let score = self.rate_device(&device);
            if score > max_score {
                max_score = score;
                best_device = Some(device);
            }
        }

        best_device.expect("No suitable physical device found.")
    }

    fn create_logical_device(&self) -> ash::Device {
        // Check queue families
        let queue_families = unsafe {
            self.instance.get_physical_device_queue_family_properties(self.physical_device.unwrap())
        };

        // Find queue families that supporting graphics commands
        let graphics_queue_family_index = queue_families
            .iter()
            .enumerate()
            .find(|(_index, queue_family_prop)| queue_family_prop.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|(index, _)| index as u32)
            .expect("No suitable graphics queue family found.");

        // Find queue families that supporting graphics commands
        let compute_queue_family_index = queue_families
            .iter()
            .enumerate()
            .find(|(_index, queue_family_prop)| queue_family_prop.queue_flags.contains(vk::QueueFlags::COMPUTE))
            .map(|(index, _)| index as u32)
            .expect("No suitable compute queue family found.");

        // Fill queue create info
        let graphics_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .queue_priorities(&[1.0])
            .build();

        let compute_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(compute_queue_family_index)
            .queue_priorities(&[1.0])
            .build();

        // Fill device create info
        let device_create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&VulkanRenderer::get_extension_names(&self.entry))
            .queue_create_infos(&[graphics_queue_create_info, compute_queue_create_info])
            .build();

        unsafe {
            self.instance.create_device(self.physical_device.unwrap(), &device_create_info, None).expect("Failed to create logical device.")
        }
    }

    fn get_extension_names(entry: &ash::Entry) -> Vec<*const c_char> {
        let mut extension_names = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::Swapchain::name().as_ptr(),
            ash::extensions::khr::WaylandSurface::name().as_ptr(),
            ash::extensions::khr::XlibSurface::name().as_ptr(),
            ash::extensions::khr::XcbSurface::name().as_ptr(),
            ash::extensions::khr::AndroidSurface::name().as_ptr(),
            ash::extensions::ext::MetalSurface::name().as_ptr(),
            vk::KhrPortabilityEnumerationFn::name().as_ptr(),
            vk::KhrGetPhysicalDeviceProperties2Fn::name().as_ptr(),
            vk::KhrWin32SurfaceFn::name().as_ptr(),
        ];

        let available_extensions = entry.enumerate_instance_extension_properties(None).expect("Failed to enumerate instance extension properties.");

        extension_names.retain(|&ext_name| {
            let ext_name_cstr = unsafe { CStr::from_ptr(ext_name) };
            available_extensions.iter().any(|ext| {
                let available_ext_name_cstr = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                ext_name_cstr == available_ext_name_cstr
            })
        });

        extension_names
    }
}

impl Renderer for VulkanRenderer {
    fn new() -> Self {
        // Load the Vulkan library.
        let entry = unsafe { ash::Entry::load() }.unwrap();

        // Define the application info.
        let app_name = CString::new("Hello Vulkan").unwrap();
        let engine_name = CString::new("No Engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&engine_name)
            .engine_version(0)
            .api_version(vk::API_VERSION_1_3);

        // Define the instance create info.
        let extension_names = VulkanRenderer::get_extension_names(&entry);
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .build();

        // Create the instance.
        let instance = unsafe { entry.create_instance(&create_info, None).unwrap() };

        VulkanRenderer {
            entry,
            instance,
            event_loop: None,
            physical_device: None,
            logical_device: None,
            surfaces: HashMap::new(),
        }
    }

    fn create_window(&mut self, event_loop: &mut EventLoopManager, title: &str, width: u32, height: u32) -> winit::window::Window {
        let event_loop = &mut event_loop.window_event_loop;

        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .build(event_loop)
            .unwrap();

        let surface = unsafe {
            ash_window::create_surface(&self.entry, &self.instance, window.display_handle().unwrap().as_raw(), window.window_handle().unwrap().as_raw(), None).expect("Failed to create surface for window.")
        };
        self.surfaces.insert(window.id(), surface);

        window
    }

    fn list_physical_devices(&self) {
        // Enumerate the physical devices.
        let physical_devices = unsafe { self.instance.enumerate_physical_devices().unwrap() };

        // Print out the physical devices.
        for (i, physical_device) in physical_devices.iter().enumerate() {
            let properties = unsafe { self.instance.get_physical_device_properties(*physical_device) };
            let device_name = unsafe { std::ffi::CStr::from_ptr(properties.device_name.as_ptr()) };
            println!("Device {}: {:?}", i, device_name);
        }
    }

    fn initialize(&mut self) {
        self.physical_device = Some(self.select_physical_device());
        self.logical_device = Some(self.create_logical_device());
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        // Cleanup.
        unsafe {
            if let Some(event_loop) = self.event_loop.take() {
                event_loop.exit();
            }
            self.instance.destroy_instance(None);
        }
    }
}
