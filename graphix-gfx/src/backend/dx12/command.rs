use crate::backend::dx12::device::Device;
use crate::backend::dx12::instance::Backend;
use crate::backend::dx12::queue;
use crate::backend::dx12::resource::FrameBuffer;
use crate::hal;

use graphix_native_dx12 as native;

pub enum CommandPoolAllocator {
    Shared(native::command_allocator::CommandAllocator), // 1 command allocator to many command lists
    Multiple(Vec<native::command_allocator::CommandAllocator>), // 1 command allocator to 1 command lists
}

pub struct CommandPool {
    device: native::device::Device,
    pool_allocator: CommandPoolAllocator,
    single_command_list: Option<native::command_list::GraphicsCommandList>,
    pool_type: native::command_list::CommandListType,
    flags: hal::CommandPoolFlags,
}

impl CommandPool {
    pub(crate) fn new(
        device: &Device,
        pool_type: hal::QueueType,
        flags: hal::CommandPoolFlags,
    ) -> Self {
        let pool_type = queue::get_native_type(pool_type);

        let pool_allocator = if flags.contains(hal::CommandPoolFlags::MULTIPLE_ALLOCATOR) {
            CommandPoolAllocator::Multiple(Vec::new())
        } else {
            let command_allocator =
                native::command_allocator::CommandAllocator::new(&device.native, pool_type)
                    .expect("Failed to create a D3D12 command allocator");

            CommandPoolAllocator::Shared(command_allocator)
        };

        CommandPool {
            device: device.native.clone(),
            pool_allocator,
            single_command_list: None,
            pool_type,
            flags,
        }
    }

    fn create_command_list(
        device: &native::device::Device,
        allocator: &native::command_allocator::CommandAllocator,
        list_type: native::command_list::CommandListType,
    ) -> native::command_list::GraphicsCommandList {
        let command_list =
            native::command_list::GraphicsCommandList::new(&device, &allocator, list_type)
                .expect("Failed to create a D3D12 graphics command list");

        // Close command list as they're initialised as recording, but only one can be recording for each allocator
        command_list
            .close()
            .expect("Failed to close D3D12 graphics command list");

        command_list
    }
}

impl hal::CommandPool<Backend> for CommandPool {
    fn reset(&self) {
        match self.pool_allocator {
            CommandPoolAllocator::Shared(ref allocator) => {
                allocator
                    .reset()
                    .expect("Failed to reset D3D12 command allocator");
            }
            CommandPoolAllocator::Multiple(ref allocators) => {
                for allocator in allocators.iter() {
                    allocator
                        .reset()
                        .expect("Failed to reset D3D12 command allocator");
                }
            }
        }
    }

    fn create_buffer(&mut self) -> CommandBuffer {
        let (command_allocator, command_list) = match self.pool_allocator {
            CommandPoolAllocator::Shared(ref allocator) => {
                let command_allocator = allocator.clone();
                let command_list = CommandPool::create_command_list(
                    &self.device,
                    &command_allocator,
                    self.pool_type,
                );

                (command_allocator, command_list)
            }
            CommandPoolAllocator::Multiple(ref mut allocators) => {
                let command_allocator =
                    native::command_allocator::CommandAllocator::new(&self.device, self.pool_type)
                        .expect("Failed to create D3D12 command allocator");

                let command_list = if self.flags.contains(hal::CommandPoolFlags::SINGLE_LIST) {
                    match self.single_command_list {
                        Some(ref command_list) => command_list.clone(),
                        None => {
                            let single_command_list = CommandPool::create_command_list(
                                &self.device,
                                &command_allocator,
                                self.pool_type,
                            );
                            self.single_command_list = Some(single_command_list);
                            (*self.single_command_list.as_ref().unwrap()).clone()
                        }
                    }
                } else {
                    CommandPool::create_command_list(
                        &self.device,
                        &command_allocator,
                        self.pool_type,
                    )
                };

                allocators.push(command_allocator.clone());
                (command_allocator, command_list)
            }
        };

        CommandBuffer::new(command_allocator, command_list, self.flags)
    }
}

pub struct CommandBuffer {
    pub(crate) command_allocator: native::command_allocator::CommandAllocator,
    pub(crate) graphics_command_list: native::command_list::GraphicsCommandList,
    flags: hal::CommandPoolFlags,
}

impl CommandBuffer {
    pub(crate) fn new(
        command_allocator: native::command_allocator::CommandAllocator,
        graphics_command_list: native::command_list::GraphicsCommandList,
        flags: hal::CommandPoolFlags,
    ) -> Self {
        CommandBuffer {
            command_allocator,
            graphics_command_list,
            flags,
        }
    }

    pub(crate) fn as_command_list(&self) -> native::command_list::CommandList {
        self.graphics_command_list.as_command_list()
    }

    fn reset(&self) {
        self.graphics_command_list
            .reset(&self.command_allocator)
            .expect("Failed to reset command list");
    }
}

impl hal::CommandBuffer<Backend> for CommandBuffer {
    fn begin(&self) {
        if self
            .flags
            .contains(hal::CommandPoolFlags::MULTIPLE_ALLOCATOR)
        {
            self.command_allocator
                .reset()
                .expect("Failed to reset command allocator");
        }

        self.reset();
    }

    fn end(&self) {
        self.graphics_command_list
            .close()
            .expect("Failed to close D3D12 command list");
    }

    fn insert_barriers(
        &self,
        barrier_point: hal::BarrierPoint,
        attachments: &[hal::Attachment],
        framebuffer: &FrameBuffer,
    ) {
        let barriers = attachments
            .iter()
            .enumerate()
            .map(|(attachment_id, attachment)| {
                let initial_state = match barrier_point {
                    hal::BarrierPoint::Pre => get_native_resource_state(attachment.states.start),
                    hal::BarrierPoint::Post => get_native_resource_state(attachment.states.end),
                };
                let target_state = match barrier_point {
                    hal::BarrierPoint::Pre => get_native_resource_state(attachment.states.end),
                    hal::BarrierPoint::Post => get_native_resource_state(attachment.states.start),
                };
                native::barrier::BarrierDesc::new(attachment_id, initial_state..target_state)
            })
            .collect::<Vec<_>>();

        let resources = framebuffer
            .attachments
            .iter()
            .map(|view| view.resource.clone())
            .collect::<Vec<_>>();

        self.graphics_command_list
            .insert_transition_barriers(&barriers, &resources);
    }

    fn clear(&self, clear_colors: &[hal::ClearColor], framebuffer: &FrameBuffer) {
        for (view, clear_color) in framebuffer.attachments.iter().zip(clear_colors.iter()) {
            self.graphics_command_list
                .clear_render_target_view(view.rtv_handle.unwrap(), *clear_color);
        }
    }
}

fn get_native_resource_state(
    attachment_mode: hal::AttachmentMode,
) -> native::resource::ResourceState {
    match attachment_mode {
        hal::AttachmentMode::Present => native::resource::ResourceState::PRESENT,
        hal::AttachmentMode::RenderTarget => native::resource::ResourceState::RENDER_TARGET,
    }
}
