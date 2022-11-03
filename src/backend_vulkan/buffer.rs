use ash::vk;
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc, Allocator},
    MemoryLocation,
};

use super::device::Device;

#[derive(Debug)]
pub struct Buffer {
    pub raw: vk::Buffer,
    pub allocation: Option<Allocation>,
}

impl Buffer {
    pub fn new(
        allocator: &mut Allocator,
        device: &Device,
        size: usize,
        usage: vk::BufferUsageFlags,
        name: impl Into<String>,
    ) -> Self {
        let name = name.into();

        // Setup vulkan info
        let vk_info = vk::BufferCreateInfo::builder()
            .size(size as u64)
            .usage(usage);

        let raw = unsafe { device.raw.create_buffer(&vk_info, None) }.unwrap();
        let requirements = unsafe { device.raw.get_buffer_memory_requirements(raw) };

        let allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: &name,
                requirements,
                location: MemoryLocation::CpuToGpu,
                linear: true,
            })
            .unwrap();

        // Bind memory to the buffer
        unsafe {
            device
                .raw
                .bind_buffer_memory(raw, allocation.memory(), allocation.offset())
                .unwrap()
        };

        Buffer {
            raw,
            allocation: Some(allocation),
        }
    }

    pub fn destroy(&mut self, device: &Device, allocator: &mut Allocator) {
        allocator.free(self.allocation.take().unwrap()).unwrap();
        unsafe { device.raw.destroy_buffer(self.raw, None) }
    }
}
