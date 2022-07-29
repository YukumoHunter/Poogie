pub mod backend_vulkan;

use anyhow::Result;
use ash::vk;
use backend_vulkan::{
    device::Device,
    instance::Instance,
    physical_device::PhysicalDevice,
    surface::Surface,
    swapchain::{Swapchain, SwapchainDesc},
};
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

        let instance = Instance::builder()
            .debug_graphics(builder.debug_graphics)
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
                    _ => (),
                }
            });
    }
}
