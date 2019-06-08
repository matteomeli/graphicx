use crate::device::Device;
use crate::dxgi::Format;
use crate::Result;

use bitflags::bitflags;
use winapi::shared::winerror;
use winapi::um::d3d12;
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::ptr;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum DescriptorHeapType {
    Rtv = d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
    Dsv = d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
    CbvSrvUav = d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
    Sampler = d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
}

bitflags! {
    pub struct DescriptorHeapFlags: u32 {
        const NONE = d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_NONE;
    }
}

pub type CPUDescriptor = d3d12::D3D12_CPU_DESCRIPTOR_HANDLE;
pub type GPUDescriptor = d3d12::D3D12_GPU_DESCRIPTOR_HANDLE;

pub struct DescriptorHeap(ComPtr<d3d12::ID3D12DescriptorHeap>);

impl DescriptorHeap {
    pub fn new(
        device: &Device,
        descriptor_type: DescriptorHeapType,
        descriptor_flags: DescriptorHeapFlags,
        capacity: u64,
        node_mask: u64,
    ) -> Result<DescriptorHeap> {
        let mut descriptor_heap: *mut d3d12::ID3D12DescriptorHeap = ptr::null_mut();

        let desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: capacity as _,
            Type: descriptor_type as _,
            Flags: descriptor_flags.bits() as _,
            NodeMask: node_mask as _,
        };

        let hr = unsafe {
            device.0.CreateDescriptorHeap(
                &desc,
                &d3d12::ID3D12DescriptorHeap::uuidof(),
                &mut descriptor_heap as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(DescriptorHeap(unsafe { ComPtr::from_raw(descriptor_heap) }))
        } else {
            Err(hr)
        }
    }

    pub fn get_cpu_descriptor_start(&self) -> CPUDescriptor {
        unsafe { self.0.GetCPUDescriptorHandleForHeapStart() }
    }

    pub fn get_gpu_descriptor_start(&self) -> GPUDescriptor {
        unsafe { self.0.GetGPUDescriptorHandleForHeapStart() }
    }
}

//#[repr(transparent)]
pub struct RenderTargetViewDesc(pub(crate) d3d12::D3D12_RENDER_TARGET_VIEW_DESC);

impl RenderTargetViewDesc {
    pub fn new(format: Format) -> Self {
        let desc = d3d12::D3D12_RENDER_TARGET_VIEW_DESC {
            Format: format as _,
            ViewDimension: d3d12::D3D12_RTV_DIMENSION_TEXTURE2D,
            ..unsafe { mem::zeroed() }
        };

        RenderTargetViewDesc(desc)
    }
}
