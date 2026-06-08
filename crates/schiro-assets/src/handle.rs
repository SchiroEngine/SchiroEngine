use std::marker::PhantomData;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::Asset;

#[derive(Debug)]
pub struct Handle<T: Asset> {
    pub(crate) inner: Arc<RwLock<Option<Arc<T>>>>,
    _phantom: PhantomData<T>,
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            _phantom: PhantomData,
        }
    }
}

impl<T: Asset> Handle<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
            _phantom: PhantomData,
        }
    }

    pub fn new_loaded(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Some(Arc::new(value)))),
            _phantom: PhantomData,
        }
    }

    pub fn resolve(&self) -> Option<Arc<T>> {
        self.inner.read().clone()
    }

    pub fn is_loaded(&self) -> bool {
        self.inner.read().is_some()
    }
}

impl<T: Asset> Default for Handle<T> {
    fn default() -> Self {
        Self::new()
    }
}
