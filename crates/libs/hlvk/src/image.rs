use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use ash::vk;
use ash::vk::Handle;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use crate::{Context, Device};

pub struct Image {
    device: Arc<Device>,
    allocator: Arc<Mutex<Allocator>>,
    pub(crate) inner: vk::Image,
    allocation: Option<Allocation>,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    /// Preventing internal referenced Image been destroyed.
    is_external_referenced: bool,
}

pub struct ImageView {
    device: Arc<Device>,
    pub(crate) inner: vk::ImageView,
}

impl Image {
    pub(crate) fn new_2d(
        device: Arc<Device>,
        allocator: Arc<Mutex<Allocator>>,
        usage: vk::ImageUsageFlags,
        memory_location: MemoryLocation,
        format: vk::Format,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let extent = vk::Extent3D {
            width,
            height,
            depth: 1,
        };

        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let inner = unsafe { device.inner.create_image(&image_info, None)? };
        let requirements = unsafe { device.inner.get_image_memory_requirements(inner) };

        let allocation = allocator.lock().unwrap().allocate(&AllocationCreateDesc {
            name: "image",
            requirements,
            location: memory_location,
            linear: true,
            allocation_scheme: AllocationScheme::DedicatedImage(inner.clone()),
        })?;

        unsafe {
            device
                .inner
                .bind_image_memory(inner, allocation.memory(), allocation.offset())?
        };

        Ok(
            Self {
                device,
                allocator,
                inner,
                allocation: Some(allocation),
                format,
                extent,
                is_external_referenced: false,
            }
        )
    }

    pub(crate) fn from_swapchain_image(
        device: Arc<Device>,
        allocator: Arc<Mutex<Allocator>>,
        swapchain_image: vk::Image,
        format: vk::Format,
        extent: vk::Extent2D
    ) -> Self {
        let extent = vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        };

        Self {
            device,
            allocator,
            inner: swapchain_image,
            allocation: None,
            format,
            extent,
            is_external_referenced: true,
        }
    }

    pub fn create_image_view(&self) -> Result<ImageView> {
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(self.inner)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(self.format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        let inner = unsafe { self.device.inner.create_image_view(&view_info, None)? };

        Ok(ImageView {
            device: self.device.clone(),
            inner,
        })
    }

    pub fn clone_external(&self) -> Self {
        assert!(self.is_external_referenced, "Only external referenced Image can be clone. Use Arc<Image> instead.");
        assert!(self.allocation.is_none(), "Allocation of external referenced Image should be None.");
        Self {
            device: self.device.clone(),
            allocator: self.allocator.clone(),
            inner: self.inner.clone(),
            allocation: None,
            format: self.format,
            extent: self.extent,
            is_external_referenced: true,
        }
    }
}

impl Context {
    pub fn create_image(
        &self,
        usage: vk::ImageUsageFlags,
        memory_location: MemoryLocation,
        format: vk::Format,
        width: u32,
        height: u32,
    ) -> Result<Image> {
        Image::new_2d(
            self.device.clone(),
            self.allocator.clone(),
            usage,
            memory_location,
            format,
            width,
            height
        )
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Image[{}] Extent {:?} Format {:?} External?: {}", self.inner.as_raw(), self.extent, self.format, self.is_external_referenced)
    }
}

impl Debug for ImageView {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ImageView({})", self.inner.as_raw())
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        if !self.is_external_referenced {
            unsafe { self.device.inner.destroy_image(self.inner, None) };
            self.allocator
                .lock()
                .unwrap()
                .free(self.allocation.take().unwrap())
                .unwrap();
        }
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe { self.device.inner.destroy_image_view(self.inner, None) }
    }
}
