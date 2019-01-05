extern crate graphicx;

use graphicx::dx12;

use winit::os::windows::WindowExt;

use std::env;

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

    let buffers_count: u32 = 3;
    let is_tearing_supported = dx12::is_tearing_supported();
    let factory = dx12::Factory4::new(if cfg!(debug_assertions) {
        dx12::FactoryCreationFlags::Debug
    } else {
        dx12::FactoryCreationFlags::None
    });
    let hwnd = window.get_hwnd() as *mut _;
    factory.make_window_association(hwnd, dx12::WindowAssociationFlags::NoAltEnter);
    let swap_chain_desc = dx12::SwapChainDesc {
        width: config.width,
        height: config.height,
        format: dx12::Format::R8G8B8A8_UNORM,
        stereo: false,
        sample_desc: dx12::SampleDesc {
            count: 1,
            quality: 0,
        },
        buffer_usage: dx12::Usage::RenderTargetOutput,
        buffer_count: buffers_count,
        scaling: dx12::Scaling::Stretch,
        swap_effect: dx12::SwapEffect::FlipDiscard,
        alpha_mode: dx12::AlphaMode::Unspecified,
        flags: if is_tearing_supported {
            dx12::Flags::AllowTearing
        } else {
            dx12::Flags::None
        },
    };
    let swap_chain = dx12::SwapChain4::new(&factory, &command_queue, &swap_chain_desc, hwnd);
    let mut current_back_buffer_index: usize = swap_chain.get_current_back_buffer_index() as _;
    let descriptor_heap = dx12::DescriptorHeap::new(
        &device,
        dx12::DescriptorHeapType::RTV,
        dx12::DescriptorHeapFlags::None,
        buffers_count,
        0,
    );
    let descriptor_size: usize =
        device.get_descriptor_increment_size(dx12::DescriptorHeapType::RTV) as _;
    let mut descriptor = descriptor_heap.get_cpu_descriptor_start();
    let mut back_buffers: Vec<dx12::Resource> = Vec::with_capacity(buffers_count as _);
    for i in 0..buffers_count {
        let back_buffer = dx12::Resource::new(&swap_chain, i as u32);

        device.create_render_target_view(&back_buffer, descriptor);
        back_buffers.push(back_buffer);

        descriptor.ptr += descriptor_size;
    }

    let mut command_allocators: Vec<dx12::CommandAllocator> =
        Vec::with_capacity(buffers_count as _);
    for _ in 0..buffers_count {
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
            for i in 0..buffers_count as _ {
                frame_fence_values[i] = frame_fence_values[current_back_buffer_index];
            }

            swap_chain.resize_buffers(buffers_count as _, width, height);

            current_back_buffer_index = swap_chain.get_current_back_buffer_index() as _;

            let mut descriptor = descriptor_heap.get_cpu_descriptor_start();

            for i in 0..buffers_count {
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
            // Reset current command allocator and command list before new commands can be recorded
            let command_allocator = &command_allocators[current_back_buffer_index];
            command_allocator.reset();
            graphics_command_list.reset(&command_allocator);

            // Clear render target
            {
                let barriers = vec![dx12::BarrierDesc::new(
                    current_back_buffer_index,
                    dx12::ResourceStates::Present..dx12::ResourceStates::RenderTarget,
                )];
                graphics_command_list.insert_transition_barriers(&barriers, &back_buffers);

                let clear_color: [f32; 4] = [0.56, 0.93, 0.56, 1.0];
                let mut rtv = descriptor_heap.get_cpu_descriptor_start();
                rtv.ptr += current_back_buffer_index * descriptor_size;

                graphics_command_list.clear_render_target_view(rtv, clear_color);
            }

            // Present the back buffer
            {
                let barriers = vec![dx12::BarrierDesc::new(
                    current_back_buffer_index,
                    dx12::ResourceStates::RenderTarget..dx12::ResourceStates::Present,
                )];
                graphics_command_list.insert_transition_barriers(&barriers, &back_buffers);

                graphics_command_list.close();

                let command_lists = vec![graphics_command_list.as_command_list()];
                command_queue.execute(&command_lists);

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
