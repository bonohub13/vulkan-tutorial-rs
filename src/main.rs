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
    let mut app = App::new(&event_loop, None, None)?;
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
                    WindowEvent::Resized(physical_size) => {
                        println!(
                            "window resized: ({}, {})",
                            physical_size.width, physical_size.height
                        );
                        app.window_resized(physical_size.width as i32, physical_size.height as i32);
                    }
                    _ => (),
                },
                Event::RedrawRequested(_) => {}
                Event::MainEventsCleared => {
                    app.draw_frame(Some(control_flow)).unwrap_or_else(|e| {
                        eprintln!("{:?}", e);
                        *control_flow = ControlFlow::ExitWithCode(0x10);
                    })
                }
                _ => (),
            }

            match *control_flow {
                ControlFlow::Exit | ControlFlow::ExitWithCode(_) => {
                    unsafe { app.device_wait_idle() }.unwrap()
                }
                _ => {
                    // Limit FPS (Frames per second) to around 144
                    let elapsed_time =
                        time::Instant::now().duration_since(start_time).as_micros() as u64;
                    let new_inst = if elapsed_time <= App::MILLISECONDS_PER_FRAME {
                        let wait_microsecond = (App::MILLISECONDS_PER_FRAME - elapsed_time).max(0);

                        start_time + time::Duration::from_micros(wait_microsecond)
                    } else {
                        start_time + time::Duration::from_micros(0)
                    };

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
