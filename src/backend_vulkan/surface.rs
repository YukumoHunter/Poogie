use super::instance::Instance;
use anyhow::Result;
use ash::{extensions::khr, vk};
use ash_window::create_surface;
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

pub struct Surface {
    pub(crate) raw: vk::SurfaceKHR,
    pub(crate) loader: khr::Surface,
}

impl Surface {
    pub fn new(instance: &Instance, window: &impl HasRawWindowHandle) -> Result<Arc<Self>> {
        Ok(Arc::new(Surface {
            raw: unsafe { create_surface(&instance.entry, &instance.raw, window, None)? },
            loader: khr::Surface::new(&instance.entry, &instance.raw),
        }))
    }
}
