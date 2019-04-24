use crate::barrier::BarrierDesc;
use crate::descriptor::CPUDescriptor;
use crate::device::Device;
use crate::resource::Resource;
use crate::sync::{Event, Fence};
use crate::{D3DResult, Error};

use bitflags::bitflags;
use winapi::shared::winerror;
use winapi::um::d3d12;
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::ptr;

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
    pub(crate) inner: ComPtr<d3d12::ID3D12CommandAllocator>,
}

impl CommandAllocator {
    pub fn create(
        device: &Device,
        command_list_type: CommandListType,
    ) -> D3DResult<CommandAllocator> {
        let mut command_allocator: *mut d3d12::ID3D12CommandAllocator = ptr::null_mut();

        let hr = unsafe {
            device.inner.CreateCommandAllocator(
                command_list_type as _,
                &d3d12::ID3D12CommandAllocator::uuidof(),
                &mut command_allocator as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(CommandAllocator {
                inner: unsafe { ComPtr::from_raw(command_allocator) },
            })
        } else {
            Err(Error::CreateCommandAllocatorFailed)
        }
    }

    pub fn reset(&self) -> D3DResult<()> {
        let hr = unsafe { self.inner.Reset() };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(Error::ResetCommandAllocatorFailed)
        }
    }
}

pub struct CommandList {
    inner: ComPtr<d3d12::ID3D12CommandList>,
}

pub struct GraphicsCommandList {
    pub(crate) inner: ComPtr<d3d12::ID3D12GraphicsCommandList>,
}

impl GraphicsCommandList {
    pub fn create(
        device: &Device,
        command_allocator: &CommandAllocator,
        command_list_type: CommandListType,
    ) -> D3DResult<GraphicsCommandList> {
        let mut graphics_command_list: *mut d3d12::ID3D12GraphicsCommandList = ptr::null_mut();

        let hr = unsafe {
            device.inner.CreateCommandList(
                0,
                command_list_type as _,
                command_allocator.inner.as_raw(),
                ptr::null_mut(),
                &d3d12::ID3D12GraphicsCommandList::uuidof(),
                &mut graphics_command_list as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            let command_list = GraphicsCommandList {
                inner: unsafe { ComPtr::from_raw(graphics_command_list) },
            };

            command_list.close()?;

            Ok(command_list)
        } else {
            Err(Error::CreateGraphicsCommandListFailed)
        }
    }

    pub fn as_command_list(&self) -> CommandList {
        CommandList {
            inner: self.inner.clone().up::<d3d12::ID3D12CommandList>(),
        }
    }

    pub fn reset(&self, command_allocator: &CommandAllocator) -> D3DResult<()> {
        let hr = unsafe {
            self.inner
                .Reset(command_allocator.inner.as_raw(), ptr::null_mut())
        };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(Error::ResetGraphicsCommandListFailed)
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
                        pResource: resources[barrier.index].inner.as_raw(),
                        Subresource: d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: barrier.states.start.bits(),
                        StateAfter: barrier.states.end.bits(),
                    };

                resource_barrier
            })
            .collect::<Vec<_>>();

        if !transition_barriers.is_empty() {
            unsafe {
                self.inner
                    .ResourceBarrier(transition_barriers.len() as _, transition_barriers.as_ptr())
            };
        }
    }

    pub fn clear_render_target_view(&self, rtv: CPUDescriptor, clear_color: [f32; 4]) {
        unsafe {
            self.inner
                .ClearRenderTargetView(rtv, &clear_color, 0, ptr::null())
        };
    }

    pub fn close(&self) -> D3DResult<()> {
        let hr = unsafe { self.inner.Close() };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(Error::CloseGraphicsCommandListFailed)
        }
    }
}

pub struct CommandQueue {
    pub(crate) inner: ComPtr<d3d12::ID3D12CommandQueue>,
}

impl CommandQueue {
    pub fn create(
        device: &Device,
        command_list_type: CommandListType,
        priority: CommandQueuePriority,
        flags: CommandQueueFlags,
        node_mask: u32,
    ) -> D3DResult<CommandQueue> {
        let mut command_queue: *mut d3d12::ID3D12CommandQueue = ptr::null_mut();

        let desc = d3d12::D3D12_COMMAND_QUEUE_DESC {
            Type: command_list_type as _,
            Priority: priority as _,
            Flags: flags.bits() as _,
            NodeMask: node_mask,
        };

        let hr = unsafe {
            device.inner.CreateCommandQueue(
                &desc,
                &d3d12::ID3D12CommandQueue::uuidof(),
                &mut command_queue as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(CommandQueue {
                inner: unsafe { ComPtr::from_raw(command_queue) },
            })
        } else {
            Err(Error::CreateCommandQueueFailed)
        }
    }

    pub fn signal(&self, fence: &Fence, value: &mut u64) -> D3DResult<u64> {
        *value += 1;
        let hr = unsafe { self.inner.Signal(fence.inner.as_raw(), *value) };
        if winerror::SUCCEEDED(hr) {
            Ok(*value)
        } else {
            Err(Error::SignalCommandQueueFailed)
        }
    }

    pub fn flush(&self, fence: &Fence, event: Event, value: &mut u64) -> D3DResult<()> {
        let value = self.signal(fence, value)?;
        fence.wait_for_value(event, value)?;
        Ok(())
    }

    pub fn execute(&self, command_lists: &[CommandList]) {
        let lists: Vec<*mut d3d12::ID3D12CommandList> = command_lists
            .iter()
            .map(|command_list| command_list.inner.as_raw())
            .collect();
        unsafe {
            self.inner
                .ExecuteCommandLists(lists.len() as _, lists.as_ptr())
        };
    }
}
