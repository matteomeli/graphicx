use crate::queue::CommandQueue;
use crate::resource::Resource;
use crate::Result;

use bitflags::bitflags;

use winapi::shared::guiddef::GUID;
use winapi::shared::windef::HWND;
use winapi::shared::{
    dxgi, dxgi1_2, dxgi1_3, dxgi1_4, dxgi1_5, dxgi1_6, dxgiformat, dxgitype, winerror,
};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::{d3d12, d3dcommon};
use winapi::Interface;
use wio::com::ComPtr;

use std::mem;
use std::ptr;

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
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

bitflags! {
    pub struct PresentFlags: u32 {
        const DO_NOT_SEQUENCE = dxgi::DXGI_PRESENT_DO_NOT_SEQUENCE;
        const TEST = dxgi::DXGI_PRESENT_TEST;
        const RESTART = dxgi::DXGI_PRESENT_RESTART;
        const DO_NOT_WAIT = dxgi::DXGI_PRESENT_DO_NOT_WAIT;
        const RESTRICT_TO_OUTPUT = dxgi::DXGI_PRESENT_RESTRICT_TO_OUTPUT;
        const PREFER_RIGHT = dxgi::DXGI_PRESENT_STEREO_PREFER_RIGHT;
        const STEREO_TEMPORARY_MONO = dxgi::DXGI_PRESENT_STEREO_TEMPORARY_MONO;
        const USE_DURATION = dxgi::DXGI_PRESENT_USE_DURATION;
        const ALLOW_TEARING = dxgi::DXGI_PRESENT_ALLOW_TEARING;
    }
}

bitflags! {
    pub struct WindowAssociationFlags: u32 {
        const NO_WINDOW_CHANGES = 1;
        const NO_ALT_ENTER = 1 << 1;
        const NO_PRINT_SCREEN = 1 << 2;
        const VALID = 0x7;
    }
}

bitflags! {
    pub struct FactoryCreationFlags: u32 {
        const DEBUG = dxgi1_3::DXGI_CREATE_FACTORY_DEBUG;
    }
}

#[repr(transparent)]
pub struct Factory(ComPtr<dxgi1_4::IDXGIFactory4>);

impl Factory {
    pub fn new(flags: FactoryCreationFlags) -> Result<Self> {
        let factory = Factory::create_factory(flags)?;

        Ok(Factory(factory))
    }

    pub fn enumerate_adapters(&self) -> Vec<Adapter> {
        let mut index = 0;
        let mut adapters = Vec::new();
        loop {
            let adapter = Adapter::enumerate(self, index);
            match adapter {
                Ok(adapter) => {
                    index += 1;

                    // Skip the Basic Render Driver adapter.
                    let mut desc: dxgi::DXGI_ADAPTER_DESC1 = unsafe { mem::zeroed() };
                    unsafe {
                        adapter.0.GetDesc1(&mut desc);
                    }
                    if (desc.Flags & dxgi::DXGI_ADAPTER_FLAG_SOFTWARE) != 0 {
                        continue;
                    }

                    // Check to see if the adapter supports Direct3D 12, but don't create the actual device yet.
                    let hr = unsafe {
                        d3d12::D3D12CreateDevice(
                            adapter.0.as_raw() as *mut _,
                            d3dcommon::D3D_FEATURE_LEVEL_11_0,
                            &d3d12::ID3D12Device::uuidof(),
                            ptr::null_mut(),
                        )
                    };
                    if !winerror::SUCCEEDED(hr) {
                        continue;
                    }

                    adapters.push(adapter);
                }
                Err(hr) if hr == winerror::DXGI_ERROR_NOT_FOUND => break,
                Err(_) => continue,
            }
        }

        adapters
    }

    pub fn enumerate_warp(&self) -> Result<Adapter> {
        Adapter::enumerate_warp(self)
    }

