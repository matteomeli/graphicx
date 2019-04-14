use super::descriptor::{CPUDescriptor, DescriptorHeapType};
use super::dxgi::Adapter;
use super::resource::Resource;
use crate::dx12::{D3DResult, Error, Format};

use bitflags::bitflags;
use log::*;
use winapi::shared::{minwindef, winerror};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3d12sdklayers, d3dcommon};
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::ptr;

bitflags! {
    pub struct MultiSampleQualityLevelFlags : u32 {
        const NONE = d3d12::D3D12_MULTISAMPLE_QUALITY_LEVELS_FLAG_NONE;
        const TILED_RESOURCE = d3d12::D3D12_MULTISAMPLE_QUALITY_LEVELS_FLAG_TILED_RESOURCE;
    }
}

pub struct Device {
    pub(crate) inner: ComPtr<d3d12::ID3D12Device2>,
    _feature_level: d3dcommon::D3D_FEATURE_LEVEL,
}

impl Device {
    pub fn create(adapter: &Adapter) -> D3DResult<Device> {
        let mut device: *mut d3d12::ID3D12Device2 = ptr::null_mut();
        let hr = unsafe {
            d3d12::D3D12CreateDevice(
                adapter.inner.as_raw() as *mut _,
                d3dcommon::D3D_FEATURE_LEVEL_11_0,
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

                    unsafe {
                        (*info_queue).Release();
                    }
                }
            }

            // Check actual feature level obtained
            let levels: [d3dcommon::D3D_FEATURE_LEVEL; 4] = [
                d3dcommon::D3D_FEATURE_LEVEL_12_1,
                d3dcommon::D3D_FEATURE_LEVEL_12_0,
                d3dcommon::D3D_FEATURE_LEVEL_11_1,
                d3dcommon::D3D_FEATURE_LEVEL_11_0,
            ];
            let mut feature_levels = d3d12::D3D12_FEATURE_DATA_FEATURE_LEVELS {
                NumFeatureLevels: 4,
                pFeatureLevelsRequested: levels.as_ptr(),
                MaxSupportedFeatureLevel: d3dcommon::D3D_FEATURE_LEVEL_11_0,
            };
            let mut feature_level = d3dcommon::D3D_FEATURE_LEVEL_11_0;
            let hr = unsafe {
                (*device).CheckFeatureSupport(
                    d3d12::D3D12_FEATURE_FEATURE_LEVELS,
                    &mut feature_levels as *mut _ as *mut _,
                    mem::size_of::<d3d12::D3D12_FEATURE_DATA_FEATURE_LEVELS>() as _,
                )
            };
            if winerror::SUCCEEDED(hr) {
                feature_level = feature_levels.MaxSupportedFeatureLevel;
            }

            Ok(Device {
                inner: unsafe { ComPtr::from_raw(device) },
                _feature_level: feature_level,
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

    pub fn get_msaa_quality(
        &self,
        format: Format,
        sample_count: u32,
        flags: MultiSampleQualityLevelFlags,
    ) -> D3DResult<u32> {
        let mut multisample_feature_data = d3d12::D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS {
            Format: format as _,
            SampleCount: sample_count,
            Flags: flags.bits() as _,
            NumQualityLevels: 0,
        };
        let hr = unsafe {
            self.inner.CheckFeatureSupport(
                d3d12::D3D12_FEATURE_MULTISAMPLE_QUALITY_LEVELS,
                &mut multisample_feature_data as *mut _ as *mut _,
                mem::size_of::<d3d12::D3D12_FEATURE_DATA_MULTISAMPLE_QUALITY_LEVELS>() as _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(multisample_feature_data.NumQualityLevels)
        } else {
            Err(Error::MultiSamplingSupportCheckFailed)
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            // Debug tracking alive objects
            let mut debug_device: *mut d3d12sdklayers::ID3D12DebugDevice = ptr::null_mut();
            let hr = unsafe {
                (*(self.inner.as_raw() as *mut IUnknown)).QueryInterface(
                    &d3d12sdklayers::ID3D12DebugDevice::uuidof(),
                    &mut debug_device as *mut *mut _ as *mut *mut _,
                )
            };
            if winerror::SUCCEEDED(hr) {
                unsafe {
                    (*debug_device).ReportLiveDeviceObjects(
                        d3d12sdklayers::D3D12_RLDO_DETAIL
                            | d3d12sdklayers::D3D12_RLDO_IGNORE_INTERNAL,
                    );
                    (*debug_device).Release();
                }
            }
        }
    }
}
