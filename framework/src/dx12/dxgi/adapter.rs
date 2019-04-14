use super::factory::Factory;
use super::GpuPreference;
use crate::dx12::{D3DResult, Error};

use winapi::shared::{dxgi, dxgi1_4, dxgi1_6, winerror};
use winapi::um::{d3d12, d3dcommon};
use winapi::Interface;
use wio::com::ComPtr;

use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::ptr;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    pub(crate) inner: ComPtr<dxgi::IDXGIAdapter1>,
    pub info: AdapterInfo,
}

impl Adapter {
    pub fn enumerate(
        factory: &Factory,
        index: u32,
        preference: GpuPreference,
    ) -> D3DResult<Adapter> {
        let adapter = Adapter::enumerate_adapters_by_gpu_preference(factory, index, preference)
            .or_else(|_| Adapter::enumerate_adapters1(factory, index))?;

        let info = Adapter::get_info(adapter.as_raw());

        // Filter out warp adapters
        if info.device_type == DeviceType::DiscreteGpu {
            Ok(Adapter {
                inner: adapter,
                info,
            })
        } else {
            Err(Error::AdapterNotFound)
        }
    }

    pub fn enumerate_warp(factory: &Factory) -> D3DResult<Adapter> {
        let mut adapter: *mut dxgi::IDXGIAdapter1 = ptr::null_mut();
        let hr = unsafe {
            (*(factory.inner.as_raw() as *mut dxgi1_4::IDXGIFactory4)).EnumWarpAdapter(
                &dxgi1_4::IDXGIAdapter3::uuidof(),
                &mut adapter as *mut *mut _ as *mut *mut _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            let info = Adapter::get_info(adapter);
            Ok(Adapter {
                inner: unsafe { ComPtr::from_raw(adapter) },
                info,
            })
        } else {
            Err(Error::AdapterNotFound)
        }
    }

    fn enumerate_adapters1(
        factory: &Factory,
        index: u32,
    ) -> D3DResult<ComPtr<dxgi::IDXGIAdapter1>> {
        let mut adapter: *mut dxgi::IDXGIAdapter1 = ptr::null_mut();

        let hr = unsafe {
            (*(factory.inner.as_raw() as *mut dxgi::IDXGIFactory1))
                .EnumAdapters1(index, &mut adapter as *mut *mut _ as *mut *mut _)
        };

        // Check to see if the adapter supports Direct3D 12,
        // but don't create the actual device yet.
        if winerror::SUCCEEDED(hr)
            && unsafe {
                winerror::SUCCEEDED(d3d12::D3D12CreateDevice(
                    adapter as *mut _,
                    d3dcommon::D3D_FEATURE_LEVEL_11_0,
                    &d3d12::ID3D12Device::uuidof(),
                    ptr::null_mut(),
                ))
            }
        {
            Ok(unsafe { ComPtr::from_raw(adapter) })
        } else {
            Err(Error::AdapterNotFound)
        }
    }

    fn enumerate_adapters_by_gpu_preference(
        factory: &Factory,
        index: u32,
        preference: GpuPreference,
    ) -> D3DResult<ComPtr<dxgi::IDXGIAdapter1>> {
        let mut adapter: *mut dxgi::IDXGIAdapter1 = ptr::null_mut();
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

    fn get_info(adapter: *mut dxgi::IDXGIAdapter1) -> AdapterInfo {
        let mut desc: dxgi::DXGI_ADAPTER_DESC1 = unsafe { mem::zeroed() };
        unsafe {
            (*adapter).GetDesc1(&mut desc);
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
}
