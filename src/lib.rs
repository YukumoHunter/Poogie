pub mod backend_vulkan;

use anyhow::Result;
use ash::vk;
use backend_vulkan::{device::Device, instance::Instance, physical_device::PhysicalDevice};
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
}

impl PoogieApp {
    pub fn new(window_width: u32, window_height: u32) -> Result<PoogieApp> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("PoogieExample")
            .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height))
            .build(&event_loop)?;

        let instance = Instance::builder().debug_graphics(true).build()?;

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

        let device = Device::create(&pdevice)?;

        Ok(PoogieApp {
            event_loop: RefCell::new(event_loop),
            window,
            instance,
            device,
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
