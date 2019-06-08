use crate::device::Device;
use crate::Result;

use winapi::shared::winerror;
use winapi::um::{d3d12, handleapi, synchapi, winbase, winnt};
use winapi::Interface;
use wio::com::ComPtr;

use std::ptr;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Event {
    handle: winnt::HANDLE,
}

impl Event {
    pub fn new(manual_reset: bool, initial_state: bool) -> Self {
        Event {
            handle: unsafe {
                synchapi::CreateEventA(
                    ptr::null_mut(),
                    manual_reset as _,
                    initial_state as _,
                    ptr::null(),
                )
            },
        }
    }

    pub fn wait(self, timeout_ms: u32) -> bool {
        let hr = unsafe { synchapi::WaitForSingleObject(self.handle, timeout_ms) };
        match hr {
            winbase::WAIT_OBJECT_0 => true,
            winbase::WAIT_ABANDONED => true,
            winerror::WAIT_TIMEOUT => false,
            _ => panic!("Unexpected event wait result"),
        }
    }

    pub fn close(self) {
        unsafe { handleapi::CloseHandle(self.handle) };
    }
}

pub struct Fence(pub(crate) ComPtr<d3d12::ID3D12Fence>);

impl Fence {
    pub fn new(device: &Device) -> Result<Fence> {
        Fence::new_with_value(device, 0)
    }

    pub fn new_with_value(device: &Device, value: u64) -> Result<Fence> {
        let mut fence: *mut d3d12::ID3D12Fence = ptr::null_mut();

        let hr = unsafe {
            device.0.CreateFence(
                value,
                d3d12::D3D12_FENCE_FLAG_NONE,
                &d3d12::ID3D12Fence::uuidof(),
                &mut fence as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(Fence(unsafe { ComPtr::from_raw(fence) }))
        } else {
            Err(hr)
        }
    }

    pub fn signal(&self, value: u64) -> Result<()> {
        let hr = unsafe { self.0.Signal(value) };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(hr)
        }
    }

    pub fn reset(&self) -> Result<()> {
        self.signal(0)
    }

    pub fn get_value(&self) -> u64 {
        unsafe { self.0.GetCompletedValue() }
    }

    pub fn set_event_on_completion(&self, event: Event, value: u64) -> Result<()> {
        let hr = unsafe { self.0.SetEventOnCompletion(value, event.handle) };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(hr)
        }
    }

    pub fn wait_for_value(&self, event: Event, value: u64) -> Result<bool> {
        self.wait_for_value_with_timeout(event, value, u64::max_value())
    }

    pub fn wait_for_value_with_timeout(
        &self,
        event: Event,
        value: u64,
        timeout_ns: u64,
    ) -> Result<bool> {
        if self.get_value() >= value {
            return Ok(true);
        }

        self.set_event_on_completion(event, value)?;
        Ok(event.wait(timeout_ns as _))
    }
}
