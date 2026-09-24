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
use std::time::Instant;

use image::{DynamicImage, GenericImageView, Pixel};
use softbuffer::{Context, Surface};
use tracing::{debug, error, info, instrument, trace};
use tracing_subscriber::EnvFilter;
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Install the tracing subscriber.
///
/// `RUST_LOG` wins when it is set; otherwise we default to debug for our own
/// crates and warnings from everything else, so `bacon view` is useful with no
/// environment plumbing.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("merle_view=debug,merle_raw=debug,merle_loader=debug,warn")
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_writer(std::io::stderr)
        .init();
}

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
    #[instrument(skip_all, fields(path = %path.display()))]
    fn new(path: &Path) -> Self {
        let image = image::open(path).unwrap();
        info!(width = image.width(), height = image.height(), "loaded image");
        Self { view: None, image }
    }

    /// Paint one frame: the image scaled to the current window size.
    #[instrument(skip_all, level = "trace")]
    fn draw(&mut self) {
        let frame_start = Instant::now();
        let Some(view) = self.view.as_mut() else {
            trace!("no surface yet, skipping frame");
            return;
        };

        let size = view.window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            // Minimised or not yet mapped — nothing to present into.
            trace!(?size, "zero-sized window, skipping frame");
            return;
        };
        let scale_start = Instant::now();
        let image_buffer =
            self.image.resize(width.get(), height.get(), image::imageops::FilterType::Gaussian);
        let scale_time = scale_start.elapsed();
        trace!(
            window_width = width.get(),
            window_height = height.get(),
            scaled_width = image_buffer.width(),
            scaled_height = image_buffer.height(),
            "drawing frame"
        );
        let surface_start = Instant::now();
        if let Err(err) = view.surface.resize(width, height) {
            error!(%err, "failed to resize surface");
            return;
        }
        let surface_resize_time = surface_start.elapsed();

        let acquire_start = Instant::now();
        let mut buffer = match view.surface.buffer_mut() {
            Ok(buffer) => buffer,
            Err(err) => {
                error!(%err, "failed to acquire buffer");
                return;
            }
        };
        let acquire_time = acquire_start.elapsed();

        // softbuffer h
        // ands us `width * height` pixels in 0RGB order.
        let width = width.get() as usize;
        let height = height.get() as usize;
        let fill_start = Instant::now();
        let width_gap = ((width - image_buffer.width() as usize) / 2) as u32;
        let height_gap = ((height - image_buffer.height() as usize) / 2) as u32;
        debug!("w_g: {width_gap} | h_g: {height_gap}");
        for (index, buffer_pixel) in buffer.iter_mut().enumerate() {
            let x = (index % width) as u32;
            let y = (index / width) as u32;
            if x >= image_buffer.width() + width_gap
                || y >= image_buffer.height() + height_gap
                || x <= width_gap as u32
                || y <= height_gap as u32
            {
                *buffer_pixel = 0;
                continue;
            }
            let pixel = image_buffer.get_pixel(x - width_gap, y - height_gap).to_rgb().0;
            let red = u32::from(pixel[0]);
            let green = u32::from(pixel[1]);
            let blue = u32::from(pixel[2]);

            *buffer_pixel = (red << 16) | (green << 8) | blue;
        }
        let fill_time = fill_start.elapsed();

        let present_start = Instant::now();
        if let Err(err) = buffer.present() {
            error!(%err, "failed to present buffer");
        }
        let present_time = present_start.elapsed();

        debug!(
            scale = ?scale_time,
            surface_resize = ?surface_resize_time,
            buffer_acquire = ?acquire_time,
            fill = ?fill_time,
            present = ?present_time,
            frame = ?frame_start.elapsed(),
            "frame timing"
        );
    }
}

impl ApplicationHandler for App {
    #[instrument(skip_all)]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = match event_loop.create_window(Window::default_attributes()) {
            Ok(window) => Rc::new(window),
            Err(err) => {
                error!(%err, "failed to create window");
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
                debug!(size = ?window.inner_size(), "surface created");
                // Ask for the first frame explicitly; whether the platform sends an
                // initial `RedrawRequested` after mapping is not guaranteed.
                window.request_redraw();
                self.view = Some(View { window, surface });
            }
            Err(err) => {
                error!(%err, "failed to create drawing surface");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("close button pressed, stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                debug!(width = size.width, height = size.height, "window resized");
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
    init_tracing();

    let event_loop = EventLoop::new()?;
    let from = if env::args_os().count() == 2 {
        env::args_os().nth(1).unwrap()
    } else {
        error!("usage: merle-view <image path>");
        std::process::exit(1);
    };
    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(Path::new(&from));
    info!("entering event loop");
    event_loop.run_app(&mut app)
}
