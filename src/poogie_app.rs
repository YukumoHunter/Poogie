use crate::vulkan::instance;
use std::cell::RefCell;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

pub struct PoogieApp {
    pub event_loop: RefCell<winit::event_loop::EventLoop<()>>,
    pub window: winit::window::Window,

    pub instance: instance::Instance,
}

impl PoogieApp {
    pub fn new(window_width: u32, window_height: u32) -> PoogieApp {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("PoogieExample")
            .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height))
            .build(&event_loop)
            .unwrap();

        let instance = instance::Instance::builder()
            .required_extensions(vec!["VK_EXT_debug_utils"]) // TEMP
            .debug_graphics(true)
            .build()
            .unwrap();

        PoogieApp {
            event_loop: RefCell::new(event_loop),
            window,
            instance,
        }
    }

    pub fn render_loop(&self) {
        self.event_loop
            .borrow_mut()
            .run_return(|event, _, control_flow| {
                *control_flow = ControlFlow::Poll;
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
