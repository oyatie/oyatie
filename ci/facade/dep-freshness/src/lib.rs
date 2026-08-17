//! Hermetic advisory gate over the committed dependency-freshness mirror (oyatie-gr1n).
//!
//! Reads ONLY committed data — the mirror, its manifest, and the `[freshness]` policy in
//! `oya-deps.toml`. No network, and no clock: the as-of date is the mirror's own `snapshot_date`.
//! That makes the verdict a pure function of committed bytes, which is what lets it be cached and
//! reproduced. It also means a stale mirror yields stale answers, honestly and visibly, rather than
//! drifting silently against wall-clock time.
//!
//! ADVISORY BY CONTRACT. `oya-deps.toml` declares `enforcement = "advisory"`, and this gate refuses
//! to run in any other mode (see [`Policy::from_toml`]). The reason is structural, not timid:
//! staleness is TIME-driven — it advances with the calendar with no repository change — so a
//! blocking form would red the required context on whichever unrelated PR is open that day. The
//! narrow blocking case worth having is CHANGE-driven (a diff that ADDS or bumps TO a stale
//! dependency) and is deliberately not implemented here, because it needs diff context this gate
//! does not have.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// The pure distiller and signal kernel.
///
/// Kept INSIDE this crate rather than as a separate `ci/core/*` crate. Every other gate in
/// `ci/facade` is a single crate, and a facade depending directly on its own capability's core
/// bypasses ports — which the `facade_core_direct_dep` rule correctly rejected. One crate is also
/// simply less to register: it removes a catalog row, a BUCK file, an OWNERS file and a lock entry.
pub mod kernel;

use kernel::{
    CrateRelease, DeclaredDependency, FRESHNESS_SCHEMA, Waivers, canonical_hash, evaluate,
};

/// The `[freshness]` policy, read as DATA from `oya-deps.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub stale_after_days: i64,
    pub enforcement: String,
    pub mirror: String,
    pub manifest: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PolicyError {
    Unparseable(String),
    MissingKey(&'static str),
    /// The policy declared an enforcement mode this gate will not implement.
    UnsupportedEnforcement(String),
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unparseable(e) => write!(f, "oya-deps.toml is not parseable TOML: {e}"),
            Self::MissingKey(k) => write!(f, "oya-deps.toml [freshness] is missing {k}"),
            Self::UnsupportedEnforcement(mode) => write!(
                f,
                "oya-deps.toml declares [freshness] enforcement = {mode:?}, but this gate only \
                 implements \"advisory\". Refusing to guess: a gate that silently downgrades a \
                 blocking policy to advisory reports success it was never authorised to report."
            ),
        }
    }
}

impl Policy {
    /// Parse `[freshness]` out of `oya-deps.toml`.
    ///
    /// Every key is required. A missing threshold is an error rather than a default, because the
    /// obvious default — a large window — silently reports nothing and is indistinguishable from a
    /// clean corpus.
    pub fn from_toml(text: &str) -> Result<Self, PolicyError> {
        let doc: toml::Value =
            toml::from_str(text).map_err(|e| PolicyError::Unparseable(e.to_string()))?;
        let table = doc
            .get("freshness")
            .ok_or(PolicyError::MissingKey("[freshness]"))?;
        let string = |key: &'static str| -> Result<String, PolicyError> {
            table
                .get(key)
                .and_then(toml::Value::as_str)
                .map(str::to_string)
                .ok_or(PolicyError::MissingKey(key))
        };
        let stale_after_days = table
            .get("stale_after_days")
            .and_then(toml::Value::as_integer)
            .ok_or(PolicyError::MissingKey("stale_after_days"))?;
        let enforcement = string("enforcement")?;
        if enforcement != "advisory" {
            return Err(PolicyError::UnsupportedEnforcement(enforcement));
        }
        Ok(Self {
            stale_after_days,
            enforcement,
            mirror: string("mirror")?,
            manifest: string("manifest")?,
        })
    }
}

/// The mirror manifest's `snapshot_date`, which is this gate's as-of date.
pub fn snapshot_date(manifest_json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(manifest_json)
        .ok()?
        .get("source")?
        .get("snapshot_date")?
        .as_str()
        .map(str::to_string)
}

/// The manifest fields that describe the mirror, so the mirror can be checked against them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub schema: String,
    pub snapshot_date: String,
    pub content_hash: String,
    pub crate_count: usize,
}

