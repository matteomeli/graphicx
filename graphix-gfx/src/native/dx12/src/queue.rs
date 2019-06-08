use crate::command_list::{CommandList, CommandListType};
use crate::device::Device;
use crate::sync::Fence;
use crate::Result;

use bitflags::bitflags;
use winapi::shared::winerror;
use winapi::um::d3d12;
use winapi::Interface;
use wio::com::ComPtr;

use std::ptr;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum CommandQueuePriority {
    Normal = d3d12::D3D12_COMMAND_QUEUE_PRIORITY_NORMAL,
}

bitflags! {
    pub struct CommandQueueFlags: u32 {
        const NONE = d3d12::D3D12_COMMAND_QUEUE_FLAG_NONE;
        const DISABLE_GPU_TIMEOUT = d3d12::D3D12_COMMAND_QUEUE_FLAG_DISABLE_GPU_TIMEOUT;
    }
}

pub struct CommandQueue(pub(crate) ComPtr<d3d12::ID3D12CommandQueue>);

impl CommandQueue {
    pub fn new(
        device: &Device,
        queue_type: CommandListType,
        priority: CommandQueuePriority,
        flags: CommandQueueFlags,
    ) -> Result<CommandQueue> {
        let mut command_queue: *mut d3d12::ID3D12CommandQueue = ptr::null_mut();

        let desc = d3d12::D3D12_COMMAND_QUEUE_DESC {
            Type: queue_type as _,
            Priority: priority as _,
            Flags: flags.bits() as _,
            NodeMask: 0,
        };

        let hr = unsafe {
            device.0.CreateCommandQueue(
                &desc,
                &d3d12::ID3D12CommandQueue::uuidof(),
                &mut command_queue as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(CommandQueue(unsafe { ComPtr::from_raw(command_queue) }))
        } else {
            Err(hr)
        }
    }

    pub fn signal(&self, fence: &Fence, value: u64) -> Result<()> {
        let hr = unsafe { self.0.Signal(fence.0.as_raw(), value) };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(hr)
        }
    }

    pub fn execute_command_lists(&self, command_lists: &[CommandList]) {
        let lists: Vec<*mut d3d12::ID3D12CommandList> = command_lists
            .iter()
            .map(|command_list| command_list.0.as_raw())
            .collect();
        unsafe { self.0.ExecuteCommandLists(lists.len() as _, lists.as_ptr()) };
    }
}
