use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::SurfaceTargetUnsafe;

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
