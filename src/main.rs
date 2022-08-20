use poogie::PoogieApp;
use std::{borrow::BorrowMut, sync::Arc};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

fn main() {
    let mut event_loop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Awesome Poogie App Winning")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap(),
    );

    env_logger::init();
    let mut poogie = PoogieApp::builder()
        .debug_graphics(true)
        .build(window)
        .unwrap();

    let mut frame = 0;

    event_loop
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
                    poogie.draw(frame).unwrap();
                    frame += 1;
                }
                _ => (),
            }
        });
}
