use std::{num::NonZeroUsize, path::PathBuf};

use lru::LruCache;
use merle_photo::MerleImage;
use tracing::info;

use crate::LoaderError;

pub(crate) struct MerleCache {
    loaded_items: LruCache<String, MerleImage>,
}

impl MerleCache {
    pub(crate) fn new(cache_size: usize) -> Self {
        Self { loaded_items: LruCache::new(NonZeroUsize::new(cache_size).unwrap()) }
    }

    pub(crate) fn request(&mut self, path_buf: &PathBuf) -> Result<&MerleImage, LoaderError> {
        let key = String::from(path_buf.as_path().to_str().expect("dont use names like this"));
        let f = || -> Result<MerleImage, LoaderError> {
            info!("!!!CACHEMISS");
            MerleImage::open(path_buf).map(MerleImage::into_rgba8).map_err(|source| {
                LoaderError::LoadImage { path: path_buf.clone(), source: Box::new(source) }
            })
        };

        self.loaded_items.try_get_or_insert(key, f)
    }
}
