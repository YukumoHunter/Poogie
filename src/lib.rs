pub mod backend_vulkan;

use anyhow::Result;
use ash::vk;
use backend_vulkan::{
    device::Device,
    instance::Instance,
    physical_device::PhysicalDevice,
    pipeline::GraphicsPipeline,
    shader::{ShaderLanguage, ShaderStage},
    surface::Surface,
    swapchain::{Swapchain, SwapchainDesc},
};
use std::{ffi::CStr, path::PathBuf, sync::Arc};

use crate::backend_vulkan::shader::ShaderDesc;

pub struct PoogieRenderer {
    pub window: Arc<winit::window::Window>,
    #[allow(dead_code)]
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    #[allow(dead_code)]
    pub surface: Arc<Surface>,
    pub swapchain: Swapchain,
    pub frame_number: u64,
    shader_descs: Vec<ShaderDesc>,
    pipeline: GraphicsPipeline,
}

pub struct PoogieRendererBuilder {
    app_name: String,
    debug_graphics: bool,
    vsync: bool,
}

impl PoogieRendererBuilder {
    pub fn default() -> Self {
        PoogieRendererBuilder {
            app_name: "PoogieApp".to_string(),
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

    pub fn build(self, window: Arc<winit::window::Window>) -> Result<PoogieRenderer> {
        PoogieRenderer::create(self, window)
    }
}

impl PoogieRenderer {
    pub fn builder() -> PoogieRendererBuilder {
        PoogieRendererBuilder::default()
    }

    pub fn create(
        builder: PoogieRendererBuilder,
        window: Arc<winit::window::Window>,
    ) -> Result<Self> {
        let window_ext = ash_window::enumerate_required_extensions(&*window)
            .expect("Failed to get required instance extensions required for window")
            .iter()
            .map(|&ext| unsafe { CStr::from_ptr(ext).to_str().unwrap() })
            .collect();

        let instance = Instance::builder()
            .app_name(builder.app_name)
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
                .filter(|device| device.dyn_rendering_supported.dynamic_rendering != 0)
                .max_by_key(|device| match device.properties.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 100,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 10,
                    vk::PhysicalDeviceType::VIRTUAL_GPU => 1,
                    _ => 0,
                })
                .expect("No suitable GPU found"),
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
            surface_format: preferred_format,
            extent: vk::Extent2D::builder()
                .width(window_size.width)
                .height(window_size.height)
                .build(),
            vsync: builder.vsync,
        };
        let swapchain = Swapchain::new(&device, &surface, swapchain_desc)?;

        let vertex_shader_desc = ShaderDesc::builder().build(
            ShaderStage::Vertex,
            ShaderLanguage::GLSL,
            PathBuf::from("./src/shaders/shader.vert"),
        );

        let fragment_shader_desc = ShaderDesc::builder().build(
            ShaderStage::Fragment,
            ShaderLanguage::GLSL,
            PathBuf::from("./src/shaders/shader.frag"),
        );

        let shader_descs = vec![vertex_shader_desc, fragment_shader_desc];

        let pipeline = GraphicsPipeline::create_pipeline(&device, &swapchain, &shader_descs)?;

        Ok(PoogieRenderer {
            window,
            instance,
            device,
            surface,
            swapchain,
            frame_number: 0,
            shader_descs,
            pipeline,
        })
    }

    pub fn recreate_swapchain(&mut self) -> Result<()> {
        self.pipeline =
            GraphicsPipeline::create_pipeline(&self.device, &self.swapchain, &self.shader_descs)?;
        self.swapchain.recreate(&self.window)
    }

    pub fn draw(&mut self) -> Result<std::time::Duration> {
        let timer = std::time::Instant::now();

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

        let command_buffer = self.device.main_command_buffer.raw;

        unsafe {
            self.device.raw.reset_fences(&[self.device.render_fence])?;

            self.device.raw.reset_command_pool(
                self.device.main_command_buffer.pool,
                vk::CommandPoolResetFlags::RELEASE_RESOURCES,
            )?;
        }

        let cmd_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .raw
                .begin_command_buffer(command_buffer, &cmd_info)?
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
                command_buffer,
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
                    float32: [0.0, ((self.frame_number as f32 % 255.) / 255.), 0.0, 1.0],
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

        unsafe {
            self.device
                .raw
                .cmd_begin_rendering(command_buffer, &rendering_info);

            self.device.raw.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );

            self.device.raw.cmd_draw(command_buffer, 3, 1, 0, 0);

            self.device.raw.cmd_end_rendering(command_buffer);
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
                command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[img_memory_barrier.build()],
            );
        }

        unsafe { self.device.raw.end_command_buffer(command_buffer)? };

        let submit_info = vk::SubmitInfo::builder()
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .wait_semaphores(&[swapchain_image.acquire_semaphore])
            .signal_semaphores(&[swapchain_image.finished_render_semaphore])
            .command_buffers(&[command_buffer])
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

        self.frame_number += 1;

        Ok(timer.elapsed())
    }

    pub fn frame_number(&self) -> u64 {
        self.frame_number
    }
}
