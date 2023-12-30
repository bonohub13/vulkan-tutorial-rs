mod app;
mod ray_tracing;

use anyhow::{bail, Result};
use app::App;
use std::{borrow::BorrowMut, time};
use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

const FRAMES_TO_RENDER: u32 = 10;

fn main() -> Result<()> {
    let mut event_loop = EventLoop::new();
    let mut app = App::new(&event_loop, None, None)?;
    let mut current_time = time::Instant::now();
    let mut keys_pressed: [Option<VirtualKeyCode>; 10] =
        [None, None, None, None, None, None, None, None, None, None];
    let mut rendered_frames = FRAMES_TO_RENDER;

    let result = event_loop
        .borrow_mut()
        .run_return(move |event, _, control_flow| {
            control_flow.set_poll();

            let new_time = time::Instant::now();
            let frame_time = (new_time - current_time).as_secs_f32();

            current_time = new_time;

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
                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::Key(KeyboardInput {
                        state,
                        virtual_keycode,
                        ..
                    }) => match state {
                        ElementState::Pressed => {
                            if !keys_pressed.contains(&virtual_keycode) {
                                if virtual_keycode == Some(VirtualKeyCode::Return) {
                                    rendered_frames = 0;
                                } else if let Some(key_unassigned) =
                                    keys_pressed.iter_mut().filter(|key| key.is_none()).next()
                                {
                                    *key_unassigned = virtual_keycode;
                                }
                            }
                        }
                        ElementState::Released => {
                            if let Some(key_assigned) = keys_pressed
                                .iter_mut()
                                .filter(|key| **key == virtual_keycode)
                                .next()
                            {
                                *key_assigned = None
                            }
                        }
                    },
                    _ => (),
                },
                Event::RedrawRequested(_) => {}
                Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {}
                Event::MainEventsCleared => {
                    // Only render if the ENTER key is pressed
                    if rendered_frames < FRAMES_TO_RENDER {
                        app.draw_frame(Some(control_flow), frame_time, &keys_pressed)
                            .unwrap_or_else(|e| {
                                eprintln!("{:?}", e);
                                *control_flow = ControlFlow::ExitWithCode(0x10);
                            });
                        rendered_frames += 1;
                    }
                }
                _ => (),
            }

            match *control_flow {
                ControlFlow::Exit | ControlFlow::ExitWithCode(_) => {
                    unsafe { app.device_wait_idle() }.unwrap()
                }
                _ => {}
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
