use std::sync::Arc;
use anyhow::Context;
use ash::vk;
use bevy_ecs::prelude::Resource;
use bevy_log::error;
use avalanche_hlvk::{CommandBuffer, CommandPool, Device, Fence, Queue, Semaphore};
use crate::context::RenderingContext;
use crate::INIT_COMMAND_POOL_NUM;

#[derive(Resource)]
pub struct FrameContext {
    render_context: RenderingContext,
    /// Cyclic frame counter
    current_frame: usize,
    command_buffers: Vec<CommandBuffer>,
    frame_finish_semaphore: Arc<Semaphore>,
    sync_fence: Arc<Fence>,
}

impl FrameContext {
    /// ## UNSAFE
    /// The method should only called at the extract stage to create a new frame context.
    ///
    /// **SAFETY of any Operation ISN'T PERFORMED in Main Thread is NOT GUARANTEED!**
    pub(crate) unsafe fn new(render_context: RenderingContext) -> Self {
        static mut COUNTER: usize = 0;
        let current_frame = COUNTER.wrapping_add(1);
        let frame_finish_semaphore = Arc::new(Semaphore::new(render_context.context.device.clone()).unwrap());
        let sync_fence = Arc::new(Fence::new(render_context.context.device.clone(), None).unwrap());
        // TODO: try to use Timeline Semaphore introduced in vk 1.2?
        // let sync_fence = Arc::new(Fence::null());
        let mut frame_context = FrameContext {
            render_context,
            current_frame,
            command_buffers: Vec::new(),
            frame_finish_semaphore,
            sync_fence,
        };

        match frame_context.allocate_command_buffer(None) {
            Err(err) => {
                error!("Failed to allocate default command when creating new [`FrameContext`]: {err}");
            },
            Ok(buffer) => {
                let _ = buffer.begin(None);
            }
        }

        frame_context
    }

    pub fn active_command_pool(&self) -> Arc<CommandPool> {
        let index = self.current_frame % INIT_COMMAND_POOL_NUM;
        self.render_context.command_pools.get(index).unwrap().clone()
    }

    pub fn active_command_pool_ref(&self) -> &CommandPool {
        let index = self.current_frame % INIT_COMMAND_POOL_NUM;
        self.render_context.command_pools.get(index).unwrap()
    }

    /// [`CommandBuffer`] doesn't use RAII,
    /// caller has duty to trace and free them.
    pub fn allocate_command_buffer(&mut self, level: Option<vk::CommandBufferLevel>) -> anyhow::Result<&CommandBuffer> {
        let command_buffer = self.active_command_pool_ref().allocate_command_buffer(level.unwrap_or(vk::CommandBufferLevel::PRIMARY))?;
        self.command_buffers.push(command_buffer);
        self.command_buffers.last().context("Unexpected error.")
    }

    #[inline]
    pub fn graphics_queue_ref(&self) -> &Queue {
        &self.render_context.graphics_queue
    }

    #[inline]
    pub fn graphics_queue(&self) -> Queue {
        self.render_context.graphics_queue.clone()
    }

    pub fn submit(&self, queue: &Queue) -> anyhow::Result<()> {
        let signal_semaphore = self.frame_finish_semaphore.as_ref();
        queue.submit(&self.command_buffers, &[], std::slice::from_ref(signal_semaphore), self.sync_fence.as_ref())
    }

    pub fn frame_finish_semaphore(&self) -> Arc<Semaphore> {
        self.frame_finish_semaphore.clone()
    }

    #[inline]
    pub fn device(&self) -> Arc<Device> {
        self.render_context.device.clone()
    }

    #[inline]
    pub fn command_buffer(&self, index: usize) -> Option<&CommandBuffer> {
        self.command_buffers.get(index)
    }

    #[inline]
    pub fn sync_fence(&self) -> Arc<Fence> {
        self.sync_fence.clone()
    }

    #[inline]
    pub fn sync_fence_ref(&self) -> &Fence {
        self.sync_fence.as_ref()
    }

    #[inline]
    pub fn render_context(&self) -> &RenderingContext {
        &self.render_context
    }

    #[inline]
    pub fn render_context_mut(&self) -> &RenderingContext {
        &self.render_context
    }
}

impl Drop for FrameContext {
    fn drop(&mut self) {
        let command_buffer = self.command_buffers
            .iter()
            .map(|buffer| buffer.inner)
            .collect::<Vec<_>>();
        unsafe {
            self.device().inner.free_command_buffers(self.active_command_pool_ref().inner, command_buffer.as_slice());
        }
    }
}