    pub fn make_window_association(&self, hwnd: HWND, flags: WindowAssociationFlags) -> Result<()> {
        let hr = unsafe { self.0.MakeWindowAssociation(hwnd, flags.bits()) };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(hr)
        }
    }

    fn create_factory(flags: FactoryCreationFlags) -> Result<ComPtr<dxgi1_4::IDXGIFactory4>> {
        // Try to get the factory at the newest DXGI version
        Factory::create_dxgi_factory(&dxgi1_6::IDXGIFactory6::uuidof(), flags)
            .or_else(|_| Factory::create_dxgi_factory(&dxgi1_5::IDXGIFactory5::uuidof(), flags))
            .or_else(|_| Factory::create_dxgi_factory(&dxgi1_4::IDXGIFactory4::uuidof(), flags))
    }

    fn create_dxgi_factory(
        guid: &GUID,
        flags: FactoryCreationFlags,
    ) -> Result<ComPtr<dxgi1_4::IDXGIFactory4>> {
        let mut factory: *mut IUnknown = ptr::null_mut();
        let hr = unsafe {
            dxgi1_3::CreateDXGIFactory2(
                flags.bits(),
                guid,
                &mut factory as *mut *mut _ as *mut *mut _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(unsafe { ComPtr::from_raw(factory as *mut _) })
        } else {
            Err(hr)
        }
    }

    pub fn is_tearing_supported(&self) -> Result<bool> {
        let mut allow_tearing: i32 = 0;
        let hr = unsafe {
            (*(self.0.as_raw() as *mut dxgi1_5::IDXGIFactory5)).CheckFeatureSupport(
                dxgi1_5::DXGI_FEATURE_PRESENT_ALLOW_TEARING,
                &mut allow_tearing as *mut _ as *mut _,
                mem::size_of::<i32>() as _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(allow_tearing == 1)
        } else {
            Err(hr)
        }
    }
}

impl Clone for Factory {
    fn clone(&self) -> Self {
        Factory(self.0.clone())
    }
}

#[repr(transparent)]
pub struct Adapter(pub(crate) ComPtr<dxgi::IDXGIAdapter1>);

impl Adapter {
    pub fn enumerate(factory: &Factory, index: u32) -> Result<Adapter> {
        let mut adapter: *mut dxgi::IDXGIAdapter1 = ptr::null_mut();
        let hr = match factory.0.cast::<dxgi1_6::IDXGIFactory6>() {
            Ok(factory6) => unsafe {
                factory6.EnumAdapterByGpuPreference(
                    index,
                    dxgi1_6::DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
                    &dxgi1_6::IDXGIAdapter4::uuidof(),
                    &mut adapter as *mut *mut _ as *mut *mut _,
                )
            },
            Err(_) => unsafe {
                factory
                    .0
                    .EnumAdapters1(index, &mut adapter as *mut *mut _ as *mut *mut _)
            },
        };

        if winerror::SUCCEEDED(hr) {
            Ok(Adapter(unsafe { ComPtr::from_raw(adapter) }))
        } else {
            Err(hr)
        }
    }

    pub fn enumerate_warp(factory: &Factory) -> Result<Adapter> {
        let mut adapter: *mut dxgi::IDXGIAdapter1 = ptr::null_mut();
        let hr = unsafe {
            factory.0.EnumWarpAdapter(
                &dxgi1_4::IDXGIAdapter3::uuidof(),
                &mut adapter as *mut *mut _ as *mut *mut _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(Adapter(unsafe { ComPtr::from_raw(adapter) }))
        } else {
            Err(hr)
        }
    }

    pub fn as_raw(&self) -> *mut dxgi::IDXGIAdapter1 {
        self.0.as_raw()
    }
}

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

#[repr(transparent)]
pub struct Swapchain(ComPtr<dxgi1_4::IDXGISwapChain3>);

impl Swapchain {
    pub fn create(
        factory: &Factory,
        command_queue: &CommandQueue,
        config: &SwapChainDesc,
        hwnd: HWND,
    ) -> Result<Swapchain> {
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
            (*(factory.0.as_raw() as *mut dxgi1_2::IDXGIFactory2)).CreateSwapChainForHwnd(
                command_queue.0.as_raw() as *mut _,
                hwnd,
                &desc,
                ptr::null(),
                ptr::null_mut(),
                &mut swap_chain1 as *mut *mut _ as *mut *mut _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            // Does not support exclusive full-screen mode and prevents DXGI from responding to the ALT+ENTER shortcut
            factory.make_window_association(hwnd, WindowAssociationFlags::NO_ALT_ENTER)?;

            let swap_chain1 = unsafe { ComPtr::from_raw(swap_chain1) };
            let swap_chain3 = swap_chain1.cast::<dxgi1_4::IDXGISwapChain3>()?;
            Ok(Swapchain(swap_chain3))
        } else {
            Err(hr)
        }
    }

    pub fn get_buffer(&self, index: u32) -> Result<Resource> {
        let mut buffer: *mut d3d12::ID3D12Resource = ptr::null_mut();
        let hr = unsafe {
            self.0.GetBuffer(
                index,
                &d3d12::ID3D12Resource::uuidof(),
                &mut buffer as *mut *mut _ as *mut *mut _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(Resource(unsafe { ComPtr::from_raw(buffer) }))
        } else {
            Err(hr)
        }
    }

    pub fn get_current_back_buffer_index(&self) -> u32 {
        unsafe { self.0.GetCurrentBackBufferIndex() }
    }

    pub fn present(&self, sync_interval: u32, flags: PresentFlags) -> Result<()> {
        let hr = unsafe { self.0.Present(sync_interval, flags.bits()) };
        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(hr)
        }
    }

    pub fn resize_buffers(&self, buffers_count: u32, width: u32, height: u32) -> Result<()> {
        let desc = self.get_desc()?;

        let hr = unsafe {
            self.0.ResizeBuffers(
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
            Err(hr)
        }
    }

    fn get_desc(&self) -> Result<dxgi::DXGI_SWAP_CHAIN_DESC> {
        let mut desc: dxgi::DXGI_SWAP_CHAIN_DESC = unsafe { mem::zeroed() };
        let hr = unsafe { self.0.GetDesc(&mut desc) };

        if winerror::SUCCEEDED(hr) {
            Ok(desc)
        } else {
            Err(hr)
        }
    }
}
