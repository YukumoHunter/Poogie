use poogie::PoogieRenderer;
use std::{borrow::BorrowMut, sync::Arc};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

fn main() {
    env_logger::init();

    let mut event_loop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Awesome Poogie App Winning")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap(),
    );

    let mut poogie = PoogieRenderer::builder()
        .debug_graphics(true)
        .vsync(false)
        .build(window.clone())
        .unwrap();

    event_loop
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
                } => {
                    log::info!("Exiting!");
                    poogie.terminate();
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    if let Err(e) = poogie.recreate_swapchain() {
                        log::warn!("Failed to create swapchain: {e:?}: {e}");
                    }
                }
                Event::MainEventsCleared => {
                    if let Ok(elapsed) = poogie.draw() {
                        window.set_title(&format!(
                            "Frame time: {:.2}ms, FPS: {}",
                            elapsed.as_secs_f64() * 1000.0,
                            (1.0 / elapsed.as_secs_f32()) as u32
                        ));
                    };
                }
                _ => (),
            }
        });
}
