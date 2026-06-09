//! Typed asset handle returned by the asset server.
//!
//! A handle is a cheap, cloneable smart pointer to an `Arc` slot. While
//! the asset is still loading the slot is empty; callers can use
//! [`Handle::resolve`] to obtain the underlying value or
//! [`Handle::is_loaded`] to poll for completion.

use std::marker::PhantomData;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::Asset;

/// Cloneable handle to a potentially-not-yet-loaded asset of type `T`.
#[derive(Debug)]
pub struct Handle<T: Asset> {
    pub(crate) inner: Arc<RwLock<Option<Arc<T>>>>,
    _phantom: PhantomData<T>,
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner), _phantom: PhantomData }
    }
}

impl<T: Asset> Handle<T> {
    /// Builds an empty handle that is not currently associated with any
    /// asset data.
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(None)), _phantom: PhantomData }
    }

    /// Builds a handle that already wraps a fully loaded asset.
    pub fn new_loaded(value: T) -> Self {
        Self { inner: Arc::new(RwLock::new(Some(Arc::new(value)))), _phantom: PhantomData }
    }

    /// Returns a clone of the underlying asset, or `None` if the asset
    /// has not finished loading.
    pub fn resolve(&self) -> Option<Arc<T>> {
        self.inner.read().clone()
    }

    /// Returns `true` once the underlying slot has been populated.
    pub fn is_loaded(&self) -> bool {
        self.inner.read().is_some()
    }
}

impl<T: Asset> Default for Handle<T> {
    fn default() -> Self {
        Self::new()
    }
}
