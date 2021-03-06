use crate::resource::ResourceState;

use std::ops::Range;

use bitflags::bitflags;
use winapi::um::d3d12;

bitflags! {
    pub struct BarrierFlags: u32 {
        const NONE = d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE;
        const BEGIN_ONLY = d3d12::D3D12_RESOURCE_BARRIER_FLAG_BEGIN_ONLY;
        const END_ONLY = d3d12::D3D12_RESOURCE_BARRIER_FLAG_END_ONLY;
    }
}

#[derive(Clone)]
pub struct BarrierDesc {
    pub index: usize,
    pub flags: BarrierFlags,
    pub states: Range<ResourceState>,
}

impl BarrierDesc {
    pub fn new(index: usize, states: Range<ResourceState>) -> Self {
        BarrierDesc {
            index,
            flags: BarrierFlags::NONE,
            states,
        }
    }
}
