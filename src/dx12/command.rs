use super::barrier::BarrierDesc;
use super::descriptor::CPUDescriptor;
use super::resource::Resource;
use super::sync::{Event, Fence};

use std::mem;
use std::ptr;

use bitflags::bitflags;
use winapi::shared::winerror;
use winapi::um::d3d12;
use wio::com::ComPtr;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum CommandListType {
    Direct = d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum CommandQueuePriority {
    Normal = d3d12::D3D12_COMMAND_QUEUE_PRIORITY_NORMAL,
}

bitflags! {
    pub struct CommandQueueFlags: u32 {
        const NONE = d3d12::D3D12_COMMAND_QUEUE_FLAG_NONE;
    }
}

pub struct CommandAllocator {
    pub(crate) raw: ComPtr<d3d12::ID3D12CommandAllocator>,
}

impl CommandAllocator {
    pub fn as_ptr(&self) -> *const d3d12::ID3D12CommandAllocator {
        self.raw.as_raw()
    }

    pub fn as_mut_ptr(&self) -> *mut d3d12::ID3D12CommandAllocator {
        self.raw.as_raw()
    }

    pub fn reset(&self) {
        let hr = unsafe { self.raw.Reset() };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on resetting command allocator: {:?}", hr);
        }
    }
}

pub struct CommandList {
    raw: ComPtr<d3d12::ID3D12CommandList>,
}

impl CommandList {
    pub fn as_ptr(&self) -> *const d3d12::ID3D12CommandList {
        self.raw.as_raw()
    }

    pub fn as_mut_ptr(&self) -> *mut d3d12::ID3D12CommandList {
        self.raw.as_raw()
    }
}

pub struct GraphicsCommandList {
    pub(crate) raw: ComPtr<d3d12::ID3D12GraphicsCommandList>,
}

impl GraphicsCommandList {
    pub fn as_command_list(&self) -> CommandList {
        CommandList {
            raw: self.raw.clone().up::<d3d12::ID3D12CommandList>(),
        }
    }

    pub fn as_ptr(&self) -> *const d3d12::ID3D12GraphicsCommandList {
        self.raw.as_raw()
    }

    pub fn as_mut_ptr(&self) -> *mut d3d12::ID3D12GraphicsCommandList {
        self.raw.as_raw()
    }

    pub fn reset(&self, command_allocator: &CommandAllocator) {
        let hr = unsafe {
            self.raw
                .Reset(command_allocator.as_mut_ptr(), ptr::null_mut())
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on resetting command list: {:?}", hr);
        }
    }

    pub fn insert_transition_barriers(&self, barriers: &[BarrierDesc], resources: &[Resource]) {
        let transition_barriers = barriers
            .iter()
            .map(|barrier| {
                let mut resource_barrier = d3d12::D3D12_RESOURCE_BARRIER {
                    Type: d3d12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                    Flags: barrier.flags.bits(),
                    u: unsafe { mem::zeroed() },
                };

                *unsafe { resource_barrier.u.Transition_mut() } =
                    d3d12::D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: resources[barrier.index].as_mut_ptr(),
                        Subresource: d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: barrier.states.start.bits(),
                        StateAfter: barrier.states.end.bits(),
                    };

                resource_barrier
            })
            .collect::<Vec<_>>();

        if !transition_barriers.is_empty() {
            unsafe {
                self.raw
                    .ResourceBarrier(transition_barriers.len() as _, transition_barriers.as_ptr())
            };
        }
    }

    pub fn clear_render_target_view(&self, rtv: CPUDescriptor, clear_color: [f32; 4]) {
        unsafe {
            self.raw
                .ClearRenderTargetView(rtv, &clear_color, 0, ptr::null())
        };
    }

    pub fn close(&self) {
        let hr = unsafe { self.raw.Close() };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on closing command list: {:?}", hr);
        }
    }
}

pub struct CommandQueue {
    pub(crate) raw: ComPtr<d3d12::ID3D12CommandQueue>,
}

impl CommandQueue {
    pub fn as_ptr(&self) -> *const d3d12::ID3D12CommandQueue {
        self.raw.as_raw()
    }

    pub fn as_mut_ptr(&self) -> *mut d3d12::ID3D12CommandQueue {
        self.raw.as_raw()
    }

    pub fn signal(&self, fence: &Fence, value: &mut u64) -> u64 {
        *value += 1;
        let hr = unsafe { self.raw.Signal(fence.as_mut_ptr(), *value) };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on signalling fence value {}: {:?}", *value, hr);
        }
        *value
    }

    pub fn flush(&self, fence: &Fence, event: Event, value: &mut u64) {
        let value = self.signal(fence, value);
        fence.wait_for_value(event, value);
    }

    pub fn execute(&self, command_lists: &[CommandList]) {
        let lists: Vec<*mut d3d12::ID3D12CommandList> = command_lists
            .iter()
            .map(|command_list| command_list.as_mut_ptr())
            .collect();
        unsafe {
            self.raw
                .ExecuteCommandLists(lists.len() as _, lists.as_ptr())
        };
    }
}
