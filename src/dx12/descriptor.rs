use super::device::Device;

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
    native: ComPtr<d3d12::ID3D12DescriptorHeap>,
}

impl DescriptorHeap {
    pub fn new(
        device: &Device,
        descriptor_type: DescriptorHeapType,
        descriptor_flags: DescriptorHeapFlags,
        descriptor_count: u32,
        node_mask: u32,
    ) -> Self {
        DescriptorHeap {
            native: device.create_descriptor_heap(
                descriptor_type,
                descriptor_flags,
                descriptor_count,
                node_mask,
            ),
        }
    }

    pub fn get_cpu_descriptor_start(&self) -> CPUDescriptor {
        unsafe { self.native.GetCPUDescriptorHandleForHeapStart() }
    }
}
