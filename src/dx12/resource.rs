use super::dxgi::SwapChain4;

use winapi::um::d3d12;
use wio::com::ComPtr;

pub struct Resource {
    native: ComPtr<d3d12::ID3D12Resource>,
}

impl Resource {
    pub fn new(swap_chain: &SwapChain4, index: u32) -> Self {
        Resource {
            native: swap_chain.get_buffer(index),
        }
    }

    pub fn as_ptr(&self) -> *const d3d12::ID3D12Resource {
        self.native.as_raw()
    }

    pub fn as_mut_ptr(&self) -> *mut d3d12::ID3D12Resource {
        self.native.as_raw()
    }
}
