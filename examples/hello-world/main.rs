use graphix_gfx as gfx;

use gfx::hal::{
    Attachment, AttachmentMode, BarrierPoint, CommandBuffer, CommandPool, CommandPoolFlags,
    CommandQueue, Device, Format, Instance, QueueType, Surface, Swapchain, SwapchainConfig,
};

use std::env;

fn main() {
    // Parse command line args into a config
    let args: Vec<String> = env::args().collect();
    let config = graphix::Config::new(&args);
    println!("{:?}", config);

    // Create main window and event loop
    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(
            f64::from(config.width),
            f64::from(config.height),
        ))
        .with_title("Rust + DirectX 12")
        .build(&events_loop)
        .unwrap();

    // The number of frames in flight
    let frame_count = 3;

    // Create graphics backend instance
    let instance = gfx::Instance::new();

    // Find a physical adapter
    let mut adapters = instance.enumerate_adapters();
    for adapter in &adapters {
        println!("{:?}", adapter.info);
    }
    let adapter = adapters.remove(0);
    println!(
        "Using adapter '{}' ({}MB dedicated video memory)",
        adapter.info.name,
        adapter.info.video_memory / 1000 / 1000
    );

    // Create a device
    let device = adapter.create_device();

    // Create main present queue
    let command_queue = device.create_command_queue(QueueType::Graphics);

    // Create a swapchain
    let surface = instance.create_surface(&window);
    let swapchain = surface.create_swapchain(
        &device,
        &command_queue,
        SwapchainConfig {
            format: Format::Rgba8Unorm,
            buffer_count: frame_count,
            width: config.width,
            height: config.height,
            sync_interval: if config.is_vsync_enabled { 1 } else { 0 },
        },
    );
    let backbuffer = swapchain.create_backbuffer();
    let mut frame_index: usize = swapchain.acquire_buffer() as _;

    let mut command_pool = device.create_command_pool(
        QueueType::Graphics,
        CommandPoolFlags::MULTIPLE_ALLOCATOR | CommandPoolFlags::SINGLE_LIST,
    );

    let mut command_buffers = Vec::with_capacity(frame_count);
    for _ in 0..frame_count {
        let command_buffer = command_pool.create_buffer();
        command_buffers.push(command_buffer);
    }

    let attachments = vec![Attachment {
        states: AttachmentMode::Present..AttachmentMode::RenderTarget,
    }];

    let clear_colors = vec![[0.56, 0.93, 0.56, 1.0]];

    let mut fence_values: [u64; 3] = [0, 0, 0];
    let fence = device.create_fence(0);

    let mut is_running = true;
    while is_running {
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
                        println!("Received request to toggle vertical sync");
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
                    }
                    _ => (),
                }
            }
        });

        // Start recording commands for the current frame
        let command_buffer = &command_buffers[frame_index];
        command_buffer.begin();

        // Record commands for command buffer
        let framebuffer = &backbuffer.framebuffers[frame_index];
        command_buffer.insert_barriers(BarrierPoint::Pre, &attachments, framebuffer);
        command_buffer.clear(&clear_colors, framebuffer);
        command_buffer.insert_barriers(BarrierPoint::Post, &attachments, framebuffer);

        // Stop recording commands
        command_buffer.end();

        // Submit commands to the command queue
        command_queue.submit(vec![command_buffer]);

        // Present
        swapchain.present();

        // Signal command queue with fence value for current frame
        let frame_fence_value = fence_values[frame_index];
        command_queue.signal_fence(&fence, frame_fence_value);

        // Wait for fence value for next frame
        frame_index = swapchain.acquire_buffer() as _;
        device.wait_for_fence(&fence, fence_values[frame_index]);
        fence_values[frame_index] = frame_fence_value + 1;
    }

    println!("Cleanup!");
    command_queue.signal_fence(&fence, fence_values[frame_index]);
    device.wait_for_fence(&fence, fence_values[frame_index]);

    println!("Bye!");
}
