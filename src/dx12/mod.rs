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
pub use self::dxgi::SwapChain4;
pub use self::resource::{Resource, ResourceStates};
pub use self::sync::{Event, Fence};

use winapi::shared::{dxgi1_3, dxgi1_4, dxgi1_5, dxgi1_6, minwindef, winerror};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3d12sdklayers, d3dcommon};
use winapi::Interface;
use wio::com::ComPtr;

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

pub fn get_adapter(use_warp: bool) -> ComPtr<dxgi1_6::IDXGIAdapter4> {
    let mut dxgi_factory: *mut dxgi1_4::IDXGIFactory4 = ptr::null_mut();
    let flags: u32 = if cfg!(debug_assertions) {
        dxgi1_3::DXGI_CREATE_FACTORY_DEBUG
    } else {
        0
    };

    let hr = unsafe {
        dxgi1_3::CreateDXGIFactory2(
            flags,
            &dxgi1_4::IDXGIFactory4::uuidof(),
            &mut dxgi_factory as *mut *mut _ as *mut *mut _,
        )
    };

    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on creating DXGI factory: {:?}", hr);
    }

    let mut dxgi_adapter1: *mut winapi::shared::dxgi::IDXGIAdapter1 = ptr::null_mut();
    let mut dxgi_adapter4: *mut dxgi1_6::IDXGIAdapter4 = ptr::null_mut();

    if use_warp {
        let hr = unsafe {
            (*dxgi_factory).EnumWarpAdapter(
                &winapi::shared::dxgi::IDXGIAdapter1::uuidof(),
                &mut dxgi_adapter1 as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on enumerating DXGI warp adapter: {:?}", hr);
        }

        // Perform QueryInterface fun, because we're not using ComPtrs.
        // TODO: Code repetition, need a function or struct to handle this
        unsafe {
            let as_unknown: &IUnknown = &*(dxgi_adapter1 as *mut IUnknown);
            let err = as_unknown.QueryInterface(
                &dxgi1_6::IDXGIAdapter4::uuidof(),
                &mut dxgi_adapter4 as *mut *mut _ as *mut *mut _,
            );
            if err < 0 {
                panic!("Failed on casting DXGI warp adapter: {:?}", hr);
            }
        }
    } else {
        let mut index = 0;
        let mut max_dedicated_vdeo_memory = 0;
        loop {
            let hr = unsafe { (*dxgi_factory).EnumAdapters1(index, &mut dxgi_adapter1) };
            if hr == winerror::DXGI_ERROR_NOT_FOUND {
                break;
            }

            index += 1;

            let mut desc: winapi::shared::dxgi::DXGI_ADAPTER_DESC1 = unsafe { mem::zeroed() };
            let hr = unsafe { (*dxgi_adapter1).GetDesc1(&mut desc) };
            if !winerror::SUCCEEDED(hr) {
                panic!("Failed on obtaining DXGI adapter description: {:?}", hr);
            }

            // We want only the hardware adapter with the largest dedicated video memory
            let hr = unsafe {
                d3d12::D3D12CreateDevice(
                    dxgi_adapter1 as *mut _,
                    d3dcommon::D3D_FEATURE_LEVEL_11_0,
                    &d3d12::ID3D12Device::uuidof(),
                    ptr::null_mut(),
                )
            };
            if (desc.Flags & winapi::shared::dxgi::DXGI_ADAPTER_FLAG_SOFTWARE) == 0
                && desc.DedicatedVideoMemory > max_dedicated_vdeo_memory
                && winerror::SUCCEEDED(hr)
            {
                max_dedicated_vdeo_memory = desc.DedicatedVideoMemory;

                // Perform QueryInterface fun, because we're not using ComPtrs.
                // TODO: Code repetition, need a function or struct to handle this
                unsafe {
                    let as_unknown: &IUnknown = &*(dxgi_adapter1 as *mut IUnknown);
                    let err = as_unknown.QueryInterface(
                        &dxgi1_6::IDXGIAdapter4::uuidof(),
                        &mut dxgi_adapter4 as *mut *mut _ as *mut *mut _,
                    );
                    if err < 0 {
                        panic!("Failed on casting into a DXGI 1.5 adapter: {:?}", hr);
                    }
                }
            }
        }
    }

    unsafe { ComPtr::from_raw(dxgi_adapter4) }
}

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
