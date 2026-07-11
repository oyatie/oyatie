//! Gate-coverage check 2/3 (born-advisory): capability-registry ⇄ derived
//! gate-policy sync.
//!
//! The #1327 defect class (c): a capability root registered in
//! `specs/capability-registry.json` was missing from the hand-maintained derived
//! gate policies (`module-membership`, `root-hygiene`, `tier-dependency`). The
//! registry is the closed authority (ADR-0562/0615) but the three policies drift
//! by hand — nothing derives them from the registry — so a newly-registered
//! capability root (e.g. the `policy` capability extracted by ADR-0615) can be
//! absent from a policy's allow-list and no gate notices.
//!
//! This check re-derives the required roots from the registry and asserts each is
//! present in every policy that must carry it:
//! - every capability `absorbs_current_dirs` entry that is a TOP-LEVEL dir (no
//!   `/`) — the capability's own root — must appear in the module-membership
//!   `allowed_top_level_dirs`, the root-hygiene `allowed_root_dirs`, AND the
//!   tier-dependency roots (`unclassified_roots` ∪ `service_roots`);
//! - every registry `meta_directories[].dir` (normalized, trailing `/` stripped)
//!   must appear in the module-membership `allowed_top_level_dirs` — the
//!   ADR-0562 closed-set authority for the meta ring. Meta dirs are DELIBERATELY
//!   not cross-required in root-hygiene / tier-dependency: those policies govern
//!   tracked-root files and crate-graph tiers respectively and legitimately omit
//!   off-ladder / not-yet-tracked meta dirs (kernel carries its own nested
//!   workspace; os/base/build/app own crates or files only once they exist). A
//!   NEW meta dir missing from the membership authority is still a regression.
//!
//! Each miss yields a Finding naming the EXACT policy file + key + dir, so the
//! FAIL output alone is actionable. Pure evaluator over a caller-assembled corpus.
//! Legitimate structural divergences (if any) are absorbed by the born-advisory
//! frozen baseline; a NEW desync is a regression that blocks.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::Finding;

/// Validator id recorded by the registry⇄derived-policy sync contract.
pub const REGISTRY_POLICY_SYNC_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/registry-derived-policy-sync";

/// The advisory violation code this check emits.
pub const REGISTRY_POLICY_DESYNC_CODE: &str = "registry_derived_policy_desync";

fn desync(key: &str) -> Finding {
    Finding::new(REGISTRY_POLICY_DESYNC_CODE, key)
}

/// A derived policy the registry roots must propagate into: the corpus-relative
/// pointer plus the resolved document.
struct Policy<'a> {
    /// The repo-relative policy path, used verbatim in the finding key.
    path: &'a str,
    document: &'a Value,
}

fn read_policy<'a>(
    policies: &'a Value,
    slot: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<Policy<'a>> {
    let entry = policies.get(slot)?;
    let (Some(path), Some(document)) = (
        entry.get("path").and_then(Value::as_str),
        entry.get("document"),
    ) else {
        findings.insert(desync(&format!("<malformed-policy-corpus>@{slot}")));
        return None;
    };
    Some(Policy { path, document })
}

/// Collect a policy document's string-array field into a set. A missing/malformed
/// field fails closed with a keyed finding (returned set is empty so downstream
/// membership tests all miss — a broken policy can never mask a desync).
fn string_set(
    document: &Value,
    field: &str,
    policy_path: &str,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    match document.get(field).and_then(Value::as_array) {
        Some(items) => {
            let mut set = BTreeSet::new();
            for item in items {
                match item.as_str() {
                    Some(text) => {
                        set.insert(text.to_owned());
                    }
                    None => {
                        findings.insert(desync(&format!("<malformed-{policy_path}#{field}>")));
                    }
                }
            }
            set
        }
        None => {
            findings.insert(desync(&format!("<malformed-{policy_path}#{field}>")));
            BTreeSet::new()
        }
    }
}

