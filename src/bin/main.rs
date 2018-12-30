extern crate graphicx;
extern crate winit;

use winapi::shared::windef;
use winapi::um::{d3d12, handleapi};
use winit::dpi::LogicalSize;
use winit::os::windows::WindowExt;
use winit::{Event, EventsLoop, KeyboardInput, VirtualKeyCode, WindowBuilder, WindowEvent};
use wio::com::ComPtr;

fn main() {
    let mut width: u32 = 1280;
    let mut height: u32 = 720;
    let use_warp = false;
    let is_vsync_enabled = true;
    //let is_fullscreen = false;

    // TODO: parse command line args for window width/height and warp mode

    // Enable debug layer
    graphicx::enable_debug_layer();

    // Create window
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_dimensions(LogicalSize::new(width as _, height as _))
        .with_title("Learning DirectX 12 with Rust")
        .build(&events_loop)
        .unwrap();
    let window_handle: windef::HWND = window.get_hwnd() as *mut _;

    let dxgi_adapter = graphicx::get_adapter(use_warp);
    let device = graphicx::create_device(dxgi_adapter);
    let command_queue =
        graphicx::create_command_queue(device.clone(), d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT);

    let back_buffers_count: usize = 3;
    let is_tearing_supported = graphicx::is_tearing_supported();
    let swap_chain = graphicx::create_swap_chain(
        command_queue.clone(),
        window_handle,
        width,
        height,
        back_buffers_count,
        is_tearing_supported,
    );
    let mut current_back_buffer_index: usize =
        unsafe { swap_chain.GetCurrentBackBufferIndex() } as _;
    let rtv_descriptor_heap = graphicx::create_descriptor_heap(
        device.clone(),
        d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
        back_buffers_count,
    );
    let rtv_descriptor_size: usize =
        unsafe { device.GetDescriptorHandleIncrementSize(d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV) }
            as _;

    let mut back_buffers: Vec<ComPtr<d3d12::ID3D12Resource>> =
        Vec::with_capacity(back_buffers_count);
    graphicx::update_render_target_views(
        device.clone(),
        swap_chain.clone(),
        rtv_descriptor_heap.clone(),
        back_buffers_count,
        &mut back_buffers,
    );

    let mut command_allocators: Vec<ComPtr<d3d12::ID3D12CommandAllocator>> =
        Vec::with_capacity(back_buffers_count);
    for _ in 0..back_buffers_count {
        command_allocators.push(graphicx::create_command_allocator(
            device.clone(),
            d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
        ));
    }
    let command_list = graphicx::create_command_list(
        device.clone(),
        command_allocators[current_back_buffer_index].clone(),
        d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
    );

    let fence = graphicx::create_fence(device.clone());
    let fence_event = graphicx::create_fence_event(false, false);
    let mut fence_value: u64 = 0;
    let mut frame_fence_values: [u64; 3] = [0, 0, 0];

    let mut running = true;
    let mut need_resize = false;
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
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    }
                    | WindowEvent::CloseRequested => {
                        println!("The close button was pressed");
                        running = false;
                    }
                    WindowEvent::Resized(LogicalSize { width, height }) => {
                        println!("The window was resized to {}x{}", width, height);
                        need_resize =
                            width as u32 != resize_width || height as u32 != resize_height;
                        if need_resize {
                            resize_width = width as _;
                            resize_height = height as _;
                        }
                    }
                    _ => (),
                }
            }
        });

        if need_resize {
            println!("Resizing...");
            graphicx::resize(
                device.clone(),
                command_queue.clone(),
                &mut back_buffers,
                &mut current_back_buffer_index,
                back_buffers_count,
                swap_chain.clone(),
                rtv_descriptor_heap.clone(),
                fence.clone(),
                &mut frame_fence_values,
                &mut fence_value,
                fence_event,
                &mut width,
                &mut height,
                resize_width,
                resize_height,
            );
            need_resize = false;
        }

        // Update and render
        graphicx::update(&mut frame_counter, &mut elapsed_time_secs, &mut t0);
        graphicx::render(
            &command_allocators,
            &back_buffers,
            &mut current_back_buffer_index,
            command_list.clone(),
            command_queue.clone(),
            rtv_descriptor_heap.clone(),
            rtv_descriptor_size,
            swap_chain.clone(),
            fence.clone(),
            &mut frame_fence_values,
            &mut fence_value,
            fence_event,
            is_tearing_supported,
            is_vsync_enabled,
        );
    }

    println!("Cleanup!");
    graphicx::flush(command_queue.clone(), fence, &mut fence_value, fence_event);
    unsafe { handleapi::CloseHandle(fence_event) };
}
