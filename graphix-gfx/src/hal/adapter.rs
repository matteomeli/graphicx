use crate::hal::Backend;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DeviceType {
    DiscreteGpu,
    VirtualGpu,
}

#[derive(Clone, Debug)]
pub struct AdapterInfo {
    pub name: String,
    pub vendor: u32,
    pub device: u32,
    pub video_memory: usize,
    pub device_type: DeviceType,
}

pub struct Adapter<B: Backend> {
    pub adapter: B::PhysicalAdapter,
    pub info: AdapterInfo,
}

impl<B: Backend> Adapter<B> {
    pub fn create_device(&self) -> B::Device {
        self.adapter.create_device()
    }
}

pub trait PhysicalAdapter<B: Backend> {
    fn create_device(&self) -> B::Device;
}
