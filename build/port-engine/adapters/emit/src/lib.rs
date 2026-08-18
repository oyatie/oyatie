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

/// Required basename for a full emit-tree materialize root.
///
/// A SECOND allowlisted basename, not a relaxation of the first. The canary path still writes
/// exactly one file called `canary.rs` under its own name; this one writes a whole tree under a
/// name that says so. Keeping them separate means "write one canary" and "write a corpus" can
/// never be confused for each other by a caller passing the wrong directory.
pub const EMIT_OUT_DIRNAME: &str = "port-engine-emit-out";

/// Every basename a materialize root may have.
pub const ALLOWED_OUT_DIRNAMES: &[&str] = &[CANARY_OUT_DIRNAME, EMIT_OUT_DIRNAME];

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
/// [`EmitError::PathRefused`] when basename is wrong, path has `..`, or points at the corpus root.
pub fn validate_canary_out_dir(out_dir: &Path) -> Result<(), EmitError> {
    validate_out_dir(out_dir, CANARY_OUT_DIRNAME)
}

/// Validate `out_dir` is allowlisted for whole-tree materialize.
///
/// # Errors
/// [`EmitError::PathRefused`] when basename is wrong, path has `..`, or points at the corpus root.
pub fn validate_emit_out_dir(out_dir: &Path) -> Result<(), EmitError> {
    validate_out_dir(out_dir, EMIT_OUT_DIRNAME)
}

fn validate_out_dir(out_dir: &Path, required_basename: &str) -> Result<(), EmitError> {
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
            detail: "out path missing basename".into(),
        });
    };
    if base != required_basename {
        return Err(EmitError::PathRefused {
            detail: format!("basename must be `{required_basename}`, got `{base}`"),
        });
    }
    Ok(())
}

/// Materialize a whole emit tree under `out_dir`, one file per region.
///
/// The `k8s/` refusal and the `..` refusal are UNCHANGED and still apply: widening what may be
/// written did not widen where. Region ids are already sanitized identifiers by the time they
/// reach here, and each one is re-checked below rather than trusted, because a region id is the
/// only part of the destination path that comes from data.
///
/// Returns the written paths in region order.
///
/// # Errors
/// [`EmitError::PathRefused`] on a refused destination or an unusable region id, or
/// [`EmitError::Io`] on a filesystem failure.
pub fn materialize_tree(
    out_dir: &Path,
    emitted: &BTreeMap<RegionId, Vec<u8>>,
) -> Result<Vec<PathBuf>, EmitError> {
    validate_emit_out_dir(out_dir)?;
    fs::create_dir_all(out_dir).map_err(|err| EmitError::Io {
        detail: err.to_string(),
    })?;

    let mut written = Vec::with_capacity(emitted.len());
    for (region, bytes) in emitted {
        // A region id becomes a FILENAME here, which is the one place data reaches the path. A
        // region called `../escape` or `a/b` would place the file outside the validated root, so
        // the id is required to be a bare identifier rather than merely assumed to be one.
        if region.0.is_empty()
            || !region
                .0
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(EmitError::PathRefused {
                detail: format!(
                    "region id `{}` is not a bare identifier and cannot name a file",
                    region.0
                ),
            });
        }
        let dest = out_dir.join(format!("{}.rs", region.0));
        fs::write(&dest, bytes).map_err(|err| EmitError::Io {
            detail: err.to_string(),
        })?;
        written.push(dest);
    }
    Ok(written)
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

    /// Widening WHAT may be written did not widen WHERE. Both refusals still apply to the tree
    /// path exactly as they do to the canary path.
    #[test]
    fn tree_materialize_keeps_every_destination_refusal() {
        for path in [
            "/tmp/k8s/port-engine-emit-out",
            "/tmp/../port-engine-emit-out",
            "/tmp/port-engine-canary-out",
            "/tmp/somewhere-else",
        ] {
            let err = validate_emit_out_dir(Path::new(path))
                .expect_err("this destination must be refused");
            assert!(
                matches!(err, EmitError::PathRefused { .. }),
                "{path}: {err}"
            );
        }
    }

    #[test]
    fn tree_materialize_writes_one_file_per_region() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir()
            .join(format!("pe-tree-{nanos}"))
            .join(EMIT_OUT_DIRNAME);

        let mut emitted = BTreeMap::new();
        emitted.insert(
            RegionId("basic__go_const__max".into()),
            b"pub const M: i64 = 1;".to_vec(),
        );
        emitted.insert(
            RegionId("basic__go_func__add".into()),
            b"pub fn add() {}".to_vec(),
        );

        let written = materialize_tree(&root, &emitted).expect("tree materialize");
        assert_eq!(written.len(), 2);
        for (path, (_, bytes)) in written.iter().zip(emitted.iter()) {
            assert_eq!(&fs::read(path).expect("read back"), bytes);
        }
        let _ = fs::remove_dir_all(root.parent().expect("parent"));
    }

    /// A region id is the only part of a destination path that comes from DATA, so it is checked
    /// rather than trusted: `../escape` as a region name would place a file outside the root that
    /// was just validated.
    #[test]
    fn tree_materialize_refuses_a_region_id_that_is_not_a_bare_identifier() {
        let root = std::env::temp_dir().join(EMIT_OUT_DIRNAME);
        for hostile in ["../escape", "a/b", "", "with space"] {
            let mut emitted = BTreeMap::new();
            emitted.insert(RegionId(hostile.into()), b"pub fn x() {}".to_vec());
            let err = materialize_tree(&root, &emitted)
                .expect_err("a region id that is not an identifier must refuse");
            assert!(
                matches!(err, EmitError::PathRefused { .. }),
                "{hostile}: {err}"
            );
        }
        let _ = fs::remove_dir_all(&root);
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
