extern crate graphicx;

use graphicx::dx12;
use winapi::um::d3d12;

use std::env;
use std::mem;

fn main() {
    // Parse command line args into a config
    let args: Vec<String> = env::args().collect();
    let mut config = graphicx::Config::new(&args);
    println!("{:?}", config);

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(
            f64::from(config.width),
            f64::from(config.height),
        ))
        .with_title("Learning DirectX 12 with Rust")
        .build(&events_loop)
        .unwrap();

    // Enable debug layer
    graphicx::dx12::enable_debug_layer();

    let dxgi_adapter = graphicx::dx12::get_adapter(config.use_warp);
    let device = dx12::Device::new(&dxgi_adapter);
    let command_queue = dx12::CommandQueue::new(
        &device,
        dx12::CommandListType::Direct,
        dx12::CommandQueuePriority::Normal,
        dx12::CommandQueueFlags::None,
        0,
    );

    let back_buffers_count: usize = 3;
    let is_tearing_supported = graphicx::dx12::is_tearing_supported();
    let swap_chain = dx12::SwapChain4::new(
        &command_queue,
        &window,
        config.width,
        config.height,
        back_buffers_count,
        is_tearing_supported,
    );
    let mut current_back_buffer_index: usize = swap_chain.get_current_back_buffer_index() as _; // TODO: Change to u32
    let descriptor_heap = dx12::DescriptorHeap::new(
        &device,
        dx12::DescriptorHeapType::RTV,
        dx12::DescriptorHeapFlags::None,
        back_buffers_count,
        0,
    );
    let descriptor_size: usize =
        device.get_descriptor_increment_size(dx12::DescriptorHeapType::RTV) as _;
    let mut descriptor = descriptor_heap.get_cpu_descriptor_start();
    let mut back_buffers: Vec<dx12::Resource> = Vec::with_capacity(back_buffers_count);
    for i in 0..back_buffers_count {
        let back_buffer = dx12::Resource::new(&swap_chain, i as u32);

        device.create_render_target_view(&back_buffer, descriptor);
        back_buffers.push(back_buffer);

        descriptor.ptr += descriptor_size;
    }

    let mut command_allocators: Vec<dx12::CommandAllocator> =
        Vec::with_capacity(back_buffers_count);
    for _ in 0..back_buffers_count {
        command_allocators.push(dx12::CommandAllocator::new(
            &device,
            dx12::CommandListType::Direct,
        ));
    }
    let graphics_command_list = dx12::GraphicsCommandList::new(
        &device,
        &command_allocators[current_back_buffer_index],
        dx12::CommandListType::Direct,
    );

    let fence = dx12::Fence::new(&device);
    let fence_event = dx12::Event::new(false, false);
    let mut fence_value: u64 = 0;
    let mut frame_fence_values: [u64; 3] = [0, 0, 0];

    let mut is_resize_requested = false;
    let mut is_fullscreen = config.is_fullscreen;
    if is_fullscreen {
        graphicx::set_fullscreen(&window, config.is_fullscreen);
    }

    let mut frame_counter: u64 = 0;
    let mut previous_frame_time = std::time::Instant::now();
    let mut elapsed_time_secs: f64 = 0.0;

    let mut is_running = true;
    while is_running {
        frame_counter += 1;
        let current_frame_time = std::time::Instant::now();
        let delta_time = current_frame_time - previous_frame_time;
        let delta_time_secs =
            delta_time.as_secs() as f64 + f64::from(delta_time.subsec_nanos()) * 1e-9;
        previous_frame_time = current_frame_time;
        elapsed_time_secs += delta_time_secs;

        // Show fps
        if cfg!(debug_assertions) && elapsed_time_secs > 1.0 {
            let fps = frame_counter as f64 / elapsed_time_secs;
            println!("FPS: {}", fps);

            frame_counter = 0;
            elapsed_time_secs = 0.0;
        }

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

        if is_resize_requested {
            println!("Resizing!");

            // Don't allow 0 size swap chain back buffers.
            let width = 1.max(config.width);
            let height = 1.max(config.height);

            // Flush the GPU queue to make sure the swap chain's back buffers
            // are not being referenced by an in-flight command list.
            command_queue.flush(&fence, fence_event, &mut fence_value);

            // Any references to the back buffers must be released
            // before the swap chain can be resized.
            while let Some(back_buffer) = back_buffers.pop() {
                std::mem::drop(back_buffer);
            }

            // Reset per-frame fence values to the fence value of the current back buffer index
            for i in 0..back_buffers_count {
                frame_fence_values[i] = frame_fence_values[current_back_buffer_index];
            }

            swap_chain.resize_buffers(back_buffers_count as _, width, height);

            current_back_buffer_index = swap_chain.get_current_back_buffer_index() as _;

            let mut descriptor = descriptor_heap.get_cpu_descriptor_start();

            for i in 0..back_buffers_count {
                let back_buffer = dx12::Resource::new(&swap_chain, i as u32);

                device.create_render_target_view(&back_buffer, descriptor);
                back_buffers.push(back_buffer);

                descriptor.ptr += descriptor_size;
            }

            is_resize_requested = false;
        }

        if config.is_fullscreen != is_fullscreen {
            config.is_fullscreen = is_fullscreen;
            graphicx::set_fullscreen(&window, config.is_fullscreen);
        }

        // Render
        {
            let command_allocator = &command_allocators[current_back_buffer_index];
            let back_buffer = &back_buffers[current_back_buffer_index];

            // Reset current command allocator and command list before new commands can be recorded
            command_allocator.reset();
            graphics_command_list.reset(&command_allocator);

            // Clear render target
            {
                let mut barrier = d3d12::D3D12_RESOURCE_BARRIER {
                    Type: d3d12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                    Flags: d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE,
                    u: unsafe { mem::zeroed() },
                };

                *unsafe { barrier.u.Transition_mut() } = d3d12::D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: back_buffer.as_mut_ptr(),
                    Subresource: d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    StateBefore: d3d12::D3D12_RESOURCE_STATE_PRESENT,
                    StateAfter: d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
                };

                graphics_command_list.add_barriers(&barrier, 1);

                let clear_color: [f32; 4] = [0.56, 0.93, 0.56, 1.0];
                let mut rtv = descriptor_heap.get_cpu_descriptor_start();
                rtv.ptr += current_back_buffer_index * descriptor_size;

                graphics_command_list.clear_render_target_view(rtv, clear_color);
            }

            // Present the back buffer
            {
                let mut barrier = d3d12::D3D12_RESOURCE_BARRIER {
                    Type: d3d12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                    Flags: d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE,
                    u: unsafe { mem::zeroed() },
                };

                *unsafe { barrier.u.Transition_mut() } = d3d12::D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: back_buffer.as_mut_ptr(),
                    Subresource: d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    StateBefore: d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
                    StateAfter: d3d12::D3D12_RESOURCE_STATE_PRESENT,
                };

                graphics_command_list.add_barriers(&barrier, 1);
                graphics_command_list.close();

                let command_lists = vec![graphics_command_list.as_command_list()];
                command_queue.execute(&command_lists.as_slice());

                let sync_interval = if config.is_vsync_enabled { 1 } else { 0 };
                let present_flags = if is_tearing_supported && !config.is_vsync_enabled {
                    winapi::shared::dxgi::DXGI_PRESENT_ALLOW_TEARING
                } else {
                    0
                };
                swap_chain.present(sync_interval, present_flags);

                // Insert a signal into the command queue with a fence value
                frame_fence_values[current_back_buffer_index] =
                    command_queue.signal(&fence, &mut fence_value);

                current_back_buffer_index = swap_chain.get_current_back_buffer_index() as _;

                // Stall the CPU until fence value signalled is reached
                fence.wait_for_value(fence_event, frame_fence_values[current_back_buffer_index]);
            }
        }
    }

    println!("Cleanup!");
    command_queue.flush(&fence, fence_event, &mut fence_value);
    fence_event.close();

    println!("Bye!");
}
