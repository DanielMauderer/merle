use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;

use image::{DynamicImage, GenericImageView, Pixel};
use softbuffer::{Context, Surface};
use tracing::{debug, error, info, instrument, trace};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

/// A window plus the CPU-side pixel buffer we present into it.
#[derive(Debug)]
struct View {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

#[derive(Default, Debug)]
pub struct App {
    view: Option<View>,
    image: DynamicImage,
}

impl App {
    #[must_use]
    pub fn new(image: DynamicImage) -> Self {
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
            self.image.resize(width.get(), height.get(), image::imageops::FilterType::Nearest);
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

        // softbuffer hands us `width * height` pixels in 0RGB order.
        let width = width.get();
        let height = height.get();
        let fill_start = Instant::now();
        let width_gap = (width - image_buffer.width()) / 2;
        let height_gap = (height - image_buffer.height()) / 2;
        debug!("w_g: {width_gap} | h_g: {height_gap}");
        let rows = buffer.chunks_exact_mut(width as usize);
        for (y, row) in (0..height).zip(rows) {
            for (x, buffer_pixel) in (0..width).zip(row.iter_mut()) {
                if x >= image_buffer.width() + width_gap
                    || y >= image_buffer.height() + height_gap
                    || x <= width_gap
                    || y <= height_gap
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
            // Preferable to drawing in `about_to_wait`: the OS can ask us to redraw
            // (damage, unminimise) and we honour it in the same place.
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }
}
