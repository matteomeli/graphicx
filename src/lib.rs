extern crate winapi;
extern crate winit;
extern crate wio;

use winapi::shared::{
    dxgi, dxgi1_2, dxgi1_3, dxgi1_4, dxgi1_5, dxgi1_6, dxgiformat, dxgitype, minwindef, windef,
    winerror,
};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3d12sdklayers, d3dcommon, synchapi, winnt};
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::ptr;
use std::time::{Duration, Instant};

// Add here missing flags for DXGIFactory::MakeWindowsAssociation
pub const DXGI_MWA_NO_WINDOW_CHANGES: minwindef::UINT = 1 << 0;
pub const DXGI_MWA_NO_ALT_ENTER: minwindef::UINT = 1 << 1;
pub const DXGI_MWA_NO_PRINT_SCREEN: minwindef::UINT = 1 << 2;
pub const DXGI_MWA_VALID: minwindef::UINT = 0x7;

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

    let mut dxgi_adapter1: *mut dxgi::IDXGIAdapter1 = ptr::null_mut();
    let mut dxgi_adapter4: *mut dxgi1_6::IDXGIAdapter4 = ptr::null_mut();

    if use_warp {
        let hr = unsafe {
            (*dxgi_factory).EnumWarpAdapter(
                &dxgi::IDXGIAdapter1::uuidof(),
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

            let mut desc: dxgi::DXGI_ADAPTER_DESC1 = unsafe { mem::zeroed() };
            let hr = unsafe { (*dxgi_adapter1).GetDesc1(&mut desc) };
            if !winerror::SUCCEEDED(hr) {
                panic!("Failed on obtaining DXGI adapter description: {:?}", hr);
            }

            // We want only the hardware adapter with the largest dedicated video memory
            let hr = unsafe {
                d3d12::D3D12CreateDevice(
                    dxgi_adapter1 as *mut IUnknown,
                    d3dcommon::D3D_FEATURE_LEVEL_11_0,
                    &d3d12::ID3D12Device::uuidof(),
                    ptr::null_mut(),
                )
            };
            if (desc.Flags & dxgi::DXGI_ADAPTER_FLAG_SOFTWARE) == 0
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

pub fn create_device(adapter: ComPtr<dxgi1_6::IDXGIAdapter4>) -> ComPtr<d3d12::ID3D12Device2> {
    let mut d3d12_device2: *mut d3d12::ID3D12Device2 = ptr::null_mut();
    let hr = unsafe {
        d3d12::D3D12CreateDevice(
            adapter.as_raw() as *mut IUnknown,
            d3dcommon::D3D_FEATURE_LEVEL_11_0,
            &d3d12::ID3D12Device::uuidof(),
            &mut d3d12_device2 as *mut *mut _ as *mut *mut _,
        )
    };
    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on creating D3D12 device: {:?}", hr);
    }

    // Setup an info queue to enable debug messages in debug mode
    if cfg!(debug_assertions) {
        let mut d3d12_info_queue: *mut d3d12sdklayers::ID3D12InfoQueue = ptr::null_mut();

        // Perform QueryInterface fun, because we're not using ComPtrs.
        // TODO: Code repetition, need a function or struct to handle this
        unsafe {
            let as_unknown: &IUnknown = &*(d3d12_device2 as *mut IUnknown);
            let err = as_unknown.QueryInterface(
                &d3d12sdklayers::ID3D12InfoQueue::uuidof(),
                &mut d3d12_info_queue as *mut *mut _ as *mut *mut _,
            );
            if err < 0 {
                panic!(
                    "Failed on casting D3D12 device into a D3D12 info queue: {:?}",
                    hr
                );
            }

            (*d3d12_info_queue).SetBreakOnSeverity(
                d3d12sdklayers::D3D12_MESSAGE_SEVERITY_CORRUPTION,
                minwindef::TRUE,
            );
            (*d3d12_info_queue).SetBreakOnSeverity(
                d3d12sdklayers::D3D12_MESSAGE_SEVERITY_ERROR,
                minwindef::TRUE,
            );
            (*d3d12_info_queue).SetBreakOnSeverity(
                d3d12sdklayers::D3D12_MESSAGE_SEVERITY_WARNING,
                minwindef::TRUE,
            );

            // Suppress whole categories of messages
            let mut categories: Vec<d3d12sdklayers::D3D12_MESSAGE_CATEGORY> = vec![];

            // Suppress messages based on their severity level
            let mut severities: Vec<d3d12sdklayers::D3D12_MESSAGE_SEVERITY> =
                vec![d3d12sdklayers::D3D12_MESSAGE_SEVERITY_INFO];

            // Suppress individual messages by their ID
            let mut deny_ids: Vec<d3d12sdklayers::D3D12_MESSAGE_ID> = vec![
                d3d12sdklayers::D3D12_MESSAGE_ID_CLEARRENDERTARGETVIEW_MISMATCHINGCLEARVALUE, // I'm really not sure how to avoid this message.
                d3d12sdklayers::D3D12_MESSAGE_ID_MAP_INVALID_NULLRANGE, // This warning occurs when using capture frame while graphics debugging.
                d3d12sdklayers::D3D12_MESSAGE_ID_UNMAP_INVALID_NULLRANGE, // This warning occurs when using capture frame while graphics debugging.
            ];

            let mut filter = d3d12sdklayers::D3D12_INFO_QUEUE_FILTER {
                AllowList: mem::zeroed(),
                DenyList: d3d12sdklayers::D3D12_INFO_QUEUE_FILTER_DESC {
                    NumCategories: categories.len() as _,
                    pCategoryList: categories.as_mut_ptr(),
                    NumSeverities: severities.len() as _,
                    pSeverityList: severities.as_mut_ptr(),
                    NumIDs: deny_ids.len() as _,
                    pIDList: deny_ids.as_mut_ptr(),
                },
            };

            let hr = (*d3d12_info_queue).PushStorageFilter(&mut filter);
            if !winerror::SUCCEEDED(hr) {
                panic!("Failed adding filter to D3D12 info queue: {:?}", hr);
            }
        }
    }

    unsafe { ComPtr::from_raw(d3d12_device2) }
}

pub fn create_command_queue(
    device: ComPtr<d3d12::ID3D12Device2>,
    command_list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
) -> ComPtr<d3d12::ID3D12CommandQueue> {
    let mut d3d12_command_queue: *mut d3d12::ID3D12CommandQueue = ptr::null_mut();

    let desc = d3d12::D3D12_COMMAND_QUEUE_DESC {
        Type: command_list_type,
        Priority: d3d12::D3D12_COMMAND_QUEUE_PRIORITY_NORMAL as _,
        Flags: d3d12::D3D12_COMMAND_QUEUE_FLAG_NONE as _,
        NodeMask: 0,
    };

    let hr = unsafe {
        device.CreateCommandQueue(
            &desc,
            &d3d12::ID3D12CommandQueue::uuidof(),
            &mut d3d12_command_queue as *mut *mut _ as *mut *mut _,
        )
    };
    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on creating D3D12 command queue: {:?}", hr);
    }

    unsafe { ComPtr::from_raw(d3d12_command_queue) }
}

pub fn is_tearing_supported() -> bool {
    let mut allow_tearing: minwindef::BOOL = minwindef::FALSE;

    let mut dxgi_factory4: *mut dxgi1_4::IDXGIFactory4 = ptr::null_mut();
    let hr = unsafe {
        dxgi::CreateDXGIFactory1(
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
    return allow_tearing == minwindef::TRUE;
}

pub fn create_swap_chain(
    command_queue: ComPtr<d3d12::ID3D12CommandQueue>,
    hwnd: windef::HWND,
    width: u32,
    height: u32,
    back_buffers_count: usize,
    is_tearing_supported: bool,
) -> ComPtr<dxgi1_5::IDXGISwapChain4> {
    let mut dxgi_factory4: *mut dxgi1_4::IDXGIFactory4 = ptr::null_mut();
    let flags: u32 = if cfg!(debug_assertions) {
        dxgi1_3::DXGI_CREATE_FACTORY_DEBUG
    } else {
        0
    };

    let hr = unsafe {
        dxgi1_3::CreateDXGIFactory2(
            flags,
            &dxgi1_4::IDXGIFactory4::uuidof(),
            &mut dxgi_factory4 as *mut *mut _ as *mut *mut _,
        )
    };
    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on creating DXGI factory: {:?}", hr);
    }

    let swap_chain_desc = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
        Width: width,
        Height: height,
        Format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
        Stereo: minwindef::FALSE,
        SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: back_buffers_count as _,
        Scaling: dxgi1_2::DXGI_SCALING_STRETCH,
        SwapEffect: dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD,
        AlphaMode: dxgi1_2::DXGI_ALPHA_MODE_UNSPECIFIED,
        // It is recommended to always allow tearing if tearing support is available.
        Flags: if is_tearing_supported {
            dxgi::DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING
        } else {
            0
        },
    };

    let mut dxgi_swap_chain1: *mut dxgi1_2::IDXGISwapChain1 = ptr::null_mut();
    let mut dxgi_swap_chain4: *mut dxgi1_5::IDXGISwapChain4 = ptr::null_mut();

    let hr = unsafe {
        (*dxgi_factory4).CreateSwapChainForHwnd(
            command_queue.as_raw() as *mut IUnknown,
            hwnd,
            &swap_chain_desc,
            ptr::null(),
            ptr::null_mut(),
            &mut dxgi_swap_chain1,
        )
    };
    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on creating a swap chain: {:?}", hr);
    }

    // Disable the Alt+Enter fullscreen toggle feature. Switching to fullscreen will be handled manually.
    let hr = unsafe { (*dxgi_factory4).MakeWindowAssociation(hwnd, DXGI_MWA_NO_ALT_ENTER) };
    if !winerror::SUCCEEDED(hr) {
        panic!(
            "Failed on disabling ALT-ENTER as fullscreen toggle: {:?}",
            hr
        );
    }

    // Perform QueryInterface fun, because we're not using ComPtrs.
    // TODO: Code repetition, need a function or struct to handle this
    unsafe {
        let as_unknown: &IUnknown = &*(dxgi_swap_chain1 as *mut IUnknown);
        let err = as_unknown.QueryInterface(
            &dxgi1_5::IDXGISwapChain4::uuidof(),
            &mut dxgi_swap_chain4 as *mut *mut _ as *mut *mut _,
        );
        if err < 0 {
            panic!("Failed on casting DXGI swap chain: {:?}", hr);
        }
    }

    unsafe { ComPtr::from_raw(dxgi_swap_chain4) }
}

