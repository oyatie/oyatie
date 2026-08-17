//! Shrink-only ratchet over the deprecated `cloud-` NAME prefix.
//!
//! ## Why a gate-owned baseline rather than a firewall code
//! `gate-baseline.generated.json` is a controller-owned frozen artifact; adding a code to it
//! requires a founder-signed door admission (see `gate-baseline.signoff.json`). This gate instead
//! owns its baseline as ordinary committed JSON, exactly as `ci/facade/crate-catalog-coverage`
//! does. Same shrink-only semantics, no governance blocker, and zero blast radius on the firewall.
//!
//! ## Why the rule is name-anchored, not a prose stem
//! A bare `cloud` substring occurs in 4,418 files — more than the entire `foundry` residue — and
//! most of it is legitimate: `cloud-provider` (1,880 occurrences), `cloud-native` (282), and
//! `oyatie-cloud-provider` is a real capability context. Freezing that would create thousands of
//! tolerated keys that mostly SHOULD NOT shrink, which is noise, not a ratchet. Anchoring on
//! renameable identifiers yields a small set that can actually reach zero.
//!
//! ## Why the key is the rename unit
//! Keying per-file would freeze 462 entries, because every file beneath `iam/cloud-iam/` counts
//! separately. Keying on the identifier gives ~304, one per thing that must be renamed, each
//! disappearing exactly when its rename lands.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

/// `cloud-<word>` compounds that are ordinary technical English rather than the deprecated
/// platform namespace. DATA, not a scanner branch.
pub const LEGITIMATE_CLOUD_COMPOUNDS: &[&str] = &[
    "native",
    "provider",
    "agnostic",
    "hyperscaler",
    "vendor",
    "region",
    "scale",
];

/// Paths that name the pattern in order to enforce it, or that can never shrink.
pub const CARVE_OUTS: &[&str] = &[
    "ci/facade/cloud-name-ratchet/",
    "evidence/audit-chain.jsonl",
];

/// Does this single name segment begin with the deprecated `cloud-` namespace prefix?
#[must_use]
pub fn is_cloud_prefixed_name(segment: &str) -> bool {
    let lower = segment.to_ascii_lowercase();
    let stem = lower.strip_prefix("oya-").unwrap_or(&lower);
    let Some(rest) = stem.strip_prefix("cloud-").filter(|rest| !rest.is_empty()) else {
        return false;
    };
    !LEGITIMATE_CLOUD_COMPOUNDS
        .iter()
        .any(|word| rest == *word || rest.starts_with(&format!("{word}-")))
}

/// Extract a declared identifier from `name = "x"` (Cargo) or `name: x` (Helm).
#[must_use]
pub fn declared_name(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("name")?.trim_start();
    let rest = rest
        .strip_prefix('=')
        .or_else(|| rest.strip_prefix(':'))?
        .trim();
    Some(rest.trim_matches(['"', '\''].as_slice()))
}

/// Every deprecated `cloud-` identifier this document contributes.
///
/// `dir:<path-through-the-offending-segment>` for a path segment — the outermost segment only,
/// since deeper ones move with it. `name:<identifier>` for a Cargo or Helm declared name.
#[must_use]
pub fn findings(path: &str, contents: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    if CARVE_OUTS.iter().any(|carve| path.starts_with(carve)) {
        return found;
    }
    let mut prefix = String::new();
    for segment in path.split('/') {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(segment);
        if is_cloud_prefixed_name(segment) {
            found.insert(format!("dir:{prefix}"));
            break;
        }
    }
    if matches!(path.rsplit('/').next(), Some("Cargo.toml" | "Chart.yaml")) {
        for name in contents.lines().filter_map(declared_name) {
            if is_cloud_prefixed_name(name) {
                found.insert(format!("name:{name}"));
            }
        }
    }
    found
}

/// The verdict of comparing today's corpus against the frozen baseline.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Verdict {
    /// New deprecated names beyond the baseline. Any entry here is a hard failure.
    pub added: BTreeSet<String>,
    /// Baselined names that are gone — burn-down. Not a failure in spirit, but the baseline must
    /// be updated in the same change so the frozen file never overstates the remaining debt.
    pub removed: BTreeSet<String>,
}

/// Compare a census against the frozen baseline.
#[must_use]
pub fn compare(current: &BTreeSet<String>, baseline: &BTreeSet<String>) -> Verdict {
    Verdict {
        added: current.difference(baseline).cloned().collect(),
        removed: baseline.difference(current).cloned().collect(),
    }
}

/// Parse the frozen baseline JSON (`{"cloud_prefixed_names": [...]}`).
#[must_use]
pub fn parse_baseline(json: &str) -> BTreeSet<String> {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|d| {
            d.get("cloud_prefixed_names")?.as_array().map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_platform_namespace_is_flagged() {
        assert!(is_cloud_prefixed_name("cloud-iam"));
        assert!(is_cloud_prefixed_name("oya-cloud-kms"));
        assert_eq!(
            findings("iam/cloud-iam/x.yaml", ""),
            BTreeSet::from(["dir:iam/cloud-iam".to_string()])
        );
    }

    #[test]
    fn adjectival_compounds_are_english_not_a_namespace() {
        assert!(!is_cloud_prefixed_name(
            "cloud-native-infrastructure-automation.md"
        ));
        assert!(!is_cloud_prefixed_name(
            "cloud-provider-full-ecosystem-north-star.md"
        ));
        // `oyatie-cloud-provider` is a real capability context.
        assert!(findings("iac/iac/oyatie-cloud-provider/x.yaml", "").is_empty());
    }

    #[test]
    fn declared_names_are_read_only_from_manifests() {
        assert!(!findings("x/Cargo.toml", "name = \"oya-cloud-kms\"\n").is_empty());
        assert!(!findings("x/Chart.yaml", "name: oya-cloud-iam\n").is_empty());
        // The same text in a source file is not a declared identifier.
        assert!(findings("src/lib.rs", "name = \"oya-cloud-kms\"").is_empty());
        // Prose in a manifest is not a name assignment.
        assert!(
            findings(
                "x/Cargo.toml",
                "# cloud-native note\nname = \"oya-audit\"\n"
            )
            .is_empty()
        );
    }

    #[test]
    fn the_outermost_segment_is_the_rename_unit() {
        // Deeper files under a flagged directory must not each become their own key.
        assert_eq!(
            findings("iam/cloud-iam/deep/nested/file.yaml", ""),
            BTreeSet::from(["dir:iam/cloud-iam".to_string()])
        );
    }

    #[test]
    fn compare_separates_growth_from_burn_down() {
        let base = BTreeSet::from(["dir:a".to_string(), "dir:b".to_string()]);
        let now = BTreeSet::from(["dir:b".to_string(), "dir:c".to_string()]);
        let v = compare(&now, &base);
        assert_eq!(v.added, BTreeSet::from(["dir:c".to_string()]));
        assert_eq!(v.removed, BTreeSet::from(["dir:a".to_string()]));
    }

    #[test]
    fn the_gate_carves_out_its_own_source() {
        assert!(findings("ci/facade/cloud-name-ratchet/src/lib.rs", "cloud-anything").is_empty());
    }

    #[test]
    fn baseline_parsing_is_fail_soft_but_visible() {
        assert!(parse_baseline("{ not json").is_empty());
        assert_eq!(
            parse_baseline(r#"{"cloud_prefixed_names":["dir:x"]}"#),
            BTreeSet::from(["dir:x".to_string()])
        );
    }
}
