use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    path::PathBuf,
};

use merle_photo::MerleImage;

use crate::LoaderError;

pub(crate) struct MerleCache {
    loaded_items: HashMap<String, MerleImage>,
}

impl MerleCache {
    pub(crate) fn new() -> Self {
        Self { loaded_items: HashMap::new() }
    }

    pub(crate) fn request(&mut self, path: &PathBuf) -> Result<&MerleImage, LoaderError> {
        let key = String::from(path.as_path().to_str().unwrap());

        let a = match self.loaded_items.entry(key) {
            Occupied(o) => o.into_mut(),
            Vacant(v) => v.insert(make(path)?),
        };
        Ok(a)
    }
}
fn make(path: &PathBuf) -> Result<MerleImage, LoaderError> {
    MerleImage::open(path)
        .map(MerleImage::into_rgba8)
        .map_err(|source| LoaderError::LoadImage { path: path.clone(), source: Box::new(source) })
}
