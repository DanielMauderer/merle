// merle — Copyright (C) 2026 Daniel Mauderer
//
// This program is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version. See the LICENSE-GPL file for the full text.

//! Minimal image viewer.
use std::env;
use std::num::NonZeroU32;
use std::path::Path;
use std::rc::Rc;

use image::{DynamicImage, GenericImageView, Pixel};
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// A window plus the CPU-side pixel buffer we present into it.
struct View {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

#[derive(Default)]
struct App {
    view: Option<View>,
    image: DynamicImage,
}

impl App {
    pub fn new(path: &Path) -> Self {
        Self { view: Option::None, image: image::open(path).unwrap() }
    }

    /// Paint one frame: a vertical gradient, so we can see that pixels land.
    fn draw(&mut self) {
        let Some(view) = self.view.as_mut() else {
            return;
        };

        let size = view.window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            // Minimised or not yet mapped — nothing to present into.
            return;
        };

        let image_buffer = self
            .image
            .resize(width.get(), height.get(), image::imageops::FilterType::Gaussian);
        if let Err(err) = view.surface.resize(width, height) {
            eprintln!("failed to resize surface: {err}");
            return;
        }

        let mut buffer = match view.surface.buffer_mut() {
            Ok(buffer) => buffer,
            Err(err) => {
                eprintln!("failed to acquire buffer: {err}");
                return;
            }
        };

        // softbuffer h
        // ands us `width * height` pixels in 0RGB order.
        let width = width.get() as usize;
        for (index, buffer_pixel) in buffer.iter_mut().enumerate() {
            let x = (index % width) as u32;
            let y = (index / width) as u32;
            if x >= image_buffer.width() || y >= image_buffer.height(){
                *buffer_pixel = 0;
                continue;
            }
            let pixel = image_buffer.get_pixel(x, y).to_rgb().0;
            let red = u32::from(pixel[0]);
            let green = u32::from(pixel[1]);
            let blue = u32::from(pixel[2]);

            *buffer_pixel = (red << 16) | (green << 8) | blue;
        }

        if let Err(err) = buffer.present() {
            eprintln!("failed to present buffer: {err}");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = match event_loop.create_window(Window::default_attributes()) {
            Ok(window) => Rc::new(window),
            Err(err) => {
                eprintln!("failed to create window: {err}");
                event_loop.exit();
                return;
            }
        };

        // `Rc<Window>` is both the display and the window handle, which keeps the
        // surface's borrow of the window trivially sound.
        let surface = Context::new(Rc::clone(&window))
            .and_then(|context| Surface::new(&context, Rc::clone(&window)));

        match surface {
            Ok(surface) => {
                // Ask for the first frame explicitly; whether the platform sends an
                // initial `RedrawRequested` after mapping is not guaranteed.
                window.request_redraw();
                self.view = Some(View { window, surface });
            }
            Err(err) => {
                eprintln!("failed to create drawing surface: {err}");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                if let Some(view) = self.view.as_ref() {
                    view.window.request_redraw();
                }
            }
            // Preferable to drawing in `about_to_wait`: the OS can ask us to redraw
            // (damage, unminimise) and we honour it in the same place.
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }
}

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new()?;
    let from = if env::args_os().count() == 2 {
        env::args_os().nth(1).unwrap()
    } else {
        println!("Please enter a from and into path.");
        std::process::exit(1);
    };
    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(Path::new(&from));
    event_loop.run_app(&mut app)
}
