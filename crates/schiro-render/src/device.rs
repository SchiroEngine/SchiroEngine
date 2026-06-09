//! Device management, swapchain handling and synchronization.
//!
//! Currently a placeholder: the wgpu device, queue and surface live on
//! the [`crate::Renderer`] struct, but the responsibilities belonging
//! here (frame pacing, fence reuse, debug markers) will be migrated
//! into dedicated types as the engine grows.
