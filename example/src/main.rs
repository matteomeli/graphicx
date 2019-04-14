use framework::dx12;

use std::env;

use winit::os::windows::WindowExt;

fn main() -> dx12::D3DResult<()> {
    // Parse command line args into a config
    let args: Vec<String> = env::args().collect();
    let mut config = framework::Config::new(&args);
    println!("{:?}", config);

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(
            f64::from(config.width),
            f64::from(config.height),
        ))
        .with_title("Rust + DirectX 12")
        .build(&events_loop)
        .unwrap();

    let hwnd = window.get_hwnd() as *mut _;
    let factory = dx12::Factory::create()?;
    factory.make_window_association(hwnd, dx12::WindowAssociationFlags::NO_ALT_ENTER)?;

    let adapter = if config.use_warp {
        dx12::Adapter::enumerate_warp(&factory)
    } else {
        let mut adapters = factory.get_adapters(dx12::GpuPreference::HighPerformance);
        for adapter in &adapters {
            println!("{:?}", adapter.info);
        }
        let adapter = adapters.remove(0);
        println!(
            "Using adapter '{}' ({}MB dedicated video memory)",
            adapter.info.name,
            adapter.info.video_memory / 1000 / 1000
        );
        Ok(adapter)
    }?;

    let device = dx12::Device::create(&adapter)?;
    let command_queue = dx12::CommandQueue::create(
        &device,
        dx12::CommandListType::Direct,
        dx12::CommandQueuePriority::Normal,
        dx12::CommandQueueFlags::NONE,
        0,
    )?;

    let buffers_count: u32 = 3;
    let is_tearing_supported = factory.is_tearing_supported;
    let swap_chain_desc = dx12::SwapChainDesc {
        width: config.width,
        height: config.height,
        format: dx12::Format::R8G8B8A8_UNORM,
        stereo: false,
        sample_desc: dx12::SampleDesc {
            count: 1,
            quality: 0,
        },
        buffer_usage: dx12::BufferUsage::RENDER_TARGET_OUTPUT,
        buffer_count: buffers_count,
        scaling: dx12::Scaling::Stretch,
        swap_effect: dx12::SwapEffect::FlipDiscard,
        alpha_mode: dx12::AlphaMode::Unspecified,
        flags: if is_tearing_supported {
            dx12::SwapChainFlags::ALLOW_TEARING
        } else {
            dx12::SwapChainFlags::NONE
        },
    };
    let swap_chain = dx12::SwapChain::create(&factory, &command_queue, &swap_chain_desc, hwnd)?;
    let mut current_back_buffer_index: usize = swap_chain.get_current_back_buffer_index() as _;
    let descriptor_heap = dx12::DescriptorHeap::create(
        &device,
        dx12::DescriptorHeapType::Rtv,
        dx12::DescriptorHeapFlags::NONE,
        buffers_count,
        0,
    )?;

    let mut back_buffers: Vec<dx12::Resource> = Vec::with_capacity(buffers_count as _);
    for i in 0..buffers_count {
        let back_buffer = swap_chain.get_buffer(i as u32)?;
        let rtv_descriptor = descriptor_heap.get_cpu_descriptor_offset(i);
        device.create_render_target_view(&back_buffer, rtv_descriptor);
        back_buffers.push(back_buffer);
    }

    let mut command_allocators: Vec<dx12::CommandAllocator> =
        Vec::with_capacity(buffers_count as _);
    for _ in 0..buffers_count {
        let command_allocator =
            dx12::CommandAllocator::create(&device, dx12::CommandListType::Direct)?;
        command_allocators.push(command_allocator);
    }
    let graphics_command_list = dx12::GraphicsCommandList::create(
        &device,
        &command_allocators[current_back_buffer_index],
        dx12::CommandListType::Direct,
    )?;

    let fence = dx12::Fence::create(&device)?;
    let fence_event = dx12::Event::new(false, false);
    let mut fence_value: u64 = 0;
    let mut frame_fence_values: [u64; 3] = [0, 0, 0]; // Triple buffering

    let mut is_resize_requested = false;
    let mut is_fullscreen = config.is_fullscreen;
    if is_fullscreen {
        framework::set_fullscreen(&window, config.is_fullscreen);
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
        if elapsed_time_secs > 1.0 {
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
            command_queue.flush(&fence, fence_event, &mut fence_value)?;

            // Any references to the back buffers must be released
            // before the swap chain can be resized.
            while let Some(back_buffer) = back_buffers.pop() {
                std::mem::drop(back_buffer);
            }

            // Reset per-frame fence values to the fence value of the current back buffer index
            for i in 0..buffers_count as _ {
                frame_fence_values[i] = frame_fence_values[current_back_buffer_index];
            }

            swap_chain.resize_buffers(buffers_count as _, width, height)?;

            current_back_buffer_index = swap_chain.get_current_back_buffer_index() as _;

            for i in 0..buffers_count {
                let back_buffer = swap_chain.get_buffer(i as u32)?;
                let rtv_descriptor = descriptor_heap.get_cpu_descriptor_offset(i);
                device.create_render_target_view(&back_buffer, rtv_descriptor);
                back_buffers.push(back_buffer);
            }

            is_resize_requested = false;
        }

        if config.is_fullscreen != is_fullscreen {
            config.is_fullscreen = is_fullscreen;
            framework::set_fullscreen(&window, config.is_fullscreen);
        }

        // Render
        {
            // Reset current command allocator and command list before new commands can be recorded
            let command_allocator = &command_allocators[current_back_buffer_index];
            command_allocator.reset()?;
            graphics_command_list.reset(&command_allocator)?;

            // Clear render target
            {
                let barriers = vec![dx12::BarrierDesc::new(
                    current_back_buffer_index,
                    dx12::ResourceStates::PRESENT..dx12::ResourceStates::RENDER_TARGET,
                )];
                graphics_command_list.insert_transition_barriers(&barriers, &back_buffers);

                let clear_color: [f32; 4] = [0.56, 0.93, 0.56, 1.0];
                let rtv_descriptor =
                    descriptor_heap.get_cpu_descriptor_offset(current_back_buffer_index as _);
                graphics_command_list.clear_render_target_view(rtv_descriptor, clear_color);
            }

            // Present the back buffer
            {
                let barriers = vec![dx12::BarrierDesc::new(
                    current_back_buffer_index,
                    dx12::ResourceStates::RENDER_TARGET..dx12::ResourceStates::PRESENT,
                )];
                graphics_command_list.insert_transition_barriers(&barriers, &back_buffers);

                graphics_command_list.close()?;

                let command_lists = vec![graphics_command_list.as_command_list()];
                command_queue.execute(&command_lists);

                let sync_interval = if config.is_vsync_enabled { 1 } else { 0 };
                let present_flags = if is_tearing_supported && !config.is_vsync_enabled {
                    dx12::PresentFlags::ALLOW_TEARING
                } else {
                    dx12::PresentFlags::NONE
                };
                swap_chain.present(sync_interval, present_flags)?;

                // Insert a signal into the command queue with a fence value
                frame_fence_values[current_back_buffer_index] =
                    command_queue.signal(&fence, &mut fence_value)?;

                current_back_buffer_index = swap_chain.get_current_back_buffer_index() as _;

                // Stall the CPU until fence value signalled is reached
                fence.wait_for_value(fence_event, frame_fence_values[current_back_buffer_index])?;
            }
        }
    }

    println!("Cleanup!");
    command_queue.flush(&fence, fence_event, &mut fence_value)?;
    fence_event.close();

    println!("Bye!");
    Ok(())
}
