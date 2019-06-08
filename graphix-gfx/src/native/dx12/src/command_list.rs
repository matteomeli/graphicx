use crate::barrier::BarrierDesc;
use crate::command_allocator::CommandAllocator;
use crate::device::Device;
use crate::heap::CPUDescriptor;
use crate::resource::Resource;
use crate::Result;

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
    Compute = d3d12::D3D12_COMMAND_LIST_TYPE_COMPUTE,
    Copy = d3d12::D3D12_COMMAND_LIST_TYPE_COPY,
}

pub struct CommandList(pub(crate) ComPtr<d3d12::ID3D12CommandList>);

pub struct GraphicsCommandList(pub(crate) ComPtr<d3d12::ID3D12GraphicsCommandList>);

impl GraphicsCommandList {
    pub fn new(
        device: &Device,
        command_allocator: &CommandAllocator,
        command_list_type: CommandListType,
    ) -> Result<GraphicsCommandList> {
        let mut graphics_command_list: *mut d3d12::ID3D12GraphicsCommandList = ptr::null_mut();

        let hr = unsafe {
            device.0.CreateCommandList(
                0,
                command_list_type as _,
                command_allocator.0.as_raw(),
                ptr::null_mut(),
                &d3d12::ID3D12GraphicsCommandList::uuidof(),
                &mut graphics_command_list as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(GraphicsCommandList(unsafe {
                ComPtr::from_raw(graphics_command_list)
            }))
        } else {
            Err(hr)
        }
    }

    pub fn as_command_list(&self) -> CommandList {
        CommandList(self.0.clone().up::<d3d12::ID3D12CommandList>())
    }

    pub fn reset(&self, command_allocator: &CommandAllocator) -> Result<()> {
        let hr = unsafe { self.0.Reset(command_allocator.0.as_raw(), ptr::null_mut()) };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(hr)
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
                        pResource: resources[barrier.index].0.as_raw(),
                        Subresource: d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: barrier.states.start.bits(),
                        StateAfter: barrier.states.end.bits(),
                    };

                resource_barrier
            })
            .collect::<Vec<_>>();

        if !transition_barriers.is_empty() {
            unsafe {
                self.0
                    .ResourceBarrier(transition_barriers.len() as _, transition_barriers.as_ptr())
            };
        }
    }

    pub fn clear_render_target_view(&self, rtv: CPUDescriptor, clear_color: [f32; 4]) {
        unsafe {
            self.0
                .ClearRenderTargetView(rtv, &clear_color, 0, ptr::null())
        };
    }

    pub fn close(&self) -> Result<()> {
        let hr = unsafe { self.0.Close() };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(hr)
        }
    }
}

impl Clone for GraphicsCommandList {
    fn clone(&self) -> Self {
        GraphicsCommandList(self.0.clone())
    }
}
