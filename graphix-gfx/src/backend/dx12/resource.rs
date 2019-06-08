use graphix_native_dx12 as native;

#[derive(Clone)]
pub struct BufferView {
    pub(crate) resource: native::resource::Resource,
    pub(crate) rtv_handle: Option<native::heap::CPUDescriptor>,
}

#[derive(Clone)]
pub struct FrameBuffer {
    pub attachments: Vec<BufferView>,
}
