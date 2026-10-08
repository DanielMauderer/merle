// merle — Copyright (C) 2026 Daniel Mauderer
//
// This program is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version. See the LICENSE-GPL file for the full text.

//! Minimal image viewer.
use std::env;
use std::path::Path;
use std::thread;

use merle_photo::MerleImage;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::Uptime;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

pub mod app;
pub mod gpu;
pub mod image_view;

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("merle_view=debug,merle_photo=debug,merle_loader=debug,warn")
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_timer(Uptime::default())
        .with_writer(std::io::stderr)
        .init();
}

fn main() {
    init_tracing();
    start_app().expect("window create failed");
}

fn start_app() -> Result<(), EventLoopError> {
    let mut args = env::args_os().skip(1);
    let (Some(from), None) = (args.next(), args.next()) else {
        error!("usage: merle-view <image path>");
        std::process::exit(1);
    };
    let event_loop = EventLoop::<MerleImage>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let proxy = event_loop.create_proxy();
    thread::spawn(move || {
        let image = MerleImage::open(Path::new(&from)).into_rgba8();
        let _ = proxy.send_event(image);
    });

    let mut app = app::App::default();
    info!("entering event loop");
    event_loop.run_app(&mut app)
}
