//! Dependency freshness kernel.
//!
//! Pure, I/O-free distiller for crates.io sparse-index records into normalized [`CrateRelease`]
//! facts, plus the two independent freshness signals computed off them. The network-bearing
//! producer feeds raw index text in; the hermetic gate consumes the vendored snapshot out. Same
//! shape as `oya-advisory-mirror-kernel`, and for the same reason: the gate must stay
//! buck2-cacheable, deterministic, and network-free at gate time.
//!
//! WHY THIS EXISTS (oyatie-gr1n). `deny.toml` already sets `advisories.unmaintained =
//! "workspace"` and the advisory mirror models `informational: unmaintained` correctly — but
//! RustSec has never filed an unmaintained advisory for `serde_yaml`, and zero workspace-declared
//! dependencies carry one. That gate is not broken; it is blind by construction, because RustSec
//! coverage is voluntary and lagging. Time-since-last-release is a LEADING indicator that needs no
//! third party to file anything: `serde_yaml`'s newest release is `0.9.34+deprecated`, published
//! 2024-03-25, and nothing in the pipeline noticed.
//!
//! TWO SIGNALS, NEVER MERGED. [`Signal::Behind`] (a newer stable exists) and [`Signal::Stale`] (no
//! release within the window) are different failures with different remedies, so they carry
//! distinct codes and must not share a threshold or a waiver. A crate can be perfectly current and
//! still abandoned — `serde_yaml` is exactly that: it is NOT behind, because `0.9.34+deprecated`
//! IS the latest version. Any tool that only answers "is a newer version available?" reports it up
//! to date forever, which is precisely why Dependabot and Renovate would not have caught this.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const FRESHNESS_SCHEMA: &str = "oya-dep-freshness/v1";

/// The distilled release facts for one crate: its newest usable version and when that shipped.
///
/// "Usable" excludes yanked releases and pre-releases, because neither is something the bump-bot
/// may propose and neither is evidence that a crate is still maintained for consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateRelease {
    pub name: String,
    pub latest_stable: String,
    /// Calendar date (`YYYY-MM-DD`) of `latest_stable`, from the index record's `pubtime`.
    pub last_release_date: String,
}

/// A dependency as the workspace declares it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredDependency {
    pub name: String,
    pub version: String,
}

/// The two freshness signals. Deliberately separate — see the module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signal {
    /// A newer stable release exists than the one the workspace declares.
    Behind {
        declared: String,
        latest_stable: String,
    },
    /// The newest release is older than the staleness window.
    Stale {
        last_release_date: String,
        days_since_release: i64,
    },
    /// The mirror has no record for a declared dependency, so neither signal can be computed.
    /// Reported rather than skipped: a silent gap is indistinguishable from a clean result.
    Unknown,
}

impl Signal {
    /// Stable machine code. Distinct per signal so waivers cannot be written once and silence both.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Behind { .. } => "DEP-FRESHNESS-BEHIND",
            Self::Stale { .. } => "DEP-FRESHNESS-STALE",
            Self::Unknown => "DEP-FRESHNESS-UNKNOWN",
        }
    }
}

/// One reported dependency, with the signal and the owner accountable for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub name: String,
    pub signal: Signal,
    /// Owning team from `specs/oss-stewardship-registry.json`, when the dependency is registered.
    pub owner_team: Option<String>,
    /// True when a waiver covers this exact (dependency, code) pair.
    pub waived: bool,
}

