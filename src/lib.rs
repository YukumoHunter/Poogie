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
use std::ffi::CStr;
use std::{cell::RefCell, sync::Arc};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

pub struct PoogieApp {
    pub event_loop: RefCell<winit::event_loop::EventLoop<()>>,
    pub window: winit::window::Window,
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub surface: Arc<Surface>,
    pub swapchain: Swapchain,
}

pub struct PoogieAppBuilder {
    title: String,
    resolution: [u32; 2],
    debug_graphics: bool,
    vsync: bool,
}

impl PoogieAppBuilder {
    pub fn default() -> Self {
        PoogieAppBuilder {
            title: "PoogieApp".to_string(),
            resolution: [1280, 720],
            debug_graphics: false,
            vsync: true,
        }
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn resolution(mut self, resolution: [u32; 2]) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn debug_graphics(mut self, debug_graphics: bool) -> Self {
        self.debug_graphics = debug_graphics;
        self
    }

    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    pub fn build(self) -> Result<PoogieApp> {
        PoogieApp::create(self)
    }
}

impl PoogieApp {
    pub fn builder() -> PoogieAppBuilder {
        PoogieAppBuilder::default()
    }

    pub fn create(builder: PoogieAppBuilder) -> Result<Self> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(builder.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                builder.resolution[0],
                builder.resolution[1],
            ))
            .build(&event_loop)?;

        let window_ext = ash_window::enumerate_required_extensions(&window)
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

        let surface = Surface::new(&instance, &window)?;

        let preferred_format = vk::SurfaceFormatKHR::builder()
            .format(vk::Format::B8G8R8A8_UNORM)
            .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .build();

        if !Swapchain::enumerate_surface_formats(&device, &surface)?.contains(&preferred_format) {
            panic!("Surface format is not supported!");
        }

        let swapchain_desc = SwapchainDesc {
            format: preferred_format,
            extent: vk::Extent2D::builder()
                .width(builder.resolution[0])
                .height(builder.resolution[1])
                .build(),
            vsync: builder.vsync,
        };
        let swapchain = Swapchain::new(&device, &surface, swapchain_desc)?;

        Ok(PoogieApp {
            event_loop: RefCell::new(event_loop),
            window,
            instance,
            device,
            surface,
            swapchain,
        })
    }

    pub fn draw(&self) -> Result<()> {
        dbg!("hi");

        unsafe {
            self.device
                .raw
                .wait_for_fences(&[self.device.render_fence], true, u64::MAX)?;
            self.device.raw.reset_fences(&[self.device.render_fence])?;
        }

        let (index, _) = unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.raw,
                1000000000,
                // u64::MAX,
                self.device.present_semaphore,
                vk::Fence::null(),
            )?
        };

        unsafe {
            self.device.raw.reset_command_buffer(
                self.device.main_command_buffer.raw,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )?;
        }

        let color_attachment_info = vk::RenderingAttachmentInfo::builder()
            .image_view(self.swapchain.image_views[index as usize]) // TODO: get correct index here
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [1.0, 0.0, 0.0, 1.0],
                },
            })
            .build();

        // log::info!("bruh");

        let color_attachments = vec![color_attachment_info];

        let rendering_info = vk::RenderingInfo::builder()
            .render_area(vk::Rect2D {
                extent: vk::Extent2D {
                    width: self.window.inner_size().width,
                    height: self.window.inner_size().height,
                },
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

        let submit_info = vk::SubmitInfo::builder()
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .wait_semaphores(&[self.device.present_semaphore])
            .signal_semaphores(&[self.device.render_semaphore])
            .command_buffers(&[self.device.main_command_buffer.raw])
            .build();

        unsafe {
            self.device.raw.queue_submit(
                self.device.graphics_queue.raw,
                &[submit_info],
                self.device.render_fence,
            )?;
        }

        // let present_info = vk::PresentInfoKHR::builder()
        // .

        Ok(())
    }

    pub fn render_loop(&self) {
        self.event_loop
            .borrow_mut()
            .run_return(|event, _, control_flow| {
                *control_flow = ControlFlow::Poll;
                #[allow(clippy::single_match)]
                match event {
                    Event::WindowEvent {
                        event:
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    Event::MainEventsCleared => {
                        self.draw().unwrap();
                    }
                    _ => (),
                }
            });
    }
}
