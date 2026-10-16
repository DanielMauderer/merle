// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A folder of photos: ordered listing, prev/next navigation and background
//! preloading of neighbours. Decoding a single file lives in `merle-photo`.
//!
//! Placeholder: no functionality yet.
mod error;

use std::path::{Path, PathBuf};

use merle_photo::MerleImage;
use tracing::info;

use crate::error::LoaderError;

pub struct MerleLoader {
    base_dir: String,
    file_index: usize,
    cb: Box<dyn Fn(MerleImage) + Send>,
}

impl MerleLoader {
    #[must_use]
    pub fn new(base_dir: &str, cb: impl Fn(MerleImage) + Send + 'static) -> Self {
        let _path = Path::new(&base_dir);
        Self { base_dir: String::from(base_dir), file_index: 0, cb: Box::new(cb) }
    }

    pub fn load_image(&mut self, name: &str) -> Result<(), LoaderError> {
        let b_path = PathBuf::from(&self.base_dir);

        let file_index = b_path
            .read_dir()
            .unwrap()
            .into_iter()
            .position(|f| {
                if let Ok(fle) = f {
                    return fle.file_name() == name;
                }
                false
            })
            .unwrap();

        self.file_index = file_index;

        let files = self.entries();
        if files.is_empty() {
            return Ok(());
        }
        self.show(&files)?;

        Ok(())
    }
    pub fn next(&mut self) -> Result<(), LoaderError> {
        let files = self.entries();
        if files.is_empty() {
            return Ok(());
        }
        self.file_index = (self.file_index + 1) % files.len();
        self.show(&files)
    }

    pub fn prev(&mut self) -> Result<(), LoaderError> {
        let files = self.entries();
        if files.is_empty() {
            return Ok(());
        }
        self.file_index = (self.file_index + files.len() - 1) % files.len();
        self.show(&files)
    }
    fn entries(&self) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&self.base_dir)
            .map(|rd| {
                rd.filter_map(Result::ok)
                    .map(|e| e.path())
                    .filter(|p| p.is_file() && is_raw(p))
                    .collect()
            })
            .unwrap_or_default();
        files.sort();
        files
    }
    fn show(&mut self, files: &[PathBuf]) -> Result<(), LoaderError> {
        let path = &files[self.file_index];
        info!("path: {}", path.to_str().unwrap());
        (self.cb)(MerleImage::open(path).map(MerleImage::into_rgba8).map_err(|source| {
            LoaderError::LoadImage { path: path.clone(), source: Box::new(source) }
        })?);
        Ok(())
    }
}

fn is_raw(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("cr2") || e.eq_ignore_ascii_case("cr3"))
}
