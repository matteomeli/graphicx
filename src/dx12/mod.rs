extern crate winapi;
extern crate winit;
extern crate wio;

pub mod command;
pub mod descriptor;
pub mod device;
pub mod dxgi;
pub mod sync;

pub use self::command::{
    CommandAllocator, CommandListType, CommandQueue, CommandQueueFlags, CommandQueuePriority,
    GraphicsCommandList,
};
pub use self::descriptor::{DescriptorHeap, DescriptorHeapFlags, DescriptorHeapType};
pub use self::device::Device;
pub use self::dxgi::SwapChain4;
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

pub fn update_render_target_views(
    device: &Device,
    swap_chain: &SwapChain4,
    descriptor_heap: &DescriptorHeap,
    back_buffers_count: usize, // TODO: Make this u32
    back_buffers: &mut Vec<ComPtr<d3d12::ID3D12Resource>>,
) {
    let descriptor_size: usize = device.get_descriptor_increment_size(DescriptorHeapType::RTV) as _;

    let mut descriptor = descriptor_heap.get_cpu_descriptor_start();

    for i in 0..back_buffers_count {
        let back_buffer = swap_chain.get_buffer(i as _);

        device.create_render_target_view(&back_buffer, descriptor);
        back_buffers.push(back_buffer);

        descriptor.ptr += descriptor_size;
    }
}

pub fn render(
    command_allocators: &[CommandAllocator],
    back_buffers: &[ComPtr<d3d12::ID3D12Resource>],
    current_back_buffer_index: &mut usize,
    graphics_command_list: &GraphicsCommandList,
    command_queue: &CommandQueue,
    descriptor_heap: &DescriptorHeap,
    descriptor_size: usize,
    swap_chain: &SwapChain4,
    fence: &Fence,
    frame_fence_values: &mut [u64],
    fence_event: Event,
    fence_value: &mut u64,
    is_tearing_supported: bool,
    is_vsync_enabled: bool,
) {
    let command_allocator = &command_allocators[*current_back_buffer_index];
    let back_buffer = &back_buffers[*current_back_buffer_index];

    // Reset current command allocator and command list before new commands can be recorded
    command_allocator.reset();
    graphics_command_list.reset(&command_allocator);

    // Clear render target
    {
        let mut barrier = d3d12::D3D12_RESOURCE_BARRIER {
            Type: d3d12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE,
            u: unsafe { mem::zeroed() },
        };

        *unsafe { barrier.u.Transition_mut() } = d3d12::D3D12_RESOURCE_TRANSITION_BARRIER {
            pResource: back_buffer.as_raw(),
            Subresource: d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
            StateBefore: d3d12::D3D12_RESOURCE_STATE_PRESENT,
            StateAfter: d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
        };

        graphics_command_list.add_barriers(&barrier, 1);

        let clear_color: [f32; 4] = [0.4, 0.6, 0.9, 1.0];
        let mut rtv = descriptor_heap.get_cpu_descriptor_start();
        rtv.ptr += *current_back_buffer_index * descriptor_size;

        graphics_command_list.clear_render_target_view(rtv, clear_color);
    }

    // Present the back buffer
    {
        let mut barrier = d3d12::D3D12_RESOURCE_BARRIER {
            Type: d3d12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE,
            u: unsafe { mem::zeroed() },
        };

        *unsafe { barrier.u.Transition_mut() } = d3d12::D3D12_RESOURCE_TRANSITION_BARRIER {
            pResource: back_buffer.as_raw(),
            Subresource: d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
            StateBefore: d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
            StateAfter: d3d12::D3D12_RESOURCE_STATE_PRESENT,
        };

        graphics_command_list.add_barriers(&barrier, 1);

        graphics_command_list.close();

        let command_lists = vec![graphics_command_list.as_command_list()];
        command_queue.execute(&command_lists.as_slice());

        let sync_interval = if is_vsync_enabled { 1 } else { 0 };
        let present_flags = if is_tearing_supported && !is_vsync_enabled {
            winapi::shared::dxgi::DXGI_PRESENT_ALLOW_TEARING
        } else {
            0
        };
        swap_chain.present(sync_interval, present_flags);

        // Insert a signal into the command queue with a fence value
        frame_fence_values[*current_back_buffer_index] = command_queue.signal(&fence, fence_value);

        *current_back_buffer_index = swap_chain.get_current_back_buffer_index() as _;

        // Stall the CPU until fence value signalled is reached
        fence.wait_for_value(fence_event, frame_fence_values[*current_back_buffer_index]);
    }
}

pub fn resize(
    device: &Device,
    command_queue: &CommandQueue,
    back_buffers: &mut Vec<ComPtr<d3d12::ID3D12Resource>>,
    current_back_buffer_index: &mut usize,
    back_buffers_count: usize, // TODO: Make this u32
    swap_chain: &SwapChain4,
    descriptor_heap: &DescriptorHeap,
    fence: &Fence,
    frame_fence_values: &mut [u64],
    fence_event: Event,
    fence_value: &mut u64,
    width: u32,
    height: u32,
) {
    // Don't allow 0 size swap chain back buffers.
    let width = 1.max(width);
    let height = 1.max(height);

    // Flush the GPU queue to make sure the swap chain's back buffers
    // are not being referenced by an in-flight command list.
    command_queue.flush(fence, fence_event, fence_value);

    // Any references to the back buffers must be released
    // before the swap chain can be resized.
    while let Some(back_buffer) = back_buffers.pop() {
        std::mem::drop(back_buffer);
    }

    // Reset per-frame fence values to the fence value of the current back buffer index
    for i in 0..back_buffers_count {
        frame_fence_values[i] = frame_fence_values[*current_back_buffer_index];
    }

    swap_chain.resize_buffers(back_buffers_count as _, width, height);

    *current_back_buffer_index = swap_chain.get_current_back_buffer_index() as _;

    update_render_target_views(
        device,
        swap_chain,
        descriptor_heap,
        back_buffers_count,
        back_buffers,
    );
}
