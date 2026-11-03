use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    num::NonZeroUsize,
    path::PathBuf,
};

use lru::LruCache;
use merle_photo::MerleImage;
use tracing::info;

use crate::LoaderError;

pub(crate) struct MerleCache {
    loaded_items: LruCache<String, MerleImage>,
}

impl MerleCache {
    pub(crate) fn new(preload: usize) -> Self {
        Self { loaded_items: LruCache::new(NonZeroUsize::new(preload).unwrap()) }
    }

    pub(crate) fn request(&mut self, path: &PathBuf) -> Result<&MerleImage, LoaderError> {
        let key = String::from(path.as_path().to_str().unwrap());
        let f = || -> Result<MerleImage, LoaderError> {
            info!("!!!CACHEMISS");
            MerleImage::open(path).map(MerleImage::into_rgba8).map_err(|source| {
                LoaderError::LoadImage { path: path.clone(), source: Box::new(source) }
            })
        };

        self.loaded_items.try_get_or_insert(key, f)
    }
}
