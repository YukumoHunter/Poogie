use super::{device::Device, surface::Surface};
use anyhow::Result;
use ash::{extensions::khr, vk};
use std::sync::Arc;

#[derive(Clone, Copy, Default)]
pub struct SwapchainDesc {
    pub format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub vsync: bool,
}

pub struct Swapchain {
    pub raw: vk::SwapchainKHR,
    pub loader: khr::Swapchain,
    pub desc: SwapchainDesc,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn enumerate_surface_formats(
        device: &Device,
        surface: &Surface,
    ) -> Result<Vec<vk::SurfaceFormatKHR>> {
        unsafe {
            Ok(surface
                .loader
                .get_physical_device_surface_formats(device.pdevice.raw, surface.raw)?)
        }
    }

    pub fn new(device: &Arc<Device>, surface: &Arc<Surface>, desc: SwapchainDesc) -> Result<Self> {
        let surface_capabilities = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(device.pdevice.raw, surface.raw)?
        };

        // try to triple-buffer
        let mut image_count = 3.max(surface_capabilities.min_image_count);
        if surface_capabilities.max_image_count != 0 {
            image_count = image_count.min(surface_capabilities.max_image_count);
        }

        log::info!("Creating swapchain with {image_count} images!");

        let surface_resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => desc.extent,
            _ => surface_capabilities.current_extent,
        };

        if surface_resolution.width == 0 || surface_resolution.height == 0 {
            anyhow::bail!("Surface resolution cannot be zero!");
        }

        let present_mode_preference = if desc.vsync {
            vec![vk::PresentModeKHR::FIFO_RELAXED, vk::PresentModeKHR::FIFO]
        } else {
            vec![vk::PresentModeKHR::MAILBOX, vk::PresentModeKHR::IMMEDIATE]
        };

        let available_present_modes = unsafe {
            surface
                .loader
                .get_physical_device_surface_present_modes(device.pdevice.raw, surface.raw)?
        };

        let present_mode = present_mode_preference
            .into_iter()
            .find(|mode| available_present_modes.contains(mode))
            // FIFO is the only presentation mode to be guaranteed to be available
            .unwrap_or(vk::PresentModeKHR::FIFO);

        log::info!("Creating swapchain using presentation mode {present_mode:?}!");

        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.raw)
            .min_image_count(image_count)
            .image_format(desc.format.format)
            .image_extent(desc.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::STORAGE)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let loader = khr::Swapchain::new(&device.instance.raw, &device.raw);
        let raw = unsafe { loader.create_swapchain(&swapchain_create_info, None)? };

        log::info!("Created swapchain!");

        let images = unsafe { loader.get_swapchain_images(raw)? };
        let image_views = images
            .iter()
            .map(|image| unsafe {
                let image_view_info = vk::ImageViewCreateInfo::builder()
                    .image(*image)
                    .format(desc.format.format)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .components(
                        vk::ComponentMapping::builder()
                            .a(vk::ComponentSwizzle::A)
                            .r(vk::ComponentSwizzle::R)
                            .g(vk::ComponentSwizzle::G)
                            .b(vk::ComponentSwizzle::B)
                            .build(),
                    )
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build(),
                    );

                device
                    .raw
                    .create_image_view(&image_view_info, None)
                    .expect("Failed to create image view!")
            })
            .collect::<Vec<vk::ImageView>>();

        Ok(Swapchain {
            raw,
            loader,
            desc,
            images,
            image_views,
        })
    }
}
