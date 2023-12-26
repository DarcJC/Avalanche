use std::sync::Arc;
use std::time::Duration;
use anyhow::{anyhow, Result};
use ash::extensions::khr::Swapchain as AshSwapchain;
use ash::vk;
use log::debug;
use crate::{Context, Device, Fence, Image, ImageView, Queue, Semaphore};

pub struct AcquiredImage {
    pub index: u32,
    pub is_suboptimal: bool,
}

pub struct Swapchain {
    device: Arc<Device>,
    inner: AshSwapchain,
    swapchain_khr: vk::SwapchainKHR,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
    pub color_space: vk::ColorSpaceKHR,
    pub present_mode: vk::PresentModeKHR,
    pub images: Vec<Image>,
    pub views: Vec<ImageView>,

    /// semaphore for acquire image
    acquire_semaphores: Vec<Arc<Semaphore>>,
    current_semaphores_index: usize,
}

impl Swapchain {
    pub fn new(context: &Context, width: u32, height: u32) -> Result<Self> {
        let device = context.device.clone();

        let format = {
            let formats = unsafe {
                context.surface.inner.get_physical_device_surface_formats(
                    context.physical_device.inner,
                    context.surface.surface_khr,
                )?
            };
            if formats.len() == 1 && formats[0].format == vk::Format::UNDEFINED {
                vk::SurfaceFormatKHR {
                    format: vk::Format::B8G8R8A8_UNORM,
                    color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
                }
            } else {
                *formats
                    .iter()
                    .find(|format| {
                        format.format == vk::Format::B8G8R8A8_UNORM
                            && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                    })
                    .unwrap_or(&formats[0])
            }
        };
        debug!("[Vulkan] Selected swapchain format is {format:?}");

        let present_mode = {
            let present_modes = unsafe {
                context
                    .surface
                    .inner
                    .get_physical_device_surface_present_modes(
                        context.physical_device.inner,
                        context.surface.surface_khr,
                    )?
            };
            if present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
                vk::PresentModeKHR::IMMEDIATE
            } else {
                vk::PresentModeKHR::FIFO
            }
        };
        debug!("[Vulkan] Selected swapchain present mode is {present_mode:?}");

        let capabilities = context.get_surface_capabilities()?;

        let extent = get_surface_suitable_extent(&capabilities, width, height);
        debug!("[Vulkan] Selected swapchain extent is {extent:?}");

        let image_count = capabilities.min_image_count + 1;
        debug!("[Vulkan] Selected swapchain image count is {image_count:?}");

        let families_indices = [
            context.graphics_queue_family.index,
            context.present_queue_family.index,
        ];
        let create_info = {
            let mut builder = vk::SwapchainCreateInfoKHR::builder()
                .surface(context.surface.surface_khr)
                .min_image_count(image_count)
                .image_format(format.format)
                .image_color_space(format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST
                );
            builder = if context.graphics_queue_family.index != context.present_queue_family.index {
                builder
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(&families_indices)
            } else {
                builder.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            };

            builder
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
        };

        let inner = AshSwapchain::new(&context.instance.inner, &context.device.inner);
        let swapchain_khr = unsafe { inner.create_swapchain(&create_info, None)? };

        let images = unsafe { inner.get_swapchain_images(swapchain_khr)? };
        let images = images
            .into_iter()
            .map(|i| {
                Image::from_swapchain_image(
                    device.clone(),
                    context.allocator.clone(),
                    i,
                    format.format,
                    extent,
                )
            })
            .collect::<Vec<_>>();

        let views = images
            .iter()
            .map(Image::create_image_view)
            .collect::<Result<Vec<_>, _>>()?;

        let acquire_semaphores = images
            .iter()
            .map(|_| {
                Arc::new(Semaphore::new(device.clone()).unwrap())
            })
            .collect::<Vec<_>>();

