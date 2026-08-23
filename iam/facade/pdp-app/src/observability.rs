//! Tracing pipeline bootstrap (structured JSON logs; K8s-native stdout).

/// Install the global tracing subscriber (idempotent: a second call is a
/// no-op so tests can race it safely — the identity precedent).
pub fn init() {
    let _ = tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();
}
