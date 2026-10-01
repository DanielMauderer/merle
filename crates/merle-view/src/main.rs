// merle — Copyright (C) 2026 Daniel Mauderer
//
// This program is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version. See the LICENSE-GPL file for the full text.

//! Minimal image viewer.
use std::env;
use std::path::Path;

use image::DynamicImage;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

pub mod sb;
pub mod wgp;
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

#[tokio::main]
async fn main() {
    init_tracing();
    start_app().expect("window create failed");
}

fn start_app() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new()?;
    let mut args = env::args_os().skip(1);
    let (Some(from), None) = (args.next(), args.next()) else {
        error!("usage: merle-view <image path>");
        std::process::exit(1);
    };
    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let image = open(Path::new(&from));
    info!(width = image.width(), height = image.height(), "loaded image");
    let mut app = wgp::App::new(image);

    info!("entering event loop");
    event_loop.run_app(&mut app)
}

fn open(path: &Path) -> DynamicImage {
    let loader = rawler::RawLoader::new();
    let source = rawler::rawsource::RawSource::new(path).expect("cant read source");

    let decoder = loader.get_decoder(&source).expect("cant decode source");
    decoder
        .preview_image(&source, &rawler::decoders::RawDecodeParams::default())
        .expect("error extracting preview image")
        .expect("source has no preview image")
}
