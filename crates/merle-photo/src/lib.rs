// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Open a single photo file (RAW via rawler, JPEG/PNG via image): embedded
//! preview, orientation, AF points and metadata. One file, synchronous, no
//! caching.
//!
//! Placeholder: no functionality yet.

use std::path::Path;

use image::{DynamicImage, GenericImageView};
use tracing::{Span, debug_span, field, info, instrument};

#[derive(Debug, Clone, Default)]
pub struct MerleImage {
    pub preview_image: DynamicImage,
}

impl MerleImage {
    #[must_use]
    #[instrument(skip_all, fields(path = %path.display(), width = field::Empty, height = field::Empty))]
    pub fn open(path: &Path) -> Self {
        let loader = rawler::RawLoader::new();
        let source = rawler::rawsource::RawSource::new(path).expect("cant read source");

        let decoder = debug_span!("get_decoder")
            .in_scope(|| loader.get_decoder(&source).expect("cant decode source"));
        let image = debug_span!("preview_image").in_scope(|| {
            decoder
                .preview_image(&source, &rawler::decoders::RawDecodeParams::default())
                .expect("error extracting preview image")
                .expect("source has no preview image")
        });

        let (width, height) = image.dimensions();
        Span::current().record("width", width).record("height", height);
        info!(width, height, color = ?image.color(), "preview loaded");

        Self { preview_image: image }
    }

    #[must_use]
    pub fn dimensions(&self) -> (u32, u32) {
        self.preview_image.dimensions()
    }
}
