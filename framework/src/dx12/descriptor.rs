use super::device::Device;
use super::{D3DResult, Error};

use bitflags::bitflags;
use winapi::shared::winerror;
use winapi::um::d3d12;
use winapi::Interface;
use wio::com::ComPtr;

use std::ptr;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum DescriptorHeapType {
    RTV = d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
}

bitflags! {
    pub struct DescriptorHeapFlags: u32 {
        const NONE = d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_NONE;
    }
}

pub type CPUDescriptor = d3d12::D3D12_CPU_DESCRIPTOR_HANDLE;

pub struct DescriptorHeap {
    pub(crate) inner: ComPtr<d3d12::ID3D12DescriptorHeap>,
}

impl DescriptorHeap {
    pub fn create(
        device: &Device,
        descriptor_type: DescriptorHeapType,
        descriptor_flags: DescriptorHeapFlags,
        descriptor_count: u32,
        node_mask: u32,
    ) -> D3DResult<DescriptorHeap> {
        let mut descriptor_heap: *mut d3d12::ID3D12DescriptorHeap = ptr::null_mut();

        let desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
            NumDescriptors: descriptor_count,
            Type: descriptor_type as _,
            Flags: descriptor_flags.bits() as _,
            NodeMask: node_mask,
        };

        let hr = unsafe {
            device.inner.CreateDescriptorHeap(
                &desc,
                &d3d12::ID3D12DescriptorHeap::uuidof(),
                &mut descriptor_heap as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(DescriptorHeap {
                inner: unsafe { ComPtr::from_raw(descriptor_heap) },
            })
        } else {
            Err(Error::CreateDescriptorHeapFailed)
        }
    }

    pub fn get_cpu_descriptor_start(&self) -> CPUDescriptor {
        unsafe { self.inner.GetCPUDescriptorHandleForHeapStart() }
    }
}