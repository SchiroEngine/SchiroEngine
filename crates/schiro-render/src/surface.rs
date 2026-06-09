//! Surface creation from a `winit` window.
//!
//! Thin wrapper around `wgpu::Instance::create_surface_unsafe`. The
//! `unsafe` call is safe because the returned surface borrows from the
//! supplied window which the caller must keep alive for at least as
//! long as the surface.

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::SurfaceTargetUnsafe;

/// Creates a wgpu surface bound to the supplied `winit` window.
///
/// # Safety
///
/// The returned [`wgpu::Surface`] must not outlive `window`. The caller
/// is responsible for upholding that invariant.
pub fn create_surface(
    instance: &wgpu::Instance,
    window: &winit::window::Window,
) -> Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError> {
    let target = SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: window.display_handle().unwrap().as_raw(),
        raw_window_handle: window.window_handle().unwrap().as_raw(),
    };
    // SAFETY: the returned Surface<'static> must not outlive the window.
    // The caller owns the window (Arc<Window>) which is guaranteed to
    // outlive any surface created from it.
    unsafe { instance.create_surface_unsafe(target) }
}