pub fn create_descriptor_heap(
    device: ComPtr<d3d12::ID3D12Device2>,
    descriptor_heap_type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE,
    descriptor_count: usize,
) -> ComPtr<d3d12::ID3D12DescriptorHeap> {
    let mut d3d12_descriptor_heap: *mut d3d12::ID3D12DescriptorHeap = ptr::null_mut();

    let descriptor_heap_desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
        NumDescriptors: descriptor_count as _,
        Type: descriptor_heap_type,
        Flags: d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
        NodeMask: 0,
    };

    let hr = unsafe {
        device.CreateDescriptorHeap(
            &descriptor_heap_desc,
            &d3d12::ID3D12DescriptorHeap::uuidof(),
            &mut d3d12_descriptor_heap as *mut *mut _ as *mut *mut _,
        )
    };
    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on creating a D3D12 descriptor heap: {:?}", hr);
    }

    unsafe { ComPtr::from_raw(d3d12_descriptor_heap) }
}

pub fn update_render_target_views(
    device: ComPtr<d3d12::ID3D12Device2>,
    swap_chain: ComPtr<dxgi1_5::IDXGISwapChain4>,
    descriptor_heap: ComPtr<d3d12::ID3D12DescriptorHeap>,
    back_buffers_count: usize,
    back_buffers: &mut Vec<ComPtr<d3d12::ID3D12Resource>>,
) {
    let rtv_descriptor_size =
        unsafe { device.GetDescriptorHandleIncrementSize(d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV) }
            as usize;

    let mut rtv_handle: d3d12::D3D12_CPU_DESCRIPTOR_HANDLE =
        unsafe { descriptor_heap.GetCPUDescriptorHandleForHeapStart() };

    for i in 0..back_buffers_count {
        let mut back_buffer: *mut d3d12::ID3D12Resource = ptr::null_mut();
        let hr = unsafe {
            swap_chain.GetBuffer(
                i as u32,
                &d3d12::ID3D12Resource::uuidof(),
                &mut back_buffer as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!(
                "Failed on obtaining back buffer resource {} from swap chain: {:?}",
                i, hr
            );
        }

        unsafe {
            device.CreateRenderTargetView(back_buffer, ptr::null(), rtv_handle);
            back_buffers.push(ComPtr::from_raw(back_buffer));
        }

        rtv_handle.ptr += rtv_descriptor_size;
    }
}

pub fn create_command_allocator(
    device: ComPtr<d3d12::ID3D12Device2>,
    command_list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
) -> ComPtr<d3d12::ID3D12CommandAllocator> {
    let mut command_allocator: *mut d3d12::ID3D12CommandAllocator = ptr::null_mut();

    let hr = unsafe {
        device.CreateCommandAllocator(
            command_list_type,
            &d3d12::ID3D12CommandAllocator::uuidof(),
            &mut command_allocator as *mut *mut _ as *mut *mut _,
        )
    };
    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on creating command allocator: {:?}", hr);
    }

    unsafe { ComPtr::from_raw(command_allocator) }
}

