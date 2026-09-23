// merle — Copyright (C) 2026 Daniel Mauderer
//
// This program is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version. See the LICENSE-GPL file for the full text.

//! The merle culler.

use tracing::info;
use tracing_subscriber::EnvFilter;

/// Install the tracing subscriber; `RUST_LOG` overrides the default filter.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("merle_app=debug,merle_raw=debug,merle_loader=debug,merle_core=debug,warn")
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_writer(std::io::stderr)
        .init();
}

fn main() {
    init_tracing();
    info!("merle-app: not implemented yet");
}
