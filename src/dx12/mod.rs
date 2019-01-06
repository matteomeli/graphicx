extern crate winapi;
extern crate winit;
extern crate wio;

pub mod barrier;
pub mod command;
pub mod descriptor;
pub mod device;
pub mod dxgi;
pub mod resource;
pub mod sync;

pub use self::barrier::{BarrierDesc, BarrierFlags};
pub use self::command::{
    CommandAllocator, CommandListType, CommandQueue, CommandQueueFlags, CommandQueuePriority,
    GraphicsCommandList,
};
pub use self::descriptor::{DescriptorHeap, DescriptorHeapFlags, DescriptorHeapType};
pub use self::device::Device;
pub use self::dxgi::{
    Adapter4, AlphaMode, Factory4, FactoryCreationFlags, Flags, Format, SampleDesc, Scaling,
    SwapChain1, SwapChain4, SwapChainDesc, SwapEffect, Usage, WindowAssociationFlags,
};
pub use self::resource::{Resource, ResourceStates};
pub use self::sync::{Event, Fence};

use winapi::shared::winerror;
use winapi::um::{d3d12, d3d12sdklayers};
use winapi::Interface;

use std::ptr;

#[cfg(debug_assertions)]
pub fn enable_debug_layer() {
    let mut debug_interface: *mut d3d12sdklayers::ID3D12Debug = ptr::null_mut();
    let hr = unsafe {
        d3d12::D3D12GetDebugInterface(
            &d3d12sdklayers::ID3D12Debug::uuidof(),
            &mut debug_interface as *mut *mut _ as *mut *mut _,
        )
    };

    if winerror::SUCCEEDED(hr) {
        unsafe {
            (*debug_interface).EnableDebugLayer();
            (*debug_interface).Release();
        }
    }
}
#[cfg(not(debug_assertions))]
pub fn enable_debug_layer() {}
