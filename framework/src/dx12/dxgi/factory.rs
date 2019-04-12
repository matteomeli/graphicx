use super::adapter::Adapter;
use super::{DxgiVersion, GpuPreference, WindowAssociationFlags};
use crate::dx12::device::Device;
use crate::dx12::{D3DResult, Error, FeatureLevel};

use winapi::shared::guiddef::GUID;
use winapi::shared::windef::HWND;
use winapi::shared::{dxgi, dxgi1_2, dxgi1_3, dxgi1_4, dxgi1_5, dxgi1_6, winerror};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3d12sdklayers, dxgidebug};
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::ptr;

pub struct Factory {
    pub(crate) inner: ComPtr<dxgi::IDXGIFactory>,
    pub(crate) version: DxgiVersion,
    pub is_tearing_supported: bool,
}

impl Factory {
    pub fn create() -> D3DResult<Factory> {
        #[cfg(debug_assertions)]
        {
            // Enable debug layer
            let mut debug_interface: *mut d3d12sdklayers::ID3D12Debug = ptr::null_mut();
            let hr = unsafe {
                d3d12::D3D12GetDebugInterface(
                    &d3d12sdklayers::ID3D12Debug::uuidof(),
                    &mut debug_interface as *mut *mut _ as *mut *mut _,
                )
            };
            if winerror::SUCCEEDED(hr) {
                unsafe {
                    (*debug_interface).EnableDebugLayer();
                    (*debug_interface).Release();
                }
            }
        }

        Factory::create_factory2().or_else(|_| Factory::create_factory1())
    }

    pub fn get_adapters(&self, preference: GpuPreference) -> Vec<Adapter> {
        let mut index = 0;
        let mut adapters = Vec::new();
        loop {
            let adapter = Adapter::enumerate(self, index, preference);

            if adapter.is_err() {
                break;
            }

            index += 1;

            let adapter = adapter.unwrap();
            if Device::create(&adapter, FeatureLevel::Lvl11_0).is_err() {
                continue;
            }

            adapters.push(adapter);
        }
        adapters
    }