/// Evaluate the registry⇄derived-policy sync corpus:
///
/// ```jsonc
/// {
///   "registry": { "capabilities": [ { "absorbs_current_dirs": ["policy", "oya/policy"] } ],
///                 "meta_directories": [ { "dir": "kernel/" } ] },
///   "policies": {
///     "module_membership": { "path": "ci/facade/module-membership/capability-membership-policy.json",
///                            "document": { "allowed_top_level_dirs": ["policy", "kernel", ...] } },
///     "root_hygiene":     { "path": "ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json",
///                            "document": { "allowed_root_dirs": ["policy", ...] } },
///     "tier_dependency":  { "path": "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json",
///                            "document": { "unclassified_roots": ["policy", ...], "service_roots": ["cloud", "oya"] } }
///   }
/// }
/// ```
pub fn evaluate_registry_derived_policy_sync(corpus: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(registry) = corpus.get("registry") else {
        findings.insert(desync("<missing-registry>"));
        return findings;
    };
    let Some(policies) = corpus.get("policies") else {
        findings.insert(desync("<missing-policies>"));
        return findings;
    };

    let (Some(membership), Some(root_hygiene), Some(tier)) = (
        read_policy(policies, "module_membership", &mut findings),
        read_policy(policies, "root_hygiene", &mut findings),
        read_policy(policies, "tier_dependency", &mut findings),
    ) else {
        return findings;
    };

    let membership_roots = string_set(
        membership.document,
        "allowed_top_level_dirs",
        membership.path,
        &mut findings,
    );
    let root_hygiene_roots = string_set(
        root_hygiene.document,
        "allowed_root_dirs",
        root_hygiene.path,
        &mut findings,
    );
    let mut tier_roots = string_set(
        tier.document,
        "unclassified_roots",
        tier.path,
        &mut findings,
    );
    tier_roots.extend(string_set(
        tier.document,
        "service_roots",
        tier.path,
        &mut findings,
    ));

    // Capability roots: every TOP-LEVEL absorbs dir must reach all three policies.
    let capability_roots = capability_top_level_roots(registry, &mut findings);
    for root in &capability_roots {
        if !membership_roots.contains(root) {
            findings.insert(desync(&format!(
                "{}#allowed_top_level_dirs:{root}",
                membership.path
            )));
        }
        if !root_hygiene_roots.contains(root) {
            findings.insert(desync(&format!(
                "{}#allowed_root_dirs:{root}",
                root_hygiene.path
            )));
        }
        if !tier_roots.contains(root) {
            findings.insert(desync(&format!(
                "{}#unclassified_roots|service_roots:{root}",
                tier.path
            )));
        }
    }

    // Meta dirs: the ADR-0562 closed-set authority is the membership whitelist.
    for meta in meta_directory_roots(registry, &mut findings) {
        if !membership_roots.contains(&meta) {
            findings.insert(desync(&format!(
                "{}#allowed_top_level_dirs:{meta}",
                membership.path
            )));
        }
    }

    findings
}

/// The distinct TOP-LEVEL (single-segment, no `/`) `absorbs_current_dirs` entries
/// across every registered capability. A malformed registry fails closed.
fn capability_top_level_roots(
    registry: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    let Some(capabilities) = registry.get("capabilities").and_then(Value::as_array) else {
        findings.insert(desync("<malformed-registry.capabilities>"));
        return BTreeSet::new();
    };
    let mut roots = BTreeSet::new();
    for capability in capabilities {
        let Some(dirs) = capability
            .get("absorbs_current_dirs")
            .and_then(Value::as_array)
        else {
            findings.insert(desync(
                "<malformed-registry.capabilities.absorbs_current_dirs>",
            ));
            continue;
        };
        for dir in dirs {
            if let Some(text) = dir.as_str().map(str::trim).filter(|d| !d.is_empty())
                && !text.contains('/')
            {
                roots.insert(text.to_owned());
            }
        }
    }
    roots
}

