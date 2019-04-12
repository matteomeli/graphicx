use winapi::um::d3dcommon;

use std::error;
use std::fmt;

mod barrier;
mod command;
mod descriptor;
mod device;
mod dxgi;
mod resource;
mod sync;

pub use self::barrier::{BarrierDesc, BarrierFlags};
pub use self::command::{
    CommandAllocator, CommandListType, CommandQueue, CommandQueueFlags, CommandQueuePriority,
    GraphicsCommandList,
};
pub use self::descriptor::{DescriptorHeap, DescriptorHeapFlags, DescriptorHeapType};
pub use self::device::Device;
pub use self::dxgi::{
    Adapter, AdapterInfo, AlphaMode, BufferUsage, Factory, Format, GpuPreference, PresentFlags,
    SampleDesc, Scaling, SwapChain, SwapChainConfig, SwapChainFlags, SwapEffect,
    WindowAssociationFlags,
};
pub use self::resource::{Resource, ResourceStates};
pub use self::sync::{Event, Fence};

pub type D3DResult<T> = Result<T, Error>;

// TODO: Include HRESULT in errors as they mostly originate from the underlying D3D layer,
// but also support custom instances of `Error` that can be created with crafted error messages
// and a particular value of `ErrorKind`.
#[derive(Debug)]
pub enum Error {
    AdapterNotFound,
    CreateFactoryFailed,
    PresentSwapChainFailed,
    GetBufferFromSwapChainFailed,
    CreateSwapChainFailed,
    ResizeSwapChainFailed,
    GetSwapChainDescFailed,
    CreateDeviceFailed,
    WarpAdapterNotSupported,
    SwapChainNotAvailable,
    MakeWindowAssociationFailed,
    CreateFenceFailed,
    SetFenceCompletionEventFailed,
    SignalFenceFailed,
    CreateDescriptorHeapFailed,
    CreateCommandQueueFailed,
    SignalCommandQueueFailed,
    CreateCommandAllocatorFailed,
    ResetCommandAllocatorFailed,
    CreateGraphicsCommandListFailed,
    CloseGraphicsCommandListFailed,
    ResetGraphicsCommandListFailed,
}

impl Error {
    pub(crate) fn as_str(&self) -> &'static str {
        match *self {
            Error::AdapterNotFound => "adapter not found",
            Error::CreateFactoryFailed => "failed to create factory",
            Error::PresentSwapChainFailed => "failed to present the swapchain",
            Error::GetBufferFromSwapChainFailed => "failed to get buffer resource from swapchain",
            Error::CreateSwapChainFailed => "failed to create the swapchain",
            Error::ResizeSwapChainFailed => "failed to resize the swapchain",
            Error::GetSwapChainDescFailed => "failed to get swapchain description",
            Error::CreateDeviceFailed => "failed to create a device",
            Error::WarpAdapterNotSupported => "warp adapter not supported",
            Error::SwapChainNotAvailable => "swap chain not available",
            Error::MakeWindowAssociationFailed => "failed to make window associtation",
            Error::CreateFenceFailed => "failed to create fence",
            Error::SetFenceCompletionEventFailed => "failed to set a fence completion event",
            Error::SignalFenceFailed => "failed to signal fence",
            Error::CreateDescriptorHeapFailed => "failed to create descriptor heap",
            Error::CreateCommandQueueFailed => "failed to create a command queue",
            Error::SignalCommandQueueFailed => "failed to signal command queue",
            Error::CreateCommandAllocatorFailed => "failed to create a commamd locator",
            Error::ResetCommandAllocatorFailed => "failed to reset command allocator",
            Error::CreateGraphicsCommandListFailed => "failed to create a graphics command list",
            Error::CloseGraphicsCommandListFailed => "failed to close graphics command list",
            Error::ResetGraphicsCommandListFailed => "failed to reset graphics command list",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DirectX12 error: {}", self.as_str())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[repr(u32)]
pub enum FeatureLevel {
    Lvl9_1 = d3dcommon::D3D_FEATURE_LEVEL_9_1,
    Lvl9_2 = d3dcommon::D3D_FEATURE_LEVEL_9_2,
    Lvl9_3 = d3dcommon::D3D_FEATURE_LEVEL_9_3,
    Lvl10_0 = d3dcommon::D3D_FEATURE_LEVEL_10_0,
    Lvl10_1 = d3dcommon::D3D_FEATURE_LEVEL_10_1,
    Lvl11_0 = d3dcommon::D3D_FEATURE_LEVEL_11_0,
    Lvl11_1 = d3dcommon::D3D_FEATURE_LEVEL_11_1,
    Lvl12_0 = d3dcommon::D3D_FEATURE_LEVEL_12_0,
    Lvl12_1 = d3dcommon::D3D_FEATURE_LEVEL_12_1,
}