/// Distill sparse-index records into normalized release facts.
///
/// Input is `(crate name, newline-delimited index JSON)` — the canonical crates.io sparse-index
/// file format, one JSON object per published version. Unparseable lines are skipped rather than
/// failing the distillation, because the index is an append-only upstream artifact this repository
/// does not control; a crate that yields NO usable version is simply absent from the output, and
/// [`evaluate`] reports that absence as [`Signal::Unknown`] rather than as silence.
#[must_use]
pub fn distill(index_files: &[(String, String)]) -> Vec<CrateRelease> {
    let mut out: Vec<CrateRelease> = index_files
        .iter()
        .filter_map(|(name, text)| newest_stable(name, text))
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn newest_stable(name: &str, text: &str) -> Option<CrateRelease> {
    let mut best: Option<(Version, String, String)> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(record) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if record.get("yanked").and_then(serde_json::Value::as_bool) == Some(true) {
            continue;
        }
        let Some(raw) = record.get("vers").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(version) = Version::parse(raw) else {
            continue;
        };
        let Some(pubtime) = record.get("pubtime").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let date = pubtime.split('T').next().unwrap_or(pubtime).to_string();
        if best.as_ref().is_none_or(|(seen, _, _)| version > *seen) {
            best = Some((version, raw.to_string(), date));
        }
    }
    best.map(|(_, raw, date)| CrateRelease {
        name: name.to_string(),
        latest_stable: raw,
        last_release_date: date,
    })
}

/// A comparable release version.
///
/// Pre-releases are rejected outright rather than ordered: the bump-bot may not propose one, and a
/// pre-release is not evidence of maintenance for consumers. Build metadata is ignored for
/// ordering, per semver — which is exactly why `0.9.34+deprecated` compares as plain `0.9.34` and
/// why a "is a newer version available?" check reports `serde_yaml` as current forever.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    fn parse(raw: &str) -> Option<Self> {
        let core = raw.split('+').next()?;
        if core.contains('-') {
            return None; // pre-release
        }
        let mut parts = core.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next().unwrap_or("0").parse().ok()?;
        let patch = parts.next().unwrap_or("0").parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

/// Waiver key: a dependency name paired with the exact signal code it excuses.
pub type Waivers = BTreeMap<(String, String), String>;

/// Evaluate declared dependencies against the mirror.
///
/// `as_of` is passed in rather than read from a clock so the gate is reproducible: the same inputs
/// always yield the same findings, which is what makes the result buck2-cacheable. `stale_after_days`
/// is likewise a parameter — the threshold lives in `oya-deps.toml` as data, never as a constant here.
///
/// Waived findings are RETAINED with `waived = true`, not dropped. A waiver records who accepted the
/// risk; deleting the finding would erase the accountability the waiver exists to create.
#[must_use]
pub fn evaluate(
    mirror: &[CrateRelease],
    declared: &[DeclaredDependency],
    stale_after_days: i64,
    as_of: &str,
    owners: &BTreeMap<String, String>,
    waivers: &Waivers,
) -> Vec<Finding> {
    let by_name: BTreeMap<&str, &CrateRelease> =
        mirror.iter().map(|r| (r.name.as_str(), r)).collect();
    let mut findings: Vec<Finding> = Vec::new();
    for dependency in declared {
        let signals = match by_name.get(dependency.name.as_str()) {
            None => vec![Signal::Unknown],
            Some(release) => {
                let mut signals = Vec::new();
                if let (Some(declared_version), Some(latest)) = (
                    Version::parse(&dependency.version),
                    Version::parse(&release.latest_stable),
                ) && latest > declared_version
                {
                    signals.push(Signal::Behind {
                        declared: dependency.version.clone(),
                        latest_stable: release.latest_stable.clone(),
                    });
                }
                if let Some(days) = days_between(&release.last_release_date, as_of)
                    && days > stale_after_days
                {
                    signals.push(Signal::Stale {
                        last_release_date: release.last_release_date.clone(),
                        days_since_release: days,
                    });
                }
                signals
            }
        };
        for signal in signals {
            let key = (dependency.name.clone(), signal.code().to_string());
            findings.push(Finding {
                name: dependency.name.clone(),
                waived: waivers.contains_key(&key),
                owner_team: owners.get(&dependency.name).cloned(),
                signal,
            });
        }
    }
    findings.sort_by(|a, b| {
        (a.name.as_str(), a.signal.code()).cmp(&(b.name.as_str(), b.signal.code()))
    });
    findings
}