pub fn create_command_list(
    device: ComPtr<d3d12::ID3D12Device2>,
    command_allocator: ComPtr<d3d12::ID3D12CommandAllocator>,
    command_list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
) -> ComPtr<d3d12::ID3D12GraphicsCommandList> {
    let mut command_list: *mut d3d12::ID3D12GraphicsCommandList = ptr::null_mut();

    let hr = unsafe {
        device.CreateCommandList(
            0,
            command_list_type,
            command_allocator.as_raw(),
            ptr::null_mut(),
            &d3d12::ID3D12GraphicsCommandList::uuidof(),
            &mut command_list as *mut *mut _ as *mut *mut _,
        )
    };
    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on creating command list: {:?}", hr);
    }

    let hr = unsafe { (*command_list).Close() };
    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on closing command list: {:?}", hr);
    }

    unsafe { ComPtr::from_raw(command_list) }
}

pub fn create_fence(device: ComPtr<d3d12::ID3D12Device2>) -> ComPtr<d3d12::ID3D12Fence> {
    let mut fence: *mut d3d12::ID3D12Fence = ptr::null_mut();

    let hr = unsafe {
        device.CreateFence(
            0,
            d3d12::D3D12_FENCE_FLAG_NONE,
            &d3d12::ID3D12Fence::uuidof(),
            &mut fence as *mut *mut _ as *mut *mut _,
        )
    };
    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on creating fence: {:?}", hr);
    }

    unsafe { ComPtr::from_raw(fence) }
}