/// Parse the mirror manifest. Every field is required — a manifest missing the fields that make
/// verification possible is not a weaker manifest, it is an unusable one.
pub fn manifest(manifest_json: &str) -> Result<Manifest, String> {
    let doc: serde_json::Value =
        serde_json::from_str(manifest_json).map_err(|e| format!("manifest is not JSON: {e}"))?;
    let string = |ptr: &str| -> Result<String, String> {
        doc.pointer(ptr)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("manifest is missing {ptr}"))
    };
    Ok(Manifest {
        schema: string("/schema")?,
        snapshot_date: string("/source/snapshot_date")?,
        content_hash: string("/content_hash")?,
        crate_count: usize::try_from(
            doc.pointer("/crate_count")
                .and_then(serde_json::Value::as_u64)
                .ok_or("manifest is missing /crate_count")?,
        )
        .map_err(|e| format!("crate_count is not a usize: {e}"))?,
    })
}

/// The mirror must be the artifact its manifest describes.
///
/// Without this, the gate reads only `snapshot_date` and ignores everything else the producer
/// recorded. Replacing `freshness.json` with `[]` while leaving the manifest untouched would then
/// report a perfectly clean corpus — a silent fail-open, and the exact failure mode this pipeline
/// exists to prevent. Schema, count and content hash are all checked, so truncation, partial
/// regeneration, and hand-editing are each caught.
pub fn verify(releases: &[CrateRelease], manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != FRESHNESS_SCHEMA {
        return Err(format!(
            "mirror schema mismatch: manifest says {:?}, this gate implements {FRESHNESS_SCHEMA:?}",
            manifest.schema
        ));
    }
    if releases.len() != manifest.crate_count {
        return Err(format!(
            "mirror holds {} crates but its manifest claims {}; the two were not produced together",
            releases.len(),
            manifest.crate_count
        ));
    }
    let actual = canonical_hash(releases);
    if actual != manifest.content_hash {
        return Err(format!(
            "mirror content hash {actual} does not match the manifest's {}; freshness.json has \
             been edited or regenerated separately from its manifest",
            manifest.content_hash
        ));
    }
    Ok(())
}

/// Parse the committed mirror.
pub fn mirror(freshness_json: &str) -> Result<Vec<CrateRelease>, String> {
    serde_json::from_str(freshness_json)
        .map_err(|e| format!("freshness.json is not parseable: {e}"))
}

/// One reported line: a stale dependency with the days it has been quiet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleEntry {
    pub name: String,
    pub latest_stable: String,
    pub last_release_date: String,
    pub days_since_release: i64,
    pub owner_team: Option<String>,
}

/// Evaluate the mirror against the policy, newest-quiet first.
///
/// The mirror IS the corpus: it is produced from the workspace's direct dependencies, so every
/// entry is something this repository actually depends on. Each entry is fed to the kernel as
/// declared-at-its-own-latest, which isolates the STALE signal — BEHIND cannot fire, by
/// construction, and that separation is the point (`serde_yaml` is stale and NOT behind).
#[must_use]
pub fn stale_entries(
    mirror: &[CrateRelease],
    policy: &Policy,
    as_of: &str,
    owners: &BTreeMap<String, String>,
) -> Vec<StaleEntry> {
    let declared: Vec<DeclaredDependency> = mirror
        .iter()
        .map(|release| DeclaredDependency {
            name: release.name.clone(),
            version: release.latest_stable.clone(),
        })
        .collect();
    let by_name: BTreeMap<&str, &CrateRelease> =
        mirror.iter().map(|r| (r.name.as_str(), r)).collect();

    let mut entries: Vec<StaleEntry> = evaluate(
        mirror,
        &declared,
        policy.stale_after_days,
        as_of,
        owners,
        &Waivers::new(),
    )
    .into_iter()
    .filter_map(|finding| match finding.signal {
        kernel::Signal::Stale {
            last_release_date,
            days_since_release,
        } => {
            let release = by_name.get(finding.name.as_str())?;
            Some(StaleEntry {
                name: finding.name,
                latest_stable: release.latest_stable.clone(),
                last_release_date,
                days_since_release,
                owner_team: finding.owner_team,
            })
        }
        _ => None,
    })
    .collect();
    entries.sort_by(|a, b| {
        b.days_since_release
            .cmp(&a.days_since_release)
            .then_with(|| a.name.cmp(&b.name))
    });
    entries
}

