use super::adapter::Adapter;
use super::{GpuPreference, WindowAssociationFlags};
use crate::{D3DResult, Error};

use log::*;
use winapi::shared::guiddef::GUID;
use winapi::shared::windef::HWND;
use winapi::shared::{dxgi, dxgi1_3, dxgi1_4, dxgi1_5, dxgi1_6, minwindef, winerror};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3d12sdklayers, dxgidebug};
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::ptr;

pub struct Factory {
    pub(crate) inner: ComPtr<dxgi::IDXGIFactory>,
    pub is_tearing_supported: bool,
}

impl Factory {
    pub fn create() -> D3DResult<Factory> {
        #[cfg(debug_assertions)]
        {
            // Enable debug layer
            let mut debug_controller: *mut d3d12sdklayers::ID3D12Debug = ptr::null_mut();
            let hr = unsafe {
                d3d12::D3D12GetDebugInterface(
                    &d3d12sdklayers::ID3D12Debug::uuidof(),
                    &mut debug_controller as *mut *mut _ as *mut *mut _,
                )
            };
            if winerror::SUCCEEDED(hr) {
                unsafe {
                    (*debug_controller).EnableDebugLayer();
                    (*debug_controller).Release();
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

            adapters.push(adapter.unwrap());

            index += 1;
        }

        #[cfg(debug_assertions)]
        {
            // If no adapters have been found, try with the warp adapter
            if adapters.is_empty() {
                if let Ok(warp_adapter) = Adapter::enumerate_warp(self) {
                    adapters.push(warp_adapter);
                }
            }
        }

        assert!(!adapters.is_empty(), "No adapter found.");
        // TODO: In the long term, implement fallback to Direct3D11

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
        // Try to get the factory at the newest DXGI version
        Factory::create_dxgi_factory2(&dxgi1_6::IDXGIFactory6::uuidof())
            .or_else(|_| Factory::create_dxgi_factory2(&dxgi1_5::IDXGIFactory5::uuidof()))
            .or_else(|_| Factory::create_dxgi_factory2(&dxgi1_4::IDXGIFactory4::uuidof()))
            .or(Err(Error::CreateFactoryFailed))
    }

    fn create_factory1() -> D3DResult<Factory> {
        // Try to get the factory at the newest DXGI version
        Factory::create_dxgi_factory1(&dxgi1_6::IDXGIFactory6::uuidof())
            .or_else(|_| Factory::create_dxgi_factory1(&dxgi1_5::IDXGIFactory5::uuidof()))
            .or_else(|_| Factory::create_dxgi_factory1(&dxgi1_4::IDXGIFactory4::uuidof()))
            .or(Err(Error::CreateFactoryFailed))
    }

    fn create_dxgi_factory2(guid: &GUID) -> D3DResult<Factory> {
        let mut factory_flags: u32 = 0;
        #[cfg(debug_assertions)]
        {
            let mut dxgi_info_queue: *mut dxgidebug::IDXGIInfoQueue = ptr::null_mut();
            let hr = unsafe {
                dxgi1_3::DXGIGetDebugInterface1(
                    0,
                    &dxgidebug::IDXGIInfoQueue::uuidof(),
                    &mut dxgi_info_queue as *mut *mut _ as *mut *mut _,
                )
            };

            if winerror::SUCCEEDED(hr) {
                let hr = unsafe {
                    (*dxgi_info_queue).SetBreakOnSeverity(
                        dxgidebug::DXGI_DEBUG_ALL,
                        dxgidebug::DXGI_INFO_QUEUE_MESSAGE_SEVERITY_CORRUPTION,
                        minwindef::TRUE,
                    )
                };
                if !winerror::SUCCEEDED(hr) {
                    warn!(
                        "Failed on setting break on severity in DXGI info queue (code {})",
                        hr
                    );
                }

                let hr = unsafe {
                    (*dxgi_info_queue).SetBreakOnSeverity(
                        dxgidebug::DXGI_DEBUG_ALL,
                        dxgidebug::DXGI_INFO_QUEUE_MESSAGE_SEVERITY_ERROR,
                        minwindef::TRUE,
                    )
                };
                if !winerror::SUCCEEDED(hr) {
                    warn!(
                        "Failed on setting break on severity in DXGI info queue (code {})",
                        hr
                    );
                }

                unsafe {
                    (*dxgi_info_queue).Release();
                }

                factory_flags = dxgi1_3::DXGI_CREATE_FACTORY_DEBUG
            }
        }

        let mut factory: *mut IUnknown = ptr::null_mut();
        let hr = unsafe {
            dxgi1_3::CreateDXGIFactory2(
                factory_flags,
                guid,
                &mut factory as *mut *mut _ as *mut *mut _,
            )
        };

        let is_tearing_supported = Factory::is_tearing_supported(factory).unwrap_or(false);

        if winerror::SUCCEEDED(hr) {
            Ok(Factory {
                inner: unsafe { ComPtr::from_raw(factory as *mut _) },
                is_tearing_supported,
            })
        } else {
            Err(Error::CreateFactoryFailed)
        }
    }

    fn create_dxgi_factory1(guid: &GUID) -> D3DResult<Factory> {
        let mut factory: *mut IUnknown = ptr::null_mut();
        let hr =
            unsafe { dxgi::CreateDXGIFactory1(guid, &mut factory as *mut *mut _ as *mut *mut _) };

        let is_tearing_supported = Factory::is_tearing_supported(factory).unwrap_or(false);

        if winerror::SUCCEEDED(hr) {
            Ok(Factory {
                inner: unsafe { ComPtr::from_raw(factory as *mut _) },
                is_tearing_supported,
            })
        } else {
            Err(Error::CreateFactoryFailed)
        }
    }

    fn is_tearing_supported(factory: *mut IUnknown) -> D3DResult<bool> {
        let mut allow_tearing: i32 = 0;
        let hr = unsafe {
            (*(factory as *mut dxgi1_5::IDXGIFactory5)).CheckFeatureSupport(
                dxgi1_5::DXGI_FEATURE_PRESENT_ALLOW_TEARING,
                &mut allow_tearing as *mut _ as *mut _,
                mem::size_of::<i32>() as _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(allow_tearing == 1)
        } else {
            Err(Error::CheckFeatureSupportFailed)
        }
    }
}

impl Drop for Factory {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            // Debug tracking alive objects
            let mut dxgi_debug_controller: *mut dxgidebug::IDXGIDebug1 = ptr::null_mut();
            let hr = unsafe {
                dxgi1_3::DXGIGetDebugInterface1(
                    0,
                    &dxgidebug::IDXGIDebug1::uuidof(),
                    &mut dxgi_debug_controller as *mut *mut _ as *mut *mut _,
                )
            };
            if winerror::SUCCEEDED(hr) {
                unsafe {
                    (*dxgi_debug_controller).ReportLiveObjects(
                        dxgidebug::DXGI_DEBUG_ALL,
                        dxgidebug::DXGI_DEBUG_RLO_SUMMARY
                            | dxgidebug::DXGI_DEBUG_RLO_IGNORE_INTERNAL,
                    );
                    (*dxgi_debug_controller).Release();
                }
            }
        }
    }
}
