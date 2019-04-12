use super::factory::Factory;
use super::{DxgiVersion, GpuPreference};
use crate::dx12::{D3DResult, Error};

use winapi::shared::{dxgi, dxgi1_2, dxgi1_4, dxgi1_6, winerror};
use winapi::Interface;
use wio::com::ComPtr;

use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::ptr;

#[derive(Copy, Clone, Debug)]
pub enum DeviceType {
    DiscreteGpu,
    VirtualGpu,
}

#[derive(Clone, Debug)]
pub struct AdapterInfo {
    pub name: String,
    pub vendor: u32,
    pub device: u32,
    pub video_memory: usize,
    pub device_type: DeviceType,
}

pub struct Adapter {
    pub(crate) inner: ComPtr<dxgi::IDXGIAdapter>,
    pub info: AdapterInfo,
}

impl Adapter {
    pub fn enumerate(
        factory: &Factory,
        index: u32,
        preference: GpuPreference,
    ) -> D3DResult<Adapter> {
        let adapter = if factory.version < DxgiVersion::Dxgi1_6 {
            Adapter::enumerate_adapters1(factory, index)
        } else {
            Adapter::enumerate_adapters_by_gpu_preference(factory, index, preference)
        };

        adapter.map(move |a| {
            let info = Adapter::get_info(a.as_raw(), factory.version);
            Adapter { inner: a, info }
        })
    }

    pub fn enumerate_warp(factory: &Factory) -> D3DResult<Adapter> {
        if factory.version < DxgiVersion::Dxgi1_4 {
            Err(Error::WarpAdapterNotSupported)
        } else {
            let mut adapter: *mut dxgi::IDXGIAdapter = ptr::null_mut();
            let hr = unsafe {
                (*(factory.inner.as_raw() as *mut dxgi1_4::IDXGIFactory4)).EnumWarpAdapter(
                    &dxgi::IDXGIAdapter1::uuidof(),
                    &mut adapter as *mut *mut _ as *mut *mut _,
                )
            };

            if winerror::SUCCEEDED(hr) {
                let info = Adapter::get_info(adapter, factory.version);
                Ok(Adapter {
                    inner: unsafe { ComPtr::from_raw(adapter) },
                    info,
                })
            } else {
                Err(Error::AdapterNotFound)
            }
        }
    }

    fn enumerate_adapters1(factory: &Factory, index: u32) -> D3DResult<ComPtr<dxgi::IDXGIAdapter>> {
        let mut adapter: *mut dxgi::IDXGIAdapter = ptr::null_mut();

        let hr = unsafe {
            (*(factory.inner.as_raw() as *mut dxgi::IDXGIFactory1))
                .EnumAdapters1(index, &mut adapter as *mut *mut _ as *mut *mut _)
        };

        if winerror::SUCCEEDED(hr) {
            Ok(unsafe { ComPtr::from_raw(adapter) })
        } else {
            Err(Error::AdapterNotFound)
        }
    }

    fn enumerate_adapters_by_gpu_preference(
        factory: &Factory,
        index: u32,
        preference: GpuPreference,
    ) -> D3DResult<ComPtr<dxgi::IDXGIAdapter>> {
        let mut adapter: *mut dxgi::IDXGIAdapter = ptr::null_mut();

        let hr = unsafe {
            (*(factory.inner.as_raw() as *mut dxgi1_6::IDXGIFactory6)).EnumAdapterByGpuPreference(
                index,
                preference as _,
                &dxgi1_6::IDXGIAdapter4::uuidof(),
                &mut adapter as *mut *mut _ as *mut *mut _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(unsafe { ComPtr::from_raw(adapter) })
        } else {
            Err(Error::AdapterNotFound)
        }
    }

    fn get_info(adapter: *mut dxgi::IDXGIAdapter, version: DxgiVersion) -> AdapterInfo {
        match version {
            DxgiVersion::Dxgi1_0 => {
                let mut desc: dxgi::DXGI_ADAPTER_DESC1 = unsafe { mem::zeroed() };
                unsafe {
                    (*(adapter as *mut dxgi::IDXGIAdapter1)).GetDesc1(&mut desc);
                }

                let device_name = {
                    let len = desc.Description.iter().take_while(|&&c| c != 0).count();
                    let name = <OsString as OsStringExt>::from_wide(&desc.Description[..len]);
                    name.to_string_lossy().into_owned()
                };

                AdapterInfo {
                    name: device_name,
                    vendor: desc.VendorId,
                    device: desc.DeviceId,
                    video_memory: desc.DedicatedVideoMemory,
                    device_type: if (desc.Flags & dxgi::DXGI_ADAPTER_FLAG_SOFTWARE) == 0 {
                        DeviceType::DiscreteGpu
                    } else {
                        DeviceType::VirtualGpu
                    },
                }
            }
            DxgiVersion::Dxgi1_2
            | DxgiVersion::Dxgi1_3
            | DxgiVersion::Dxgi1_4
            | DxgiVersion::Dxgi1_5 => {
                let mut desc: dxgi1_2::DXGI_ADAPTER_DESC2 = unsafe { mem::zeroed() };
                unsafe {
                    (*(adapter as *mut dxgi1_2::IDXGIAdapter2)).GetDesc2(&mut desc);
                }

                let device_name = {
                    let len = desc.Description.iter().take_while(|&&c| c != 0).count();
                    let name = <OsString as OsStringExt>::from_wide(&desc.Description[..len]);
                    name.to_string_lossy().into_owned()
                };

                AdapterInfo {
                    name: device_name,
                    vendor: desc.VendorId,
                    device: desc.DeviceId,
                    video_memory: desc.DedicatedVideoMemory,
                    device_type: if (desc.Flags & dxgi::DXGI_ADAPTER_FLAG_SOFTWARE) == 0 {
                        DeviceType::DiscreteGpu
                    } else {
                        DeviceType::VirtualGpu
                    },
                }
            }
            DxgiVersion::Dxgi1_6 => {
                let mut desc: dxgi1_6::DXGI_ADAPTER_DESC3 = unsafe { mem::zeroed() };
                unsafe {
                    (*(adapter as *mut dxgi1_6::IDXGIAdapter4)).GetDesc3(&mut desc);
                }

                let device_name = {
                    let len = desc.Description.iter().take_while(|&&c| c != 0).count();
                    let name = <OsString as OsStringExt>::from_wide(&desc.Description[..len]);
                    name.to_string_lossy().into_owned()
                };

                AdapterInfo {
                    name: device_name,
                    vendor: desc.VendorID,
                    device: desc.DeviceID,
                    video_memory: desc.DedicatedVideoMemory,
                    device_type: if (desc.Flags & dxgi1_6::DXGI_ADAPTER_FLAG3_SOFTWARE) == 0 {
                        DeviceType::DiscreteGpu
                    } else {
                        DeviceType::VirtualGpu
                    },
                }
            }
        }
    }
}
