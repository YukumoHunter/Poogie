use ash::vk;
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc, Allocator},
    MemoryLocation,
};

use crate::PoogieRenderer;

use super::device::Device;

pub struct Buffer {
    pub raw: vk::Buffer,
    pub allocation: Allocation,
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

        Buffer { raw, allocation }
    }
}

impl PoogieRenderer {
    pub fn destroy_buffer(&mut self, buffer: Buffer) {
        self.allocator.free(buffer.allocation).unwrap();
        unsafe { self.device.raw.destroy_buffer(buffer.raw, None) }
    }
}
