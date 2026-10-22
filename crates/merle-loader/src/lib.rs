// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A folder of photos: ordered listing, prev/next navigation and background
//! preloading of neighbours. Decoding a single file lives in `merle-photo`.
//!
//! Placeholder: no functionality yet.
mod error;

use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};

use merle_photo::MerleImage;
use tracing::info;

pub use crate::error::LoaderError;

pub struct MerleLoader {
    base_dir: PathBuf,
    file_index: usize,
    cb: Box<dyn Fn(MerleImage) + Send>,
}

impl fmt::Debug for MerleLoader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MerleLoader")
            .field("base_dir", &self.base_dir)
            .field("file_index", &self.file_index)
            .finish_non_exhaustive()
    }
}

impl MerleLoader {
    #[must_use]
    pub fn new(base_dir: PathBuf, cb: impl Fn(MerleImage) + Send + 'static) -> Self {
        Self { base_dir, file_index: 0, cb: Box::new(cb) }
    }

    pub fn load_image(&mut self, name: &OsStr) -> Result<(), LoaderError> {
        let mut path = self.base_dir.clone();
        path.push(name);

        let files = self.entries()?;
        self.file_index =
            files.iter().position(|f| *f == path).ok_or(LoaderError::NotInDirectory { path })?;
        self.show(&files)
    }

    pub fn load_next(&mut self) -> Result<(), LoaderError> {
        let files = self.entries()?;
        if files.is_empty() {
            return Ok(());
        }
        self.file_index = (self.file_index + 1) % files.len();
        self.show(&files)
    }

    pub fn load_prev(&mut self) -> Result<(), LoaderError> {
        let files = self.entries()?;
        if files.is_empty() {
            return Ok(());
        }
        self.file_index = (self.file_index + files.len() - 1) % files.len();
        self.show(&files)
    }

    fn entries(&self) -> Result<Vec<PathBuf>, LoaderError> {
        let read_dir = std::fs::read_dir(&self.base_dir)
            .map_err(|source| LoaderError::ReadDir { path: self.base_dir.clone(), source })?;
        let mut files: Vec<PathBuf> = read_dir
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_file() && is_raw(p))
            .collect();
        files.sort();
        Ok(files)
    }

    fn show(&mut self, files: &[PathBuf]) -> Result<(), LoaderError> {
        let path = &files[self.file_index];
        info!("path: {}", path.display());
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
