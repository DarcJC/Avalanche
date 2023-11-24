use std::any::Any;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::rc::Rc;
use anyhow::{Context, Result};
use ash::vk;
use ash::vk::DeviceSize;
use enumflags2::BitFlags;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use crate::ash_window;
use crate::core::event_loop::EventLoopManager;
use crate::core::renderer_trait::{Buffer, buffer_cast, Renderer};
use crate::core::renderer_types::{GraphicsBufferCreationFlags, GraphicsBufferShareModes, GraphicsBufferUsageFlags};
use crate::core::renderer_vulkan::{compare_c_str_value, CompatibilityFlags, QueueInfo, retain_available_names, VulkanAccelerationStructureState, VulkanBuffer, VulkanRenderer};

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
        if !supports_graphics || !supports_compute {
            result -= 4;
        }
        result
    }

    pub(crate) fn select_physical_device(&self) -> vk::PhysicalDevice {
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

    pub(crate) fn create_logical_device(&mut self) -> ash::Device {
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
        self.queue_info.graphics = graphics_queue_family_index;

        // Find queue families that supporting graphics commands
        let compute_queue_family_index = queue_families
            .iter()
            .enumerate()
            .find(|(_index, queue_family_prop)| queue_family_prop.queue_flags.contains(vk::QueueFlags::COMPUTE))
            .map(|(index, _)| index as u32)
            .expect("No suitable compute queue family found.");
        self.queue_info.compute = compute_queue_family_index;

        // Fill queue create info
        let mut queue_create_infos = Vec::new();

        let graphics_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .queue_priorities(&[1.0])
            .build();
        queue_create_infos.push(graphics_queue_create_info);

        if compute_queue_family_index != graphics_queue_family_index {
            let compute_queue_create_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(compute_queue_family_index)
                .queue_priorities(&[1.0])
                .build();
            queue_create_infos.push(compute_queue_create_info);
        }

        // Fill device create info
        let extension_names = self.get_device_extension_names();
        let device_create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&extension_names)
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&vk::PhysicalDeviceFeatures::default())
            .build();

        unsafe {
            self.instance.create_device(self.physical_device.unwrap(), &device_create_info, None).expect("Failed to create logical device.")
        }
    }

    pub(crate) fn get_instance_extension_names(entry: &ash::Entry) -> Vec<*const c_char> {
        let mut extension_names = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            vk::KhrPortabilityEnumerationFn::name().as_ptr(),
            vk::KhrGetPhysicalDeviceProperties2Fn::name().as_ptr(),
            vk::KhrWin32SurfaceFn::name().as_ptr(),
            ash::extensions::khr::WaylandSurface::name().as_ptr(),
            ash::extensions::khr::XlibSurface::name().as_ptr(),
            ash::extensions::khr::XcbSurface::name().as_ptr(),
            ash::extensions::khr::AndroidSurface::name().as_ptr(),
            ash::extensions::ext::MetalSurface::name().as_ptr(),
        ];

        #[cfg(debug_assertions)]
        extension_names.push(ash::extensions::ext::DebugUtils::name().as_ptr());

        let available_extensions = entry.enumerate_instance_extension_properties(None).expect("Failed to enumerate instance extension properties.");

        retain_available_names(&mut extension_names, &available_extensions.iter().map(|ext| ext.extension_name.as_ptr()).collect::<Vec<_>>());

        extension_names
    }

    fn get_device_extension_names(&self) -> Vec<*const c_char> {
        let mut extension_names = vec![
            ash::extensions::khr::Swapchain::name().as_ptr(),
        ];

        let available_extensions = unsafe { self.instance.enumerate_device_extension_properties(self.physical_device.unwrap()).expect("Failed to enumerate device extensions.") };

        retain_available_names(&mut extension_names, &available_extensions.iter().map(|ext| ext.extension_name.as_ptr()).collect::<Vec<_>>());

        extension_names
    }

    pub(crate) fn get_layer_names(entry: &ash::Entry) -> Vec<*const c_char> {
        let mut layers = vec![];

        #[cfg(debug_assertions)]
        unsafe {
            layers.push(
                CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0").as_ptr()
            );
        }

        let available_layers = entry.enumerate_instance_layer_properties().expect("Failed to enumerate instance layer properties.");

        retain_available_names(&mut layers, &available_layers.iter().map(|layer| layer.layer_name.as_ptr()).collect::<Vec<_>>());
        layers
    }

    pub(crate) fn check_compatibility(&mut self) {
        let device_extensions = unsafe {
            self.instance.enumerate_device_extension_properties(self.physical_device.unwrap()).expect("Failed to enumerate device extension properties.")
        };
        self.compatibility_flags.khr_acceleration_structure = device_extensions
            .iter()
            .any(
                |prop|
                    compare_c_str_value(
                        &prop.extension_name.as_ptr(),
                        &ash::extensions::khr::AccelerationStructure::name().as_ptr(),
                    )
            );
        self.compatibility_flags.khr_ray_tracing_pipeline = device_extensions
            .iter()
            .any(
                |prop|
                    compare_c_str_value(
                        &prop.extension_name.as_ptr(),
                        &ash::extensions::khr::RayTracingPipeline::name().as_ptr(),
                    )
            );

        let mut props2 = vk::PhysicalDeviceProperties2::default();
        let mut device_ray_tracing_prop = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
        props2.p_next = &mut device_ray_tracing_prop as *mut _ as *mut c_void;
        unsafe { self.instance.get_physical_device_properties2(self.physical_device.unwrap(), &mut props2) };
        self.compatibility_flags.rt_max_ray_recursion_depth = device_ray_tracing_prop.max_ray_recursion_depth;

        println!("System Compatibility: {:?}", self.compatibility_flags);
    }

    pub fn find_memory_type(&self, type_filter_bits: u32, properties: vk::MemoryPropertyFlags) -> Option<u32> {
        let mem_prop = unsafe {
            self.instance
                .get_physical_device_memory_properties(
                    self.physical_device.unwrap()
                )
        };

        for i in 0..mem_prop.memory_type_count {
            if (type_filter_bits & (1 << i) != 0) && mem_prop.memory_types[i as usize].property_flags.contains(properties) {
                return Some(i);
            }
        }

        None
    }

    pub(crate) fn get_buffer_device_address(&self, buffer: &VulkanBuffer) -> Result<vk::DeviceAddress> {
        let device = self.logical_device.as_ref().context("Renderer hasn't initialized.")?;
        let handle = buffer.resource.context("Buffer hasn't initialized could not get device address.")?;
        let info = vk::BufferDeviceAddressInfo::builder().buffer(handle).build();
        Ok(unsafe {
            device.get_buffer_device_address(&info)
        })
    }
}

