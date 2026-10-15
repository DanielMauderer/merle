use tracing::{instrument, trace};

#[derive(Debug)]
pub struct Viewport {
    pub target_size: (u32, u32),
    pub zoom: f64,
    pub center: (f64, f64),
}

impl Viewport {
    #[instrument(level = "trace", skip(self,))]
    pub fn reset(&mut self) {
        self.center();
        self.zoom = 1.0;
    }

    #[instrument(level = "trace", skip(self,))]
    pub fn center(&mut self) {
        self.center = (0.0, 0.0);
    }

    #[instrument(level = "trace", skip(self,))]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.target_size = (width, height);
    }

    #[instrument(level = "trace", skip(self,))]
    pub fn move_pixel(&mut self, pixels: (f64, f64)) {
        let aspect = pixel_to_windows(self.target_size);
        let delta = (pixels.0 * aspect.0, -pixels.1 * aspect.1);
        self.move_relative(delta);
    }

    #[instrument(level = "trace", skip(self,))]
    pub fn move_relative(&mut self, delta: (f64, f64)) {
        self.center.0 += delta.0;
        self.center.1 += delta.1;
    }

    #[instrument(level = "trace", skip(self,))]
    pub fn move_to_pixel(&mut self, pixels: (f64, f64)) {
        let aspect = pixel_to_windows(self.target_size);
        let delta = (pixels.0 * aspect.0, pixels.1 * aspect.1);
        self.move_absolut(delta);
    }

    #[instrument(level = "trace", skip(self,))]
    pub fn move_absolut(&mut self, delta: (f64, f64)) {
        self.center.0 = delta.0;
        self.center.1 = -delta.1;
    }

    #[instrument(level = "trace", skip(self))]
    pub fn zoom(&mut self, delta: f64, zoom_point: (f64, f64)) {
        let lower_limit = 0.5;
        let upper_limit = 10.0;
        self.zoom = (self.zoom + delta * 0.1).clamp(lower_limit, upper_limit);
        self.move_to_pixel((zoom_point.0 / 2.0, zoom_point.1 / 2.0));
        trace!(self.zoom, "scale");
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self { target_size: (0, 0), zoom: 1.0, center: (0.0, 0.0) }
    }
}

fn pixel_to_windows((win_w, win_h): (u32, u32)) -> (f64, f64) {
    (2.0 / f64::from(win_w), 2.0 / f64::from(win_h))
}
