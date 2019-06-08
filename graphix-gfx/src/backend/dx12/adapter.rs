use crate::backend::dx12::device::Device;
use crate::backend::dx12::instance::Backend;
use crate::hal;

use graphix_native_dx12 as native;

pub struct PhysicalAdapter {
    pub(crate) native: native::dxgi::Adapter,
}

impl hal::PhysicalAdapter<Backend> for PhysicalAdapter {
    fn create_device(&self) -> Device {
        Device::new(self)
    }
}
