use super::command::CommandQueue;

use winapi::shared::windef::HWND;
use winapi::shared::{
    dxgi, dxgi1_2, dxgi1_3, dxgi1_4, dxgi1_5, dxgi1_6, dxgiformat, dxgitype, winerror,
};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3dcommon};
use winapi::Interface;
use wio::com::ComPtr;

// Add here missing flags for DXGIFactory::MakeWindowsAssociation
bitflags! {
    pub struct WindowAssociationFlags: u32 {
        const NoWindowChanges = 1;
        const NoAltEnter = 1 << 1;
        const NoPrintScreen = 1 << 2;
        const Valid = 0x7;
    }
}

use std::mem;
use std::ptr;

// TODO: Adapter wrap

bitflags! {
    pub struct FactoryCreationFlags: u32 {
        const None = 0;
        const Debug = dxgi1_3::DXGI_CREATE_FACTORY_DEBUG;
    }
}

pub struct SampleDesc {
    pub count: u32,
    pub quality: u32,
}

bitflags! {
    pub struct Usage: u32 {
        const AccessNone = dxgitype::DXGI_CPU_ACCESS_NONE;
        const AccessDynamic = dxgitype::DXGI_CPU_ACCESS_DYNAMIC;
        const AccessReadWrite = dxgitype::DXGI_CPU_ACCESS_READ_WRITE;
        const AccessScratch = dxgitype::DXGI_CPU_ACCESS_SCRATCH;
        const AccessField = dxgitype::DXGI_CPU_ACCESS_FIELD;
        const ShaderInput = dxgitype::DXGI_USAGE_SHADER_INPUT;
        const RenderTargetOutput = dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT;
        const BackBuffer = dxgitype::DXGI_USAGE_BACK_BUFFER;
        const Shared = dxgitype::DXGI_USAGE_SHARED;
        const ReadOnly = dxgitype::DXGI_USAGE_READ_ONLY;
        const DiscardOnPresent = dxgitype::DXGI_USAGE_DISCARD_ON_PRESENT;
        const UnorderedAccess = dxgitype::DXGI_USAGE_UNORDERED_ACCESS;
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Scaling {
    Stretch = dxgi1_2::DXGI_SCALING_STRETCH,
    None = dxgi1_2::DXGI_SCALING_NONE,
    AspectRatioStretch = dxgi1_2::DXGI_SCALING_ASPECT_RATIO_STRETCH,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum SwapEffect {
    Discard = dxgi::DXGI_SWAP_EFFECT_DISCARD,
    Sequential = dxgi::DXGI_SWAP_EFFECT_SEQUENTIAL,
    FlipSequential = dxgi::DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
    FlipDiscard = dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum AlphaMode {
    Unspecified = dxgi1_2::DXGI_ALPHA_MODE_UNSPECIFIED,
    Premultiplied = dxgi1_2::DXGI_ALPHA_MODE_PREMULTIPLIED,
    Straight = dxgi1_2::DXGI_ALPHA_MODE_STRAIGHT,
    Ignore = dxgi1_2::DXGI_ALPHA_MODE_IGNORE,
    ForceDword = dxgi1_2::DXGI_ALPHA_MODE_FORCE_DWORD,
}

bitflags! {
    pub struct Flags: u32 {
        const None = 0;
        const NonPrerotated = dxgi::DXGI_SWAP_CHAIN_FLAG_NONPREROTATED;
        const AllowModeSwitch = dxgi::DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH;
        const GDICompatible = dxgi::DXGI_SWAP_CHAIN_FLAG_GDI_COMPATIBLE;
        const RestrictedContent = dxgi::DXGI_SWAP_CHAIN_FLAG_RESTRICTED_CONTENT;
        const RestrictSharedResourceDriver = dxgi::DXGI_SWAP_CHAIN_FLAG_RESTRICT_SHARED_RESOURCE_DRIVER;
        const DispatchOnly = dxgi::DXGI_SWAP_CHAIN_FLAG_DISPLAY_ONLY;
        const FrameLatencyWaitableObject = dxgi::DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT;
        const ForegroundLayer = dxgi::DXGI_SWAP_CHAIN_FLAG_FOREGROUND_LAYER;
        const FullscreenVideo = dxgi::DXGI_SWAP_CHAIN_FLAG_FULLSCREEN_VIDEO;
        const YUVVideo = dxgi::DXGI_SWAP_CHAIN_FLAG_YUV_VIDEO;
        const HWProtected = dxgi::DXGI_SWAP_CHAIN_FLAG_HW_PROTECTED;
        const AllowTearing = dxgi::DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING;
    }
}

pub struct SwapChainDesc {
    pub width: u32,
    pub height: u32,
    pub format: Format,
    pub stereo: bool,
    pub sample_desc: SampleDesc,
    pub buffer_usage: Usage,
    pub buffer_count: u32,
    pub scaling: Scaling,
    pub swap_effect: SwapEffect,
    pub alpha_mode: AlphaMode,
    pub flags: Flags,
}

pub struct Adapter4 {
    native: ComPtr<dxgi1_6::IDXGIAdapter4>,
}

impl Adapter4 {
    pub fn new(use_warp: bool) -> Self {
        let mut dxgi_factory: *mut dxgi1_4::IDXGIFactory4 = ptr::null_mut();
        let flags: u32 = if cfg!(debug_assertions) {
            dxgi1_3::DXGI_CREATE_FACTORY_DEBUG
        } else {
            0
        };

        let hr = unsafe {
            dxgi1_3::CreateDXGIFactory2(
                flags,
                &dxgi1_4::IDXGIFactory4::uuidof(),
                &mut dxgi_factory as *mut *mut _ as *mut *mut _,
            )
        };

        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating DXGI factory: {:?}", hr);
        }

        let mut dxgi_adapter1: *mut winapi::shared::dxgi::IDXGIAdapter1 = ptr::null_mut();
        let mut dxgi_adapter4: *mut dxgi1_6::IDXGIAdapter4 = ptr::null_mut();

        if use_warp {
            let hr = unsafe {
                (*dxgi_factory).EnumWarpAdapter(
                    &winapi::shared::dxgi::IDXGIAdapter1::uuidof(),
                    &mut dxgi_adapter1 as *mut *mut _ as *mut *mut _,
                )
            };
            if !winerror::SUCCEEDED(hr) {
                panic!("Failed on enumerating DXGI warp adapter: {:?}", hr);
            }

            // Perform QueryInterface fun, because we're not using ComPtrs.
            // TODO: Code repetition, need a function or struct to handle this
            unsafe {
                let as_unknown: &IUnknown = &*(dxgi_adapter1 as *mut IUnknown);
                let err = as_unknown.QueryInterface(
                    &dxgi1_6::IDXGIAdapter4::uuidof(),
                    &mut dxgi_adapter4 as *mut *mut _ as *mut *mut _,
                );
                if err < 0 {
                    panic!("Failed on casting DXGI warp adapter: {:?}", hr);
                }
            }
        } else {
            let mut index = 0;
            let mut max_dedicated_vdeo_memory = 0;
            loop {
                let hr = unsafe { (*dxgi_factory).EnumAdapters1(index, &mut dxgi_adapter1) };
                if hr == winerror::DXGI_ERROR_NOT_FOUND {
                    break;
                }

                index += 1;

                let mut desc: winapi::shared::dxgi::DXGI_ADAPTER_DESC1 = unsafe { mem::zeroed() };
                let hr = unsafe { (*dxgi_adapter1).GetDesc1(&mut desc) };
                if !winerror::SUCCEEDED(hr) {
                    panic!("Failed on obtaining DXGI adapter description: {:?}", hr);
                }

                // We want only the hardware adapter with the largest dedicated video memory
                let hr = unsafe {
                    d3d12::D3D12CreateDevice(
                        dxgi_adapter1 as *mut _,
                        d3dcommon::D3D_FEATURE_LEVEL_11_0,
                        &d3d12::ID3D12Device::uuidof(),
                        ptr::null_mut(),
                    )
                };
                if (desc.Flags & winapi::shared::dxgi::DXGI_ADAPTER_FLAG_SOFTWARE) == 0
                    && desc.DedicatedVideoMemory > max_dedicated_vdeo_memory
                    && winerror::SUCCEEDED(hr)
                {
                    max_dedicated_vdeo_memory = desc.DedicatedVideoMemory;

                    // Perform QueryInterface fun, because we're not using ComPtrs.
                    // TODO: Code repetition, need a function or struct to handle this
                    unsafe {
                        let as_unknown: &IUnknown = &*(dxgi_adapter1 as *mut IUnknown);
                        let err = as_unknown.QueryInterface(
                            &dxgi1_6::IDXGIAdapter4::uuidof(),
                            &mut dxgi_adapter4 as *mut *mut _ as *mut *mut _,
                        );
                        if err < 0 {
                            panic!("Failed on casting into a DXGI 1.5 adapter: {:?}", hr);
                        }
                    }
                }
            }
        }

        Adapter4 {
            native: unsafe { ComPtr::from_raw(dxgi_adapter4) },
        }
    }

    pub fn as_ptr(&self) -> *const dxgi1_6::IDXGIAdapter4 {
        self.native.as_raw()
    }

    pub fn as_mut_ptr(&self) -> *mut dxgi1_6::IDXGIAdapter4 {
        self.native.as_raw()
    }
}

pub struct Factory2 {
    native: ComPtr<dxgi1_2::IDXGIFactory2>,
}

impl Factory2 {
    pub fn create_swap_chain_for_hwnd(
        &self,
        command_queue: &CommandQueue,
        desc: &SwapChainDesc,
        hwnd: HWND,
    ) -> *mut dxgi1_2::IDXGISwapChain1 {
        let desc = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            Width: desc.width,
            Height: desc.height,
            Format: desc.format as _,
            Stereo: desc.stereo as _,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: desc.sample_desc.count,
                Quality: desc.sample_desc.quality,
            },
            BufferUsage: desc.buffer_usage.bits(),
            BufferCount: desc.buffer_count,
            Scaling: desc.scaling as _,
            SwapEffect: desc.swap_effect as _,
            AlphaMode: desc.alpha_mode as _,
            Flags: desc.flags.bits(),
        };

        let mut swap_chain: *mut dxgi1_2::IDXGISwapChain1 = ptr::null_mut();
        let hr = unsafe {
            self.native.CreateSwapChainForHwnd(
                command_queue.as_mut_ptr() as *mut _,
                hwnd,
                &desc,
                ptr::null(),
                ptr::null_mut(),
                &mut swap_chain,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating a swap chain: {:?}", hr);
        }

        swap_chain
    }

    pub fn make_window_association(&self, hwnd: HWND, flags: WindowAssociationFlags) {
        let hr = unsafe { self.native.MakeWindowAssociation(hwnd, flags.bits()) };
        if !winerror::SUCCEEDED(hr) {
            panic!(
                "Failed on disabling ALT-ENTER as fullscreen toggle: {:?}",
                hr
            );
        }
    }
}

pub struct Factory4 {
    native: ComPtr<dxgi1_4::IDXGIFactory4>,
}

impl Factory4 {
    pub fn new(flags: FactoryCreationFlags) -> Self {
        let mut factory: *mut dxgi1_4::IDXGIFactory4 = ptr::null_mut();
        let hr = unsafe {
            dxgi1_3::CreateDXGIFactory2(
                flags.bits(),
                &dxgi1_4::IDXGIFactory4::uuidof(),
                &mut factory as *mut *mut _ as *mut *mut _,
            )
        };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on creating DXGI factory: {:?}", hr);
        }

        Factory4 {
            native: unsafe { ComPtr::from_raw(factory) },
        }
    }

    pub fn as_ptr(&self) -> *const dxgi1_4::IDXGIFactory4 {
        self.native.as_raw()
    }

    pub fn as_mut_ptr(&self) -> *mut dxgi1_4::IDXGIFactory4 {
        self.native.as_raw()
    }

    pub fn as_factory2(&self) -> Factory2 {
        Factory2 {
            native: self.native.cast::<dxgi1_2::IDXGIFactory2>().unwrap(),
        }
    }

    pub fn create_swap_chain_for_hwnd(
        &self,
        command_queue: &CommandQueue,
        desc: &SwapChainDesc,
        hwnd: HWND,
    ) -> *mut dxgi1_2::IDXGISwapChain1 {
        self.as_factory2()
            .create_swap_chain_for_hwnd(command_queue, desc, hwnd)
    }

    pub fn make_window_association(&self, hwnd: HWND, flags: WindowAssociationFlags) {
        self.as_factory2().make_window_association(hwnd, flags);
    }
}

pub struct SwapChain {
    native: ComPtr<dxgi::IDXGISwapChain>,
}

impl SwapChain {
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

    pub fn get_desc(&self, desc: &mut dxgi::DXGI_SWAP_CHAIN_DESC) {
        let hr = unsafe { self.native.GetDesc(desc) };
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on obtaining swap chain description: {:?}", hr);
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

    pub fn resize_buffers(&self, buffers_count: u32, width: u32, height: u32) {
        let mut desc: winapi::shared::dxgi::DXGI_SWAP_CHAIN_DESC = unsafe { mem::zeroed() };
        self.get_desc(&mut desc);

        let hr = unsafe {
            self.native.ResizeBuffers(
                buffers_count,
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
}

pub struct SwapChain1 {
    native: ComPtr<dxgi1_2::IDXGISwapChain1>,
}

impl SwapChain1 {
    pub fn as_swap_chain(&self) -> SwapChain {
        SwapChain {
            native: self.native.cast::<dxgi::IDXGISwapChain>().unwrap(),
        }
    }
}

pub struct SwapChain3 {
    native: ComPtr<dxgi1_4::IDXGISwapChain3>,
}

impl SwapChain3 {
    pub fn get_current_back_buffer_index(&self) -> u32 {
        unsafe { self.native.GetCurrentBackBufferIndex() }
    }
}

pub struct SwapChain4 {
    native: ComPtr<dxgi1_5::IDXGISwapChain4>,
}

impl SwapChain4 {
    pub fn new(
        factory: &Factory4,
        command_queue: &CommandQueue,
        desc: &SwapChainDesc,
        hwnd: HWND,
    ) -> Self {
        let swap_chain1 = unsafe {
            ComPtr::from_raw(factory.create_swap_chain_for_hwnd(command_queue, desc, hwnd))
        };
        SwapChain4 {
            native: swap_chain1.cast::<dxgi1_5::IDXGISwapChain4>().unwrap(),
        }
    }

    pub fn as_swap_chain(&self) -> SwapChain {
        SwapChain {
            native: self.native.cast::<dxgi::IDXGISwapChain>().unwrap(),
        }
    }

    pub fn as_swap_chain1(&self) -> SwapChain1 {
        SwapChain1 {
            native: self.native.cast::<dxgi1_2::IDXGISwapChain1>().unwrap(),
        }
    }

    pub fn as_swap_chain3(&self) -> SwapChain3 {
        SwapChain3 {
            native: self.native.cast::<dxgi1_4::IDXGISwapChain3>().unwrap(),
        }
    }

    pub fn get_current_back_buffer_index(&self) -> u32 {
        self.as_swap_chain3().get_current_back_buffer_index()
    }

    pub fn get_desc(&self, desc: &mut dxgi::DXGI_SWAP_CHAIN_DESC) {
        self.as_swap_chain().get_desc(desc);
    }

    pub fn get_buffer(&self, index: u32) -> ComPtr<d3d12::ID3D12Resource> {
        self.as_swap_chain().get_buffer(index)
    }

    pub fn resize_buffers(&self, buffers_count: u32, width: u32, height: u32) {
        self.as_swap_chain()
            .resize_buffers(buffers_count, width, height);
    }

    pub fn present(&self, sync_interval: u32, flags: u32) {
        self.as_swap_chain().present(sync_interval, flags);
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum Format {
    UNKNOWN = dxgiformat::DXGI_FORMAT_UNKNOWN,
    R32G32B32A32_TYPELESS = dxgiformat::DXGI_FORMAT_R32G32B32A32_TYPELESS,
    R32G32B32A32_FLOAT = dxgiformat::DXGI_FORMAT_R32G32B32A32_FLOAT,
    R32G32B32A32_UINT = dxgiformat::DXGI_FORMAT_R32G32B32A32_UINT,
    R32G32B32A32_SINT = dxgiformat::DXGI_FORMAT_R32G32B32A32_SINT,
    R32G32B32_TYPELESS = dxgiformat::DXGI_FORMAT_R32G32B32_TYPELESS,
    R32G32B32_FLOAT = dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
    R32G32B32_UINT = dxgiformat::DXGI_FORMAT_R32G32B32_UINT,
    R32G32B32_SINT = dxgiformat::DXGI_FORMAT_R32G32B32_SINT,
    R16G16B16A16_TYPELESS = dxgiformat::DXGI_FORMAT_R16G16B16A16_TYPELESS,
    R16G16B16A16_FLOAT = dxgiformat::DXGI_FORMAT_R16G16B16A16_FLOAT,
    R16G16B16A16_UNORM = dxgiformat::DXGI_FORMAT_R16G16B16A16_UNORM,
    R16G16B16A16_UINT = dxgiformat::DXGI_FORMAT_R16G16B16A16_UINT,
    R16G16B16A16_SNORM = dxgiformat::DXGI_FORMAT_R16G16B16A16_SNORM,
    R16G16B16A16_SINT = dxgiformat::DXGI_FORMAT_R16G16B16A16_SINT,
    R32G32_TYPELESS = dxgiformat::DXGI_FORMAT_R32G32_TYPELESS,
    R32G32_FLOAT = dxgiformat::DXGI_FORMAT_R32G32_FLOAT,
    R32G32_UINT = dxgiformat::DXGI_FORMAT_R32G32_UINT,
    R32G32_SINT = dxgiformat::DXGI_FORMAT_R32G32_SINT,
    R32G8X24_TYPELESS = dxgiformat::DXGI_FORMAT_R32G8X24_TYPELESS,
    D32_FLOAT_S8X24_UINT = dxgiformat::DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
    R32_FLOAT_X8X24_TYPELESS = dxgiformat::DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS,
    X32_TYPELESS_G8X24_UINT = dxgiformat::DXGI_FORMAT_X32_TYPELESS_G8X24_UINT,
    R10G10B10A2_TYPELESS = dxgiformat::DXGI_FORMAT_R10G10B10A2_TYPELESS,
    R10G10B10A2_UNORM = dxgiformat::DXGI_FORMAT_R10G10B10A2_UNORM,
    R10G10B10A2_UINT = dxgiformat::DXGI_FORMAT_R10G10B10A2_UINT,
    R11G11B10_FLOAT = dxgiformat::DXGI_FORMAT_R11G11B10_FLOAT,
    R8G8B8A8_TYPELESS = dxgiformat::DXGI_FORMAT_R8G8B8A8_TYPELESS,
    R8G8B8A8_UNORM = dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
    R8G8B8A8_UNORM_SRGB = dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
    R8G8B8A8_UINT = dxgiformat::DXGI_FORMAT_R8G8B8A8_UINT,
    R8G8B8A8_SNORM = dxgiformat::DXGI_FORMAT_R8G8B8A8_SNORM,
    R8G8B8A8_SINT = dxgiformat::DXGI_FORMAT_R8G8B8A8_SINT,
    R16G16_TYPELESS = dxgiformat::DXGI_FORMAT_R16G16_TYPELESS,
    R16G16_FLOAT = dxgiformat::DXGI_FORMAT_R16G16_FLOAT,
    R16G16_UNORM = dxgiformat::DXGI_FORMAT_R16G16_UNORM,
    R16G16_UINT = dxgiformat::DXGI_FORMAT_R16G16_UINT,
    R16G16_SNORM = dxgiformat::DXGI_FORMAT_R16G16_SNORM,
    R16G16_SINT = dxgiformat::DXGI_FORMAT_R16G16_SINT,
    R32_TYPELESS = dxgiformat::DXGI_FORMAT_R32_TYPELESS,
    D32_FLOAT = dxgiformat::DXGI_FORMAT_D32_FLOAT,
    R32_FLOAT = dxgiformat::DXGI_FORMAT_R32_FLOAT,
    R32_UINT = dxgiformat::DXGI_FORMAT_R32_UINT,
    R32_SINT = dxgiformat::DXGI_FORMAT_R32_SINT,
    R24G8_TYPELESS = dxgiformat::DXGI_FORMAT_R24G8_TYPELESS,
    D24_UNORM_S8_UINT = dxgiformat::DXGI_FORMAT_D24_UNORM_S8_UINT,
    R24_UNORM_X8_TYPELESS = dxgiformat::DXGI_FORMAT_R24_UNORM_X8_TYPELESS,
    X24_TYPELESS_G8_UINT = dxgiformat::DXGI_FORMAT_X24_TYPELESS_G8_UINT,
    R8G8_TYPELESS = dxgiformat::DXGI_FORMAT_R8G8_TYPELESS,
    R8G8_UNORM = dxgiformat::DXGI_FORMAT_R8G8_UNORM,
    R8G8_UINT = dxgiformat::DXGI_FORMAT_R8G8_UINT,
    R8G8_SNORM = dxgiformat::DXGI_FORMAT_R8G8_SNORM,
    R8G8_SINT = dxgiformat::DXGI_FORMAT_R8G8_SINT,
    R16_TYPELESS = dxgiformat::DXGI_FORMAT_R16_TYPELESS,
    R16_FLOAT = dxgiformat::DXGI_FORMAT_R16_FLOAT,
    D16_UNORM = dxgiformat::DXGI_FORMAT_D16_UNORM,
    R16_UNORM = dxgiformat::DXGI_FORMAT_R16_UNORM,
    R16_UINT = dxgiformat::DXGI_FORMAT_R16_UINT,
    R16_SNORM = dxgiformat::DXGI_FORMAT_R16_SNORM,
    R16_SINT = dxgiformat::DXGI_FORMAT_R16_SINT,
    R8_TYPELESS = dxgiformat::DXGI_FORMAT_R8_TYPELESS,
    R8_UNORM = dxgiformat::DXGI_FORMAT_R8_UNORM,
    R8_UINT = dxgiformat::DXGI_FORMAT_R8_UINT,
    R8_SNORM = dxgiformat::DXGI_FORMAT_R8_SNORM,
    R8_SINT = dxgiformat::DXGI_FORMAT_R8_SINT,
    A8_UNORM = dxgiformat::DXGI_FORMAT_A8_UNORM,
    R1_UNORM = dxgiformat::DXGI_FORMAT_R1_UNORM,
    R9G9B9E5_SHAREDEXP = dxgiformat::DXGI_FORMAT_R9G9B9E5_SHAREDEXP,
    R8G8_B8G8_UNORM = dxgiformat::DXGI_FORMAT_R8G8_B8G8_UNORM,
    G8R8_G8B8_UNORM = dxgiformat::DXGI_FORMAT_G8R8_G8B8_UNORM,
    BC1_TYPELESS = dxgiformat::DXGI_FORMAT_BC1_TYPELESS,
    BC1_UNORM = dxgiformat::DXGI_FORMAT_BC1_UNORM,
    BC1_UNORM_SRGB = dxgiformat::DXGI_FORMAT_BC1_UNORM_SRGB,
    BC2_TYPELESS = dxgiformat::DXGI_FORMAT_BC2_TYPELESS,
    BC2_UNORM = dxgiformat::DXGI_FORMAT_BC2_UNORM,
    BC2_UNORM_SRGB = dxgiformat::DXGI_FORMAT_BC2_UNORM_SRGB,
    BC3_TYPELESS = dxgiformat::DXGI_FORMAT_BC3_TYPELESS,
    BC3_UNORM = dxgiformat::DXGI_FORMAT_BC3_UNORM,
    BC3_UNORM_SRGB = dxgiformat::DXGI_FORMAT_BC3_UNORM_SRGB,
    BC4_TYPELESS = dxgiformat::DXGI_FORMAT_BC4_TYPELESS,
    BC4_UNORM = dxgiformat::DXGI_FORMAT_BC4_UNORM,
    BC4_SNORM = dxgiformat::DXGI_FORMAT_BC4_SNORM,
    BC5_TYPELESS = dxgiformat::DXGI_FORMAT_BC5_TYPELESS,
    BC5_UNORM = dxgiformat::DXGI_FORMAT_BC5_UNORM,
    BC5_SNORM = dxgiformat::DXGI_FORMAT_BC5_SNORM,
    B5G6R5_UNORM = dxgiformat::DXGI_FORMAT_B5G6R5_UNORM,
    B5G5R5A1_UNORM = dxgiformat::DXGI_FORMAT_B5G5R5A1_UNORM,
    B8G8R8A8_UNORM = dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
    B8G8R8X8_UNORM = dxgiformat::DXGI_FORMAT_B8G8R8X8_UNORM,
    R10G10B10_XR_BIAS_A2_UNORM = dxgiformat::DXGI_FORMAT_R10G10B10_XR_BIAS_A2_UNORM,
    B8G8R8A8_TYPELESS = dxgiformat::DXGI_FORMAT_B8G8R8A8_TYPELESS,
    B8G8R8A8_UNORM_SRGB = dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
    B8G8R8X8_TYPELESS = dxgiformat::DXGI_FORMAT_B8G8R8X8_TYPELESS,
    B8G8R8X8_UNORM_SRGB = dxgiformat::DXGI_FORMAT_B8G8R8X8_UNORM_SRGB,
    BC6H_TYPELESS = dxgiformat::DXGI_FORMAT_BC6H_TYPELESS,
    BC6H_UF16 = dxgiformat::DXGI_FORMAT_BC6H_UF16,
    BC6H_SF16 = dxgiformat::DXGI_FORMAT_BC6H_SF16,
    BC7_TYPELESS = dxgiformat::DXGI_FORMAT_BC7_TYPELESS,
    BC7_UNORM = dxgiformat::DXGI_FORMAT_BC7_UNORM,
    BC7_UNORM_SRGB = dxgiformat::DXGI_FORMAT_BC7_UNORM_SRGB,
    AYUV = dxgiformat::DXGI_FORMAT_AYUV,
    Y410 = dxgiformat::DXGI_FORMAT_Y410,
    Y416 = dxgiformat::DXGI_FORMAT_Y416,
    NV12 = dxgiformat::DXGI_FORMAT_NV12,
    P010 = dxgiformat::DXGI_FORMAT_P010,
    P016 = dxgiformat::DXGI_FORMAT_P016,
    F420_OPAQUE = dxgiformat::DXGI_FORMAT_420_OPAQUE,
    YUY2 = dxgiformat::DXGI_FORMAT_YUY2,
    Y210 = dxgiformat::DXGI_FORMAT_Y210,
    Y216 = dxgiformat::DXGI_FORMAT_Y216,
    NV11 = dxgiformat::DXGI_FORMAT_NV11,
    AI44 = dxgiformat::DXGI_FORMAT_AI44,
    IA44 = dxgiformat::DXGI_FORMAT_IA44,
    P8 = dxgiformat::DXGI_FORMAT_P8,
    A8P8 = dxgiformat::DXGI_FORMAT_A8P8,
    B4G4R4A4_UNORM = dxgiformat::DXGI_FORMAT_B4G4R4A4_UNORM,
    P208 = dxgiformat::DXGI_FORMAT_P208,
    V208 = dxgiformat::DXGI_FORMAT_V208,
    V408 = dxgiformat::DXGI_FORMAT_V408,
}
