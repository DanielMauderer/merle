use std::sync::Arc;

use merle_photo::MerleImage;
use tracing::instrument;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, OwnedDisplayHandle},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::gpu::Gpu;
use crate::image_view::ImageView;

#[derive(Debug)]
pub struct State {
    window: Arc<Window>,
    gpu: Gpu,
    image_view: ImageView,
}

impl State {
    #[instrument(name = "State::new", skip_all)]
    pub async fn new(window: Arc<Window>, display: OwnedDisplayHandle) -> Self {
        let gpu = Gpu::new(Arc::clone(&window), display).await;
        let image_view = ImageView::new(&gpu.device, gpu.format());

        let size = window.inner_size();
        let mut state = Self { window, gpu, image_view };
        state.resize(size.width, size.height);
        state
    }

    pub fn set_image(&mut self, image: &MerleImage) {
        self.image_view.set_image(&self.gpu.device, &self.gpu.queue, image);
        self.window.request_redraw();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
        self.image_view.resize(&self.gpu.queue, width, height);
        self.window.request_redraw();
    }

    pub fn render(&mut self) {
        self.gpu.render(|render_pass| self.image_view.draw(render_pass));
    }
}

#[derive(Debug, Default)]
pub struct App {
    state: Option<State>,
    pending: Option<MerleImage>,
}

impl ApplicationHandler<MerleImage> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop.create_window(Window::default_attributes()).expect("cant create window"),
        );
        let mut state = pollster::block_on(State::new(window, event_loop.owned_display_handle()));
        if let Some(image) = self.pending.take() {
            state.set_image(&image);
        }
        self.state = Some(state);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, image: MerleImage) {
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
            WindowEvent::RedrawRequested => state.render(),
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(code), state: key_state, .. },
                ..
            } => {
                if let (KeyCode::KeyQ, true) = (code, key_state.is_pressed()) {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}
