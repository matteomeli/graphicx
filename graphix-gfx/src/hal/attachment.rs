use std::ops::Range;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum AttachmentMode {
    Present,
    RenderTarget,
}

#[derive(Clone)]
pub struct Attachment {
    pub states: Range<AttachmentMode>,
}
