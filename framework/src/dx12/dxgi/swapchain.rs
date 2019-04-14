use super::factory::Factory;
use super::{Format, PresentFlags};
use crate::dx12::command::CommandQueue;
use crate::dx12::resource::Resource;
use crate::dx12::{D3DResult, Error};

use bitflags::bitflags;
use winapi::shared::windef::HWND;
use winapi::shared::{dxgi, dxgi1_2, dxgi1_4, dxgitype, winerror};
use winapi::um::d3d12;
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::ptr;

#[derive(Copy, Clone, Debug)]
pub struct SampleDesc {
    pub count: u32,
    pub quality: u32,
}

bitflags! {
    pub struct BufferUsage: u32 {
        const ACCESS_NONE = dxgitype::DXGI_CPU_ACCESS_NONE;
        const ACCESS_DYNAMIC = dxgitype::DXGI_CPU_ACCESS_DYNAMIC;
        const ACCESS_READ_WRITE = dxgitype::DXGI_CPU_ACCESS_READ_WRITE;
        const ACCESS_SCRATCH = dxgitype::DXGI_CPU_ACCESS_SCRATCH;
        const ACCESS_FIELD = dxgitype::DXGI_CPU_ACCESS_FIELD;
        const SHADER_INPUT = dxgitype::DXGI_USAGE_SHADER_INPUT;
        const RENDER_TARGET_OUTPUT = dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT;
        const BACK_BUFFER = dxgitype::DXGI_USAGE_BACK_BUFFER;
        const SHARED = dxgitype::DXGI_USAGE_SHARED;
        const READ_ONLY = dxgitype::DXGI_USAGE_READ_ONLY;
        const DISCARD_ON_PRESENT = dxgitype::DXGI_USAGE_DISCARD_ON_PRESENT;
        const UNORDERED_ACCESS = dxgitype::DXGI_USAGE_UNORDERED_ACCESS;
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Scaling {
    Stretch = dxgi1_2::DXGI_SCALING_STRETCH,
    None = dxgi1_2::DXGI_SCALING_NONE,
    AspectRatioStretch = dxgi1_2::DXGI_SCALING_ASPECT_RATIO_STRETCH,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum SwapEffect {
    Discard = dxgi::DXGI_SWAP_EFFECT_DISCARD,
    Sequential = dxgi::DXGI_SWAP_EFFECT_SEQUENTIAL,
    FlipSequential = dxgi::DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
    FlipDiscard = dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum AlphaMode {
    Unspecified = dxgi1_2::DXGI_ALPHA_MODE_UNSPECIFIED,
    Premultiplied = dxgi1_2::DXGI_ALPHA_MODE_PREMULTIPLIED,
    Straight = dxgi1_2::DXGI_ALPHA_MODE_STRAIGHT,
    Ignore = dxgi1_2::DXGI_ALPHA_MODE_IGNORE,
    ForceDword = dxgi1_2::DXGI_ALPHA_MODE_FORCE_DWORD,
}

bitflags! {
    pub struct SwapChainFlags: u32 {
        const NONE = 0;
        const NON_PREROTATED = dxgi::DXGI_SWAP_CHAIN_FLAG_NONPREROTATED;
        const ALLOW_MODE_SWITCH = dxgi::DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH;
        const GDO_COMPATIBLE = dxgi::DXGI_SWAP_CHAIN_FLAG_GDI_COMPATIBLE;
        const RESTRICTED_CONTENT = dxgi::DXGI_SWAP_CHAIN_FLAG_RESTRICTED_CONTENT;
        const RESTRICT_SHARED_RESOURCE_DRIVER = dxgi::DXGI_SWAP_CHAIN_FLAG_RESTRICT_SHARED_RESOURCE_DRIVER;
        const DISPATCH_ONLY = dxgi::DXGI_SWAP_CHAIN_FLAG_DISPLAY_ONLY;
        const FRAME_LATENCY_WAITABLE_OBJECT = dxgi::DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT;
        const FOREGROUND_LAYER = dxgi::DXGI_SWAP_CHAIN_FLAG_FOREGROUND_LAYER;
        const FULLSCREEN_VIDEO = dxgi::DXGI_SWAP_CHAIN_FLAG_FULLSCREEN_VIDEO;
        const YUV_VIDEO = dxgi::DXGI_SWAP_CHAIN_FLAG_YUV_VIDEO;
        const HW_PROTECTED = dxgi::DXGI_SWAP_CHAIN_FLAG_HW_PROTECTED;
        const ALLOW_TEARING = dxgi::DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SwapChainDesc {
    pub width: u32,
    pub height: u32,
    pub format: Format,
    pub stereo: bool,
    pub sample_desc: SampleDesc,
    pub buffer_usage: BufferUsage,
    pub buffer_count: u32,
    pub scaling: Scaling,
    pub swap_effect: SwapEffect,
    pub alpha_mode: AlphaMode,
    pub flags: SwapChainFlags,
}

pub struct SwapChain {
    inner: ComPtr<dxgi1_4::IDXGISwapChain3>,
}

impl SwapChain {
    pub fn create(
        factory: &Factory,
        command_queue: &CommandQueue,
        config: &SwapChainDesc,
        hwnd: HWND,
    ) -> D3DResult<SwapChain> {
        let desc = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            Width: config.width,
            Height: config.height,
            Format: config.format as _,
            Stereo: config.stereo as _,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: config.sample_desc.count,
                Quality: config.sample_desc.quality,
            },
            BufferUsage: config.buffer_usage.bits(),
            BufferCount: config.buffer_count,
            Scaling: config.scaling as _,
            SwapEffect: config.swap_effect as _,
            AlphaMode: config.alpha_mode as _,
            Flags: config.flags.bits(),
        };

        let mut swap_chain1: *mut dxgi1_2::IDXGISwapChain1 = ptr::null_mut();
        let hr = unsafe {
            (*(factory.inner.as_raw() as *mut dxgi1_2::IDXGIFactory2)).CreateSwapChainForHwnd(
                command_queue.inner.as_raw() as *mut _,
                hwnd,
                &desc,
                ptr::null(),
                ptr::null_mut(),
                &mut swap_chain1 as *mut *mut _ as *mut *mut _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            let swap_chain1 = unsafe { ComPtr::from_raw(swap_chain1) };
            let swap_chain3 = swap_chain1
                .cast::<dxgi1_4::IDXGISwapChain3>()
                .map_err(|_| Error::CreateSwapChainFailed)?;
            Ok(SwapChain { inner: swap_chain3 })
        } else {
            Err(Error::CreateSwapChainFailed)
        }
    }

    pub fn get_buffer(&self, index: u32) -> D3DResult<Resource> {
        let mut buffer: *mut d3d12::ID3D12Resource = ptr::null_mut();
        let hr = unsafe {
            self.inner.GetBuffer(
                index,
                &d3d12::ID3D12Resource::uuidof(),
                &mut buffer as *mut *mut _ as *mut *mut _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(Resource {
                inner: unsafe { ComPtr::from_raw(buffer) },
            })
        } else {
            Err(Error::GetBufferFromSwapChainFailed)
        }
    }

    pub fn get_current_back_buffer_index(&self) -> u32 {
        unsafe { self.inner.GetCurrentBackBufferIndex() }
    }

    pub fn present(&self, sync_interval: u32, flags: PresentFlags) -> D3DResult<()> {
        let hr = unsafe { self.inner.Present(sync_interval, flags.bits()) };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(Error::PresentSwapChainFailed)
        }
    }

    pub fn resize_buffers(&self, buffers_count: u32, width: u32, height: u32) -> D3DResult<()> {
        let desc = self.get_desc()?;

        let hr = unsafe {
            self.inner.ResizeBuffers(
                buffers_count,
                width,
                height,
                desc.BufferDesc.Format,
                desc.Flags,
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(Error::ResizeSwapChainFailed)
        }
    }

    fn get_desc(&self) -> D3DResult<dxgi::DXGI_SWAP_CHAIN_DESC> {
        let mut desc: dxgi::DXGI_SWAP_CHAIN_DESC = unsafe { mem::zeroed() };
        let hr = unsafe { self.inner.GetDesc(&mut desc) };

        if winerror::SUCCEEDED(hr) {
            Ok(desc)
        } else {
            Err(Error::GetSwapChainDescFailed)
        }
    }
}
