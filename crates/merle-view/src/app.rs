use std::sync::Arc;

use merle_photo::MerleImage;
use tokio::{runtime::Handle, task::JoinHandle};
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
    /// Sets up the GPU and pipeline while `image` is still decoding, and only
    /// waits for it right before the texture upload.
    pub async fn new(
        window: Arc<Window>,
        display: OwnedDisplayHandle,
        image: JoinHandle<MerleImage>,
    ) -> Self {
        let gpu = Gpu::new(Arc::clone(&window), display).await;
        let mut image_view = ImageView::new(&gpu.device, gpu.format());
        let image = image.await.expect("image load task failed");
        image_view.set_image(&gpu.device, &gpu.queue, &image);

        let size = window.inner_size();
        let mut state = Self { window, gpu, image_view };
        state.resize(size.width, size.height);
        state
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

#[derive(Debug)]
pub struct App {
    runtime: Handle,
    state: Option<State>,
    /// In-flight decode, consumed by the first `resumed`.
    image: Option<JoinHandle<MerleImage>>,
}

impl App {
    #[must_use]
    pub fn new(runtime: Handle, image: JoinHandle<MerleImage>) -> Self {
        Self { runtime, state: None, image: Some(image) }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(image) = self.image.take() else { return };
        let window = Arc::new(
            event_loop.create_window(Window::default_attributes()).expect("cant create window"),
        );

        self.state = Some(self.runtime.block_on(State::new(
            window,
            event_loop.owned_display_handle(),
            image,
        )));
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
