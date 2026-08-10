//! # port-engine-rust-ir — Rust IR stub and renderer placeholder (W0-B Slice 3).
//!
//! ADR-0637 D1 core face: holds `TargetIr` rendering with stable ordering and normalized
//! formatting. Slice 3 lands the adapter stub and deterministic empty renderer; syn/quote path
//! and leakage-forbidden tests land in Slice 5.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use port_engine_api::{Digest, PortError, RegionId, Renderer, TargetIr};

/// Fail-closed readiness gate. `true` once Slice 3 adapter stub is wired.
pub const fn w0_ready() -> bool {
    true
}

/// Minimal in-memory Rust IR stub for driver wiring tests (no syn/quote yet).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustIr {
    regions: Vec<RegionId>,
}

impl RustIr {
    /// Construct an IR declaring `region_ids` in deterministic order.
    #[must_use]
    pub fn new(region_ids: &[&str]) -> Self {
        Self {
            regions: region_ids
                .iter()
                .map(|id| RegionId((*id).to_owned()))
                .collect(),
        }
    }
}

impl TargetIr for RustIr {
    fn target_language(&self) -> &str {
        "rust"
    }

    fn regions(&self) -> Vec<RegionId> {
        self.regions.clone()
    }
}

/// Deterministic empty renderer: emits zero-byte blobs for every declared region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmptyRenderer {
    formatter_digest: Digest,
}

impl EmptyRenderer {
    /// Renderer with a fixed formatter digest for receipt wiring tests.
    #[must_use]
    pub fn new(formatter_digest: impl Into<String>) -> Self {
        Self {
            formatter_digest: Digest(formatter_digest.into()),
        }
    }
}

impl Renderer for EmptyRenderer {
    fn target_language(&self) -> &str {
        "rust"
    }

    fn formatter_digest(&self) -> Digest {
        self.formatter_digest.clone()
    }

    fn render(&self, ir: &dyn TargetIr) -> Result<BTreeMap<RegionId, Vec<u8>>, PortError> {
        let mut out = BTreeMap::new();
        for region in ir.regions() {
            out.insert(region, Vec::new());
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_renderer_matches_declared_regions() {
        let ir = RustIr::new(&["root"]);
        let renderer = EmptyRenderer::new("fmt-stub-v0");
        let out = renderer.render(&ir).expect("empty stub must succeed");
        assert_eq!(out.len(), 1);
        assert!(out.contains_key(&RegionId("root".to_owned())));
        assert!(out.get(&RegionId("root".to_owned())).unwrap().is_empty());
    }
}
