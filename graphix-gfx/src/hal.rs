use std::any::Any;

pub use crate::hal::adapter::{Adapter, AdapterInfo, DeviceType, PhysicalAdapter};
pub use crate::hal::attachment::{Attachment, AttachmentMode};
pub use crate::hal::command::{
    BarrierPoint, ClearColor, CommandBuffer, CommandPool, CommandPoolFlags,
};
pub use crate::hal::device::Device;
pub use crate::hal::format::Format;
pub use crate::hal::queue::{CommandQueue, QueueType};
pub use crate::hal::window::{
    BackBuffer, Surface, Swapchain, SwapchainBufferIndex, SwapchainConfig,
};

pub mod adapter;
pub mod attachment;
pub mod command;
pub mod device;
pub mod format;
pub mod queue;
pub mod window;

pub trait Backend: Sized {
    type PhysicalAdapter: PhysicalAdapter<Self>;
    type Device: Device<Self>;

    type CommandQueue: CommandQueue<Self>;
    type CommandPool: CommandPool<Self>;
    type CommandBuffer: CommandBuffer<Self>;

    type Surface: Surface<Self>;
    type Swapchain: Swapchain<Self>;
    type FrameBuffer: Any;

    type Fence: Any;
}

pub trait Instance {
    type Backend: Backend;

    fn enumerate_adapters(&self) -> Vec<Adapter<Self::Backend>>;
}
