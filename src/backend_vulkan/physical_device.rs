use super::instance::Instance;
use anyhow::Result;
use ash::vk;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct QueueFamily {
    pub index: u32,
    pub properties: vk::QueueFamilyProperties,
}

pub struct PhysicalDevice {
    pub raw: vk::PhysicalDevice,
    pub instance: Arc<Instance>,
    pub properties: vk::PhysicalDeviceProperties,
    pub(crate) queue_families: Vec<QueueFamily>,
    // pub(crate) presentation_requested: bool,
    // pub memory_properties: PhysicalDeviceMemoryProperties,
}

impl PhysicalDevice {
    pub fn enumerate_physical_devices(instance: &Arc<Instance>) -> Result<Vec<PhysicalDevice>> {
        let pdevices = unsafe { instance.raw.enumerate_physical_devices()? };

        Ok(pdevices
            .into_iter()
            .map(|pdevice| {
                let properties = unsafe { instance.raw.get_physical_device_properties(pdevice) };

                let queue_families = unsafe {
                    instance
                        .raw
                        .get_physical_device_queue_family_properties(pdevice)
                        .into_iter()
                        .enumerate()
                        .map(|(index, properties)| QueueFamily {
                            index: index as _,
                            properties,
                        })
                        .collect()
                };

                PhysicalDevice {
                    raw: pdevice,
                    instance: instance.clone(),
                    properties,
                    queue_families,
                }
            })
            .collect())
    }
}
