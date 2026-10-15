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

use crate::error::LoaderError;

#[derive(Debug)]
pub struct MerleLoader<F: Fn(MerleImage) -> ()> {
    base_dir: String,
    file_index: usize,
    cb: F,
}

impl<F> MerleLoader<F>
where
    F: Fn(MerleImage) -> (),
{
    #[must_use]
    pub fn new(base_dir: &str, cb: F) -> Self {
        let path = Path::new(&base_dir);
        Self { base_dir: String::from(base_dir), file_index: 0, cb }
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

        let path_string = format!(r"{}\{}", self.base_dir, name);
        let path = Path::new(&path_string);
        ();

        (self.cb)(MerleImage::open(path).map(MerleImage::into_rgba8).map_err(|source| {
            LoaderError::LoadImage { path: path.to_owned(), source: Box::new(source) }
        })?);

        Ok(())
    }

    pub fn next(&mut self) -> Result<(), LoaderError>
    where
        F: Fn() -> (),
    {
        let path = Path::new(&self.base_dir);

        self.file_index += 1;
        if path.read_dir().iter().count() >= self.file_index {
            self.file_index = 0;
        }
        let f = path.read_dir().unwrap().into_iter().nth(self.file_index).unwrap().unwrap();
        (self.cb)(MerleImage::open(&f.path()).map(MerleImage::into_rgba8).map_err(|source| {
            LoaderError::LoadImage { path: path.to_owned(), source: Box::new(source) }
        })?);
        Ok(())
    }

    pub fn prev(&mut self) -> Result<(), LoaderError> {
        let path = Path::new(&self.base_dir);

        self.file_index -= 1;
        if self.file_index <= 0 {
            self.file_index = path.read_dir().iter().count() - 1;
        }
        let f = path.read_dir().unwrap().into_iter().nth(self.file_index).unwrap().unwrap();
        (self.cb)(MerleImage::open(&f.path()).map(MerleImage::into_rgba8).map_err(|source| {
            LoaderError::LoadImage { path: path.to_owned(), source: Box::new(source) }
        })?);
        Ok(())
    }
}
