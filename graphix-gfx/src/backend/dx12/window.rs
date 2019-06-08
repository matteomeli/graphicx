use crate::backend::dx12::device::Device;
use crate::backend::dx12::heap::DescriptorHeap;
use crate::backend::dx12::instance::{Backend, Instance};
use crate::backend::dx12::queue::CommandQueue;
use crate::backend::dx12::resource::{BufferView, FrameBuffer};
use crate::hal;

use graphix_native_dx12 as native;

use winapi::shared::windef::HWND;

impl Instance {
    pub fn create_surface_from_hwnd(&self, window_handle: HWND) -> Surface {
        Surface {
            factory: self.factory.clone(),
            window_handle,
        }
    }

    #[cfg(feature = "winit")]
    pub fn create_surface(&self, window: &winit::Window) -> Surface {
        use winit::os::windows::WindowExt;
        self.create_surface_from_hwnd(window.get_hwnd() as *mut _)
    }
}

pub struct Surface {
    pub(crate) factory: native::dxgi::Factory,
    pub(crate) window_handle: HWND,
}

impl hal::Surface<Backend> for Surface {
    fn create_swapchain(
        &self,
        device: &Device,
        command_queue: &CommandQueue,
        config: hal::SwapchainConfig,
    ) -> Swapchain {
        Swapchain::new(self, device, command_queue, config)
    }
}

pub struct Swapchain {
    pub(crate) native: native::dxgi::Swapchain,
    pub(crate) heap: DescriptorHeap,
    pub(crate) resources: Vec<native::resource::Resource>,
}

impl Swapchain {
    pub fn new(
        surface: &Surface,
        device: &Device,
        command_queue: &CommandQueue,
        config: hal::window::SwapchainConfig,
    ) -> Self {
        let swap_chain_desc = native::dxgi::SwapChainDesc {
            width: config.width,
            height: config.height,
            format: get_native_format(config.format),
            stereo: false,
            sample_desc: native::dxgi::SampleDesc {
                count: 1,
                quality: 0,
            },
            buffer_usage: native::dxgi::BufferUsage::RENDER_TARGET_OUTPUT,
            buffer_count: config.buffer_count as _,
            scaling: native::dxgi::Scaling::Stretch,
            swap_effect: native::dxgi::SwapEffect::FlipDiscard,
            alpha_mode: native::dxgi::AlphaMode::Unspecified,
            // TODO: Add proper support for tearing
            flags: if false {
                native::dxgi::SwapChainFlags::ALLOW_TEARING
            } else {
                native::dxgi::SwapChainFlags::empty()
            },
        };

        let swapchain = native::dxgi::Swapchain::create(
            &surface.factory,
            &command_queue.native,
            &swap_chain_desc,
            surface.window_handle,
        )
        .expect("Failed to create DXGI swap chain");

        let heap = DescriptorHeap::new(
            device,
            native::heap::DescriptorHeapType::Rtv,
            config.buffer_count,
        );

        let rtv_desc = native::heap::RenderTargetViewDesc::new(get_native_format(config.format));

        let mut resources = Vec::with_capacity(config.buffer_count);
        for i in 0..config.buffer_count {
            let resource = swapchain
                .get_buffer(i as _)
                .expect("Failed to obtain D3D12 resource");
            let rtv_handle = heap.offset(i as _).cpu;
            device
                .native
                .create_render_target_view(&resource, &rtv_desc, rtv_handle);
            resources.push(resource);
        }

        Swapchain {
            native: swapchain,
            heap,
            resources,
        }
    }
}

impl hal::Swapchain<Backend> for Swapchain {
    fn acquire_buffer(&self) -> hal::SwapchainBufferIndex {
        self.native.get_current_back_buffer_index()
    }

    fn present(&self) {
        // TODO: Add proper support for tearing
        self.native
            .present(1, native::dxgi::PresentFlags::empty())
            .expect("Failed to present DXGI swapchain")
    }

    fn create_backbuffer(&self) -> hal::BackBuffer<Backend> {
        let framebuffers = self
            .resources
            .iter()
            .enumerate()
            .map(|(i, resource)| {
                let rtv_handle = self.heap.offset(i as _).cpu;
                let buffer_view = BufferView {
                    resource: resource.clone(),
                    rtv_handle: Some(rtv_handle),
                };

                FrameBuffer {
                    attachments: vec![buffer_view],
                }
            })
            .collect();

        hal::BackBuffer { framebuffers }
    }
}

fn get_native_format(format: hal::format::Format) -> native::dxgi::Format {
    match format {
        hal::format::Format::Rgba8Unorm => native::dxgi::Format::R8G8B8A8_UNORM,
        // TODO: Add conversions for other formats
    }
}
