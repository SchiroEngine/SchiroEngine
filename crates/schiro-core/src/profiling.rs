//! Tracing initialization helpers.
//!
//! Wraps [`tracing_subscriber`] so that the rest of the engine can simply
//! call [`init`] once during startup to get a sensible default logger.

pub use tracing::info;

/// Installs a global [`tracing`] subscriber using a compact formatter and
/// the `RUST_LOG` environment filter, falling back to `info` when the
/// variable is unset.
///
/// Calling this more than once is a no-op.
pub fn init() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_names(true)
        .compact()
        .finish();

    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("tracing subscriber already set: {e}");
    }
}