/// `dep_name -> owner_team` from `specs/oss-stewardship-registry.json`.
///
/// Enrichment only, never a filter. Measured on the 2026-08-17 corpus, that registry covers 2 of 75
/// direct crate dependencies (it enumerates platform components — distros, `postgresql`, `istio` —
/// rather than crates), so treating an absent owner as a signal would flag 26 of 27 stale entries
/// and collapse straight back into plain staleness. Tracked as oyatie-g5k3.
#[must_use]
pub fn owner_index(registry_json: &str) -> BTreeMap<String, String> {
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(registry_json) else {
        return BTreeMap::new();
    };
    doc.get("entries")
        .and_then(serde_json::Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    Some((
                        entry.get("dep_name")?.as_str()?.to_string(),
                        entry.get("owner_team")?.as_str()?.to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: &str = r#"
[freshness]
mirror = "m.json"
manifest = "mm.json"
stale_after_days = 90
enforcement = "advisory"
"#;

    #[test]
    fn policy_reads_the_threshold_as_data() {
        let policy = Policy::from_toml(POLICY).expect("parses");
        assert_eq!(policy.stale_after_days, 90);
        assert_eq!(policy.enforcement, "advisory");
    }

    #[test]
    fn a_blocking_policy_is_refused_rather_than_silently_downgraded() {
        let blocking = POLICY.replace("\"advisory\"", "\"blocking\"");
        assert_eq!(
            Policy::from_toml(&blocking),
            Err(PolicyError::UnsupportedEnforcement("blocking".into()))
        );
    }

    #[test]
    fn a_missing_threshold_is_an_error_not_a_default() {
        let no_threshold = POLICY.replace("stale_after_days = 90", "");
        assert_eq!(
            Policy::from_toml(&no_threshold),
            Err(PolicyError::MissingKey("stale_after_days"))
        );
    }

    #[test]
    fn stale_entries_are_ordered_quietest_first_and_carry_no_behind_signal() {
        let mirror = vec![
            CrateRelease {
                name: "old".into(),
                latest_stable: "1.0.0".into(),
                last_release_date: "2020-01-01".into(),
            },
            CrateRelease {
                name: "older".into(),
                latest_stable: "1.0.0".into(),
                last_release_date: "2018-01-01".into(),
            },
            CrateRelease {
                name: "fresh".into(),
                latest_stable: "1.0.0".into(),
                last_release_date: "2026-08-01".into(),
            },
        ];
        let policy = Policy::from_toml(POLICY).expect("parses");
        let got = stale_entries(&mirror, &policy, "2026-08-17", &BTreeMap::new());
        assert_eq!(
            got.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            ["older", "old"],
            "quietest first, and `fresh` must not appear"
        );
    }

    #[test]
    fn owner_index_is_enrichment_and_survives_a_registry_it_cannot_parse() {
        assert!(owner_index("{ not json").is_empty());
        let index = owner_index(r#"{"entries":[{"dep_name":"a","owner_team":"axis-x"}]}"#);
        assert_eq!(index.get("a").map(String::as_str), Some("axis-x"));
    }

    fn one_release() -> Vec<CrateRelease> {
        vec![CrateRelease {
            name: "c".into(),
            latest_stable: "1.0.0".into(),
            last_release_date: "2020-01-01".into(),
        }]
    }

    fn good_manifest(releases: &[CrateRelease]) -> Manifest {
        Manifest {
            schema: FRESHNESS_SCHEMA.to_string(),
            snapshot_date: "2026-08-17".into(),
            content_hash: canonical_hash(releases),
            crate_count: releases.len(),
        }
    }

    #[test]
    fn a_truncated_mirror_is_caught_rather_than_read_as_a_clean_corpus() {
        // The fail-open this exists to stop: replace freshness.json with [] and leave the manifest
        // alone. Without verification the gate reports zero stale dependencies and looks green.
        let m = good_manifest(&one_release());
        let err = verify(&[], &m).expect_err("an empty mirror must not verify");
        assert!(err.contains("manifest claims"), "{err}");
    }

    #[test]
    fn a_hand_edited_mirror_is_caught_by_the_content_hash() {
        let mut edited = one_release();
        let m = good_manifest(&edited);
        edited[0].last_release_date = "2026-08-01".into(); // same count, different bytes
        let err = verify(&edited, &m).expect_err("edited content must not verify");
        assert!(err.contains("content hash"), "{err}");
    }

    #[test]
    fn a_manifest_from_another_schema_is_refused() {
        let releases = one_release();
        let mut m = good_manifest(&releases);
        m.schema = "oya-dep-freshness/v99".into();
        assert!(verify(&releases, &m).is_err());
    }

    #[test]
    fn a_matching_pair_verifies() {
        let releases = one_release();
        assert!(verify(&releases, &good_manifest(&releases)).is_ok());
    }

    #[test]
    fn a_manifest_missing_a_verification_field_is_unusable_not_merely_weaker() {
        assert!(manifest(r#"{"schema":"x","source":{"snapshot_date":"2026-08-17"}}"#).is_err());
        assert!(manifest("{ not json").is_err());
    }

    #[test]
    fn snapshot_date_comes_from_the_manifest_so_the_gate_needs_no_clock() {
        let manifest = r#"{"source":{"snapshot_date":"2026-08-17"}}"#;
        assert_eq!(snapshot_date(manifest).as_deref(), Some("2026-08-17"));
        assert_eq!(snapshot_date("{}"), None);
    }
}
