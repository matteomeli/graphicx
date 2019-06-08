use crate::hal::Backend;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QueueType {
    Graphics,
    Compute,
    Transfer,
}

pub trait CommandQueue<B: Backend> {
    fn submit(&self, command_buffers: Vec<&B::CommandBuffer>);

    fn signal_fence(&self, fence: &B::Fence, value: u64);
}
