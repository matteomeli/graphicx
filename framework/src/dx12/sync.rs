use super::device::Device;
use super::{D3DResult, Error};

use winapi::shared::winerror;
use winapi::um::{d3d12, handleapi, synchapi, winnt};
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

    pub fn wait(self, timeout_ms: u32) -> u32 {
        unsafe { synchapi::WaitForSingleObject(self.handle, timeout_ms) }
    }

    pub fn close(self) {
        unsafe { handleapi::CloseHandle(self.handle) };
    }
}

pub struct Fence {
    pub(crate) inner: ComPtr<d3d12::ID3D12Fence>,
}

impl Fence {
    pub fn create(device: &Device) -> D3DResult<Fence> {
        Fence::create_with_value(device, 0)
    }

    pub fn create_with_value(device: &Device, value: u64) -> D3DResult<Fence> {
        let mut fence: *mut d3d12::ID3D12Fence = ptr::null_mut();

        let hr = unsafe {
            device.inner.CreateFence(
                value,
                d3d12::D3D12_FENCE_FLAG_NONE,
                &d3d12::ID3D12Fence::uuidof(),
                &mut fence as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(Fence {
                inner: unsafe { ComPtr::from_raw(fence) },
            })
        } else {
            Err(Error::CreateFenceFailed)
        }
    }

    pub fn signal(&self, value: u64) -> D3DResult<()> {
        let hr = unsafe { self.inner.Signal(value) };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(Error::SignalFenceFailed)
        }
    }

    pub fn get_value(&self) -> u64 {
        unsafe { self.inner.GetCompletedValue() }
    }

    pub fn set_event_on_completion(&self, event: Event, value: u64) -> D3DResult<()> {
        let hr = unsafe { self.inner.SetEventOnCompletion(value, event.handle) };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(Error::SetFenceCompletionEventFailed)
        }
    }

    pub fn wait_for_value(&self, event: Event, value: u64) -> D3DResult<()> {
        self.wait_for_value_timeout(event, value, u32::max_value())
    }

    pub fn wait_for_value_timeout(
        &self,
        event: Event,
        value: u64,
        timeout_ms: u32,
    ) -> D3DResult<()> {
        if self.get_value() >= value {
            return Ok(());
        }

        self.set_event_on_completion(event, value)?;
        event.wait(timeout_ms);

        Ok(())
    }
}
