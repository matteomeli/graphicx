
use crate::hal::attachment::Attachment;
use crate::hal::Backend;

use bitflags::bitflags;

bitflags! {
    pub struct CommandPoolFlags: u8 {
        const MULTIPLE_ALLOCATOR = 0x1;
        const SINGLE_LIST = 0x2;
    }
}

pub enum BarrierPoint {
    Pre,
    Post,
}

pub type ClearColor = [f32; 4];

pub trait CommandPool<B: Backend> {
    fn reset(&self);

    fn create_buffer(&mut self) -> B::CommandBuffer {
        self.create_buffers(1).pop().unwrap()
    }

    fn create_buffers(&mut self, count: usize) -> Vec<B::CommandBuffer> {
        (0..count).map(|_| self.create_buffer()).collect()
    }
}

pub trait CommandBuffer<B: Backend> {
    fn begin(&self);
    fn end(&self);

    fn insert_barriers(
        &self,
        barrier_point: BarrierPoint,
        attachments: &[Attachment],
        framebuffer: &B::FrameBuffer,
    );
    fn clear(&self, clear_colors: &[ClearColor], framebuffer: &B::FrameBuffer);
}
