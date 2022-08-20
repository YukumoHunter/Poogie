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
    pub images: Vec<Arc<vk::Image>>,
    pub image_views: Vec<vk::ImageView>,
    pub acquire_semaphores: Vec<vk::Semaphore>,
    pub finished_render_semaphores: Vec<vk::Semaphore>,
    pub next_semaphore: usize,
}

pub struct SwapchainImage {
    pub image: Arc<vk::Image>,
    pub index: u32,
    pub acquire_semaphore: vk::Semaphore,
    pub finished_render_semaphore: vk::Semaphore,
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
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let loader = khr::Swapchain::new(&device.instance.raw, &device.raw);
        let raw = unsafe { loader.create_swapchain(&swapchain_create_info, None)? };

        log::info!("Created swapchain!");

        let images = unsafe { loader.get_swapchain_images(raw)? }
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();
        let image_views = images
            .iter()
            .map(|image| unsafe {
                let image_view_info = vk::ImageViewCreateInfo::builder()
                    .image(**image)
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

        let acquire_semaphores = (0..images.len())
            .map(|_| unsafe {
                device
                    .raw
                    .create_semaphore(&vk::SemaphoreCreateInfo::builder(), None)
                    .unwrap()
            })
            .collect();

        let finished_render_semaphores = (0..images.len())
            .map(|_| unsafe {
                device
                    .raw
                    .create_semaphore(&vk::SemaphoreCreateInfo::builder(), None)
                    .unwrap()
            })
            .collect();

        Ok(Swapchain {
            raw,
            loader,
            desc,
            images,
            image_views,
            acquire_semaphores,
            finished_render_semaphores,
            next_semaphore: 0,
        })
    }

    pub fn acquire_next_image(&mut self) -> Result<SwapchainImage> {
        let acquire_semaphore = self.acquire_semaphores[self.next_semaphore];
        let finished_render_semaphore = self.finished_render_semaphores[self.next_semaphore];

        let present_result = unsafe {
            self.loader.acquire_next_image(
                self.raw,
                1000000000,
                // u64::MAX,
                acquire_semaphore,
                vk::Fence::null(),
            )
        };

        match present_result {
            Ok((present_index, _)) => {
                self.next_semaphore = (self.next_semaphore + 1) % self.images.len();
                Ok(SwapchainImage {
                    image: self.images[present_index as usize].clone(),
                    index: present_index,
                    acquire_semaphore,
                    finished_render_semaphore,
                })
            }
            Err(err)
                if err == vk::Result::ERROR_OUT_OF_DATE_KHR
                    || err == vk::Result::SUBOPTIMAL_KHR =>
            {
                // recreate framebuffer
                anyhow::bail!("Recreate framebuffer");
            }
            err => {
                panic!("Could not acquire swapchain image: {:?}", err);
            }
        }
    }
}
