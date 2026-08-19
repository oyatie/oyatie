//! Shrink-only ratchet over the deprecated `cloud-` NAME prefix.
//!
//! ## Why a gate-owned baseline rather than a firewall code
//! `gate-baseline.generated.json` is a controller-owned frozen artifact; adding a code to it
//! requires a founder-signed door admission (see `gate-baseline.signoff.json`). This gate instead
//! owns its baseline as ordinary committed JSON, exactly as `ci/facade/crate-catalog-coverage`
//! does. Same shrink-only semantics, no governance blocker, and zero blast radius on the firewall.
//!
//! ## Why the rule is name-anchored, not a prose stem
//! A bare `cloud` substring occurs in 4,418 files — more than any other retired brand stem — and
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
///
/// BOTH separators are forbidden. The accepted naming grammar rejects a leading `cloud-` and a
/// leading `cloud_` alike (ADR-0711), and the same holds for the `oya-`/`oya_` wrapper. Matching
/// only the hyphen let `secrets/cloud_new_service` — or a Cargo package named `oya_cloud_thing` —
/// walk straight past a blocking gate by changing one character.
///
/// Does this name segment begin with a deprecated brand-namespace prefix?
///
/// BOTH `cloud-` and `oya-` are shrink-to-zero targets. They are the same kind of debt — a brand
/// baked into an identifier — and de-branding is already 64% complete: of 1,759 packages, 1,129
/// carry neither prefix. The `crate-name-prefix` gate was built for this, scoring de-branded rows
/// as advisory rather than as violations, so dropping a prefix is the supported direction rather
/// than something CI resists.
#[must_use]
pub fn is_deprecated_prefixed_name(segment: &str) -> bool {
    is_cloud_prefixed_name(segment) || is_oya_prefixed_name(segment)
}

/// `oya-` / `oya_` prefixed identifiers.
///
/// The bare word `oya` is not a prefix, and `oyatie-` is the ORGANISATION rather than the retired
/// crate prefix — `oyatie-cloud-provider` must not match here, or the ratchet would demand a rename
/// of the company's own name.
#[must_use]
pub fn is_oya_prefixed_name(segment: &str) -> bool {
    let lower = segment.to_ascii_lowercase();
    strip_either(&lower, "oya").is_some_and(|rest| !rest.is_empty())
}

pub fn is_cloud_prefixed_name(segment: &str) -> bool {
    let lower = segment.to_ascii_lowercase();
    let stem = strip_either(&lower, "oya").unwrap_or(&lower);
    let Some(rest) = strip_either(stem, "cloud").filter(|rest| !rest.is_empty()) else {
        return false;
    };
    !LEGITIMATE_CLOUD_COMPOUNDS.iter().any(|word| {
        rest == *word
            || rest.starts_with(&format!("{word}-"))
            || rest.starts_with(&format!("{word}_"))
    })
}

/// Strip `<word>-` or `<word>_`, so one separator cannot dodge the other.
fn strip_either<'a>(value: &'a str, word: &str) -> Option<&'a str> {
    value
        .strip_prefix(&format!("{word}-"))
        .or_else(|| value.strip_prefix(&format!("{word}_")))
}

/// Extract a declared identifier from `name = "x"` / `package.name = "x"` (Cargo) or
/// `name: x` (Helm).
///
/// Cargo accepts the dotted-table spelling `package.name = "cloud-new-service"`, which is a
/// perfectly valid manifest; recognizing only a bare leading `name` let such a member declare a
/// deprecated identifier with no finding at all. `names = [...]` and similar longer keys must
/// still NOT match, so the key is compared exactly rather than by prefix.
#[must_use]
pub fn declared_name(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let (key, value) = trimmed
        .split_once('=')
        .or_else(|| trimmed.split_once(':'))?;
    let key = key.trim();
    if key != "name" && key != "package.name" {
        return None;
    }
    Some(value.trim().trim_matches(['"', '\''].as_slice()))
}

