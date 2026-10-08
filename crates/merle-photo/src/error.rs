// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use rawler::RawlerError;

#[derive(Debug)]
pub enum PhotoError {
    Read { path: PathBuf, source: io::Error },
    Decode { path: PathBuf, source: Box<RawlerError> },
    Preview { path: PathBuf, source: Box<RawlerError> },
    NoPreview { path: PathBuf },
}

impl fmt::Display for PhotoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, .. } => write!(f, "cannot read {}", path.display()),
            Self::Decode { path, .. } => write!(f, "cannot decode {}", path.display()),
            Self::Preview { path, .. } => {
                write!(f, "cannot extract preview from {}", path.display())
            }
            Self::NoPreview { path } => write!(f, "{} has no embedded preview", path.display()),
        }
    }
}

impl Error for PhotoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Decode { source, .. } | Self::Preview { source, .. } => Some(source.as_ref()),
            Self::NoPreview { .. } => None,
        }
    }
}