pub fn create_fence_event(manual_reset: bool, initial_state: bool) -> winnt::HANDLE {
    let event_handle = unsafe {
        synchapi::CreateEventA(
            ptr::null_mut(),
            manual_reset as _,
            initial_state as _,
            ptr::null(),
        )
    };

    assert!(!event_handle.is_null(), "Failed to create fence event");

    event_handle
}

pub fn signal_fence_gpu(
    command_queue: ComPtr<d3d12::ID3D12CommandQueue>,
    fence: ComPtr<d3d12::ID3D12Fence>,
    fence_value: &mut u64,
) -> u64 {
    *fence_value += 1;
    let fence_value_for_signal = *fence_value;
    let hr = unsafe { command_queue.Signal(fence.as_raw(), fence_value_for_signal) };
    if !winerror::SUCCEEDED(hr) {
        panic!(
            "Failed on signalling fence value {}: {:?}",
            fence_value_for_signal, hr
        );
    }
    fence_value_for_signal
}

pub fn wait_for_fence_value(
    fence: ComPtr<d3d12::ID3D12Fence>,
    fence_value: u64,
    fence_event: winnt::HANDLE,
) {
    wait_for_fence_value_with_timeout(fence, fence_value, fence_event, u32::max_value());
}

pub fn wait_for_fence_value_with_timeout(
    fence: ComPtr<d3d12::ID3D12Fence>,
    fence_value: u64,
    fence_event: winnt::HANDLE,
    timeout_ms: u32,
) {
    unsafe {
        if fence.GetCompletedValue() >= fence_value {
            return;
        }
    }

    let hr = unsafe { fence.SetEventOnCompletion(fence_value, fence_event) };
    if !winerror::SUCCEEDED(hr) {
        panic!("Failed on setting fence event on completion: {:?}", hr);
    }

    unsafe { synchapi::WaitForSingleObject(fence_event, timeout_ms) };
}

