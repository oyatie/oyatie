#![forbid(unsafe_code)]
//! Realtime collaboration WebSocket/SSE gateway scaffold.
//!
//! This is a first-execution deployable scaffold for the Oya Office public-SaaS
//! office suite. Runtime wiring, dependencies, and service contracts are added
//! only through later verified Ultragoal stories.

/// Stable application identifier used by workspace and Buck2 scaffold verification.
pub const APP_NAME: &str = "oya-office-collab-gateway-app";

/// Product vertical slice owned by this deployable.
pub const VERTICAL_SLICE: &str = "collab";

/// Source-shaped deployable layer represented by this scaffold.
pub const DEPLOYABLE_LAYER: &str = "gateway";

/// Starts the scaffolded application entrypoint.
///
/// Later stories replace this no-op with real Rust runtime wiring while keeping
/// the app horizontally scalable and free of global mutable singleton state.
pub fn run() {}

#[cfg(test)]
mod tests {
    use super::{APP_NAME, DEPLOYABLE_LAYER, VERTICAL_SLICE};

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!APP_NAME.is_empty());
        assert!(!DEPLOYABLE_LAYER.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }
}
