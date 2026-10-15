use crate::error::{Error, GpuError};
use crate::gpu::Gpu;
use crate::image_view::ImageView;
use crate::input::Pointer;
use crate::viewport::Viewport;
use merle_photo::{MerleImage, PhotoError};
use std::sync::Arc;
use tracing::instrument;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, OwnedDisplayHandle},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

#[derive(Debug)]
pub struct State {
    window: Arc<Window>,
    gpu: Gpu,
    image_view: ImageView,
    pointer: Pointer,
    viewport: Viewport,
}

impl State {
    #[instrument(name = "State::new", skip_all)]
    pub async fn new(window: Arc<Window>, display: OwnedDisplayHandle) -> Result<Self, GpuError> {
        let gpu = Gpu::new(Arc::clone(&window), display).await?;
        let image_view = ImageView::new(&gpu.device, gpu.format());

        let size = window.inner_size();
        let mut state = Self {
            window,
            gpu,
            image_view,
            pointer: Pointer::default(),
            viewport: Viewport::default(),
        };
        state.resize(size.width, size.height);
        Ok(state)
    }

    pub fn begin_mouse_pan(&mut self) {
        self.pointer.press();
    }

    pub fn end_mouse_pan(&mut self) {
        self.pointer.release();
    }

    pub fn set_image(&mut self, image: &MerleImage) {
        self.image_view.set_image(&self.gpu.device, &self.gpu.queue, image);
        self.window.request_redraw();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
        self.viewport.resize(width, height);
        self.window.request_redraw();
    }

    pub fn move_relative(&mut self, delta: (f64, f64)) {
        self.viewport.move_relative(delta);
        self.window.request_redraw();
    }

    pub fn zoom(&mut self, delta: f64) {
        self.viewport.zoom(delta, self.pointer.position());
        self.window.request_redraw();
    }

    pub fn center(&mut self) {
        self.viewport.center();
        self.window.request_redraw();
    }

    pub fn reset(&mut self) {
        self.viewport.reset();
        self.window.request_redraw();
    }

    pub fn render(&mut self) -> Result<(), GpuError> {
        self.image_view.write_uniform(&self.gpu.queue, &self.viewport);
        self.gpu.render(|render_pass| self.image_view.draw(render_pass))
    }

    pub fn update_mouse_position(&mut self, pixels: (f64, f64)) {
        if let Some(delta) = self.pointer.moved(pixels) {
            self.viewport.move_pixel(delta);
            self.window.request_redraw();
        }
    }
}

#[derive(Debug, Default)]
pub struct App {
    state: Option<State>,
    pending: Option<MerleImage>,
    error: Option<Error>,
}

impl App {
    pub fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: Error) {
        self.error.get_or_insert(error);
        event_loop.exit();
    }
}

impl ApplicationHandler<Result<MerleImage, PhotoError>> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let window = match event_loop.create_window(Window::default_attributes()) {
            Ok(window) => Arc::new(window),
            Err(err) => return self.fail(event_loop, err.into()),
        };
        let mut state =
            match pollster::block_on(State::new(window, event_loop.owned_display_handle())) {
                Ok(state) => state,
                Err(err) => return self.fail(event_loop, err.into()),
            };
        if let Some(image) = self.pending.take() {
            state.set_image(&image);
        }
        self.state = Some(state);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, image: Result<MerleImage, PhotoError>) {
        let image = match image {
            Ok(image) => image,
            Err(err) => return self.fail(event_loop, err.into()),
        };
        match &mut self.state {
            Some(state) => state.set_image(&image),
            None => self.pending = Some(image),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else { return };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if let Err(err) = state.render() {
                    self.fail(event_loop, err.into());
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(code), state: key_state, .. },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::KeyQ, true) => event_loop.exit(),
                (KeyCode::KeyW, true) => state.move_relative((0.0, -0.1)),
                (KeyCode::KeyS, true) => state.move_relative((0.0, 0.1)),
                (KeyCode::KeyA, true) => state.move_relative((0.1, 0.0)),
                (KeyCode::KeyD, true) => state.move_relative((-0.1, 0.0)),
                (KeyCode::KeyR, true) => state.reset(),
                (KeyCode::KeyC, true) => state.center(),
                _ => (),
            },
            WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
                let change = match delta {
                    MouseScrollDelta::LineDelta(_, y) => f64::from(y),
                    MouseScrollDelta::PixelDelta(d) => d.y,
                };
                state.zoom(change);
            }
            WindowEvent::MouseInput { device_id: _, state: button_state, button } => {
                if button != MouseButton::Left {
                    return;
                }
                match button_state {
                    ElementState::Pressed => state.begin_mouse_pan(),
                    ElementState::Released => state.end_mouse_pan(),
                }
            }
            WindowEvent::CursorMoved { device_id: _, position } => {
                state.update_mouse_position((position.x, position.y));
            }
            _ => {}
        }
    }
}
