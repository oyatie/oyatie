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
//! - the same COVERAGE-SCOPE rule extended to the two OTHER hand-maintained
//!   scan-root arrays that a reorg move can silently exit (W2, ADR-0562 §10.29):
//!   tier-field-coverage `governed_service_roots` and tier-dependency
//!   `crate_root_globs` (matched on the glob's FIRST path segment). Each must
//!   carry every capability root AND every crate-owning meta dir. Patching these
//!   one at a time after a move has already failed twice; deriving them from the
//!   closed registry is what kills the CLASS.
//!
//!   NOT derived, deliberately: rust-first-automation
//!   `scan.cli_package_authority.roots`. That array's root set doubles as the
//!   gate's GRANDFATHERING boundary — the dimension is absolute over its roots
//!   with no baseline, so a root is admitted only once it holds no `-cli`
//!   package. `marketplace` (marketplace-dev-cli) and `tenancy` (tenancy-cli)
//!   cannot be admitted today, and neither escape is available: the scan-scope
//!   ratchet (`validate_scan_scope_ceiling`) makes `exclude_prefixes`
//!   shrink-only, and the cross-artifact coverage baseline is asserted
//!   born-EMPTY. Requiring it here would therefore only be satisfiable by
//!   weakening one of those two invariants. The array is still shrink-PROOF
//!   (the same ratchet forbids dropping a term) and every reorg destination it
//!   can zero-delta admit was widened in this change; completeness lands with
//!   the CLI-retirement wave that renames those two packages.
//! - the module-membership `meta_directories` mirror must equal the registry's
//!   own meta ring. It is a projection of the ADR-0562 authority, so a meta dir
//!   added to the registry (e.g. `third-party/` by ADR-0615) and not mirrored
//!   here is exactly the drift this check exists to name.
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
    let Some(entry) = policies.get(slot) else {
        // Fail CLOSED: an absent slot used to return an empty finding set, which
        // reads as GREEN. A corpus that forgets a policy must never look synced.
        findings.insert(desync(&format!("<missing-policy-corpus>@{slot}")));
        return None;
    };
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

    let (Some(membership), Some(root_hygiene), Some(tier), Some(tier_field)) = (
        read_policy(policies, "module_membership", &mut findings),
        read_policy(policies, "root_hygiene", &mut findings),
        read_policy(policies, "tier_dependency", &mut findings),
        read_policy(policies, "tier_field_coverage", &mut findings),
    ) else {
        return findings;
    };

    let membership_roots = string_set(
        membership.document,
        "allowed_top_level_dirs",
        membership.path,
        &mut findings,
    );
    let membership_scan_roots =
        string_set(membership.document, "scan_roots", membership.path, &mut findings);
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
        // Coverage scope: a capability root the membership lint never walks is a
        // silent hole, not a narrower lint.
        if !membership_scan_roots.contains(root) {
            findings.insert(desync(&format!("{}#scan_roots:{root}", membership.path)));
        }
    }

    // Meta dirs: the ADR-0562 closed-set authority is the membership whitelist;
    // root-hygiene must additionally admit the dir or the destination cannot be
    // created at all; and a crate-OWNING meta dir must be inside the scan scope.
    let meta_dirs = meta_directory_roots(registry, &mut findings);
    let membership_meta_mirror = string_set(
        membership.document,
        "meta_directories",
        membership.path,
        &mut findings,
    );
    for (meta, owns_crates) in &meta_dirs {
        if !membership_roots.contains(meta) {
            findings.insert(desync(&format!(
                "{}#allowed_top_level_dirs:{meta}",
                membership.path
            )));
        }
        if !root_hygiene_roots.contains(meta) {
            findings.insert(desync(&format!(
                "{}#allowed_root_dirs:{meta}",
                root_hygiene.path
            )));
        }
        if *owns_crates && !membership_scan_roots.contains(meta) {
            findings.insert(desync(&format!("{}#scan_roots:{meta}", membership.path)));
        }
        // The membership policy mirrors the registry's meta ring; a registry meta
        // dir absent from the mirror is drift in the projection of the authority.
        if !membership_meta_mirror.contains(meta) {
            findings.insert(desync(&format!(
                "{}#meta_directories:{meta}",
                membership.path
            )));
        }
    }

    // COVERAGE-SCOPE, extended (W2). Two more hand-maintained scan-root arrays
    // must each carry every root a reorg move can land crates in: capability
    // roots plus crate-owning meta dirs. A root missing here is the exact
    // "moved crate silently exits the gate" class.
    let mut crate_bearing_roots = capability_roots.clone();
    crate_bearing_roots.extend(
        meta_dirs
            .iter()
            .filter(|(_, owns)| **owns)
            .map(|(dir, _)| dir.clone()),
    );

    let governed_service_roots = string_set(
        tier_field.document,
        "governed_service_roots",
        tier_field.path,
        &mut findings,
    );
    let glob_roots: BTreeSet<String> = string_set(
        tier.document,
        "crate_root_globs",
        tier.path,
        &mut findings,
    )
    .iter()
    .filter_map(|glob| glob.split('/').next().map(str::to_owned))
    .filter(|segment| !segment.is_empty())
    .collect();

    for root in &crate_bearing_roots {
        if !governed_service_roots.contains(root) {
            findings.insert(desync(&format!(
                "{}#governed_service_roots:{root}",
                tier_field.path
            )));
        }
        if !glob_roots.contains(root) {
            findings.insert(desync(&format!("{}#crate_root_globs:{root}", tier.path)));
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
                    { "absorbs_current_dirs": ["policy", "oya/policy"] },
                    { "absorbs_current_dirs": ["cell", "cloud/cloud-cell"] }
                ],
                "meta_directories": [
                    { "dir": "kernel/", "owns_crates": true },
                    { "dir": "governance/", "owns_crates": false }
                ]
            },
            "policies": {
                "module_membership": {
                    "path": "ci/facade/module-membership/capability-membership-policy.json",
                    "document": {
                        "allowed_top_level_dirs": ["policy", "cell", "kernel", "governance"],
                        "scan_roots": ["policy", "cell", "kernel", "governance"],
                        "meta_directories": ["kernel", "governance"]
                    }
                },
                "root_hygiene": {
                    "path": "ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json",
                    "document": { "allowed_root_dirs": ["policy", "cell", "kernel", "governance"] }
                },
                "tier_dependency": {
                    "path": "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json",
                    "document": {
                        "unclassified_roots": ["policy", "cell"],
                        "service_roots": ["cloud", "oya"],
                        "crate_root_globs": ["policy/*/*", "cell/*/*", "kernel/*/*"]
                    }
                },
                "tier_field_coverage": {
                    "path": "ci/facade/service-tier-metadata/tier-field-coverage-policy.json",
                    "document": { "governed_service_roots": ["policy", "cell", "kernel"] }
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
        corpus["policies"]["module_membership"]["document"]["scan_roots"] =
            json!(["cloud", "kernel", "governance"]);
        corpus["policies"]["root_hygiene"]["document"]["allowed_root_dirs"] =
            json!(["cloud", "kernel", "governance"]);
        corpus["policies"]["tier_dependency"]["document"]["crate_root_globs"] =
            json!(["cloud/*/crates/*", "kernel/*/*"]);
        corpus["policies"]["tier_field_coverage"]["document"]["governed_service_roots"] =
            json!(["cloud", "kernel"]);
        // "cloud" is only in service_roots, not unclassified_roots.
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert!(findings.is_empty(), "{:?}", keys(&findings));
    }

    #[test]
    fn a_crate_bearing_root_missing_from_governed_service_roots_is_flagged() {
        // W2: vacating oya/ into app/ silently removes the whole product surface
        // from tier-field-coverage unless the destination is a governed root.
        let mut corpus = green_corpus();
        corpus["policies"]["tier_field_coverage"]["document"]["governed_service_roots"] =
            json!(["policy", "cell"]); // dropped the crate-owning meta dir "kernel"
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec![
                "ci/facade/service-tier-metadata/tier-field-coverage-policy.json#governed_service_roots:kernel"
                    .to_owned()
            ]
        );
    }

    #[test]
    fn a_crate_bearing_root_missing_from_crate_root_globs_is_flagged() {
        let mut corpus = green_corpus();
        corpus["policies"]["tier_dependency"]["document"]["crate_root_globs"] =
            json!(["policy/*/*", "kernel/*/*"]); // dropped every "cell" glob
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec![
                "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json#crate_root_globs:cell"
                    .to_owned()
            ]
        );
    }

    #[test]
    fn a_crate_root_glob_matches_on_its_first_path_segment_only() {
        // `cell/*/*` covers the `cell` root; a glob rooted elsewhere does not.
        let mut corpus = green_corpus();
        corpus["policies"]["tier_dependency"]["document"]["crate_root_globs"] =
            json!(["policy/*/*", "kernel/*/*", "oya/cell/crates/*"]);
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec![
                "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json#crate_root_globs:cell"
                    .to_owned()
            ]
        );
    }



    #[test]
    fn a_registry_meta_dir_missing_from_the_membership_mirror_is_flagged() {
        // ADR-0615 promoted `third-party/` in the registry; the membership policy
        // kept the pre-amendment six-entry mirror. That drift must be named.
        let mut corpus = green_corpus();
        corpus["policies"]["module_membership"]["document"]["meta_directories"] = json!(["kernel"]);
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec![
                "ci/facade/module-membership/capability-membership-policy.json#meta_directories:governance"
                    .to_owned()
            ]
        );
    }

    #[test]
    fn an_absent_policy_slot_fails_closed_instead_of_reading_as_synced() {
        let mut corpus = green_corpus();
        corpus["policies"]
            .as_object_mut()
            .expect("policies object")
            .remove("tier_field_coverage");
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec!["<missing-policy-corpus>@tier_field_coverage".to_owned()]
        );
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
    fn a_meta_dir_missing_from_root_hygiene_is_flagged() {
        // The `app/` blocker: root-hygiene is a default-DENY allowlist, so a
        // declared destination absent from it cannot be created at all.
        let mut corpus = green_corpus();
        corpus["policies"]["root_hygiene"]["document"]["allowed_root_dirs"] =
            json!(["policy", "cell", "governance"]); // dropped "kernel"
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec![
                "ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json#allowed_root_dirs:kernel"
                    .to_owned()
            ]
        );
    }

    #[test]
    fn a_crate_owning_meta_dir_missing_from_scan_roots_is_flagged() {
        // COVERAGE-SCOPE anti-laundering: dropping a crate-owning destination
        // from scan_roots silently removes every crate moved there from the
        // membership lint. It must never be a quiet policy-list trim.
        let mut corpus = green_corpus();
        corpus["policies"]["module_membership"]["document"]["scan_roots"] =
            json!(["policy", "cell", "governance"]); // dropped "kernel"
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec![
                "ci/facade/module-membership/capability-membership-policy.json#scan_roots:kernel"
                    .to_owned()
            ]
        );
    }

    #[test]
    fn a_capability_root_missing_from_scan_roots_is_flagged() {
        let mut corpus = green_corpus();
        corpus["policies"]["module_membership"]["document"]["scan_roots"] =
            json!(["cell", "kernel", "governance"]); // dropped "policy"
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert_eq!(
            keys(&findings),
            vec![
                "ci/facade/module-membership/capability-membership-policy.json#scan_roots:policy"
                    .to_owned()
            ]
        );
    }

    #[test]
    fn a_non_crate_owning_meta_dir_is_not_required_in_scan_roots() {
        // governance/ carries owns_crates:false — walking it for crates is not
        // required, so its absence from scan_roots is not a coverage hole.
        let mut corpus = green_corpus();
        corpus["policies"]["module_membership"]["document"]["scan_roots"] =
            json!(["policy", "cell", "kernel"]); // dropped "governance"
        assert!(evaluate_registry_derived_policy_sync(&corpus).is_empty());
    }

    #[test]
    fn a_meta_dir_without_owns_crates_fails_closed_into_scan_coverage() {
        // An undeclared owns_crates must be read as crate-owning: a new meta dir
        // may never enter the registry outside the membership lint's scan scope.
        let mut corpus = green_corpus();
        corpus["registry"]["meta_directories"] = json!([{ "dir": "app/" }]);
        let findings = evaluate_registry_derived_policy_sync(&corpus);
        assert!(
            findings.iter().any(|f| f.key
                == "ci/facade/module-membership/capability-membership-policy.json#scan_roots:app"),
            "{:?}",
            keys(&findings)
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
