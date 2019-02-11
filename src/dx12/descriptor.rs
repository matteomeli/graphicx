use bitflags::bitflags;
use winapi::um::d3d12;
use wio::com::ComPtr;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum DescriptorHeapType {
    RTV = d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
}

bitflags! {
    pub struct DescriptorHeapFlags: u32 {
        const None = d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_NONE;
    }
}

pub type CPUDescriptor = d3d12::D3D12_CPU_DESCRIPTOR_HANDLE;

pub struct DescriptorHeap {
    pub(crate) raw: ComPtr<d3d12::ID3D12DescriptorHeap>,
}

impl DescriptorHeap {
    pub fn get_cpu_descriptor_start(&self) -> CPUDescriptor {
        unsafe { self.raw.GetCPUDescriptorHandleForHeapStart() }
    }
}
