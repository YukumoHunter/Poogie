pub mod backend_vulkan;

use anyhow::Result;
use ash::{extensions::khr, vk};
use backend_vulkan::{
    device::Device,
    instance::Instance,
    physical_device::PhysicalDevice,
    surface::Surface,
    swapchain::{Swapchain, SwapchainDesc},
};
use std::{ffi::CStr, sync::Arc};

pub struct PoogieApp {
    pub window: Arc<winit::window::Window>,
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub surface: Arc<Surface>,
    pub swapchain: Swapchain,
}

pub struct PoogieAppBuilder {
    debug_graphics: bool,
    vsync: bool,
}

impl PoogieAppBuilder {
    pub fn default() -> Self {
        PoogieAppBuilder {
            debug_graphics: false,
            vsync: true,
        }
    }

    pub fn debug_graphics(mut self, debug_graphics: bool) -> Self {
        self.debug_graphics = debug_graphics;
        self
    }

    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    pub fn build(self, window: Arc<winit::window::Window>) -> Result<PoogieApp> {
        PoogieApp::create(self, window)
    }
}

impl PoogieApp {
    pub fn builder() -> PoogieAppBuilder {
        PoogieAppBuilder::default()
    }

    pub fn create(builder: PoogieAppBuilder, window: Arc<winit::window::Window>) -> Result<Self> {
        let window_ext = ash_window::enumerate_required_extensions(&*window)
            .expect("Failed to get required instance extensions required for window")
            .iter()
            .map(|&ext| unsafe { CStr::from_ptr(ext).to_str().unwrap() })
            .collect();

        let instance = Instance::builder()
            .debug_graphics(builder.debug_graphics)
            .required_extensions(window_ext)
            .build()?;

        let pdevices = PhysicalDevice::enumerate_physical_devices(&instance)?;
        let pdevice = Arc::new(
            pdevices
                .into_iter()
                // `max_by_key` selects the last device in case there are multiple
                // we want to their order preserved
                .rev()
                .max_by_key(|device| match device.properties.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 100,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 10,
                    vk::PhysicalDeviceType::VIRTUAL_GPU => 1,
                    _ => 0,
                })
                .unwrap(),
        );

        let device = Device::new(&pdevice)?;

        let surface = Surface::new(&instance, &*window)?;

        let preferred_format = vk::SurfaceFormatKHR::builder()
            .format(vk::Format::B8G8R8A8_UNORM)
            .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .build();

        if !Swapchain::enumerate_surface_formats(&device, &surface)?.contains(&preferred_format) {
            panic!("Surface format is not supported!");
        }

        let window_size = window.inner_size();
        let swapchain_desc = SwapchainDesc {
            format: preferred_format,
            extent: vk::Extent2D::builder()
                .width(window_size.width)
                .height(window_size.height)
                .build(),
            vsync: builder.vsync,
        };
        let swapchain = Swapchain::new(&device, &surface, swapchain_desc)?;

        Ok(PoogieApp {
            window,
            instance,
            device,
            surface,
            swapchain,
        })
    }

    pub fn draw(&mut self, frame_number: i32) -> Result<()> {
        unsafe {
            self.device
                .raw
                .wait_for_fences(&[self.device.render_fence], true, u64::MAX)?;
        }

        let swapchain_image = match self.swapchain.acquire_next_image() {
            Some(img) => img,
            None => {
                anyhow::bail!("Bad swapchain image");
            }
        };

        unsafe {
            self.device.raw.reset_fences(&[self.device.render_fence])?;

            self.device.raw.reset_command_buffer(
                self.device.main_command_buffer.raw,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )?;
        }

        let cmd_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .raw
                .begin_command_buffer(self.device.main_command_buffer.raw, &cmd_info)?
        };

        // manually set image to a renderable layout
        let img_memory_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .image(*swapchain_image.image)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .base_mip_level(0)
                    .layer_count(1)
                    .base_array_layer(0)
                    .build(),
            );

        unsafe {
            self.device.raw.cmd_pipeline_barrier(
                self.device.main_command_buffer.raw,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[img_memory_barrier.build()],
            );
        }

        let color_attachment_info = vk::RenderingAttachmentInfo::builder()
            .image_view(self.swapchain.image_views[swapchain_image.index as usize])
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL_KHR)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [((frame_number as f32 % 255.) / 255.), 0.0, 0.0, 1.0],
                },
            })
            .build();

        let color_attachments = vec![color_attachment_info];

        let rendering_info = vk::RenderingInfo::builder()
            .render_area(vk::Rect2D {
                extent: vk::Extent2D::builder()
                    .width(self.swapchain.desc.extent.width)
                    .height(self.swapchain.desc.extent.height)
                    .build(),
                ..Default::default()
            })
            .layer_count(1)
            .color_attachments(&color_attachments);

        let dyn_rendering_loader = khr::DynamicRendering::new(&self.instance.raw, &self.device.raw);
        unsafe {
            dyn_rendering_loader
                .cmd_begin_rendering(self.device.main_command_buffer.raw, &rendering_info);

            // self.device
            //     .raw
            //     .cmd_draw(self.device.main_command_buffer.raw, 1, 1, 0, 0);

            dyn_rendering_loader.cmd_end_rendering(self.device.main_command_buffer.raw);
        }

        // manually set image to a presentable layout
        let img_memory_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .image(*swapchain_image.image)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .base_mip_level(0)
                    .layer_count(1)
                    .base_array_layer(0)
                    .build(),
            );

        unsafe {
            self.device.raw.cmd_pipeline_barrier(
                self.device.main_command_buffer.raw,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[img_memory_barrier.build()],
            );
        }

        unsafe {
            self.device
                .raw
                .end_command_buffer(self.device.main_command_buffer.raw)?
        };

        let submit_info = vk::SubmitInfo::builder()
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .wait_semaphores(&[swapchain_image.acquire_semaphore])
            .signal_semaphores(&[swapchain_image.finished_render_semaphore])
            .command_buffers(&[self.device.main_command_buffer.raw])
            .build();

        unsafe {
            self.device.raw.queue_submit(
                self.device.graphics_queue.raw,
                &[submit_info],
                self.device.render_fence,
            )?;
        }

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[swapchain_image.finished_render_semaphore])
            .swapchains(&[self.swapchain.raw])
            .image_indices(&[swapchain_image.index])
            .build();

        unsafe {
            match self
                .swapchain
                .loader
                .queue_present(self.device.graphics_queue.raw, &present_info)
            {
                Ok(_) => (),
                Err(e) => {
                    if e == vk::Result::ERROR_OUT_OF_DATE_KHR || e == vk::Result::SUBOPTIMAL_KHR {
                        anyhow::bail!("Bad swapchain image");
                    }
                }
            }
        };

        Ok(())
    }
}
