mod app;

use anyhow::{bail, Result};
use app::App;
use std::borrow::BorrowMut;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

fn main() -> Result<()> {
    let mut event_loop = EventLoop::new();
    let mut app = App::new(&event_loop)?;
    let result = event_loop
        .borrow_mut()
        .run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == app.window().id() => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                },
                Event::MainEventsCleared => app.draw_frame().unwrap_or_else(|e| {
                    eprintln!("{:?}", e);
                    *control_flow = ControlFlow::ExitWithCode(0x10);
                }),
                _ => (),
            }

            unsafe { app.device_wait_idle() }.unwrap_or_else(|e| {
                eprintln!("Device failed to wait idle!: {:?}", e);
                *control_flow = ControlFlow::ExitWithCode(0x20);
            });
        });

    match result {
        0 => Ok(()),
        _ => bail!("Exit with status: {:0x}", result),
    }
}
