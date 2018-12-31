extern crate graphicx;
extern crate winit;

use winapi::shared::windef;
use winapi::um::{d3d12, handleapi};
use winit::dpi::LogicalSize;
use winit::os::windows::WindowExt;
use winit::{
    ElementState, Event, EventsLoop, KeyboardInput, ModifiersState, VirtualKeyCode, WindowBuilder,
    WindowEvent,
};
use wio::com::ComPtr;

fn main() {
    let mut width: u32 = 1280;
    let mut height: u32 = 720;
    let use_warp = false;
    let mut is_vsync_enabled = false;
    let mut is_fullscreen = false;

    // TODO: parse command line args for window width/height and warp mode

    // Enable debug layer
    graphicx::enable_debug_layer();

    // Create window
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_dimensions(LogicalSize::new(f64::from(width), f64::from(height)))
        .with_title("Learning DirectX 12 with Rust")
        .build(&events_loop)
        .unwrap();
    let window_handle: windef::HWND = window.get_hwnd() as *mut _;

    let dxgi_adapter = graphicx::get_adapter(use_warp);
    let device = graphicx::create_device(&dxgi_adapter);
    let command_queue =
        graphicx::create_command_queue(&device, d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT);

    let back_buffers_count: usize = 3;
    let is_tearing_supported = graphicx::is_tearing_supported();
    let swap_chain = graphicx::create_swap_chain(
        &command_queue,
        window_handle,
        width,
        height,
        back_buffers_count,
        is_tearing_supported,
    );
    let mut current_back_buffer_index: usize =
        unsafe { swap_chain.GetCurrentBackBufferIndex() } as _;
    let rtv_descriptor_heap = graphicx::create_descriptor_heap(
        &device,
        d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
        back_buffers_count,
    );
    let rtv_descriptor_size: usize =
        unsafe { device.GetDescriptorHandleIncrementSize(d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV) }
            as _;

    let mut back_buffers: Vec<ComPtr<d3d12::ID3D12Resource>> =
        Vec::with_capacity(back_buffers_count);
    graphicx::update_render_target_views(
        &device,
        &swap_chain,
        &rtv_descriptor_heap,
        back_buffers_count,
        &mut back_buffers,
    );

    let mut command_allocators: Vec<ComPtr<d3d12::ID3D12CommandAllocator>> =
        Vec::with_capacity(back_buffers_count);
    for _ in 0..back_buffers_count {
        command_allocators.push(graphicx::create_command_allocator(
            &device,
            d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
        ));
    }
    let command_list = graphicx::create_command_list(
        &device,
        &command_allocators[current_back_buffer_index],
        d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
    );

    let fence = graphicx::create_fence(&device);
    let fence_event = graphicx::create_fence_event(false, false);
    let mut fence_value: u64 = 0;
    let mut frame_fence_values: [u64; 3] = [0, 0, 0];

    let mut running = true;
    let mut is_resize_requested = false;
    let mut fullscreen_toggle_requested = false;
    let mut resize_width: u32 = width;
    let mut resize_height: u32 = height;

    let mut frame_counter: u64 = 0;
    let mut elapsed_time_secs = 0.0;
    let mut t0 = std::time::Instant::now();

    while running {
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::V),
                                state: ElementState::Released,
                                modifiers: ModifiersState { alt: true, .. },
                                ..
                            },
                        ..
                    } => {
                        println!(
                            "Received request to toggle vertical sync to {}",
                            !is_vsync_enabled
                        );
                        is_vsync_enabled = !is_vsync_enabled;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::F),
                                state: ElementState::Released,
                                modifiers: ModifiersState { alt: true, .. },
                                ..
                            },
                        ..
                    } => {
                        println!("Received request to toggle fullscreen");
                        fullscreen_toggle_requested = true;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    }
                    | WindowEvent::CloseRequested => {
                        println!("Received request to close the window");
                        running = false;
                    }
                    WindowEvent::Resized(LogicalSize { width, height }) => {
                        println!(
                            "Received request to resize the window to {}x{}",
                            width, height
                        );
                        is_resize_requested =
                            width as u32 != resize_width || height as u32 != resize_height;
                        if is_resize_requested {
                            resize_width = width as _;
                            resize_height = height as _;
                        }
                    }
                    _ => (),
                }
            }
        });

        if is_resize_requested {
            println!("Resizing!");
            graphicx::resize(
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
                &mut width,
                &mut height,
                resize_width,
                resize_height,
            );
            is_resize_requested = false;
        }

        if fullscreen_toggle_requested {
            is_fullscreen = !is_fullscreen;
            graphicx::set_fullscreen(&window, is_fullscreen);
            fullscreen_toggle_requested = false;
        }

        // Update and render
        graphicx::update(&mut frame_counter, &mut elapsed_time_secs, &mut t0);
        graphicx::render(
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
            is_vsync_enabled,
        );
    }

    println!("Cleanup!");
    graphicx::flush(&command_queue, &fence, &mut fence_value, fence_event);
    unsafe { handleapi::CloseHandle(fence_event) };

    println!("Bye!");
}
