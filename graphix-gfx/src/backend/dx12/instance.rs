use crate::backend::dx12::adapter::PhysicalAdapter;
use crate::backend::dx12::command::{CommandBuffer, CommandPool};
use crate::backend::dx12::device::Device;
use crate::backend::dx12::queue::CommandQueue;
use crate::backend::dx12::resource::FrameBuffer;
use crate::backend::dx12::window::{Surface, Swapchain};
use crate::hal;

use graphix_native_dx12 as native;

use winapi::shared::{dxgi, dxgi1_3, winerror};
use winapi::um::dxgidebug;
use winapi::Interface;

use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::ptr;

pub enum Backend {}
impl hal::Backend for Backend {
    type PhysicalAdapter = PhysicalAdapter;
    type Device = Device;

    type CommandQueue = CommandQueue;
    type CommandPool = CommandPool;
    type CommandBuffer = CommandBuffer;

    type Surface = Surface;
    type Swapchain = Swapchain;

    type FrameBuffer = FrameBuffer;

    type Fence = native::sync::Fence;
}

pub struct Instance {
    pub(crate) factory: native::dxgi::Factory,
}

impl Instance {
    pub fn new() -> Self {
        Default::default()
    }

    fn setup_dxgi_debug_layer() -> bool {
        let mut info_queue: *mut dxgidebug::IDXGIInfoQueue = ptr::null_mut();
        let hr = unsafe {
            dxgi1_3::DXGIGetDebugInterface1(
                0,
                &dxgidebug::IDXGIInfoQueue::uuidof(),
                &mut info_queue as *mut *mut _ as *mut *mut _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            unsafe {
                (*info_queue).SetBreakOnSeverity(
                    dxgidebug::DXGI_DEBUG_ALL,
                    dxgidebug::DXGI_INFO_QUEUE_MESSAGE_SEVERITY_CORRUPTION,
                    1,
                );
            }

            unsafe {
                (*info_queue).SetBreakOnSeverity(
                    dxgidebug::DXGI_DEBUG_ALL,
                    dxgidebug::DXGI_INFO_QUEUE_MESSAGE_SEVERITY_ERROR,
                    1,
                );
            }

            unsafe {
                (*info_queue).SetBreakOnSeverity(
                    dxgidebug::DXGI_DEBUG_ALL,
                    dxgidebug::DXGI_INFO_QUEUE_MESSAGE_SEVERITY_WARNING,
                    0,
                );
            }

            unsafe {
                (*info_queue).Release();
            }

            true
        } else {
            false
        }
    }

    fn get_adapter_info(adapter: &native::dxgi::Adapter) -> hal::AdapterInfo {
        let mut desc: dxgi::DXGI_ADAPTER_DESC1 = unsafe { mem::zeroed() };
        unsafe {
            (*adapter.as_raw()).GetDesc1(&mut desc);
        }

        let device_name = {
            let len = desc.Description.iter().take_while(|&&c| c != 0).count();
            let name = <OsString as OsStringExt>::from_wide(&desc.Description[..len]);
            name.to_string_lossy().into_owned()
        };

        hal::AdapterInfo {
            name: device_name,
            vendor: desc.VendorId,
            device: desc.DeviceId,
            video_memory: desc.DedicatedVideoMemory,
            device_type: if (desc.Flags & dxgi::DXGI_ADAPTER_FLAG_SOFTWARE) == 0 {
                hal::DeviceType::DiscreteGpu
            } else {
                hal::DeviceType::VirtualGpu
            },
        }
    }
}

impl Default for Instance {
    fn default() -> Self {
        let factory_flags = if cfg!(debug_assertions) && Instance::setup_dxgi_debug_layer() {
            native::dxgi::FactoryCreationFlags::DEBUG
        } else {
            native::dxgi::FactoryCreationFlags::empty()
        };

        Instance {
            factory: native::dxgi::Factory::new(factory_flags)
                .expect("Failed to create DXGI factory."),
        }
    }
}

impl hal::Instance for Instance {
    type Backend = Backend;

    fn enumerate_adapters(&self) -> Vec<hal::Adapter<Backend>> {
        self.factory
            .enumerate_adapters()
            .into_iter()
            .map(|native_adapter| {
                let info = Instance::get_adapter_info(&native_adapter);
                hal::Adapter {
                    adapter: PhysicalAdapter {
                        native: native_adapter,
                    },
                    info,
                }
            })
            .collect()
    }
}

impl Drop for Instance {
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
