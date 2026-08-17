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

use oya_dep_freshness_kernel::{CrateRelease, DeclaredDependency, Waivers, evaluate};

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
        oya_dep_freshness_kernel::Signal::Stale {
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

    #[test]
    fn snapshot_date_comes_from_the_manifest_so_the_gate_needs_no_clock() {
        let manifest = r#"{"source":{"snapshot_date":"2026-08-17"}}"#;
        assert_eq!(snapshot_date(manifest).as_deref(), Some("2026-08-17"));
        assert_eq!(snapshot_date("{}"), None);
    }
}
