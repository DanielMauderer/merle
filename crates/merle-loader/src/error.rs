use core::fmt;
use std::{error::Error, io, path::PathBuf};

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use merle_photo::PhotoError;

#[derive(Debug)]
pub enum LoaderError {
    ReadDir { path: PathBuf, source: io::Error },
    NotInDirectory { path: PathBuf },
    LoadImage { path: PathBuf, source: Box<PhotoError> },
}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDir { path, .. } => write!(f, "cannot read directory {}", path.display()),
            Self::NotInDirectory { path } => {
                write!(f, "{} is not a supported image in its directory", path.display())
            }
            Self::LoadImage { path, .. } => write!(f, "cannot read image {}", path.display()),
        }
    }
}

impl Error for LoaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadDir { source, .. } => Some(source),
            Self::NotInDirectory { .. } => None,
            Self::LoadImage { source, .. } => Some(source.as_ref()),
        }
    }
}
