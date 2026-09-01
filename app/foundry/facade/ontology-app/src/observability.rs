//! Structured logging. Installed once at start-up; a second installation is
//! not an error worth aborting a boot over, so it is ignored deliberately.

/// Install the tracing subscriber from `RUST_LOG`, defaulting to `info`.
pub fn init() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .try_init();
}
