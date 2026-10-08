// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Open a single photo file (RAW via rawler, JPEG/PNG via image): embedded
//! preview, orientation, AF points and metadata. One file, synchronous, no
//! caching.
//!
//! Placeholder: no functionality yet.

use std::borrow::Cow;
use std::num::NonZero;
use std::path::Path;
use std::thread;

use image::{DynamicImage, GenericImageView, RgbaImage};
use tracing::{Span, debug_span, field, info, instrument};

#[derive(Debug, Clone, Default)]
pub struct MerleImage {
    pub preview_image: DynamicImage,
}

impl MerleImage {
    #[must_use]
    #[instrument(skip_all, fields(path = %path.display(), width = field::Empty, height = field::Empty))]
    pub fn open(path: &Path) -> Self {
        let loader = debug_span!("raw_loader").in_scope(rawler::RawLoader::new);
        let source = debug_span!("raw_source")
            .in_scope(|| rawler::rawsource::RawSource::new(path).expect("cant read source"));

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

    #[must_use]
    #[instrument(skip_all)]
    pub fn into_rgba8(mut self) -> Self {
        if !matches!(self.preview_image, DynamicImage::ImageRgba8(_)) {
            self.preview_image = to_rgba8(&self.preview_image).into();
        }
        self
    }

    #[must_use]
    pub fn rgba8(&self, max_dim: u32) -> Cow<'_, RgbaImage> {
        let (width, height) = self.dimensions();
        let fits = width <= max_dim && height <= max_dim;
        match &self.preview_image {
            DynamicImage::ImageRgba8(rgba) if fits => Cow::Borrowed(rgba),
            preview if fits => Cow::Owned(to_rgba8(preview)),
            preview => Cow::Owned(to_rgba8(&preview.thumbnail(max_dim, max_dim))),
        }
    }
}

fn to_rgba8(image: &DynamicImage) -> RgbaImage {
    let DynamicImage::ImageRgb8(rgb) = image else { return image.to_rgba8() };
    let (width, height) = rgb.dimensions();
    let src = rgb.as_raw();
    let mut dst = vec![0; src.len() / 3 * 4];

    let threads = thread::available_parallelism().map_or(1, NonZero::get);
    let band_px = (src.len() / 3).div_ceil(threads).max(1);
    thread::scope(|scope| {
        for (dst, src) in dst.chunks_mut(band_px * 4).zip(src.chunks(band_px * 3)) {
            scope.spawn(move || {
                for (out, px) in dst.as_chunks_mut::<4>().0.iter_mut().zip(src.as_chunks::<3>().0) {
                    *out = [px[0], px[1], px[2], u8::MAX];
                }
            });
        }
    });

    RgbaImage::from_raw(width, height, dst).expect("buffer matches dimensions")
}
