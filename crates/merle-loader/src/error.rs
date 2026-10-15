use core::fmt;
use std::{error::Error, path::PathBuf};

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use merle_photo::PhotoError;

#[derive(Debug)]
pub enum LoaderError {
    LoadImage { path: PathBuf, source: Box<PhotoError> },
}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LoadImage { path, .. } => write!(f, "cannot read image {}", path.display()),
        }
    }
}

impl Error for LoaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LoadImage { source, .. } => Some(source.as_ref()),
        }
    }
}
