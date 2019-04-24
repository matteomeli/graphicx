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
    pub(crate) inner: ComPtr<d3d12::ID3D12Resource>,
}
