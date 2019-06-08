use crate::hal::command::CommandPoolFlags;
use crate::hal::queue::QueueType;
use crate::hal::Backend;

pub trait Device<B: Backend> {
    fn create_command_queue(&self, queue_type: QueueType) -> B::CommandQueue;

    fn create_command_pool(&self, queue_type: QueueType, flags: CommandPoolFlags)
        -> B::CommandPool;

    fn create_fence(&self, initial_value: u64) -> B::Fence;
    fn reset_fence(&self, fence: &B::Fence);
    fn wait_for_fence(&self, fence: &B::Fence, value: u64) -> bool {
        self.wait_for_fence_with_timeout(fence, value, u64::max_value())
    }
    fn wait_for_fence_with_timeout(&self, fence: &B::Fence, value: u64, timeout_ns: u64) -> bool;
}
