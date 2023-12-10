mod app;

use anyhow::{bail, Result};
use app::App;
use std::borrow::BorrowMut;
use std::time;
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
            let start_time = time::Instant::now();

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

            match *control_flow {
                ControlFlow::Exit => (),
                _ => {
                    // Limit FPS (Frames per second) to around 144
                    let elapsed_time =
                        time::Instant::now().duration_since(start_time).as_micros() as u64;
                    let wait_microsecond = (App::MILLISECONDS_PER_FRAME - elapsed_time).max(0);
                    let new_inst = start_time + time::Duration::from_micros(wait_microsecond);

                    *control_flow = ControlFlow::WaitUntil(new_inst);
                }
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