pub fn flush(
    command_queue: ComPtr<d3d12::ID3D12CommandQueue>,
    fence: ComPtr<d3d12::ID3D12Fence>,
    fence_value: &mut u64,
    fence_event: winnt::HANDLE,
) {
    let fence_value_for_signal = signal_fence_gpu(command_queue, fence.clone(), fence_value);
    wait_for_fence_value(fence.clone(), fence_value_for_signal, fence_event);
}

pub fn update(frame_counter: &mut u64, elapsed_time_secs: &mut f64, t0: &mut Instant) {
    *frame_counter += 1;

    let t1 = Instant::now();
    let delta_time: Duration = t1 - *t0;
    *t0 = t1;

    *elapsed_time_secs += delta_time.as_secs() as f64 + delta_time.subsec_nanos() as f64 * 1e-9;
    if *elapsed_time_secs > 1.0 {
        let fps = *frame_counter as f64 / *elapsed_time_secs;
        println!("FPS: {}", fps);

        *frame_counter = 0;
        *elapsed_time_secs = 0.0;
    }
}

pub fn render(
    command_allocators: &[ComPtr<d3d12::ID3D12CommandAllocator>],
    back_buffers: &[ComPtr<d3d12::ID3D12Resource>],
    current_back_buffer_index: &mut usize,
    command_list: ComPtr<d3d12::ID3D12GraphicsCommandList>,
    command_queue: ComPtr<d3d12::ID3D12CommandQueue>,
    rtv_descriptor_heap: ComPtr<d3d12::ID3D12DescriptorHeap>,
    rtv_descriptor_size: usize,
    swap_chain: ComPtr<dxgi1_5::IDXGISwapChain4>,
    fence: ComPtr<d3d12::ID3D12Fence>,
    frame_fence_values: &mut [u64],
    fence_value: &mut u64,
    fence_event: winnt::HANDLE,
    is_tearing_supported: bool,
    is_vsync_enabled: bool,
) {
    let command_allocator = &command_allocators[*current_back_buffer_index];
    let back_buffer = &back_buffers[*current_back_buffer_index];

    // Reset current command allocator and command list before new commands can be recorded
    unsafe {
        let hr = command_allocator.Reset();
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on resetting command allocator: {:?}", hr);
        }

        let hr = command_list.Reset(command_allocator.as_raw(), ptr::null_mut());
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on resetting command list: {:?}", hr);
        }
    }

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

        unsafe { command_list.ResourceBarrier(1, &barrier) };

        let clear_color: [f32; 4] = [0.4, 0.6, 0.9, 1.0];
        let mut rtv_handle: d3d12::D3D12_CPU_DESCRIPTOR_HANDLE =
            unsafe { rtv_descriptor_heap.GetCPUDescriptorHandleForHeapStart() };
        rtv_handle.ptr += *current_back_buffer_index * rtv_descriptor_size;

        unsafe { command_list.ClearRenderTargetView(rtv_handle, &clear_color, 0, ptr::null()) };
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

        unsafe { command_list.ResourceBarrier(1, &barrier) };

        let hr = unsafe { command_list.Close() };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on closing command list: {:?}", hr);
        }

        let command_lists = vec![command_list.as_raw() as *mut _];
        unsafe {
            command_queue.ExecuteCommandLists(command_lists.len() as _, command_lists.as_ptr())
        };

        let sync_interval = if is_vsync_enabled { 1 } else { 0 };
        let present_flags = if is_tearing_supported && !is_vsync_enabled {
            dxgi::DXGI_PRESENT_ALLOW_TEARING
        } else {
            0
        };
        let hr = unsafe { swap_chain.Present(sync_interval, present_flags) };
        if !winerror::SUCCEEDED(hr) {
            panic!(
                "Failed on presenting the swap chain's current back buffer: {:?}",
                hr
            );
        }

        // Insert a signal into the command queue with a fence value
        frame_fence_values[*current_back_buffer_index] =
            signal_fence_gpu(command_queue, fence.clone(), fence_value);

        unsafe { *current_back_buffer_index = swap_chain.GetCurrentBackBufferIndex() as _ };

        // Stall the CPU until fence value signalled is reached
        wait_for_fence_value(
            fence.clone(),
            frame_fence_values[*current_back_buffer_index],
            fence_event,
        );
    }
}

