use crate::backend::dx12::device::Device;

use graphix_native_dx12 as native;

pub struct DescriptorHeapHandle {
    pub(crate) cpu: native::heap::CPUDescriptor,
    pub(crate) gpu: native::heap::GPUDescriptor,
}

pub struct DescriptorHeap {
    pub(crate) native: native::heap::DescriptorHeap,
    pub(crate) handle_size: u64,
    pub(crate) handle_count: u64,
    pub(crate) start: DescriptorHeapHandle,
}

impl DescriptorHeap {
    pub(crate) fn new(
        device: &Device,
        heap_type: native::heap::DescriptorHeapType,
        capacity: usize,
    ) -> Self {
        let heap = native::heap::DescriptorHeap::new(
            &device.native,
            heap_type,
            native::heap::DescriptorHeapFlags::NONE,
            capacity as _,
            0,
        )
        .expect("Failed to create D3D12 descriptor heap");

        let handle_size = device.native.get_descriptor_increment_size(heap_type);
        let cpu_handle = heap.get_cpu_descriptor_start();
        let gpu_handle = heap.get_gpu_descriptor_start();

        DescriptorHeap {
            native: heap,
            handle_size: handle_size as _,
            handle_count: capacity as _,
            start: DescriptorHeapHandle {
                cpu: cpu_handle,
                gpu: gpu_handle,
            },
        }
    }

    pub(crate) fn offset(&self, index: u64) -> DescriptorHeapHandle {
        DescriptorHeapHandle {
            cpu: native::heap::CPUDescriptor {
                ptr: self.start.cpu.ptr + (self.handle_size * index) as usize,
            },
            gpu: native::heap::GPUDescriptor {
                ptr: self.start.gpu.ptr + (self.handle_size * index) as u64,
            },
        }
    }
}