/// The registry meta directories, normalized (trailing `/` stripped). A malformed
/// registry fails closed.
fn meta_directory_roots(registry: &Value, findings: &mut BTreeSet<Finding>) -> BTreeSet<String> {
    let Some(metas) = registry.get("meta_directories").and_then(Value::as_array) else {
        findings.insert(desync("<malformed-registry.meta_directories>"));
        return BTreeSet::new();
    };
    let mut roots = BTreeSet::new();
    for meta in metas {
        if let Some(dir) = meta.get("dir").and_then(Value::as_str) {
            let normalized = dir.trim().trim_end_matches('/');
            if !normalized.is_empty() {
                roots.insert(normalized.to_owned());
            }
        }
    }
    roots
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use serde_json::json;

    use super::*;

    fn keys(findings: &BTreeSet<Finding>) -> Vec<String> {
        findings.iter().map(|f| f.key.clone()).collect()
    }

    fn green_corpus() -> Value {
        json!({
            "registry": {
                "capabilities": [
                    { "absorbs_current_dirs": ["policy", "oya/policy"] },
                    { "absorbs_current_dirs": ["cell", "cloud/cloud-cell"] }
                ],
                "meta_directories": [ { "dir": "kernel/" }, { "dir": "governance/" } ]
            },
            "policies": {
                "module_membership": {
                    "path": "ci/facade/module-membership/capability-membership-policy.json",
                    "document": { "allowed_top_level_dirs": ["policy", "cell", "kernel", "governance"] }
                },
                "root_hygiene": {
                    "path": "ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json",
                    "document": { "allowed_root_dirs": ["policy", "cell", "kernel", "governance"] }
                },
                "tier_dependency": {
                    "path": "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json",
                    "document": { "unclassified_roots": ["policy", "cell"], "service_roots": ["cloud", "oya"] }
                }
            }
        })
    }

    #[test]
    fn a_fully_synced_registry_and_policies_is_green() {
        assert!(evaluate_registry_derived_policy_sync(&green_corpus()).is_empty());
    }

    #[test]
    fn a_capability_root_missing_from_membership_is_flagged_with_exact_file_and_key() {
        let mut corpus = green_corpus();
        corpus["policies"]["module_membership"]["document"]["allowed_top_level_dirs"] =
            json!(["cell", "kernel", "governance"]); // dropped "policy"
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec![
                "ci/facade/module-membership/capability-membership-policy.json#allowed_top_level_dirs:policy"
                    .to_owned()
            ]
        );
    }

    #[test]
    fn a_capability_root_missing_from_root_hygiene_and_tier_is_flagged_per_policy() {
        let mut corpus = green_corpus();
        corpus["policies"]["root_hygiene"]["document"]["allowed_root_dirs"] =
            json!(["policy", "kernel", "governance"]); // dropped "cell"
        corpus["policies"]["tier_dependency"]["document"]["unclassified_roots"] = json!(["policy"]); // dropped "cell"
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec![
                "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json#unclassified_roots|service_roots:cell".to_owned(),
                "ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json#allowed_root_dirs:cell".to_owned(),
            ]
        );
    }

    #[test]
    fn a_service_root_is_accepted_from_tier_service_roots_not_only_unclassified() {
        // A capability root that lives in tier `service_roots` (cloud/oya style)
        // must not be reported as a tier desync.
        let mut corpus = green_corpus();
        corpus["registry"]["capabilities"] = json!([{ "absorbs_current_dirs": ["cloud"] }]);
        corpus["policies"]["module_membership"]["document"]["allowed_top_level_dirs"] =
            json!(["cloud", "kernel", "governance"]);
        corpus["policies"]["root_hygiene"]["document"]["allowed_root_dirs"] =
            json!(["cloud", "kernel", "governance"]);
        // "cloud" is only in service_roots, not unclassified_roots.
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert!(findings.is_empty(), "{:?}", keys(&findings));
    }

    #[test]
    fn a_meta_dir_missing_from_membership_authority_is_flagged() {
        let mut corpus = green_corpus();
        corpus["policies"]["module_membership"]["document"]["allowed_top_level_dirs"] =
            json!(["policy", "cell", "kernel"]); // dropped "governance"
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec![
                "ci/facade/module-membership/capability-membership-policy.json#allowed_top_level_dirs:governance"
                    .to_owned()
            ]
        );
    }

    #[test]
    fn nested_absorb_dirs_are_not_treated_as_top_level_roots() {
        // "oya/policy" is nested and must NOT be required as a top-level root.
        let mut corpus = green_corpus();
        // Remove the top-level "policy" but keep the nested "oya/policy".
        corpus["registry"]["capabilities"] = json!([{ "absorbs_current_dirs": ["oya/policy"] }]);
        corpus["policies"]["module_membership"]["document"]["allowed_top_level_dirs"] =
            json!(["cell", "kernel", "governance"]);
        corpus["policies"]["root_hygiene"]["document"]["allowed_root_dirs"] =
            json!(["cell", "kernel", "governance"]);
        corpus["policies"]["tier_dependency"]["document"]["unclassified_roots"] = json!(["cell"]);
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert!(findings.is_empty(), "{:?}", keys(&findings));
    }

    #[test]
    fn a_malformed_policy_corpus_fails_closed() {
        let findings = evaluate_registry_derived_policy_sync(&json!({ "registry": {} }));
        assert_eq!(keys(&findings), vec!["<missing-policies>".to_owned()]);
    }

    #[test]
    fn every_finding_uses_the_advisory_code() {
        let mut corpus = green_corpus();
        corpus["policies"]["module_membership"]["document"]["allowed_top_level_dirs"] = json!([]);
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert!(!findings.is_empty());
        for finding in &findings {
            assert_eq!(finding.code, REGISTRY_POLICY_DESYNC_CODE);
        }
    }
}
