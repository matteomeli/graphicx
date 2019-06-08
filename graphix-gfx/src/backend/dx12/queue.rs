use crate::backend::dx12::command::CommandBuffer;
use crate::backend::dx12::device::Device;
use crate::backend::dx12::instance::Backend;
use crate::hal;

use graphix_native_dx12 as native;

pub struct CommandQueue {
    pub(crate) native: native::queue::CommandQueue,
}

impl CommandQueue {
    pub(crate) fn new(device: &Device, queue_type: hal::QueueType) -> Self {
        let queue = native::queue::CommandQueue::new(
            &device.native,
            get_native_type(queue_type),
            native::queue::CommandQueuePriority::Normal,
            native::queue::CommandQueueFlags::empty(),
        )
        .expect("Failed to create D3D12 command queue.");

        CommandQueue { native: queue }
    }
}

impl hal::CommandQueue<Backend> for CommandQueue {
    fn submit(&self, commnad_buffers: Vec<&CommandBuffer>) {
        let lists = commnad_buffers
            .into_iter()
            .map(CommandBuffer::as_command_list)
            .collect::<Vec<_>>();
        self.native.execute_command_lists(&lists);
    }

    fn signal_fence(&self, fence: &native::sync::Fence, value: u64) {
        self.native
            .signal(fence, value)
            .expect("Failed to signal D3D12 command queue");
    }
}

pub(crate) fn get_native_type(queue_type: hal::QueueType) -> native::command_list::CommandListType {
    match queue_type {
        hal::QueueType::Graphics => native::command_list::CommandListType::Direct,
        hal::QueueType::Compute => native::command_list::CommandListType::Compute,
        hal::QueueType::Transfer => native::command_list::CommandListType::Copy,
    }
}
