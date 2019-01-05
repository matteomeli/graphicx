use super::resource::ResourceStates;

use winapi::um::d3d12;

use std::ops::Range;

bitflags! {
    pub struct BarrierFlags: u32 {
        const None = d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE;
        const Beginnly = d3d12::D3D12_RESOURCE_BARRIER_FLAG_BEGIN_ONLY;
        const EndOnly = d3d12::D3D12_RESOURCE_BARRIER_FLAG_END_ONLY;
    }
}

pub struct BarrierDesc {
    pub index: usize,
    pub flags: BarrierFlags,
    pub states: Range<ResourceStates>,
}

impl BarrierDesc {
    pub fn new(index: usize, states: Range<ResourceStates>) -> Self {
        BarrierDesc {
            index,
            flags: BarrierFlags::None,
            states,
        }
    }
}
