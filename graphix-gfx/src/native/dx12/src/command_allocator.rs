use crate::command_list::CommandListType;
use crate::device::Device;
use crate::Result;

use winapi::shared::winerror;
use winapi::um::d3d12;
use winapi::Interface;
use wio::com::ComPtr;

use std::ptr;

pub struct CommandAllocator(pub(crate) ComPtr<d3d12::ID3D12CommandAllocator>);

impl CommandAllocator {
    pub fn new(device: &Device, command_list_type: CommandListType) -> Result<CommandAllocator> {
        let mut command_allocator: *mut d3d12::ID3D12CommandAllocator = ptr::null_mut();

        let hr = unsafe {
            device.0.CreateCommandAllocator(
                command_list_type as _,
                &d3d12::ID3D12CommandAllocator::uuidof(),
                &mut command_allocator as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(CommandAllocator(unsafe {
                ComPtr::from_raw(command_allocator)
            }))
        } else {
            Err(hr)
        }
    }

    pub fn reset(&self) -> Result<()> {
        let hr = unsafe { self.0.Reset() };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(hr)
        }
    }
}

impl Clone for CommandAllocator {
    fn clone(&self) -> Self {
        CommandAllocator(self.0.clone())
    }
}