/// Whole days from `from` to `to`, both `YYYY-MM-DD`. `None` if either fails to parse.
#[must_use]
pub fn days_between(from: &str, to: &str) -> Option<i64> {
    Some(days_from_civil(to)? - days_from_civil(from)?)
}

/// Days since the civil epoch, via Howard Hinnant's `days_from_civil`. Implemented here rather than
/// pulling a date crate: the kernel's whole value is adding ZERO new rows to `Cargo.lock`.
fn days_from_civil(date: &str) -> Option<i64> {
    let mut parts = date.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}

/// Deterministic content hash over the distilled set, for the mirror manifest.
#[must_use]
pub fn canonical_hash(releases: &[CrateRelease]) -> String {
    let mut hasher = Sha256::new();
    for release in releases {
        hasher.update(release.name.as_bytes());
        hasher.update([0]);
        hasher.update(release.latest_stable.as_bytes());
        hasher.update([0]);
        hasher.update(release.last_release_date.as_bytes());
        hasher.update([0]);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(vers: &str, pubtime: &str, yanked: bool) -> String {
        format!(r#"{{"name":"c","vers":"{vers}","yanked":{yanked},"pubtime":"{pubtime}"}}"#)
    }

    #[test]
    fn distill_picks_the_highest_stable_version_not_the_last_line() {
        let text = [
            line("1.0.0", "2020-01-01T00:00:00Z", false),
            line("1.10.0", "2021-01-01T00:00:00Z", false),
            line("1.9.0", "2022-01-01T00:00:00Z", false),
        ]
        .join("\n");
        let got = distill(&[("c".into(), text)]);
        // 1.10.0 > 1.9.0 numerically; a string sort would wrongly choose 1.9.0.
        assert_eq!(got[0].latest_stable, "1.10.0");
        assert_eq!(got[0].last_release_date, "2021-01-01");
    }

    #[test]
    fn distill_skips_yanked_and_prerelease_versions() {
        let text = [
            line("1.0.0", "2020-01-01T00:00:00Z", false),
            line("2.0.0", "2021-01-01T00:00:00Z", true),
            line("3.0.0-rc.1", "2022-01-01T00:00:00Z", false),
        ]
        .join("\n");
        assert_eq!(distill(&[("c".into(), text)])[0].latest_stable, "1.0.0");
    }

    #[test]
    fn build_metadata_is_ignored_for_ordering_so_deprecated_still_compares() {
        // The exact serde_yaml shape: `+deprecated` is build metadata, not a pre-release.
        assert_eq!(
            Version::parse("0.9.34+deprecated"),
            Version::parse("0.9.34")
        );
        assert!(Version::parse("0.9.34-rc.1").is_none());
    }

    #[test]
    fn a_crate_can_be_current_and_still_stale() {
        // serde_yaml: NOT behind (0.9.34+deprecated is the latest), but abandoned since 2024.
        // This is the case Dependabot and Renovate both report as up to date forever.
        let mirror = vec![CrateRelease {
            name: "serde_yaml".into(),
            latest_stable: "0.9.34+deprecated".into(),
            last_release_date: "2024-03-25".into(),
        }];
        let declared = vec![DeclaredDependency {
            name: "serde_yaml".into(),
            version: "0.9.34+deprecated".into(),
        }];
        let findings = evaluate(
            &mirror,
            &declared,
            90,
            "2026-08-17",
            &BTreeMap::new(),
            &Waivers::new(),
        );
        assert_eq!(
            findings.len(),
            1,
            "exactly one signal, not two: {findings:?}"
        );
        assert_eq!(findings[0].signal.code(), "DEP-FRESHNESS-STALE");
    }

    #[test]
    fn behind_and_stale_are_independent_signals_on_one_dependency() {
        let mirror = vec![CrateRelease {
            name: "c".into(),
            latest_stable: "2.0.0".into(),
            last_release_date: "2024-01-01".into(),
        }];
        let declared = vec![DeclaredDependency {
            name: "c".into(),
            version: "1.0.0".into(),
        }];
        let findings = evaluate(
            &mirror,
            &declared,
            90,
            "2026-08-17",
            &BTreeMap::new(),
            &Waivers::new(),
        );
        let codes: Vec<_> = findings.iter().map(|f| f.signal.code()).collect();
        assert_eq!(codes, ["DEP-FRESHNESS-BEHIND", "DEP-FRESHNESS-STALE"]);
    }

    #[test]
    fn a_waiver_marks_but_never_deletes_a_finding() {
        let mirror = vec![CrateRelease {
            name: "c".into(),
            latest_stable: "1.0.0".into(),
            last_release_date: "2020-01-01".into(),
        }];
        let declared = vec![DeclaredDependency {
            name: "c".into(),
            version: "1.0.0".into(),
        }];
        let mut waivers = Waivers::new();
        waivers.insert(
            ("c".into(), "DEP-FRESHNESS-STALE".into()),
            "accepted by axis-foundry".into(),
        );
        let owners = BTreeMap::from([("c".to_string(), "axis-foundry".to_string())]);
        let findings = evaluate(&mirror, &declared, 90, "2026-08-17", &owners, &waivers);
        assert_eq!(findings.len(), 1, "the finding survives the waiver");
        assert!(findings[0].waived);
        assert_eq!(findings[0].owner_team.as_deref(), Some("axis-foundry"));
    }

    #[test]
    fn a_waiver_for_one_code_does_not_silence_the_other() {
        let mirror = vec![CrateRelease {
            name: "c".into(),
            latest_stable: "2.0.0".into(),
            last_release_date: "2020-01-01".into(),
        }];
        let declared = vec![DeclaredDependency {
            name: "c".into(),
            version: "1.0.0".into(),
        }];
        let mut waivers = Waivers::new();
        waivers.insert(
            ("c".into(), "DEP-FRESHNESS-STALE".into()),
            "accepted".into(),
        );
        let findings = evaluate(
            &mirror,
            &declared,
            90,
            "2026-08-17",
            &BTreeMap::new(),
            &waivers,
        );
        let waived: Vec<_> = findings
            .iter()
            .map(|f| (f.signal.code(), f.waived))
            .collect();
        assert_eq!(
            waived,
            [
                ("DEP-FRESHNESS-BEHIND", false),
                ("DEP-FRESHNESS-STALE", true)
            ]
        );
    }

    #[test]
    fn a_dependency_missing_from_the_mirror_is_reported_not_skipped() {
        let findings = evaluate(
            &[],
            &[DeclaredDependency {
                name: "ghost".into(),
                version: "1.0.0".into(),
            }],
            90,
            "2026-08-17",
            &BTreeMap::new(),
            &Waivers::new(),
        );
        assert_eq!(findings[0].signal.code(), "DEP-FRESHNESS-UNKNOWN");
    }

    #[test]
    fn days_between_handles_leap_years_and_epoch_boundaries() {
        assert_eq!(days_between("2024-02-28", "2024-03-01"), Some(2)); // 2024 is a leap year
        assert_eq!(days_between("2023-02-28", "2023-03-01"), Some(1));
        assert_eq!(days_between("1970-01-01", "1970-01-01"), Some(0));
        assert_eq!(days_between("2024-03-25", "2026-08-17"), Some(875));
        assert_eq!(days_between("not-a-date", "2026-08-17"), None);
    }

    #[test]
    fn the_hash_is_order_independent_of_input_but_sensitive_to_content() {
        let a = distill(&[
            ("b".into(), line("1.0.0", "2020-01-01T00:00:00Z", false)),
            ("a".into(), line("1.0.0", "2020-01-01T00:00:00Z", false)),
        ]);
        let b = distill(&[
            ("a".into(), line("1.0.0", "2020-01-01T00:00:00Z", false)),
            ("b".into(), line("1.0.0", "2020-01-01T00:00:00Z", false)),
        ]);
        assert_eq!(canonical_hash(&a), canonical_hash(&b));
        let c = distill(&[("a".into(), line("1.0.1", "2020-01-01T00:00:00Z", false))]);
        assert_ne!(canonical_hash(&a), canonical_hash(&c));
    }
}
