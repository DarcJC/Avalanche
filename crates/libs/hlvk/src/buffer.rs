use std::fmt::{Debug, Formatter};
use std::mem::{align_of, size_of_val};
use std::sync::{Arc, Mutex};
use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use anyhow::Result;
use ash::vk::Handle;
use crate::Device;

pub struct Buffer {
    device: Arc<Device>,
    allocator: Arc<Mutex<Allocator>>,
    pub(crate) inner: vk::Buffer,
    allocation: Option<Allocation>,
    pub size: vk::DeviceSize,
}

impl Buffer {
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<Mutex<Allocator>>,
        usage: vk::BufferUsageFlags,
        memory_location: MemoryLocation,
        size: vk::DeviceSize,
    ) -> Result<Self> {
        let create_info = vk::BufferCreateInfo::builder().size(size).usage(usage);
        let inner = unsafe { device.inner.create_buffer(&create_info, None)? };
        let requirements = unsafe { device.inner.get_buffer_memory_requirements(inner) };
        let allocation = allocator.lock().unwrap().allocate(&AllocationCreateDesc {
            name: "buffer",
            requirements,
            location: memory_location,
            linear: true,
            allocation_scheme: AllocationScheme::DedicatedBuffer(inner),
        })?;

        unsafe {
            device
                .inner
                .bind_buffer_memory(inner, allocation.memory(), allocation.offset())?
        };

        Ok(Self {
            device,
            allocator,
            inner,
            allocation: Some(allocation),
            size
        })
    }

    pub fn copy_data_to_buffer<T: Copy>(&self, data: &[T]) -> Result<()> {
        unsafe {
            let data_ptr = self
                .allocation
                .as_ref()
                .unwrap()
                .mapped_ptr()
                .unwrap()
                .as_ptr();
            let mut align =
                ash::util::Align::new(data_ptr, align_of::<T>() as _, size_of_val(data) as _);
            align.copy_from_slice(data);
        };

        Ok(())
    }

    pub fn get_device_address(&self) -> u64 {
        let addr_info = vk::BufferDeviceAddressInfo::builder().buffer(self.inner);
        unsafe { self.device.inner.get_buffer_device_address(&addr_info) }
    }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Buffer {} (size: {})", self.inner.as_raw(), self.size)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { self.device.inner.destroy_buffer(self.inner, None); }
        self.allocator
            .lock()
            .unwrap()
            .free(self.allocation.take().unwrap())
            .unwrap();
    }
}