impl Renderer for VulkanRenderer {
    type BufferType = VulkanBuffer;

    fn new() -> Self {
        // Load the Vulkan library.
        let entry = unsafe { ash::Entry::load() }.unwrap();

        // Define the application info.
        let app_name = CString::new("Avalanche").unwrap();
        let engine_name = CString::new("Avalanche").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&engine_name)
            .engine_version(0)
            .api_version(vk::API_VERSION_1_3);

        // Define the instance create info.
        let extension_names = VulkanRenderer::get_instance_extension_names(&entry);
        let layer_names = VulkanRenderer::get_layer_names(&entry);
        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layer_names)
            .flags(create_flags)
            .build();

        // Create the instance.
        let instance = unsafe { entry.create_instance(&create_info, None).unwrap() };

        VulkanRenderer {
            entry,
            instance,
            physical_device: None,
            logical_device: None,
            surfaces: HashMap::new(),
            queue_info: QueueInfo::default(),
            compatibility_flags: CompatibilityFlags::default(),
            rt_as_state: VulkanAccelerationStructureState::default(),
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
            let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
            println!("Device {}: {:?}", i, device_name);
        }
    }

    fn initialize(&mut self) {
        self.physical_device = Some(self.select_physical_device());
        self.logical_device = Some(self.create_logical_device());
        self.check_compatibility();
    }

    fn support_ray_tracing(&self) -> bool {
        self.compatibility_flags.khr_acceleration_structure && self.compatibility_flags.khr_ray_tracing_pipeline
    }

    fn create_buffer_resource(&self, buffer: &mut impl Buffer) -> Result<()> {
        let buffer = buffer_cast::<VulkanBuffer, _>(buffer).unwrap();
        let device = self.logical_device.as_ref().context("Logical device isn't created.")?;
        buffer.resource = Some(unsafe { device.create_buffer(&buffer.create_info, None).expect("Failed to create buffer.") });
        let memory_requirement = unsafe {
            device.get_buffer_memory_requirements(buffer.resource.unwrap())
        };
        let memory_type_index = self.find_memory_type(
            memory_requirement.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE //| vk::MemoryPropertyFlags::HOST_COHERENT
        ).context("Failed to find compatible memory.")?;
        let allocation_info = vk::MemoryAllocateInfo::builder()
            .memory_type_index(memory_type_index)
            .allocation_size(memory_requirement.size)
            .build();
        let device_memory = unsafe {
            device.allocate_memory(&allocation_info, None)
        }?;
        buffer.device_memory = Some(device_memory);
        unsafe { device.bind_buffer_memory(buffer.resource.unwrap(), device_memory, 0)?; };
        Ok(())
    }

    fn map_buffer_memory(&self, buffer: &mut impl Buffer) -> Result<*mut c_void> {
        let buffer = buffer_cast::<VulkanBuffer, _>(buffer).unwrap();
        let buffer_memory = buffer.device_memory.context("Buffer hasn't created.")?;
        let device = self.logical_device.as_ref().context("Logical device isn't created.")?;
        Ok(unsafe { device.map_memory(buffer_memory, 0, buffer.create_info.size, vk::MemoryMapFlags::empty())? })
    }

    fn unmap_buffer_memory(&self, buffer: &mut impl Buffer) -> Result<()> {
        let buffer = buffer_cast::<VulkanBuffer, _>(buffer).unwrap();
        let buffer_memory = buffer.device_memory.context("Buffer hasn't created.")?;
        let device = self.logical_device.as_ref().context("Logical device isn't created.")?;
        unsafe { device.unmap_memory(buffer_memory); }
        Ok(())
    }

    fn get_buffer_creation_info(share_modes: BitFlags<GraphicsBufferShareModes>, usage: BitFlags<GraphicsBufferUsageFlags>, _flags: BitFlags<GraphicsBufferCreationFlags>, size: usize) -> Result<Rc<dyn Any>> {
        let mut info = vk::BufferCreateInfo::default();

        if share_modes.contains(GraphicsBufferShareModes::Concurrent) {
            info.sharing_mode = vk::SharingMode::CONCURRENT;
        } else {
            info.sharing_mode = vk::SharingMode::EXCLUSIVE;
        }

        if usage.contains(GraphicsBufferUsageFlags::VertexBuffer) {
            info.usage |= vk::BufferUsageFlags::VERTEX_BUFFER;
        }
        if usage.contains(GraphicsBufferUsageFlags::IndexBuffer) {
            info.usage |= vk::BufferUsageFlags::INDEX_BUFFER;
        }

        info.size = size as DeviceSize;

        Ok(Rc::new(info))
    }
}
