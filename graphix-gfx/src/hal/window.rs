use crate::hal::format::Format;
use crate::hal::Backend;

pub type SwapchainBufferIndex = u32;

pub struct SwapchainConfig {
    pub format: Format,
    pub buffer_count: usize,
    pub width: u32,
    pub height: u32,
    pub sync_interval: u32, // < 0 - The presentation occurs immediately, there is no synchronization. 1 through 4 - Synchronize presentation after the nth vertical blank.
}

pub struct BackBuffer<B: Backend> {
    pub framebuffers: Vec<B::FrameBuffer>,
}

pub trait Surface<B: Backend> {
    fn create_swapchain(
        &self,
        device: &B::Device,
        command_queue: &B::CommandQueue,
        config: SwapchainConfig,
    ) -> B::Swapchain;
}

pub trait Swapchain<B: Backend> {
    fn acquire_buffer(&self) -> SwapchainBufferIndex;

    fn present(&self);

    fn create_backbuffer(&self) -> BackBuffer<B>;
}
