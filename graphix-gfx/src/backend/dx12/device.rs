use crate::backend::dx12::adapter::PhysicalAdapter;
use crate::backend::dx12::command::CommandPool;
use crate::backend::dx12::instance::Backend;
use crate::backend::dx12::queue::CommandQueue;
use crate::hal;

use graphix_native_dx12 as native;

use winapi::shared::winerror;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3d12sdklayers, d3dcommon};
use winapi::Interface;

use std::mem;
use std::ptr;

pub struct Device {
    pub(crate) native: native::device::Device,
    feature_level: d3dcommon::D3D_FEATURE_LEVEL,
}

impl Device {
    pub(crate) fn new(adapter: &PhysicalAdapter) -> Self {
        Device::setup_debug_layer();

        let device =
            native::device::Device::new(&adapter.native, native::device::FeatureLevel::L11_0)
                .expect("Failed to create D3D12 device.");

        // Check actual feature level obtained
        let min_feature_level = d3dcommon::D3D_FEATURE_LEVEL_11_0;
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

        let feature_level = match device.check_feature_support(
            native::device::Feature::FeatureLevels,
            &mut feature_levels as *mut _ as *mut _,
            mem::size_of::<d3d12::D3D12_FEATURE_DATA_FEATURE_LEVELS>() as _,
        ) {
            Ok(_) => feature_levels.MaxSupportedFeatureLevel,
            Err(_) => min_feature_level,
        };

        Device::setup_debug_settings(&device);

        Device {
            native: device,
            feature_level,
        }
    }

    fn setup_debug_layer() {
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
                let mut debug_controller1: *mut d3d12sdklayers::ID3D12Debug1 = ptr::null_mut();
                let hr2 = unsafe {
                    (*(debug_controller as *mut IUnknown)).QueryInterface(
                        &d3d12sdklayers::ID3D12Debug1::uuidof(),
                        &mut debug_controller1 as *mut *mut _ as *mut *mut _,
                    )
                };
                if winerror::SUCCEEDED(hr2) {
                    unsafe {
                        (*debug_controller1).SetEnableGPUBasedValidation(1);
                        (*debug_controller1).SetEnableSynchronizedCommandQueueValidation(1);
                        (*debug_controller1).Release();
                    }
                }

                unsafe {
                    (*debug_controller).EnableDebugLayer();
                    (*debug_controller).Release();
                }
            }
        }
    }

    fn setup_debug_settings(device: &native::device::Device) {
        #[cfg(debug_assertions)]
        {
            // Setup an info queue to enable debug messages in debug mode
            device.init_info_queue();
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            // Debug tracking alive objects
            self.native.report_live_objects();
        }
    }
}

impl hal::Device<Backend> for Device {
    fn create_command_queue(&self, queue_type: hal::QueueType) -> CommandQueue {
        CommandQueue::new(self, queue_type)
    }

    fn create_command_pool(
        &self,
        pool_type: hal::QueueType,
        flags: hal::CommandPoolFlags,
    ) -> CommandPool {
        CommandPool::new(self, pool_type, flags)
    }

    fn create_fence(&self, initial_value: u64) -> native::sync::Fence {
        native::sync::Fence::new_with_value(&self.native, initial_value)
            .expect("Failed to create D3D12 fence")
    }

    fn reset_fence(&self, fence: &native::sync::Fence) {
        fence.reset().expect("Failed to reset D3D12 fence")
    }

    fn wait_for_fence_with_timeout(
        &self,
        fence: &native::sync::Fence,
        value: u64,
        timeout: u64,
    ) -> bool {
        let event = native::sync::Event::new(false, false);
        fence
            .wait_for_value_with_timeout(event, value, timeout)
            .expect("Failed to wait for D3D12 fence")
    }
}
