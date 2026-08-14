//! # port-engine-frontend-go — Go SourceModel snapshot consumer (W0-B Slice 4).
//!
//! ADR-0638 D3 snapshot firewall: this adapter consumes **SourceModel snapshot bytes only** and
//! must never invoke a Go toolchain in-process or from the `verify()` path. Slice 4 lands decode
//! of the canonical snapshot envelope plus an architecture test that refuses spawning the `go`
//! binary from library sources used by verify.
#![forbid(unsafe_code)]

use std::fmt;

use port_engine_api::{Digest, UnitId};
use serde::Deserialize;

/// Canonical bootstrap extractor identity (ADR-0638 D3).
pub const PRODUCER_BOOTSTRAP_GO: &str = "bootstrap-go-packages-go-types";

/// Owned Rust front-end producer identity (authorized only after W2 equivalence).
pub const PRODUCER_OWNED_RUST: &str = "owned-rust-go-front-end";

/// Fail-closed readiness gate. `true` once Slice 4 snapshot decode is present.
pub const fn w0_ready() -> bool {
    true
}

/// Typed refusal from snapshot decode / producer validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    /// JSON could not be parsed.
    Parse {
        /// Parser detail (no path — adapter receives bytes only).
        detail: String,
    },
    /// Required field missing or wrong type / empty.
    Schema {
        /// Which field failed.
        field: &'static str,
    },
    /// Package producer is not one of the ADR-0638 canonical identities.
    UnknownProducer {
        /// Producer string found on a package.
        actual: String,
    },
    /// Duplicate `unit_id` — non-deterministic model shape.
    DuplicateUnit {
        /// The repeated unit id.
        unit_id: String,
    },
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { detail } => {
                write!(f, "source-model snapshot JSON parse failed: {detail}")
            }
            Self::Schema { field } => {
                write!(
                    f,
                    "source-model snapshot schema missing or invalid: {field}"
                )
            }
            Self::UnknownProducer { actual } => write!(
                f,
                "source-model snapshot package producer must be `{PRODUCER_BOOTSTRAP_GO}` or `{PRODUCER_OWNED_RUST}`, got `{actual}`"
            ),
            Self::DuplicateUnit { unit_id } => {
                write!(f, "source-model snapshot has duplicate unit_id `{unit_id}`")
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

#[derive(Deserialize)]
struct SnapshotDocument {
    language: String,
    snapshot_digest: String,
    packages: Vec<PackageEntry>,
}

#[derive(Deserialize)]
struct PackageEntry {
    unit_id: String,
    producer: String,
}

/// Decoded Go SourceModel snapshot (identity + order only; no Go toolchain).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceModel {
    language: String,
    snapshot_digest: Digest,
    units: Vec<UnitId>,
    /// Parallel to [`Self::units`] — ADR-0638 D3 package→producer map (one producer per package).
    producers: Vec<String>,
}

impl GoSourceModel {
    /// Decode snapshot JSON bytes into an unadmitted Go model.
    ///
    /// # Errors
    /// [`SnapshotError`] on parse failure, schema violation, unknown producer, or duplicate unit.
    pub fn decode(bytes: &[u8]) -> Result<Self, SnapshotError> {
        let text = std::str::from_utf8(bytes).map_err(|err| SnapshotError::Parse {
            detail: format!("utf-8: {err}"),
        })?;
        Self::decode_str(text)
    }

    /// Decode snapshot JSON from an in-memory string (test hook and future adapter input).
    ///
    /// # Errors
    /// [`SnapshotError`] on parse failure, schema violation, unknown producer, or duplicate unit.
    pub fn decode_str(json: &str) -> Result<Self, SnapshotError> {
        let doc: SnapshotDocument =
            serde_json::from_str(json).map_err(|err| SnapshotError::Parse {
                detail: err.to_string(),
            })?;
        if doc.language.is_empty() {
            return Err(SnapshotError::Schema { field: "language" });
        }
        if doc.snapshot_digest.is_empty() {
            return Err(SnapshotError::Schema {
                field: "snapshot_digest",
            });
        }
        let mut units = Vec::with_capacity(doc.packages.len());
        let mut producers = Vec::with_capacity(doc.packages.len());
        let mut seen = std::collections::BTreeSet::new();
        for pkg in doc.packages {
            if pkg.unit_id.is_empty() {
                return Err(SnapshotError::Schema {
                    field: "packages.unit_id",
                });
            }
            if pkg.producer != PRODUCER_BOOTSTRAP_GO && pkg.producer != PRODUCER_OWNED_RUST {
                return Err(SnapshotError::UnknownProducer {
                    actual: pkg.producer,
                });
            }
            if !seen.insert(pkg.unit_id.clone()) {
                return Err(SnapshotError::DuplicateUnit {
                    unit_id: pkg.unit_id,
                });
            }
            units.push(UnitId(pkg.unit_id));
            producers.push(pkg.producer);
        }
        Ok(Self {
            language: doc.language,
            snapshot_digest: Digest(doc.snapshot_digest),
            units,
            producers,
        })
    }

