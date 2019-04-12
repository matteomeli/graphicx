use super::descriptor::{CPUDescriptor, DescriptorHeapType};
use super::dxgi::Adapter;
use super::resource::Resource;
use crate::dx12::{D3DResult, Error, FeatureLevel};

use log::*;
use winapi::shared::{minwindef, winerror};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3d12sdklayers};
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::ptr;

pub struct Device {
    pub(crate) inner: ComPtr<d3d12::ID3D12Device2>,
}

impl Device {
    pub fn create(adapter: &Adapter, feature_level: FeatureLevel) -> D3DResult<Device> {
        let mut device: *mut d3d12::ID3D12Device2 = ptr::null_mut();
        let hr = unsafe {
            d3d12::D3D12CreateDevice(
                adapter.inner.as_raw() as *mut _,
                feature_level as _,
                &d3d12::ID3D12Device::uuidof(),
                &mut device as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            #[cfg(debug_assertions)]
            {
                // Setup an info queue to enable debug messages in debug mode
                let mut info_queue: *mut d3d12sdklayers::ID3D12InfoQueue = ptr::null_mut();

                let hr = unsafe {
                    (*(device as *mut IUnknown)).QueryInterface(
                        &d3d12sdklayers::ID3D12InfoQueue::uuidof(),
                        &mut info_queue as *mut *mut _ as *mut *mut _,
                    )
                };

                if !winerror::SUCCEEDED(hr) {
                    warn!(
                        "Failed on casting D3D12 device into a D3D12 info queue (code {})",
                        hr
                    );
                } else {
                    let hr = unsafe {
                        (*info_queue).SetBreakOnSeverity(
                            d3d12sdklayers::D3D12_MESSAGE_SEVERITY_CORRUPTION,
                            minwindef::TRUE,
                        )
                    };
                    if !winerror::SUCCEEDED(hr) {
                        warn!(
                            "Failed on setting break on severity in D3D12 info queue (code {})",
                            hr
                        );
                    }

                    let hr = unsafe {
                        (*info_queue).SetBreakOnSeverity(
                            d3d12sdklayers::D3D12_MESSAGE_SEVERITY_ERROR,
                            minwindef::TRUE,
                        )
                    };
                    if !winerror::SUCCEEDED(hr) {
                        warn!(
                            "Failed on setting break on severity in D3D12 info queue (code {})",
                            hr
                        );
                    }

                    let hr = unsafe {
                        (*info_queue).SetBreakOnSeverity(
                            d3d12sdklayers::D3D12_MESSAGE_SEVERITY_WARNING,
                            minwindef::TRUE,
                        )
                    };
                    if !winerror::SUCCEEDED(hr) {
                        warn!(
                            "Failed on setting break on severity in D3D12 info queue (code {})",
                            hr
                        );
                    }

                    // Suppress whole categories of messages
                    let mut categories: Vec<d3d12sdklayers::D3D12_MESSAGE_CATEGORY> = vec![];

                    // Suppress messages based on their severity level
                    let mut severities: Vec<d3d12sdklayers::D3D12_MESSAGE_SEVERITY> =
                        vec![d3d12sdklayers::D3D12_MESSAGE_SEVERITY_INFO];

                    // Suppress individual messages by their ID
                    let mut deny_ids: Vec<d3d12sdklayers::D3D12_MESSAGE_ID> = vec![
                    d3d12sdklayers::D3D12_MESSAGE_ID_CLEARRENDERTARGETVIEW_MISMATCHINGCLEARVALUE,
                    d3d12sdklayers::D3D12_MESSAGE_ID_MAP_INVALID_NULLRANGE,
                    d3d12sdklayers::D3D12_MESSAGE_ID_UNMAP_INVALID_NULLRANGE,
                    ];

                    let mut filter = d3d12sdklayers::D3D12_INFO_QUEUE_FILTER {
                        AllowList: unsafe { mem::zeroed() },
                        DenyList: d3d12sdklayers::D3D12_INFO_QUEUE_FILTER_DESC {
                            NumCategories: categories.len() as _,
                            pCategoryList: categories.as_mut_ptr(),
                            NumSeverities: severities.len() as _,
                            pSeverityList: severities.as_mut_ptr(),
                            NumIDs: deny_ids.len() as _,
                            pIDList: deny_ids.as_mut_ptr(),
                        },
                    };

                    let hr = unsafe { (*info_queue).PushStorageFilter(&mut filter) };
                    if !winerror::SUCCEEDED(hr) {
                        warn!(
                            "Failed on pushing storage filter in D3D12 info queue (code {})",
                            hr
                        );
                    }
                }
            }

            Ok(Device {
                inner: unsafe { ComPtr::from_raw(device) },
            })
        } else {
            Err(Error::CreateDeviceFailed)
        }
    }

    pub fn create_render_target_view(&self, resource: &Resource, descriptor: CPUDescriptor) {
        unsafe {
            self.inner
                .CreateRenderTargetView(resource.inner.as_raw(), ptr::null(), descriptor)
        };
    }

    pub fn get_descriptor_increment_size(&self, descriptor_heap_type: DescriptorHeapType) -> u32 {
        unsafe {
            self.inner
                .GetDescriptorHandleIncrementSize(descriptor_heap_type as _)
        }
    }
}
