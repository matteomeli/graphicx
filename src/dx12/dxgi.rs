use super::command::CommandQueue;

use winapi::shared::{
    dxgi, dxgi1_2, dxgi1_3, dxgi1_4, dxgi1_5, dxgiformat, dxgitype, minwindef, windef, winerror,
};
use winapi::um::d3d12;
use winapi::um::unknwnbase::IUnknown;
use winapi::Interface;
use wio::com::ComPtr;

// Add here missing flags for DXGIFactory::MakeWindowsAssociation
pub const DXGI_MWA_NO_WINDOW_CHANGES: minwindef::UINT = 1;
pub const DXGI_MWA_NO_ALT_ENTER: minwindef::UINT = 1 << 1;
pub const DXGI_MWA_NO_PRINT_SCREEN: minwindef::UINT = 1 << 2;
pub const DXGI_MWA_VALID: minwindef::UINT = 0x7;

use winit::os::windows::WindowExt;

use std::mem;
use std::ptr;

pub struct SwapChain4 {
    native: ComPtr<dxgi1_5::IDXGISwapChain4>,
}

impl SwapChain4 {
    pub fn new(
        command_queue: &CommandQueue,
        window: &winit::Window,
        width: u32,
        height: u32,
        back_buffers_count: usize,
        is_tearing_supported: bool,
    ) -> Self {
        let mut dxgi_factory4: *mut dxgi1_4::IDXGIFactory4 = ptr::null_mut();
        let flags: u32 = if cfg!(debug_assertions) {
            dxgi1_3::DXGI_CREATE_FACTORY_DEBUG
        } else {
            0
        };

        let hr = unsafe {
            dxgi1_3::CreateDXGIFactory2(
                flags,
                &dxgi1_4::IDXGIFactory4::uuidof(),
                &mut dxgi_factory4 as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating DXGI factory: {:?}", hr);
        }

        let swap_chain_desc = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            Stereo: minwindef::FALSE,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: back_buffers_count as _,
            Scaling: dxgi1_2::DXGI_SCALING_STRETCH,
            SwapEffect: dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: dxgi1_2::DXGI_ALPHA_MODE_UNSPECIFIED,
            // It is recommended to always allow tearing if tearing support is available.
            Flags: if is_tearing_supported {
                dxgi::DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING
            } else {
                0
            },
        };

        let mut dxgi_swap_chain1: *mut dxgi1_2::IDXGISwapChain1 = ptr::null_mut();
        let mut dxgi_swap_chain4: *mut dxgi1_5::IDXGISwapChain4 = ptr::null_mut();

        let hwnd: windef::HWND = window.get_hwnd() as *mut _;

        let hr = unsafe {
            (*dxgi_factory4).CreateSwapChainForHwnd(
                command_queue.as_mut_ptr() as *mut _,
                hwnd,
                &swap_chain_desc,
                ptr::null(),
                ptr::null_mut(),
                &mut dxgi_swap_chain1,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating a swap chain: {:?}", hr);
        }

        // Disable the Alt+Enter fullscreen toggle feature. Switching to fullscreen will be handled manually.
        let hr = unsafe { (*dxgi_factory4).MakeWindowAssociation(hwnd, DXGI_MWA_NO_ALT_ENTER) };
        if !winerror::SUCCEEDED(hr) {
            panic!(
                "Failed on disabling ALT-ENTER as fullscreen toggle: {:?}",
                hr
            );
        }

        // Perform QueryInterface fun, because we're not using ComPtrs.
        // TODO: Code repetition, need a function or struct to handle this
        unsafe {
            let as_unknown: &IUnknown = &*(dxgi_swap_chain1 as *mut IUnknown);
            let err = as_unknown.QueryInterface(
                &dxgi1_5::IDXGISwapChain4::uuidof(),
                &mut dxgi_swap_chain4 as *mut *mut _ as *mut *mut _,
            );
            if err < 0 {
                panic!("Failed on casting DXGI swap chain: {:?}", hr);
            }
        }

        SwapChain4 {
            native: unsafe { ComPtr::from_raw(dxgi_swap_chain4) },
        }
    }

    pub fn get_current_back_buffer_index(&self) -> u32 {
        unsafe { self.native.GetCurrentBackBufferIndex() }
    }

    pub fn get_desc(&self, desc: &mut dxgi::DXGI_SWAP_CHAIN_DESC) {
        let hr = unsafe { self.native.GetDesc(desc) };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on obtaining swap chain description: {:?}", hr);
        }
    }

    pub fn get_buffer(&self, index: u32) -> ComPtr<d3d12::ID3D12Resource> {
        let mut back_buffer: *mut d3d12::ID3D12Resource = ptr::null_mut();
        let hr = unsafe {
            self.native.GetBuffer(
                index,
                &d3d12::ID3D12Resource::uuidof(),
                &mut back_buffer as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!(
                "Failed on obtaining back buffer resource {} from swap chain: {:?}",
                index, hr
            );
        }

        unsafe { ComPtr::from_raw(back_buffer) }
    }

    pub fn resize_buffers(&self, back_buffers_count: u32, width: u32, height: u32) {
        let mut desc: winapi::shared::dxgi::DXGI_SWAP_CHAIN_DESC = unsafe { mem::zeroed() };
        self.get_desc(&mut desc);

        let hr = unsafe {
            self.native.ResizeBuffers(
                back_buffers_count,
                width,
                height,
                desc.BufferDesc.Format,
                desc.Flags,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on resizing swap chain buffers: {:?}", hr);
        }
    }

    pub fn present(&self, sync_interval: u32, flags: u32) {
        let hr = unsafe { self.native.Present(sync_interval, flags) };
        if !winerror::SUCCEEDED(hr) {
            panic!(
                "Failed on presenting the swap chain's current back buffer: {:?}",
                hr
            );
        }
    }
}