pub fn resize(
    device: ComPtr<d3d12::ID3D12Device2>,
    command_queue: ComPtr<d3d12::ID3D12CommandQueue>,
    back_buffers: &mut Vec<ComPtr<d3d12::ID3D12Resource>>,
    current_back_buffer_index: &mut usize,
    back_buffers_count: usize,
    swap_chain: ComPtr<dxgi1_5::IDXGISwapChain4>,
    descriptor_heap: ComPtr<d3d12::ID3D12DescriptorHeap>,
    fence: ComPtr<d3d12::ID3D12Fence>,
    frame_fence_values: &mut [u64],
    fence_value: &mut u64,
    fence_event: winnt::HANDLE,
    client_width: &mut u32,
    client_height: &mut u32,
    width: u32,
    height: u32,
) {
    if *client_width != width || *client_height != height {
        // Don't allow 0 size swap chain back buffers.
        *client_width = 1.max(width);
        *client_height = 1.max(height);

        // Flush the GPU queue to make sure the swap chain's back buffers
        // are not being referenced by an in-flight command list.
        flush(command_queue, fence, fence_value, fence_event);

        // Any references to the back buffers must be released
        // before the swap chain can be resized.
        while let Some(back_buffer) = back_buffers.pop() {
            std::mem::drop(back_buffer);
        }

        // Reset per-frame fence values to the fence value of the current back buffer index
        for i in 0..back_buffers_count {
            frame_fence_values[i] = frame_fence_values[*current_back_buffer_index];
        }

        let mut swap_chain_desc: dxgi::DXGI_SWAP_CHAIN_DESC = unsafe { mem::zeroed() };
        let hr = unsafe { swap_chain.GetDesc(&mut swap_chain_desc) };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on obtaining swap chain description: {:?}", hr);
        }

        let hr = unsafe {
            swap_chain.ResizeBuffers(
                back_buffers_count as _,
                *client_width,
                *client_height,
                swap_chain_desc.BufferDesc.Format,
                swap_chain_desc.Flags,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on resizing swap chain buffers: {:?}", hr);
        }

        unsafe { *current_back_buffer_index = swap_chain.GetCurrentBackBufferIndex() as _ };

        update_render_target_views(
            device,
            swap_chain,
            descriptor_heap,
            back_buffers_count,
            back_buffers,
        );
    }
}

pub fn set_fullscreen(window: &winit::Window, is_fullscreen: bool) {
    if is_fullscreen {
        // Turn off decorations
        window.set_decorations(false);
        // Make sure window is on top
        window.set_always_on_top(true);
        // Maximize window
        window.set_maximized(true);
    // Sets the window fullscreen
    //window.set_fullscreen(Some(window.get_current_monitor()));
    } else {
        // Turn off decorations
        window.set_decorations(true);
        // Make sure window is on top
        window.set_always_on_top(false);
        // Maximize window
        window.set_maximized(false);
        // Sets the window fullscreen
        //window.set_fullscreen(None);
    }
}
