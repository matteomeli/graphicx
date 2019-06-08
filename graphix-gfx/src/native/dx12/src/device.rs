use crate::dxgi::Adapter;
use crate::heap::{CPUDescriptor, DescriptorHeapType, RenderTargetViewDesc};
use crate::resource::Resource;
use crate::Result;

use bitflags::bitflags;
use winapi::shared::winerror;
use winapi::um::{d3d12, d3d12sdklayers, d3dcommon};
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::os::raw::c_void;
use std::ptr;

bitflags! {
    pub struct MultiSampleQualityLevelFlags : u32 {
        const NONE = d3d12::D3D12_MULTISAMPLE_QUALITY_LEVELS_FLAG_NONE;
        const TILED_RESOURCE = d3d12::D3D12_MULTISAMPLE_QUALITY_LEVELS_FLAG_TILED_RESOURCE;
    }
}

#[repr(u32)]
pub enum FeatureLevel {
    L9_1 = d3dcommon::D3D_FEATURE_LEVEL_9_1,
    L9_2 = d3dcommon::D3D_FEATURE_LEVEL_9_2,
    L9_3 = d3dcommon::D3D_FEATURE_LEVEL_9_3,
    L10_0 = d3dcommon::D3D_FEATURE_LEVEL_10_0,
    L10_1 = d3dcommon::D3D_FEATURE_LEVEL_10_1,
    L11_0 = d3dcommon::D3D_FEATURE_LEVEL_11_0,
    L11_1 = d3dcommon::D3D_FEATURE_LEVEL_11_1,
    L12_0 = d3dcommon::D3D_FEATURE_LEVEL_12_0,
    L12_1 = d3dcommon::D3D_FEATURE_LEVEL_12_1,
}

#[repr(u32)]
pub enum Feature {
    Options = d3d12::D3D12_FEATURE_D3D12_OPTIONS,
    Architecture = d3d12::D3D12_FEATURE_ARCHITECTURE,
    FeatureLevels = d3d12::D3D12_FEATURE_FEATURE_LEVELS,
    FormatSupport = d3d12::D3D12_FEATURE_FORMAT_SUPPORT,
    MultisampleQualityLevels = d3d12::D3D12_FEATURE_MULTISAMPLE_QUALITY_LEVELS,
    FormatInfo = d3d12::D3D12_FEATURE_FORMAT_INFO,
    GpuVirtualAddressSupport = d3d12::D3D12_FEATURE_GPU_VIRTUAL_ADDRESS_SUPPORT,
    ShaderModel = d3d12::D3D12_FEATURE_SHADER_MODEL,
    Options1 = d3d12::D3D12_FEATURE_D3D12_OPTIONS1,
    RootSignature = d3d12::D3D12_FEATURE_ROOT_SIGNATURE,
    Architecture1 = d3d12::D3D12_FEATURE_ARCHITECTURE1,
    Options2 = d3d12::D3D12_FEATURE_D3D12_OPTIONS2,
    ShaderCache = d3d12::D3D12_FEATURE_SHADER_CACHE,
    CommandQueuePriority = d3d12::D3D12_FEATURE_COMMAND_QUEUE_PRIORITY,
}

pub struct Device(pub(crate) ComPtr<d3d12::ID3D12Device2>);

impl Device {
    pub fn new(adapter: &Adapter, feature_level: FeatureLevel) -> Result<Device> {
        let mut device: *mut d3d12::ID3D12Device2 = ptr::null_mut();
        let hr = unsafe {
            d3d12::D3D12CreateDevice(
                adapter.0.as_raw() as *mut _,
                feature_level as _,
                &d3d12::ID3D12Device::uuidof(),
                &mut device as *mut *mut _ as *mut *mut _,
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(Device(unsafe { ComPtr::from_raw(device) }))
        } else {
            Err(hr)
        }
    }

    pub fn create_render_target_view(
        &self,
        resource: &Resource,
        desc: &RenderTargetViewDesc,
        descriptor: CPUDescriptor,
    ) {
        unsafe {
            self.0
                .CreateRenderTargetView(resource.0.as_raw(), &desc.0 as *const _, descriptor)
        };
    }

    pub fn get_descriptor_increment_size(&self, descriptor_heap_type: DescriptorHeapType) -> u32 {
        unsafe {
            self.0
                .GetDescriptorHandleIncrementSize(descriptor_heap_type as _)
        }
    }

    pub fn check_feature_support(
        &self,
        feature: Feature,
        data: *mut c_void,
        size: usize,
    ) -> Result<()> {
        let hr = unsafe {
            self.0
                .CheckFeatureSupport(feature as _, data as *mut _ as *mut _, size as _)
        };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(hr)
        }
    }

    pub fn init_info_queue(&self) {
        if let Ok(info_queue) = self.0.cast::<d3d12sdklayers::ID3D12InfoQueue>() {
            unsafe {
                info_queue.SetBreakOnSeverity(d3d12sdklayers::D3D12_MESSAGE_SEVERITY_CORRUPTION, 1);
                info_queue.SetBreakOnSeverity(d3d12sdklayers::D3D12_MESSAGE_SEVERITY_ERROR, 1);
                info_queue.SetBreakOnSeverity(d3d12sdklayers::D3D12_MESSAGE_SEVERITY_WARNING, 0);
            }

            let mut categories: Vec<d3d12sdklayers::D3D12_MESSAGE_CATEGORY> = vec![];
            let mut severities: Vec<d3d12sdklayers::D3D12_MESSAGE_SEVERITY> =
                vec![d3d12sdklayers::D3D12_MESSAGE_SEVERITY_INFO];
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

            unsafe {
                info_queue.AddStorageFilterEntries(&mut filter);
            }
        }
    }

    pub fn report_live_objects(&self) {
        if let Ok(debug_device) = self.0.cast::<d3d12sdklayers::ID3D12DebugDevice>() {
            unsafe {
                debug_device.ReportLiveDeviceObjects(
                    d3d12sdklayers::D3D12_RLDO_DETAIL | d3d12sdklayers::D3D12_RLDO_IGNORE_INTERNAL,
                );
            }
        }
    }
}

impl Clone for Device {
    fn clone(&self) -> Self {
        Device(self.0.clone())
    }
}