        Ok(Self {
            device,
            inner,
            swapchain_khr,
            extent,
            format: format.format,
            color_space: format.color_space,
            present_mode,
            images,
            views,
            acquire_semaphores,
            current_semaphores_index: 0,
        })
    }

    pub fn resize(&mut self, context: &Context, width: u32, height: u32) -> Result<()> {
        self.destroy();

        let capabilities = context.get_surface_capabilities()?;
        let extent = get_surface_suitable_extent(&capabilities, width, height);
        debug!("[Vulkan] Resizing swapchain to {}x{}", extent.width, extent.height);

        let image_count = capabilities.min_image_count + 1;

        let families_indices = [
            context.graphics_queue_family.index,
            context.present_queue_family.index,
        ];
        let create_info = {
            let mut builder = vk::SwapchainCreateInfoKHR::builder()
                .surface(context.surface.surface_khr)
                .min_image_count(image_count)
                .image_format(self.format)
                .image_color_space(self.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                );
            builder = if context.graphics_queue_family.index != context.present_queue_family.index {
                builder
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(&families_indices)
            } else {
                builder.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            };

            builder
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(self.present_mode)
                .clipped(true)
        };

        let swapchain_khr = unsafe { self.inner.create_swapchain(&create_info, None)? };

        // Swapchain images and image views
        let images = unsafe { self.inner.get_swapchain_images(swapchain_khr)? };
        let images = images
            .into_iter()
            .map(|i| {
                Image::from_swapchain_image(
                    self.device.clone(),
                    context.allocator.clone(),
                    i,
                    self.format,
                    extent,
                )
            })
            .collect::<Vec<_>>();

        let views = images
            .iter()
            .map(Image::create_image_view)
            .collect::<Result<Vec<_>, _>>()?;

        if self.images.len() != image_count as usize {
            self.acquire_semaphores = images
                .iter()
                .map(|_| {
                    Arc::new(Semaphore::new(self.device.clone()).unwrap())
                })
                .collect::<Vec<_>>();
            self.current_semaphores_index = 0;
        }

        self.swapchain_khr = swapchain_khr;
        self.extent = extent;
        self.images = images;
        self.views = views;

        Ok(())
    }

    fn next_semaphore(&mut self) -> Result<Arc<Semaphore>> {
        self.current_semaphores_index = (self.current_semaphores_index + 1) % self.images.len();
        self.acquire_semaphores[self.current_semaphores_index] = Arc::new(Semaphore::new(self.device.clone())?);
        Ok(self.current_acquire_semaphore())
    }

    pub fn current_acquire_semaphore(&self) -> Arc<Semaphore> {
        self.acquire_semaphores[self.current_semaphores_index].clone()
    }

    pub fn acquire_next_image(&mut self, timeout: Duration, fence: Option<&Fence>) -> Result<AcquiredImage> {
        let timeout = timeout.as_nanos() as u64;
        let semaphore = self.next_semaphore()?;
        let (index, is_suboptimal) = unsafe {
            self.inner.acquire_next_image(
                self.swapchain_khr,
                timeout,
                semaphore.inner,
                if let Some(fence) = fence { fence.inner } else { vk::Fence::null() },
            )?
        };

        Ok(AcquiredImage {
            index,
            is_suboptimal,
        })
    }

    pub fn acquire_next_image_v2(&self, timeout: Duration, fence: Option<&Fence>, semaphore: Option<&Semaphore>) -> Result<AcquiredImage> {
        if fence.is_none() && semaphore.is_none() {
            return Err(anyhow!("Fence and semaphore should not both none."));
        }
        let timeout = timeout.as_nanos() as u64;
        let (index, is_suboptimal) = unsafe {
            self.inner.acquire_next_image2(
                &vk::AcquireNextImageInfoKHR::builder()
                    .swapchain(self.swapchain_khr)
                    .timeout(timeout)
                    .fence(if let Some(fence) = fence { fence.inner } else { vk::Fence::null() })
                    .semaphore(if let Some(semaphore) = semaphore { semaphore.inner } else { vk::Semaphore::null() })
                    .build()
            )?
        };

        Ok(AcquiredImage {
            index,
            is_suboptimal,
        })
    }

    pub fn queue_present(
        &self,
        image_index: u32,
        wait_semaphores: &[&Semaphore],
        queue: &Queue,
    ) -> Result<bool> {
        let swapchains = [self.swapchain_khr];
        let images_indices = [image_index];
        let wait_semaphores = wait_semaphores.iter().map(|s| s.inner).collect::<Vec<_>>();

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&images_indices);

        let result = unsafe { self.inner.queue_present(queue.inner, &present_info)? };

        Ok(result)
    }

    fn destroy(&mut self) {
        self.views.clear();
        self.images.clear();
        unsafe {
            self.inner.destroy_swapchain(self.swapchain_khr, None)
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        self.destroy()
    }
}

pub fn get_surface_suitable_extent(capabilities: &vk::SurfaceCapabilitiesKHR, target_width: u32, target_height: u32) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        let min = capabilities.min_image_extent;
        let max = capabilities.max_image_extent;
        let width = target_width.clamp(min.width, max.width);
        let height = target_height.clamp(min.height, max.height);
        vk::Extent2D { width, height }
    }
}

impl Context {
    pub fn get_surface_capabilities(&self) -> Result<vk::SurfaceCapabilitiesKHR> {
        Ok(unsafe {
            self
                .surface
                .inner
                .get_physical_device_surface_capabilities(
                    self.physical_device.inner,
                    self.surface.surface_khr,
                )?
        })
    }
}
