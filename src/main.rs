mod app;

use anyhow::Result;
use app::App;
use std::borrow::BorrowMut;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

fn main() -> Result<()> {
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
                } if window_id == app.window().id() => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::MainEventsCleared => app.run(),
                _ => (),
            }
        });

    Ok(())
}
