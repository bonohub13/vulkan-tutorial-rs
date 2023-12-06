use anyhow::Result;
use lve_rs;
use std::borrow::BorrowMut;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

struct App {
    window: lve_rs::Window,
}

impl App {
    pub const WIDTH: i32 = 800;
    pub const HEIGHT: i32 = 600;

    pub fn new<T>(event_loop: &EventLoop<T>) -> Result<Self> {
        let window = lve_rs::Window::new(event_loop, Self::WIDTH, Self::HEIGHT, "Hello Vulkan!")?;

        Ok(Self { window })
    }

    pub fn run(&self) {}
}

fn main() -> Result<()> {
    env_logger::init();

    let mut event_loop = EventLoop::new();
    let app = App::new(&event_loop)?;

    event_loop
        .borrow_mut()
        .run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == app.window.window().id() => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::MainEventsCleared => app.run(),
                _ => (),
            }
        });

    Ok(())
}
