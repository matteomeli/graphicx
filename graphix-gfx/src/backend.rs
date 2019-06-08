#[cfg(feature = "dx12")]
extern crate graphix_native_dx12;

#[cfg(feature = "dx12")]
pub use crate::backend::dx12::{adapter, command, device, heap, instance, queue, resource, window};

#[cfg(feature = "dx12")]
mod dx12 {
    pub mod adapter;
    pub mod command;
    pub mod device;
    pub mod heap;
    pub mod instance;
    pub mod queue;
    pub mod resource;
    pub mod window;
}
