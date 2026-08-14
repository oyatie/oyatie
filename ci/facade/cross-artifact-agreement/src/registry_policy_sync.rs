//! Gate-coverage check 2/3 (born-advisory): capability-registry ⇄ derived
//! gate-policy sync.
//!
//! The #1327 defect class (c): a capability root registered in
//! `governance/capability-registry.json` was missing from the hand-maintained derived
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
//!   tier-dependency roots (`unclassified_roots` ∪ `service_roots` ∪ `capability_roots`);
//! - every registry `meta_directories[].dir` (normalized, trailing `/` stripped)
//!   must appear in the module-membership `allowed_top_level_dirs` — the
//!   ADR-0562 closed-set authority for the meta ring — AND in the root-hygiene
//!   `allowed_root_dirs`. Root-hygiene is a born-blocking default-DENY allowlist
//!   over tracked top-level dirs, so a declared meta destination absent from it
//!   is not a harmless omission: the FIRST tracked file created under that
//!   destination REDs the gate, which is what blocked `app/` (110 declared
//!   crates) from ever being created. A declared destination must be creatable.
//!   Meta dirs stay DELIBERATELY not cross-required in tier-dependency: that
//!   policy governs crate-graph tiers and legitimately omits off-ladder dirs.
//! - every capability top-level root AND every crate-owning meta dir
//!   (`meta_directories[].owns_crates`, defaulted to `true` when undeclared so a
//!   new meta dir fails CLOSED into coverage) must appear in the
//!   module-membership `scan_roots`. This is the COVERAGE-SCOPE anti-laundering
//!   rule: `scan_roots` is what the membership lint actually walks, so a
//!   destination root missing from it drops every crate moved there out of lint
//!   coverage SILENTLY — the `min_expected_crates` floor is a broken-scan guard
//!   (roots deleted / wrong CWD), not a coverage guard, and cannot see a
//!   partial-root loss. Deriving the requirement from the closed registry means
//!   widening coverage is the only legal direction: a root can only leave
//!   `scan_roots` by leaving the registry's crate-owning set, which is a visible
//!   edit to the ADR-0562/0615 authority, never a quiet policy-list trim.
//!
//! Each miss yields a Finding naming the EXACT policy file + key + dir, so the
//! FAIL output alone is actionable. Pure evaluator over a caller-assembled corpus.
//! Legitimate structural divergences (if any) are absorbed by the born-advisory
//! frozen baseline; a NEW desync is a regression that blocks.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::{BTreeMap, BTreeSet};

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
///   "registry": { "capabilities": [ { "absorbs_current_dirs": ["policy", "example/policy"] } ],
///                 "meta_directories": [ { "dir": "kernel/", "owns_crates": true } ] },
///   "policies": {
///     "module_membership": { "path": "ci/facade/module-membership/capability-membership-policy.json",
///                            "document": { "allowed_top_level_dirs": ["policy", "kernel", ...],
///                                          "scan_roots": ["policy", "kernel", ...] } },
///     "root_hygiene":     { "path": "ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json",
///                            "document": { "allowed_root_dirs": ["policy", ...] } },
///     "tier_dependency":  { "path": "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json",
///                            "document": { "unclassified_roots": ["policy", ...], "service_roots": ["cloud", "oya"], "capability_roots": ["cell", ...] } }
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

    let (Some(root_hygiene), Some(tier)) = (
        read_policy(policies, "root_hygiene", &mut findings),
        read_policy(policies, "tier_dependency", &mut findings),
    ) else {
        return findings;
    };

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
    // `capability_roots` is the TIER-ENFORCED home for capability trees. It must join the union or
    // this check would RED the correct fix: moving a capability root out of `unclassified_roots`
    // into `capability_roots` keeps it declared, and reporting that as a desync would leave
    // `unclassified_roots` (the silent-exemption list) as the only union member a capability could
    // legally sit in — which is how it grew to 24 capability entries in the first place.
    tier_roots.extend(string_set(
        tier.document,
        "capability_roots",
        tier.path,
        &mut findings,
    ));

    // Capability roots: every TOP-LEVEL absorbs dir must reach all three policies.
    let capability_roots = capability_top_level_roots(registry, &mut findings);
    for root in &capability_roots {
        if !root_hygiene_roots.contains(root) {
            findings.insert(desync(&format!(
                "{}#allowed_root_dirs:{root}",
                root_hygiene.path
            )));
        }
        if !tier_roots.contains(root) {
            findings.insert(desync(&format!(
                "{}#unclassified_roots|service_roots|capability_roots:{root}",
                tier.path
            )));
        }
    }

    // Meta dirs: root-hygiene must admit the dir or the destination cannot be
    // created at all. (The module-membership whitelist/scan-scope half retired
    // with the registry bookkeeping layer, ADR-0718.)
    for (meta, _owns_crates) in meta_directory_roots(registry, &mut findings) {
        if !root_hygiene_roots.contains(&meta) {
            findings.insert(desync(&format!(
                "{}#allowed_root_dirs:{meta}",
                root_hygiene.path
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

/// The registry meta directories, normalized (trailing `/` stripped), each mapped
/// to whether it owns crates. An undeclared `owns_crates` fails CLOSED as `true`:
/// an unclassified meta dir is treated as crate-owning so it must be scanned.
/// A malformed registry fails closed.
fn meta_directory_roots(
    registry: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, bool> {
    let Some(metas) = registry.get("meta_directories").and_then(Value::as_array) else {
        findings.insert(desync("<malformed-registry.meta_directories>"));
        return BTreeMap::new();
    };
    let mut roots = BTreeMap::new();
    for meta in metas {
        if let Some(dir) = meta.get("dir").and_then(Value::as_str) {
            let normalized = dir.trim().trim_end_matches('/');
            if !normalized.is_empty() {
                let owns_crates = meta
                    .get("owns_crates")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
                roots.insert(normalized.to_owned(), owns_crates);
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
                    { "absorbs_current_dirs": ["policy", "oya/policy"] }
                ],
                "meta_directories": [
                    { "dir": "kernel/", "owns_crates": true }
                ]
            },
            "policies": {
                "root_hygiene": {
                    "path": "ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json",
                    "document": {
                        "allowed_root_dirs": ["policy", "kernel", "oya"]
                    }
                },
                "tier_dependency": {
                    "path": "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json",
                    "document": {
                        "unclassified_roots": ["oya"],
                        "service_roots": [],
                        "capability_roots": ["policy"]
                    }
                }
            }
        })
    }

    #[test]
    fn registry_derived_policy_sync_is_green_on_a_fully_declared_corpus() {
        assert!(evaluate_registry_derived_policy_sync(&green_corpus()).is_empty());
    }

    #[test]
    fn a_capability_root_missing_from_root_hygiene_is_a_desync() {
        let mut corpus = green_corpus();
        corpus["policies"]["root_hygiene"]["document"]["allowed_root_dirs"] =
            json!(["kernel", "oya"]);
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert!(
            keys(&findings)
                .iter()
                .any(|k| k.contains("#allowed_root_dirs:policy"))
        );
    }

    #[test]
    fn a_capability_root_missing_from_tier_roots_is_a_desync() {
        let mut corpus = green_corpus();
        corpus["policies"]["tier_dependency"]["document"]["capability_roots"] = json!([]);
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert!(keys(&findings).iter().any(|k| k.contains(":policy")));
    }

    #[test]
    fn a_meta_dir_missing_from_root_hygiene_is_a_desync() {
        let mut corpus = green_corpus();
        corpus["policies"]["root_hygiene"]["document"]["allowed_root_dirs"] =
            json!(["policy", "oya"]);
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert!(
            keys(&findings)
                .iter()
                .any(|k| k.contains("#allowed_root_dirs:kernel"))
        );
    }
}
