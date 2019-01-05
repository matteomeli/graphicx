use super::command::{CommandAllocator, CommandListType, CommandQueueFlags, CommandQueuePriority};
use super::descriptor::{CPUDescriptor, DescriptorHeapFlags, DescriptorHeapType};
use super::dxgi::Adapter4;
use super::resource::Resource;

use winapi::shared::{minwindef, winerror};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3d12sdklayers, d3dcommon};
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::ptr;

pub struct Device {
    native: ComPtr<d3d12::ID3D12Device2>,
}

impl Device {
    pub fn new(adapter: &Adapter4) -> Self {
        let mut device: *mut d3d12::ID3D12Device2 = ptr::null_mut();
        let hr = unsafe {
            d3d12::D3D12CreateDevice(
                adapter.as_mut_ptr() as *mut IUnknown,
                d3dcommon::D3D_FEATURE_LEVEL_11_0,
                &d3d12::ID3D12Device::uuidof(),
                &mut device as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating D3D12 device: {:?}", hr);
        }

        // Setup an info queue to enable debug messages in debug mode
        if cfg!(debug_assertions) {
            let mut info_queue: *mut d3d12sdklayers::ID3D12InfoQueue = ptr::null_mut();

            // Perform QueryInterface fun, because we're not using ComPtrs.
            // TODO: Code repetition, need a function or struct to handle this
            unsafe {
                let as_unknown: &IUnknown = &*(device as *mut IUnknown);
                let err = as_unknown.QueryInterface(
                    &d3d12sdklayers::ID3D12InfoQueue::uuidof(),
                    &mut info_queue as *mut *mut _ as *mut *mut _,
                );
                if err < 0 {
                    panic!(
                        "Failed on casting D3D12 device into a D3D12 info queue: {:?}",
                        hr
                    );
                }

                (*info_queue).SetBreakOnSeverity(
                    d3d12sdklayers::D3D12_MESSAGE_SEVERITY_CORRUPTION,
                    minwindef::TRUE,
                );
                (*info_queue).SetBreakOnSeverity(
                    d3d12sdklayers::D3D12_MESSAGE_SEVERITY_ERROR,
                    minwindef::TRUE,
                );
                (*info_queue).SetBreakOnSeverity(
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
                    d3d12sdklayers::D3D12_MESSAGE_ID_CLEARRENDERTARGETVIEW_MISMATCHINGCLEARVALUE,
                    d3d12sdklayers::D3D12_MESSAGE_ID_MAP_INVALID_NULLRANGE,
                    d3d12sdklayers::D3D12_MESSAGE_ID_UNMAP_INVALID_NULLRANGE,
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

                let hr = (*info_queue).PushStorageFilter(&mut filter);
                if !winerror::SUCCEEDED(hr) {
                    panic!("Failed adding filter to D3D12 info queue: {:?}", hr);
                }
            }
        }

        Device {
            native: unsafe { ComPtr::from_raw(device) },
        }
    }

    pub fn create_fence(&self, initial_value: u64) -> ComPtr<d3d12::ID3D12Fence> {
        let mut fence: *mut d3d12::ID3D12Fence = ptr::null_mut();

        let hr = unsafe {
            self.native.CreateFence(
                initial_value,
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

    pub fn create_descriptor_heap(
        &self,
        descriptor_type: DescriptorHeapType,
        descriptor_flags: DescriptorHeapFlags,
        descriptor_count: u32,
        node_mask: u32,
    ) -> ComPtr<d3d12::ID3D12DescriptorHeap> {
        let mut descriptor_heap: *mut d3d12::ID3D12DescriptorHeap = ptr::null_mut();

        let desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: descriptor_count,
            Type: descriptor_type as _,
            Flags: descriptor_flags.bits() as _,
            NodeMask: node_mask,
        };

        let hr = unsafe {
            self.native.CreateDescriptorHeap(
                &desc,
                &d3d12::ID3D12DescriptorHeap::uuidof(),
                &mut descriptor_heap as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating a D3D12 descriptor heap: {:?}", hr);
        }

        unsafe { ComPtr::from_raw(descriptor_heap) }
    }

    pub fn create_command_queue(
        &self,
        command_list_type: CommandListType,
        priority: CommandQueuePriority,
        flags: CommandQueueFlags,
        node_mask: u32,
    ) -> ComPtr<d3d12::ID3D12CommandQueue> {
        let mut command_queue: *mut d3d12::ID3D12CommandQueue = ptr::null_mut();

        let desc = d3d12::D3D12_COMMAND_QUEUE_DESC {
            Type: command_list_type as _,
            Priority: priority as _,
            Flags: flags.bits() as _,
            NodeMask: node_mask,
        };

        let hr = unsafe {
            self.native.CreateCommandQueue(
                &desc,
                &d3d12::ID3D12CommandQueue::uuidof(),
                &mut command_queue as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating D3D12 command queue: {:?}", hr);
        }

        unsafe { ComPtr::from_raw(command_queue) }
    }

    pub fn create_command_allocator(
        &self,
        command_list_type: CommandListType,
    ) -> ComPtr<d3d12::ID3D12CommandAllocator> {
        let mut command_allocator: *mut d3d12::ID3D12CommandAllocator = ptr::null_mut();

        let hr = unsafe {
            self.native.CreateCommandAllocator(
                command_list_type as _,
                &d3d12::ID3D12CommandAllocator::uuidof(),
                &mut command_allocator as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating command allocator: {:?}", hr);
        }

        unsafe { ComPtr::from_raw(command_allocator) }
    }

    pub fn create_graphics_command_list(
        &self,
        command_allocator: &CommandAllocator,
        command_list_type: CommandListType,
    ) -> ComPtr<d3d12::ID3D12GraphicsCommandList> {
        let mut graphics_command_list: *mut d3d12::ID3D12GraphicsCommandList = ptr::null_mut();

        let hr = unsafe {
            self.native.CreateCommandList(
                0,
                command_list_type as _,
                command_allocator.as_mut_ptr(),
                ptr::null_mut(),
                &d3d12::ID3D12GraphicsCommandList::uuidof(),
                &mut graphics_command_list as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating command list: {:?}", hr);
        }

        let hr = unsafe { (*graphics_command_list).Close() };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on closing command list: {:?}", hr);
        }

        unsafe { ComPtr::from_raw(graphics_command_list) }
    }

    pub fn create_render_target_view(&self, resource: &Resource, descriptor: CPUDescriptor) {
        unsafe {
            self.native
                .CreateRenderTargetView(resource.as_mut_ptr(), ptr::null(), descriptor)
        };
    }

    pub fn get_descriptor_increment_size(&self, descriptor_heap_type: DescriptorHeapType) -> u32 {
        unsafe {
            self.native
                .GetDescriptorHandleIncrementSize(descriptor_heap_type as _)
        }
    }
}