    pub fn make_window_association(
        &self,
        hwnd: HWND,
        flags: WindowAssociationFlags,
    ) -> D3DResult<()> {
        let hr = unsafe { self.inner.MakeWindowAssociation(hwnd, flags.bits()) };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(Error::MakeWindowAssociationFailed)
        }
    }

    fn create_factory2() -> D3DResult<Factory> {
        Factory::create_dxgi_factory2(&dxgi1_6::IDXGIFactory6::uuidof(), DxgiVersion::Dxgi1_6)
            .or_else(|_| {
                Factory::create_dxgi_factory2(
                    &dxgi1_5::IDXGIFactory5::uuidof(),
                    DxgiVersion::Dxgi1_5,
                )
            })
            .or_else(|_| {
                Factory::create_dxgi_factory2(
                    &dxgi1_4::IDXGIFactory4::uuidof(),
                    DxgiVersion::Dxgi1_4,
                )
            })
            .or_else(|_| {
                Factory::create_dxgi_factory2(
                    &dxgi1_3::IDXGIFactory3::uuidof(),
                    DxgiVersion::Dxgi1_3,
                )
            })
            .or_else(|_| {
                Factory::create_dxgi_factory2(
                    &dxgi1_2::IDXGIFactory2::uuidof(),
                    DxgiVersion::Dxgi1_2,
                )
            })
            .or_else(|_| {
                Factory::create_dxgi_factory2(&dxgi::IDXGIFactory1::uuidof(), DxgiVersion::Dxgi1_0)
            })
            .or_else(|_| {
                Factory::create_dxgi_factory2(&dxgi::IDXGIFactory::uuidof(), DxgiVersion::Dxgi1_0)
            })
    }

    fn create_factory1() -> D3DResult<Factory> {
        Factory::create_dxgi_factory1(&dxgi1_6::IDXGIFactory6::uuidof(), DxgiVersion::Dxgi1_6)
            .or_else(|_| {
                Factory::create_dxgi_factory1(
                    &dxgi1_5::IDXGIFactory5::uuidof(),
                    DxgiVersion::Dxgi1_5,
                )
            })
            .or_else(|_| {
                Factory::create_dxgi_factory1(
                    &dxgi1_4::IDXGIFactory4::uuidof(),
                    DxgiVersion::Dxgi1_4,
                )
            })
            .or_else(|_| {
                Factory::create_dxgi_factory1(
                    &dxgi1_3::IDXGIFactory3::uuidof(),
                    DxgiVersion::Dxgi1_3,
                )
            })
            .or_else(|_| {
                Factory::create_dxgi_factory1(
                    &dxgi1_2::IDXGIFactory2::uuidof(),
                    DxgiVersion::Dxgi1_2,
                )
            })
            .or_else(|_| {
                Factory::create_dxgi_factory1(&dxgi::IDXGIFactory1::uuidof(), DxgiVersion::Dxgi1_0)
            })
            .or_else(|_| {
                Factory::create_dxgi_factory1(&dxgi::IDXGIFactory::uuidof(), DxgiVersion::Dxgi1_0)
            })
    }

    fn create_dxgi_factory2(guid: &GUID, version: DxgiVersion) -> D3DResult<Factory> {
        let mut queue: *mut dxgidebug::IDXGIInfoQueue = ptr::null_mut();
        let hr = unsafe {
            dxgi1_3::DXGIGetDebugInterface1(
                0,
                &dxgidebug::IDXGIInfoQueue::uuidof(),
                &mut queue as *mut *mut _ as *mut *mut _,
            )
        };

        let factory_flags = if winerror::SUCCEEDED(hr) {
            unsafe {
                (*queue).Release();
            }
            dxgi1_3::DXGI_CREATE_FACTORY_DEBUG
        } else {
            0
        };

        let mut factory: *mut IUnknown = ptr::null_mut();
        let hr = unsafe {
            dxgi1_3::CreateDXGIFactory2(
                factory_flags,
                guid,
                &mut factory as *mut *mut _ as *mut *mut _,
            )
        };

        let is_tearing_supported = if version >= DxgiVersion::Dxgi1_5 {
            Factory::is_tearing_supported(factory)
        } else {
            false
        };

        if winerror::SUCCEEDED(hr) {
            Ok(Factory {
                inner: unsafe { ComPtr::from_raw(factory as *mut _) },
                version,
                is_tearing_supported,
            })
        } else {
            Err(Error::CreateFactoryFailed)
        }
    }

    fn create_dxgi_factory1(guid: &GUID, version: DxgiVersion) -> D3DResult<Factory> {
        let mut factory: *mut IUnknown = ptr::null_mut();
        let hr =
            unsafe { dxgi::CreateDXGIFactory1(guid, &mut factory as *mut *mut _ as *mut *mut _) };

        let is_tearing_supported = if version >= DxgiVersion::Dxgi1_5 {
            Factory::is_tearing_supported(factory)
        } else {
            false
        };

        if winerror::SUCCEEDED(hr) {
            Ok(Factory {
                inner: unsafe { ComPtr::from_raw(factory as *mut _) },
                version,
                is_tearing_supported,
            })
        } else {
            Err(Error::CreateFactoryFailed)
        }
    }

    fn is_tearing_supported(interface: *mut IUnknown) -> bool {
        let mut allow_tearing: i32 = 0;
        let hr = unsafe {
            (*(interface as *mut dxgi1_5::IDXGIFactory5)).CheckFeatureSupport(
                dxgi1_5::DXGI_FEATURE_PRESENT_ALLOW_TEARING,
                &mut allow_tearing as *mut _ as *mut _,
                mem::size_of::<i32>() as _,
            )
        };

        winerror::SUCCEEDED(hr) && allow_tearing == 1
    }
}
