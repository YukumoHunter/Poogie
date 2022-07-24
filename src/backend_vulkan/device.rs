use super::{instance::Instance, physical_device::PhysicalDevice};
use anyhow::Result;
use ash;
use ash::{extensions::khr, vk};
use std::{collections::HashSet, ffi::CStr, os::raw::c_char, sync::Arc};

pub struct Device {
    pub raw: ash::Device,
    #[allow(dead_code)]
    pub(crate) pdevice: Arc<PhysicalDevice>,
    #[allow(dead_code)]
    pub(crate) instance: Arc<Instance>,
    pub has_transfer_queue: bool,
}

impl Device {
    pub fn create(pdevice: &Arc<PhysicalDevice>) -> Result<Arc<Self>> {
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

        let device_ext_names = vec![
            khr::Swapchain::name().as_ptr(),
            khr::DynamicRendering::name().as_ptr(),
        ];

        unsafe {
            for &ext in &device_ext_names {
                let ext = std::ffi::CStr::from_ptr(ext).to_string_lossy();
                if !supported_extensions.contains(ext.as_ref()) {
                    panic!("Unsupported device extension: {ext}");
                }
            }
        }

        let graphics_queue = pdevice
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

        let transfer_queue = pdevice
            .queue_families
            .iter()
            .filter(|family| {
                family
                    .properties
                    .queue_flags
                    .contains(vk::QueueFlags::TRANSFER)
                    && !family
                        .properties
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS)
            })
            .copied()
            .next();

        let priorities = [1.0f32];

        let mut queue_create_infos = vec![vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue.index)
            .queue_priorities(&priorities)
            .build()];

        let mut has_transfer_queue = false;
        if let Some(transfer_queue) = transfer_queue {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(transfer_queue.index)
                    .queue_priorities(&priorities)
                    .build(),
            );

            has_transfer_queue = true;
        };

        let device_create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&device_ext_names)
            .queue_create_infos(&queue_create_infos);

        let device = unsafe {
            pdevice
                .instance
                .raw
                .create_device(pdevice.raw, &device_create_info, None)?
        };

        log::info!("Created Vulkan logical device");

        Ok(Arc::new(Device {
            raw: device,
            pdevice: pdevice.clone(),
            instance: pdevice.instance.clone(),
            has_transfer_queue,
        }))
    }
}
