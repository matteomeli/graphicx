use winapi::shared::winerror;
use winapi::um::{d3d12, d3d12sdklayers};
use winapi::Interface;

use wio::com::ComPtr;

use std::ptr;

pub struct Debug {
    raw: ComPtr<d3d12sdklayers::ID3D12Debug>,
}

impl Debug {
    pub fn get_interface() -> Self {
        let mut debug_interface: *mut d3d12sdklayers::ID3D12Debug = ptr::null_mut();
        let hr = unsafe {
            d3d12::D3D12GetDebugInterface(
                &d3d12sdklayers::ID3D12Debug::uuidof(),
                &mut debug_interface as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating debug interface: {:?}", hr);
        }

        Debug {
            raw: unsafe { ComPtr::from_raw(debug_interface) },
        }
    }

    pub fn enable_layer(&self) {
        unsafe { self.raw.EnableDebugLayer() }
    }
}
