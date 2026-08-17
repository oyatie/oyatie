//! # port-engine-emit — single-fixture canary emit seam (W0-B Slice 13/14).
//!
//! Selects **one** canary region from a rendered emit tree, compares it to a hermetic golden,
//! and optionally materializes that single file under an allowlisted canary-out directory.
//! Slice 14 adds read-back round-trip after materialize.
//!
//! Hard stops (W0-B / ADR-0704):
//! - NEVER writes under `k8s/`
//! - NEVER bulk-emits a corpus (exactly one canary file)
//! - Destination directory basename MUST be `port-engine-canary-out`
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use port_engine_api::{Digest, RegionId};
use port_engine_hash::digest_bytes;

/// Fail-closed readiness gate. `true` once Slice 13 canary emit is present.
pub const fn w0_ready() -> bool {
    true
}

/// Required basename for a canary materialize root (envelope fence).
pub const CANARY_OUT_DIRNAME: &str = "port-engine-canary-out";

/// Filename written inside the canary-out directory.
pub const CANARY_FILENAME: &str = "canary.rs";

/// Region id suffix produced by `empty_canary` construction (Slice 11).
pub const CANARY_RULE_SUFFIX: &str = "__canary_empty_unit";

/// Embedded golden canary source (post syn/quote spelling of the mini fixture).
const GOLDEN_CANARY_RS: &str = include_str!("golden-canary-v0.txt");

/// Selected canary region + bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanaryArtifact {
    /// Region id selected from the emit tree.
    pub region: RegionId,
    /// Emitted source bytes for that region.
    pub bytes: Vec<u8>,
    /// Content digest of [`Self::bytes`].
    pub digest: Digest,
}

/// Typed refusal from canary emit / materialize.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmitError {
    /// No canary region present in the emit tree.
    MissingCanary,
    /// More than one canary region — bulk/ambiguous emit refused.
    AmbiguousCanary {
        /// How many canary-shaped regions were found.
        count: usize,
    },
    /// Bytes do not match the embedded golden.
    GoldenMismatch {
        /// Digest of the emitted canary.
        actual: String,
        /// Digest of the golden.
        expected: String,
        /// UTF-8 lossy spelling of emitted bytes (for golden authoring).
        actual_utf8: String,
    },
    /// Destination path escapes the canary-out allowlist.
    PathRefused {
        /// Why the path was refused.
        detail: String,
    },
    /// Filesystem IO failed.
    Io {
        /// OS detail.
        detail: String,
    },
}

impl fmt::Display for EmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCanary => {
                write!(f, "canary emit: no `{CANARY_RULE_SUFFIX}` region in tree")
            }
            Self::AmbiguousCanary { count } => write!(
                f,
                "canary emit: expected exactly one canary region, found {count}"
            ),
            Self::GoldenMismatch {
                actual,
                expected,
                actual_utf8,
            } => write!(
                f,
                "canary emit golden mismatch: actual `{actual}`, expected `{expected}`, bytes={actual_utf8:?}"
            ),
            Self::PathRefused { detail } => write!(f, "canary emit path refused: {detail}"),
            Self::Io { detail } => write!(f, "canary emit io failed: {detail}"),
        }
    }
}

impl std::error::Error for EmitError {}

/// Select the single canary region from a rendered emit tree.
///
/// # Errors
/// [`EmitError::MissingCanary`] or [`EmitError::AmbiguousCanary`].
pub fn select_canary(emitted: &BTreeMap<RegionId, Vec<u8>>) -> Result<CanaryArtifact, EmitError> {
    let mut matches: Vec<(&RegionId, &Vec<u8>)> = emitted
        .iter()
        .filter(|(id, _)| id.0.ends_with(CANARY_RULE_SUFFIX))
        .collect();
    match matches.len() {
        0 => Err(EmitError::MissingCanary),
        1 => {
            let (region, bytes) = matches.pop().expect("len checked");
            Ok(CanaryArtifact {
                region: region.clone(),
                digest: digest_bytes(bytes),
                bytes: bytes.clone(),
            })
        }
        n => Err(EmitError::AmbiguousCanary { count: n }),
    }
}

/// Golden canary source bytes (UTF-8).
#[must_use]
pub fn golden_canary_bytes() -> &'static [u8] {
    GOLDEN_CANARY_RS.as_bytes()
}

/// Digest of the embedded golden canary.
#[must_use]
pub fn golden_canary_digest() -> Digest {
    digest_bytes(golden_canary_bytes())
}

/// Fail closed unless `artifact` matches the embedded golden byte-for-byte.
///
/// # Errors
/// [`EmitError::GoldenMismatch`].
pub fn assert_matches_golden(artifact: &CanaryArtifact) -> Result<(), EmitError> {
    let expected = golden_canary_digest();
    if artifact.bytes != golden_canary_bytes() {
        return Err(EmitError::GoldenMismatch {
            actual: artifact.digest.0.clone(),
            expected: expected.0,
            actual_utf8: String::from_utf8_lossy(&artifact.bytes).into_owned(),
        });
    }
    Ok(())
}