    /// Source-language slug claimed by the decoded snapshot.
    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Semantic digest claimed by the decoded snapshot.
    #[must_use]
    pub fn snapshot_digest(&self) -> Digest {
        self.snapshot_digest.clone()
    }

    /// Units in decoded snapshot order.
    #[must_use]
    pub fn units(&self) -> Vec<UnitId> {
        self.units.clone()
    }

    /// Producer identity for `unit`, if present in the snapshot map.
    #[must_use]
    pub fn producer_for(&self, unit: &UnitId) -> Option<&str> {
        self.units
            .iter()
            .position(|u| u == unit)
            .map(|idx| producers_at(self, idx))
    }
}

fn producers_at(model: &GoSourceModel, idx: usize) -> &str {
    &model.producers[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
  "language": "go",
  "snapshot_digest": "sha256:fixture-slice4",
  "packages": [
    {"unit_id": "example.com/a", "producer": "bootstrap-go-packages-go-types"},
    {"unit_id": "example.com/b", "producer": "bootstrap-go-packages-go-types"}
  ]
}"#;

    #[test]
    fn slice4_claims_readiness() {
        assert!(w0_ready());
    }

    #[test]
    fn decodes_ordered_units_and_producers() {
        let model = GoSourceModel::decode_str(FIXTURE).expect("fixture must decode");
        assert_eq!(model.language(), "go");
        assert_eq!(
            model.snapshot_digest(),
            Digest("sha256:fixture-slice4".into())
        );
        assert_eq!(
            model.units(),
            vec![
                UnitId("example.com/a".into()),
                UnitId("example.com/b".into())
            ]
        );
        assert_eq!(
            model.producer_for(&UnitId("example.com/a".into())),
            Some(PRODUCER_BOOTSTRAP_GO)
        );
    }

    #[test]
    fn refuses_unknown_producer() {
        let json = r#"{"language":"go","snapshot_digest":"d","packages":[{"unit_id":"x","producer":"gccgo"}]}"#;
        let err = GoSourceModel::decode_str(json).expect_err("unknown producer must refuse");
        assert!(matches!(err, SnapshotError::UnknownProducer { .. }));
    }

    #[test]
    fn refuses_duplicate_unit() {
        let json = r#"{"language":"go","snapshot_digest":"d","packages":[{"unit_id":"x","producer":"bootstrap-go-packages-go-types"},{"unit_id":"x","producer":"bootstrap-go-packages-go-types"}]}"#;
        let err = GoSourceModel::decode_str(json).expect_err("duplicate unit must refuse");
        assert!(matches!(err, SnapshotError::DuplicateUnit { .. }));
    }

    /// ADR-0638 D3 architecture fence: library sources used by verify must not spawn `go`.
    #[test]
    fn library_source_never_spawns_go_command() {
        let src = include_str!("lib.rs");
        // Build needles without embedding the forbidden call site as a contiguous literal in
        // production code paths (this test body is the only place that may mention the pattern).
        let cmd_new = ["Command", "::", "new"].concat();
        let go_lit = ["\"", "go", "\""].concat();
        let forbidden_call = format!("{cmd_new}({go_lit})");
        let process_cmd = ["std", "::", "process", "::", "Command"].concat();
        // Strip this #[cfg(test)] module so the assertion text does not self-fail.
        let production = src
            .split("#[cfg(test)]")
            .next()
            .expect("lib.rs must have a production section");
        assert!(
            !production.contains(&forbidden_call),
            "port-engine-frontend-go production sources must not invoke Go via {forbidden_call}"
        );
        assert!(
            !production.contains(&process_cmd),
            "port-engine-frontend-go production sources must not import {process_cmd} (Go firewall)"
        );
    }
}
