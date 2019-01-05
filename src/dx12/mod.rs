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

use winapi::shared::{dxgi1_4, dxgi1_5, minwindef, winerror};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3d12sdklayers};
use winapi::Interface;

use std::mem;
use std::ptr;

#[cfg(debug_assertions)]
pub fn enable_debug_layer() {
    let mut d3d12_debug_interface: *mut d3d12sdklayers::ID3D12Debug = ptr::null_mut();
    let hr = unsafe {
        d3d12::D3D12GetDebugInterface(
            &d3d12sdklayers::ID3D12Debug::uuidof(),
            &mut d3d12_debug_interface as *mut *mut _ as *mut *mut _,
        )
    };

    if winerror::SUCCEEDED(hr) {
        unsafe {
            (*d3d12_debug_interface).EnableDebugLayer();
            (*d3d12_debug_interface).Release();
        }
    }
}
#[cfg(not(debug_assertions))]
pub fn enable_debug_layer() {}

// TODO: Add this as a function to Factory
pub fn is_tearing_supported() -> bool {
    let mut allow_tearing: minwindef::BOOL = minwindef::FALSE;

    let mut dxgi_factory4: *mut dxgi1_4::IDXGIFactory4 = ptr::null_mut();
    let hr = unsafe {
        winapi::shared::dxgi::CreateDXGIFactory1(
            &dxgi1_4::IDXGIFactory4::uuidof(),
            &mut dxgi_factory4 as *mut *mut _ as *mut *mut _,
        )
    };
    if winerror::SUCCEEDED(hr) {
        let mut dxgi_factory5: *mut dxgi1_5::IDXGIFactory5 = ptr::null_mut();

        // Perform QueryInterface fun, because we're not using ComPtrs.
        // TODO: Code repetition, need a function or struct to handle this
        unsafe {
            let as_unknown: &IUnknown = &*(dxgi_factory4 as *mut IUnknown);
            let err = as_unknown.QueryInterface(
                &dxgi1_5::IDXGIFactory5::uuidof(),
                &mut dxgi_factory5 as *mut *mut _ as *mut *mut _,
            );
            if err < 0 {
                panic!("Failed on casting into a DXGI 1.5 factory: {:?}", hr);
            }

            let hr = (*dxgi_factory5).CheckFeatureSupport(
                dxgi1_5::DXGI_FEATURE_PRESENT_ALLOW_TEARING,
                &mut allow_tearing as *mut _ as *mut _,
                mem::size_of::<minwindef::BOOL>() as _,
            );
            if !winerror::SUCCEEDED(hr) {
                allow_tearing = minwindef::FALSE;
            }
        }
    }

    allow_tearing == minwindef::TRUE
}
