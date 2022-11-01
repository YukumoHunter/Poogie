use super::{
    instance::Instance,
    physical_device::{PhysicalDevice, QueueFamily},
};
use anyhow::Result;
use ash::{extensions::khr, vk};
use std::{collections::HashSet, ffi::CStr, os::raw::c_char, sync::Arc};

pub struct Queue {
    pub raw: vk::Queue,
    pub family: QueueFamily,
}

pub struct CommandBuffer {
    pub raw: vk::CommandBuffer,
    pub pool: vk::CommandPool,
    pub submit_done_fence: vk::Fence,
}

impl CommandBuffer {
    pub fn new(device: &ash::Device, queue_family: &QueueFamily, amount: u32) -> Result<Self> {
        let pool_create_info =
            vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family.index);

        let pool = unsafe { device.create_command_pool(&pool_create_info, None)? };

        let cmd_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(amount)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let cmd_buffer = unsafe { device.allocate_command_buffers(&cmd_buffer_allocate_info)?[0] };

        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        let submit_done_fence = unsafe { device.create_fence(&fence_create_info, None)? };

        Ok(CommandBuffer {
            raw: cmd_buffer,
            pool,
            submit_done_fence,
        })
    }

    pub fn immediate_submit() {}
}

pub struct Device {
    pub raw: ash::Device,
    pub(crate) pdevice: Arc<PhysicalDevice>,
    pub(crate) instance: Arc<Instance>,

    pub graphics_queue: Queue,
    pub transfer_queue: Option<Queue>, // TODO: currently unused
    // TODO: create an optional queue for compute as well
    pub main_command_buffer: CommandBuffer,

    pub render_fence: vk::Fence,
}

impl Device {
    pub fn new(pdevice: &Arc<PhysicalDevice>) -> Result<Arc<Self>> {
        let supported_extensions: HashSet<String> = unsafe {
            pdevice
                .instance
                .raw
                .enumerate_device_extension_properties(pdevice.raw)?
                .iter()
                .map(|ext| {
                    CStr::from_ptr(ext.extension_name.as_ptr() as *const c_char)
                        .to_string_lossy()
                        .as_ref()
                        .to_owned()
                })
                .collect()
        };

        let device_ext_names = vec![khr::Swapchain::name().as_ptr()];

        unsafe {
            for &ext in &device_ext_names {
                let ext = std::ffi::CStr::from_ptr(ext).to_string_lossy();
                if !supported_extensions.contains(ext.as_ref()) {
                    panic!("Unsupported device extension: {ext}");
                }
            }
        }

        let graphics_queue_family = pdevice
            .queue_families
            .iter()
            .filter(|family| {
                family
                    .properties
                    .queue_flags
                    .contains(vk::QueueFlags::GRAPHICS)
            })
            .copied()
            .next()
            .expect("No suitable graphics queue family found");

        let transfer_queue_family = pdevice
            .queue_families
            .iter()
            .filter(|family| {
                family
                    .properties
                    .queue_flags
                    .contains(vk::QueueFlags::TRANSFER)
                    && !family.index == graphics_queue_family.index
            })
            .copied()
            .next();

        let priorities = [1.0f32];

        let mut queue_create_infos = vec![vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family.index)
            .queue_priorities(&priorities)
            .build()];

        if let Some(transfer_queue) = transfer_queue_family {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(transfer_queue.index)
                    .queue_priorities(&priorities)
                    .build(),
            );
        };

        let mut features13 = vk::PhysicalDeviceVulkan13Features::builder().dynamic_rendering(true);
        let mut features = vk::PhysicalDeviceFeatures2::builder().push_next(&mut features13);

        let device_create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&device_ext_names)
            .queue_create_infos(&queue_create_infos)
            .push_next(&mut features);

        let device = unsafe {
            pdevice
                .instance
                .raw
                .create_device(pdevice.raw, &device_create_info, None)?
        };

        log::debug!("Created Vulkan logical device!");

        let graphics_queue = Queue {
            raw: unsafe { device.get_device_queue(graphics_queue_family.index, 0) },
            family: graphics_queue_family,
        };

        let transfer_queue = transfer_queue_family.map(|family| Queue {
            raw: unsafe { device.get_device_queue(family.index, 0) },
            family,
        });

        let main_command_buffer = CommandBuffer::new(&device, &graphics_queue_family, 1)?;

        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        let render_fence = unsafe { device.create_fence(&fence_create_info, None)? };

        Ok(Arc::new(Device {
            raw: device,
            pdevice: pdevice.clone(),
            instance: pdevice.instance.clone(),
            graphics_queue,
            transfer_queue,
            main_command_buffer,
            render_fence,
        }))
    }
}