/// Validate `out_dir` is allowlisted for single-file canary materialize.
///
/// # Errors
/// [`EmitError::PathRefused`] when basename is wrong, path has `..`, or points at `k8s`.
pub fn validate_canary_out_dir(out_dir: &Path) -> Result<(), EmitError> {
    let raw = out_dir.to_string_lossy();
    if raw.contains("..") {
        return Err(EmitError::PathRefused {
            detail: "path must not contain `..`".into(),
        });
    }
    // Refuse any k8s corpus path component (W0-B bulk emit hard stop).
    for component in out_dir.components() {
        if let std::path::Component::Normal(name) = component
            && name == "k8s"
        {
            return Err(EmitError::PathRefused {
                detail: "refusing materialize under `k8s/` (bulk corpus forbidden in W0-B)".into(),
            });
        }
    }
    let Some(base) = out_dir.file_name().and_then(|s| s.to_str()) else {
        return Err(EmitError::PathRefused {
            detail: "canary-out path missing basename".into(),
        });
    };
    if base != CANARY_OUT_DIRNAME {
        return Err(EmitError::PathRefused {
            detail: format!("basename must be `{CANARY_OUT_DIRNAME}`, got `{base}`"),
        });
    }
    Ok(())
}

/// Materialize the single canary file under `out_dir` (create dir if needed).
///
/// # Errors
/// Path refusal, IO failure.
pub fn materialize_canary(out_dir: &Path, artifact: &CanaryArtifact) -> Result<PathBuf, EmitError> {
    validate_canary_out_dir(out_dir)?;
    assert_matches_golden(artifact)?;
    fs::create_dir_all(out_dir).map_err(|err| EmitError::Io {
        detail: err.to_string(),
    })?;
    let dest = out_dir.join(CANARY_FILENAME);
    fs::write(&dest, &artifact.bytes).map_err(|err| EmitError::Io {
        detail: err.to_string(),
    })?;
    Ok(dest)
}

/// Select canary, check golden, return artifact (no filesystem write).
///
/// # Errors
/// Selection or golden mismatch.
pub fn emit_canary_checked(
    emitted: &BTreeMap<RegionId, Vec<u8>>,
) -> Result<CanaryArtifact, EmitError> {
    let artifact = select_canary(emitted)?;
    assert_matches_golden(&artifact)?;
    Ok(artifact)
}

/// Materialize the canary then read it back; refuse on byte drift.
///
/// # Errors
/// Path refusal, IO failure, or round-trip mismatch.
pub fn materialize_canary_roundtrip(
    out_dir: &Path,
    artifact: &CanaryArtifact,
) -> Result<PathBuf, EmitError> {
    let dest = materialize_canary(out_dir, artifact)?;
    let written = fs::read(&dest).map_err(|err| EmitError::Io {
        detail: err.to_string(),
    })?;
    if written != artifact.bytes {
        return Err(EmitError::Io {
            detail: format!(
                "canary round-trip mismatch at {}: wrote {} bytes, read {}",
                dest.display(),
                artifact.bytes.len(),
                written.len()
            ),
        });
    }
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn slice13_claims_emit_readiness() {
        assert!(w0_ready());
    }

    #[test]
    fn selects_exactly_one_canary_region() {
        let mut tree = BTreeMap::new();
        tree.insert(
            RegionId("example_com_a__identity".into()),
            b"pub fn example_com_a__identity() {}".to_vec(),
        );
        tree.insert(
            RegionId("example_com_b__canary_empty_unit".into()),
            golden_canary_bytes().to_vec(),
        );
        let art = select_canary(&tree).expect("one canary");
        assert_eq!(art.region.0, "example_com_b__canary_empty_unit");
        assert_matches_golden(&art).expect("golden");
    }

    #[test]
    fn refuses_missing_and_ambiguous() {
        let empty = BTreeMap::new();
        assert!(matches!(
            select_canary(&empty),
            Err(EmitError::MissingCanary)
        ));
        let mut two = BTreeMap::new();
        two.insert(RegionId("a__canary_empty_unit".into()), b"a".to_vec());
        two.insert(RegionId("b__canary_empty_unit".into()), b"b".to_vec());
        assert!(matches!(
            select_canary(&two),
            Err(EmitError::AmbiguousCanary { count: 2 })
        ));
    }

    #[test]
    fn refuses_k8s_destination() {
        let err = validate_canary_out_dir(Path::new("/tmp/k8s/port-engine-canary-out"))
            .expect_err("k8s path");
        assert!(matches!(err, EmitError::PathRefused { .. }));
    }

    #[test]
    fn refuses_wrong_basename() {
        let err = validate_canary_out_dir(Path::new("/tmp/not-canary-out")).expect_err("basename");
        assert!(matches!(err, EmitError::PathRefused { .. }));
    }

    #[test]
    fn materialize_writes_single_file_under_allowlisted_dir() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir()
            .join(format!("pe-canary-{nanos}"))
            .join(CANARY_OUT_DIRNAME);
        let artifact = CanaryArtifact {
            region: RegionId("example_com_b__canary_empty_unit".into()),
            bytes: golden_canary_bytes().to_vec(),
            digest: golden_canary_digest(),
        };
        let dest = materialize_canary_roundtrip(&root, &artifact).expect("materialize+roundtrip");
        assert_eq!(
            dest.file_name().and_then(|s| s.to_str()),
            Some(CANARY_FILENAME)
        );
        let written = fs::read(&dest).expect("read back");
        assert_eq!(written, golden_canary_bytes());
        let _ = fs::remove_dir_all(root.parent().expect("parent"));
    }

    /// ADR-0704 / W0-B fence: production emit must refuse `k8s` path components (bulk corpus).
    #[test]
    fn production_source_documents_k8s_refuse() {
        let src = include_str!("lib.rs");
        let production = src
            .split("#[cfg(test)]")
            .next()
            .expect("lib.rs must have a production section");
        assert!(
            production.contains("refusing materialize under"),
            "emit adapter must keep an explicit k8s materialize refuse path"
        );
        // Never construct a default destination under the corpus root.
        let bad = ["\"", "k8s", "/", "\""].concat();
        assert!(
            !production.contains(&bad),
            "emit adapter must not hard-code a k8s/ destination string"
        );
    }
}
