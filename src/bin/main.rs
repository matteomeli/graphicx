extern crate graphicx;

use graphicx::GameLoop;

use winapi::shared::windef;
use winapi::um::{d3d12, handleapi};
use winit::os::windows::WindowExt;

use wio::com::ComPtr;

use std::env;

fn main() {
    // Parse command line args into a config
    let args: Vec<String> = env::args().collect();
    let mut config = graphicx::Config::new(&args);
    println!("{:?}", config);

    let mut game_loop = GameLoop::new();
    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(
            f64::from(config.width),
            f64::from(config.height),
        ))
        .with_title("Learning DirectX 12 with Rust")
        .build(&events_loop)
        .unwrap();

    let window_handle: windef::HWND = window.get_hwnd() as *mut _;

    // Enable debug layer
    graphicx::dx12::enable_debug_layer();

    let dxgi_adapter = graphicx::dx12::get_adapter(config.use_warp);
    let device = graphicx::dx12::create_device(&dxgi_adapter);
    let command_queue =
        graphicx::dx12::create_command_queue(&device, d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT);

    let back_buffers_count: usize = 3;
    let is_tearing_supported = graphicx::dx12::is_tearing_supported();
    let swap_chain = graphicx::dx12::create_swap_chain(
        &command_queue,
        window_handle,
        config.width,
        config.height,
        back_buffers_count,
        is_tearing_supported,
    );
    let mut current_back_buffer_index: usize =
        unsafe { swap_chain.GetCurrentBackBufferIndex() } as _;
    let rtv_descriptor_heap = graphicx::dx12::create_descriptor_heap(
        &device,
        d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
        back_buffers_count,
    );
    let rtv_descriptor_size: usize =
        unsafe { device.GetDescriptorHandleIncrementSize(d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV) }
            as _;

    let mut back_buffers: Vec<ComPtr<d3d12::ID3D12Resource>> =
        Vec::with_capacity(back_buffers_count);
    graphicx::dx12::update_render_target_views(
        &device,
        &swap_chain,
        &rtv_descriptor_heap,
        back_buffers_count,
        &mut back_buffers,
    );

    let mut command_allocators: Vec<ComPtr<d3d12::ID3D12CommandAllocator>> =
        Vec::with_capacity(back_buffers_count);
    for _ in 0..back_buffers_count {
        command_allocators.push(graphicx::dx12::create_command_allocator(
            &device,
            d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
        ));
    }
    let command_list = graphicx::dx12::create_command_list(
        &device,
        &command_allocators[current_back_buffer_index],
        d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
    );

    let fence = graphicx::dx12::create_fence(&device);
    let fence_event = graphicx::dx12::create_fence_event(false, false);
    let mut fence_value: u64 = 0;
    let mut frame_fence_values: [u64; 3] = [0, 0, 0];

    let mut is_resize_requested = false;
    let mut is_fullscreen = config.is_fullscreen;
    if is_fullscreen {
        graphicx::window::set_fullscreen(&window, config.is_fullscreen);
    }

    let mut is_running = true;
    while is_running {
        game_loop.frame();

        events_loop.poll_events(|event| {
            if let winit::Event::WindowEvent { event, .. } = event {
                match event {
                    winit::WindowEvent::KeyboardInput {
                        input:
                            winit::KeyboardInput {
                                virtual_keycode: Some(winit::VirtualKeyCode::V),
                                state: winit::ElementState::Released,
                                modifiers: winit::ModifiersState { alt: true, .. },
                                ..
                            },
                        ..
                    } => {
                        println!(
                            "Received request to toggle vertical sync to {}",
                            !config.is_vsync_enabled
                        );
                        config.is_vsync_enabled = !config.is_vsync_enabled;
                    }
                    winit::WindowEvent::KeyboardInput {
                        input:
                            winit::KeyboardInput {
                                virtual_keycode: Some(winit::VirtualKeyCode::F),
                                state: winit::ElementState::Released,
                                modifiers: winit::ModifiersState { alt: true, .. },
                                ..
                            },
                        ..
                    } => {
                        println!("Received request to toggle fullscreen");
                        is_fullscreen = !is_fullscreen;
                    }
                    winit::WindowEvent::KeyboardInput {
                        input:
                            winit::KeyboardInput {
                                virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    }
                    | winit::WindowEvent::CloseRequested => {
                        println!("Received request to close the window");
                        is_running = false;
                    }
                    winit::WindowEvent::Resized(winit::dpi::LogicalSize { width, height }) => {
                        println!(
                            "Received request to resize the window to {}x{}",
                            width, height
                        );
                        if width as u32 != config.width || height as u32 != config.height {
                            is_resize_requested = true;
                            config.width = width as _;
                            config.height = height as _;
                        }
                    }
                    _ => (),
                }
            }
        });

        game_loop.update();

        if is_resize_requested {
            println!("Resizing!");
            graphicx::dx12::resize(
                &device,
                &command_queue,
                &mut back_buffers,
                &mut current_back_buffer_index,
                back_buffers_count,
                &swap_chain,
                &rtv_descriptor_heap,
                &fence,
                &mut frame_fence_values,
                &mut fence_value,
                fence_event,
                config.width,
                config.height,
            );
            is_resize_requested = false;
        }

        if config.is_fullscreen != is_fullscreen {
            config.is_fullscreen = is_fullscreen;
            graphicx::window::set_fullscreen(&window, config.is_fullscreen);
        }

        // Render
        graphicx::dx12::render(
            &command_allocators,
            &back_buffers,
            &mut current_back_buffer_index,
            &command_list,
            &command_queue,
            &rtv_descriptor_heap,
            rtv_descriptor_size,
            &swap_chain,
            &fence,
            &mut frame_fence_values,
            &mut fence_value,
            fence_event,
            is_tearing_supported,
            config.is_vsync_enabled,
        );
    }

    println!("Cleanup!");
    graphicx::dx12::flush(&command_queue, &fence, &mut fence_value, fence_event);
    unsafe { handleapi::CloseHandle(fence_event) };

    println!("Bye!");
}
