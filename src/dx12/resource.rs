use bitflags::bitflags;
use winapi::um::d3d12;
use wio::com::ComPtr;

bitflags! {
    pub struct ResourceStates: u32 {
        const PRESENT = d3d12::D3D12_RESOURCE_STATE_PRESENT;
        const RENDER_TARGET = d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET;
    }
}

pub struct Resource {
    pub(crate) raw: ComPtr<d3d12::ID3D12Resource>,
}

impl Resource {
    pub fn as_ptr(&self) -> *const d3d12::ID3D12Resource {
        self.raw.as_raw()
    }

    pub fn as_mut_ptr(&self) -> *mut d3d12::ID3D12Resource {
        self.raw.as_raw()
    }
}