/// Extract a durable capability identifier from a catalog row's `capability: <id>` line.
#[must_use]
pub fn declared_capability(line: &str) -> Option<&str> {
    let (key, value) = line.trim().split_once(':')?;
    if key.trim() != "capability" {
        return None;
    }
    Some(value.trim().trim_matches(['"', '\''].as_slice()))
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
        if is_deprecated_prefixed_name(segment) {
            found.insert(format!("dir:{prefix}"));
            break;
        }
    }
    // Capability identifiers in the crate catalog are durable names too. The ratchet scanned only
    // paths plus Cargo/Helm declarations, so a catalog row could mint `cloud-ci-<thing>` and stay
    // green — which is precisely how this gate's OWN row introduced the debt it exists to prevent.
    if path.starts_with("registry/catalog/") && path.ends_with(".yaml") {
        for capability in contents.lines().filter_map(declared_capability) {
            if is_deprecated_prefixed_name(capability) {
                found.insert(format!("name:{path}:{capability}"));
            }
        }
    }
    if matches!(path.rsplit('/').next(), Some("Cargo.toml" | "Chart.yaml")) {
        for name in contents.lines().filter_map(declared_name) {
            if is_deprecated_prefixed_name(name) {
                // Keyed by MANIFEST PATH as well as identifier. A bare `name:<id>` key collapsed
                // in the BTreeSet whenever a second manifest declared an already-baselined name,
                // so adding another Cargo or Helm artifact called `oya-cloud-iam` produced no
                // addition and passed the blocking ratchet without any baseline edit.
                found.insert(format!("name:{path}:{name}"));
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

    /// Reviewer finding: only `cloud-` was recognized, so one separator change walked past a
    /// blocking gate. The grammar forbids both leading forms (ADR-0711).
    #[test]
    fn underscore_variants_are_forbidden_too() {
        assert!(is_cloud_prefixed_name("cloud_new_service"));
        assert!(is_cloud_prefixed_name("oya_cloud_thing"));
        assert!(is_cloud_prefixed_name("oya-cloud_thing"));
        // legitimate technical compounds stay allowed on BOTH separators
        assert!(!is_cloud_prefixed_name("cloud_native"));
        assert!(!is_cloud_prefixed_name("cloud-provider"));
    }

    /// Reviewer finding: `package.name = "..."` is a valid Cargo spelling that produced no
    /// finding, while a longer key such as `names = [...]` must still not match.
    #[test]
    fn dotted_cargo_package_names_are_parsed_and_lookalikes_are_not() {
        assert_eq!(
            declared_name("package.name = \"cloud-new-service\""),
            Some("cloud-new-service")
        );
        assert_eq!(declared_name("name = \"cloud-x\""), Some("cloud-x"));
        assert_eq!(declared_name("name: cloud-x"), Some("cloud-x"));
        assert_eq!(declared_name("names = [\"cloud-x\"]"), None);
        assert_eq!(declared_name("nameserver = \"cloud-x\""), None);
    }

    /// Reviewer finding: a bare `name:<id>` key collapsed in the set, so a SECOND manifest
    /// declaring an already-baselined name produced no addition.
    #[test]
    fn the_same_declared_name_in_two_manifests_is_two_keys() {
        let first = findings("a/Cargo.toml", "name = \"oya-cloud-iam\"\n");
        let second = findings("b/Chart.yaml", "name: oya-cloud-iam\n");
        assert_ne!(first, second);
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
    }

    /// Reviewer finding: catalog capability identifiers were never scanned, which is how this
    /// gate's own row minted `cloud-ci-cloud-name-ratchet` while staying green.
    #[test]
    fn crate_catalog_capability_identifiers_are_scanned() {
        let found = findings("registry/catalog/x.yaml", "capability: cloud-ci-thing\n");
        assert!(found.contains("name:registry/catalog/x.yaml:cloud-ci-thing"));
        let neutral = findings(
            "registry/catalog/y.yaml",
            "capability: ci-name-prefix-ratchet\n",
        );
        assert!(neutral.is_empty());
    }

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
        // Prose in a manifest is not a name assignment. The control name must itself be
        // de-branded: `oya-audit` became a rename target when `oya-` joined the ratchet.
        assert!(
            findings(
                "x/Cargo.toml",
                "# cloud-native note\nname = \"audit-chain\"\n"
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

#[cfg(test)]
mod oya_prefix_tests {
    use super::*;

    #[test]
    fn oya_prefixed_identifiers_are_flagged() {
        assert!(is_oya_prefixed_name("oya-audit-chain"));
        assert!(is_oya_prefixed_name("oya_workspace_meet_api"));
        assert!(is_deprecated_prefixed_name("oya-cloud-kms"));
    }

    #[test]
    fn the_organisation_name_is_not_the_retired_crate_prefix() {
        // `oyatie-cloud-provider` is a real capability context. Matching it would demand a rename
        // of the company's own name, which is not what de-branding means.
        assert!(!is_oya_prefixed_name("oyatie-cloud-provider"));
        assert!(!is_oya_prefixed_name("oyatie"));
        assert!(!is_oya_prefixed_name("oya"));
        assert!(findings("iac/iac/oyatie-cloud-provider/x.yaml", "").is_empty());
    }

    #[test]
    fn already_debranded_names_stay_clean() {
        for n in [
            "ci-dep-freshness",
            "audit-chain-emission-kernel",
            "port-engine-kernel",
        ] {
            assert!(!is_deprecated_prefixed_name(n), "{n} must not be flagged");
        }
    }
}
