use tracing::instrument;

#[derive(Debug, Default)]
pub struct Pointer {
    position: (f64, f64),
    dragging: bool,
}

impl Pointer {
    #[instrument(name = "Pointer::press", level = "trace", skip_all)]
    pub fn press(&mut self) {
        self.dragging = true;
    }

    #[instrument(name = "Pointer::release", level = "trace", skip_all)]
    pub fn release(&mut self) {
        self.dragging = false;
    }

    pub fn moved(&mut self, position: (f64, f64)) -> Option<(f64, f64)> {
        let delta = (position.0 - self.position.0, position.1 - self.position.1);
        self.position = position;
        self.dragging.then_some(delta)
    }

    #[must_use]
    pub fn position(&self) -> (f64, f64) {
        self.position
    }
}
