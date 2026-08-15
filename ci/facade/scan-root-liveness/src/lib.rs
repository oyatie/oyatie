//! cloud-ci-scan-root-liveness — every coverage-bearing scan root a gate declares
//! must actually resolve, or be declared forward.
//!
//! ## The defect this closes
//!
//! Gate policies declare scan roots. The fleet has an anti-weakening ratchet, so a
//! root cannot be REMOVED without firing a scope-narrowing violation. But nothing
//! detects a declared root that no longer RESOLVES. The two combine badly:
//!
//!   1. a reorg move empties `oya/<service>/`
//!   2. the gate's scan root silently matches nothing — coverage drops to zero
//!   3. the gate still reports GREEN, because it found no violations in no files
//!   4. the anti-narrowing ratchet now BLOCKS deleting the dead root
//!
//! So the ratchet, whose job is to stop coverage shrinking, ends up preserving the
//! evidence of coverage that already shrank. `automation-language-policy` documents
//! this in its own policy comment: a root is "RETAINED because removing a root fires
//! rust_first_automation_scan_scope_narrowing".
//!
//! With ~250 crates still to move out of the legacy `oya/` and `cloud/` roots under
//! ADR-0562, every remaining move can silently blind a gate and nothing reports it.
//!
//! ## The distinction that makes this a real gate
//!
//! Not every declared path that is absent is a defect. Four classes:
//!
//! - COVERAGE-BEARING (`roots`, `scan_roots`, `crate_root_globs`, ...) — a dead entry
//!   means the gate scans less than it claims. THIS is the defect.
//! - VOCABULARY (`exclude_prefixes`, `allowed_paths`, `forbidden_prefixes`) — a dead
//!   entry excludes or permits nothing. Stale, not blinding. Out of scope here.
//! - FORWARD — declared deliberately BEFORE the path exists. `module-membership`
//!   lists `app`, `base` and `policy` as allowed homes so the reorg can land there
//!   without a policy edit. Flagging those would punish exactly the good practice of
//!   declaring a destination ahead of the move.
//! - RETIRED TOMBSTONE — a protected scan term names a root that was deliberately
//!   deleted. The literal remains to preserve the anti-narrowing ceiling, while this
//!   gate turns reappearance into a blocking event instead of silently reviving it.
//!
//! A naive "every declared path must exist" check gets the third class wrong, which
//! is why forward declarations are explicit DATA with a stated reason rather than a
//! heuristic.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

pub const GATE_ID: &str = "cloud-ci-scan-root-liveness";

/// A coverage-bearing scan root resolves to nothing, and is not declared forward.
pub const CODE_DEAD_SCAN_ROOT: &str = "dead_scan_root";
/// A forward declaration whose path now EXISTS. The declaration has served its
/// purpose and must be retired, or it silently becomes an unaudited permanent entry.
pub const CODE_FORWARD_DECLARATION_LANDED: &str = "forward_declaration_landed";
/// A gate policy file declares coverage-bearing roots but is not registered with this
/// gate. Without this, a new gate escapes liveness checking silently — the exact way
/// a fleet-wide checker goes vacuous.
pub const CODE_UNREGISTERED_POLICY_FILE: &str = "unregistered_policy_file";
/// The observed corpus is implausibly small — a collector bug must not read as clean.
pub const CODE_IMPLAUSIBLE_CORPUS: &str = "scan_root_liveness_implausible_corpus";
/// An optional-local allowance is malformed or no longer names a declared coverage root.
pub const CODE_INVALID_OPTIONAL_LOCAL_ROOT: &str = "invalid_optional_local_root";
/// A retired-root tombstone is malformed, overlaps another allowance, or no longer names a
/// currently declared coverage root by its exact full key.
pub const CODE_INVALID_RETIRED_ROOT_TOMBSTONE: &str = "invalid_retired_root_tombstone";
/// A path/glob classified as deliberately retired resolves again. Reintroduction must be an
/// explicit coverage-policy change, never an unnoticed directory birth.
pub const CODE_RETIRED_ROOT_REAPPEARED: &str = "retired_scan_root_reappeared";

pub const VIOLATION_CODES: [&str; 7] = [
    CODE_DEAD_SCAN_ROOT,
    CODE_FORWARD_DECLARATION_LANDED,
    CODE_UNREGISTERED_POLICY_FILE,
    CODE_IMPLAUSIBLE_CORPUS,
    CODE_INVALID_OPTIONAL_LOCAL_ROOT,
    CODE_INVALID_RETIRED_ROOT_TOMBSTONE,
    CODE_RETIRED_ROOT_REAPPEARED,
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Finding {
    pub code: String,
    pub subject: String,
    pub detail: String,
}

/// One declared root, as observed in a gate policy.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DeclaredRoot {
    /// The policy file that declares it.
    pub policy_file: String,
    /// The FULL JSON POINTER it was declared under, e.g.
    /// `/scan/workflow_inline_shell/roots` — not the leaf name. `roots` occurs at
    /// three different nesting levels inside `rust-first-automation-policy.json`
    /// alone, so leaf-name keying would collapse distinct declarations and let one
    /// baselined entry silently tolerate another.
    pub key: String,
    /// The literal declared value (may contain glob metacharacters).
    pub value: String,
    /// Whether the collector could resolve it: a path that exists, or a glob that
    /// matches at least one path.
    pub resolves: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Observed {
    pub roots: Vec<DeclaredRoot>,
    /// Every policy file the collector saw declaring coverage-bearing keys.
    pub policy_files_with_roots: BTreeSet<String>,
}

/// A path declared before it exists, on purpose.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForwardDeclaration {
    pub value: String,
    /// WHY it does not exist yet, and what will create it. Required: an unexplained
    /// forward declaration is indistinguishable from a dead root.
    pub reason: String,
}

#[derive(Clone, Debug, Default)]
pub struct Policy {
    /// Policy files registered with this gate (the completeness universe).
    pub registered_policy_files: BTreeSet<String>,
    /// Policy files deliberately out of scope, with a reason.
    pub exempt_policy_files: BTreeMap<String, String>,
    /// Paths declared ahead of creation, keyed by declared value.
    pub forward_declarations: BTreeMap<String, ForwardDeclaration>,
    /// Frozen, shrink-only debt: dead roots tolerated today.
    pub baselined_dead_roots: BTreeSet<String>,
    /// Ignored machine-local roots that are intentionally absent in clean CI checkouts but remain
    /// coverage-bearing whenever an operator creates them locally.
    pub optional_local_roots: BTreeMap<String, ForwardDeclaration>,
    /// Permanently deleted roots whose protected policy literals remain solely to preserve the
    /// immutable anti-narrowing ceiling. Absence is required; reappearance fails closed.
    pub retired_root_tombstones: BTreeMap<String, ForwardDeclaration>,
    /// False-green floor on the number of declared roots collected.
    pub min_expected_roots: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub verdict: Verdict,
    pub findings: Vec<Finding>,
    pub roots_checked: usize,
    pub dead_tolerated: usize,
    pub retired_tombstones: usize,
}

/// Key used for baselining and forward-declaration lookup. Includes the policy file
/// so the same literal value in two gates is tracked independently — one gate's dead
/// root must not silently license another's.
fn root_key(r: &DeclaredRoot) -> String {
    format!("{}::{}::{}", r.policy_file, r.key, r.value)
}

/// Evaluate root liveness. Pure: no I/O, no clock, no environment.
pub fn evaluate(observed: &Observed, policy: &Policy) -> Report {
    let mut findings: Vec<Finding> = Vec::new();

    let observed_by_key: BTreeMap<String, &DeclaredRoot> = observed
        .roots
        .iter()
        .map(|root| (root_key(root), root))
        .collect();
    for (key, declaration) in &policy.optional_local_roots {
        let matches_declared_root = observed_by_key
            .get(key)
            .is_some_and(|root| root.value == declaration.value);
        if declaration.value.trim().is_empty()
            || declaration.reason.trim().is_empty()
            || !matches_declared_root
        {
            findings.push(Finding {
                code: CODE_INVALID_OPTIONAL_LOCAL_ROOT.to_owned(),
                subject: key.clone(),
                detail: "optional_local_roots entries must uniquely name a currently declared coverage root by full key and carry non-empty value/reason; remove stale entries or repair the declaration".to_owned(),
            });
        }
    }
    for (key, declaration) in &policy.retired_root_tombstones {
        let matches_declared_root = observed_by_key
            .get(key)
            .is_some_and(|root| root.value == declaration.value);
        let overlaps_allowance = policy.forward_declarations.contains_key(key)
            || policy.optional_local_roots.contains_key(key)
            || policy.baselined_dead_roots.contains(key);
        if declaration.value.trim().is_empty()
            || declaration.reason.trim().is_empty()
            || !matches_declared_root
            || overlaps_allowance
        {
            findings.push(Finding {
                code: CODE_INVALID_RETIRED_ROOT_TOMBSTONE.to_owned(),
                subject: key.clone(),
                detail: "retired_root_tombstones entries must uniquely name a currently declared coverage root by full key, carry non-empty value/reason, and not overlap forward/optional/baselined allowances".to_owned(),
            });
        }
    }

    if observed.roots.len() < policy.min_expected_roots {
        findings.push(Finding {
            code: CODE_IMPLAUSIBLE_CORPUS.to_owned(),
            subject: format!("{} roots", observed.roots.len()),
            detail: format!(
                "collected {} declared roots, below the floor of {}; the collector is broken or \
                 the policy glob stopped matching — this is a gate failure, never coverage",
                observed.roots.len(),
                policy.min_expected_roots
            ),
        });
    }

    // COMPLETENESS: a policy file declaring roots must be registered or exempt,
    // otherwise a newly-added gate silently escapes liveness checking.
    for file in &observed.policy_files_with_roots {
        if policy.registered_policy_files.contains(file)
            || policy.exempt_policy_files.contains_key(file)
        {
            continue;
        }
        findings.push(Finding {
            code: CODE_UNREGISTERED_POLICY_FILE.to_owned(),
            subject: file.clone(),
            detail: format!(
                "{file} declares coverage-bearing scan roots but is not in \
                 registered_policy_files (nor exempt_policy_files with a reason). Register it, \
                 or this gate silently stops covering it — which is the same class of blind spot \
                 it exists to detect."
            ),
        });
    }

    let mut dead_tolerated = 0usize;
    let mut retired_tombstones = 0usize;

    for r in &observed.roots {
        let key = root_key(r);
        if r.resolves {
            if policy.retired_root_tombstones.contains_key(&key) {
                findings.push(Finding {
                    code: CODE_RETIRED_ROOT_REAPPEARED.to_owned(),
                    subject: key.clone(),
                    detail: format!(
                        "retired coverage root `{}` resolves again. Remove the tombstone only in an explicit reintroduction that reviews every declaring gate and its anti-narrowing boundary.",
                        r.value
                    ),
                });
            }
            // A forward declaration whose path now exists has done its job.
            if policy.forward_declarations.contains_key(&key) {
                findings.push(Finding {
                    code: CODE_FORWARD_DECLARATION_LANDED.to_owned(),
                    subject: key.clone(),
                    detail: format!(
                        "`{}` was declared forward, but the path now resolves. Remove it from \
                         forward_declarations — a landed declaration left in the list becomes a \
                         permanent unaudited exemption that would hide the path going dead later.",
                        r.value
                    ),
                });
            }
            continue;
        }
        // Does not resolve.
        if policy.forward_declarations.contains_key(&key) {
            continue; // declared ahead of creation, with a stated reason
        }
        if policy.optional_local_roots.contains_key(&key) {
            continue; // absent by default; still scanned whenever local runtime state exists
        }
        if policy.retired_root_tombstones.contains_key(&key) {
            retired_tombstones += 1;
            continue; // deliberate permanent absence; reappearance is checked above
        }
        if policy.baselined_dead_roots.contains(&key) {
            dead_tolerated += 1;
            continue;
        }
        findings.push(Finding {
            code: CODE_DEAD_SCAN_ROOT.to_owned(),
            subject: key,
            detail: format!(
                "{} declares coverage-bearing root `{}` under `{}`, which resolves to NOTHING. \
                 The gate scans less than it claims and will report GREEN over an empty set. If \
                 the path moved, repoint it. If it is gone for good, remove it (and expect the \
                 anti-narrowing ratchet to require that removal be the SUBJECT of the change, not \
                 a side effect). If it does not exist YET, add it to forward_declarations with the \
                 reason and what will create it.",
                r.policy_file, r.value, r.key
            ),
        });
    }

    findings.sort();
    let verdict = if findings.is_empty() {
        Verdict::Green
    } else {
        Verdict::Red
    };
    Report {
        verdict,
        findings,
        roots_checked: observed.roots.len(),
        dead_tolerated,
        retired_tombstones,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(file: &str, key: &str, value: &str, resolves: bool) -> DeclaredRoot {
        DeclaredRoot {
            policy_file: file.to_owned(),
            key: key.to_owned(),
            value: value.to_owned(),
            resolves,
        }
    }

    fn observed(roots: Vec<DeclaredRoot>) -> Observed {
        let files = roots.iter().map(|r| r.policy_file.clone()).collect();
        Observed {
            roots,
            policy_files_with_roots: files,
        }
    }

    fn policy(files: &[&str]) -> Policy {
        Policy {
            registered_policy_files: files.iter().map(|f| (*f).to_owned()).collect(),
            ..Policy::default()
        }
    }

    #[test]
    fn all_roots_resolving_is_green() {
        let o = observed(vec![
            root("p.json", "roots", "libs", true),
            root("p.json", "roots", "ci", true),
        ]);
        let r = evaluate(&o, &policy(&["p.json"]));
        assert_eq!(r.verdict, Verdict::Green);
        assert_eq!(r.roots_checked, 2);
    }

    /// THE CORE DEFECT: a root that resolves to nothing means silent blindness.
    #[test]
    fn dead_scan_root_fails_closed() {
        let o = observed(vec![root("p.json", "roots", "cloud/cloud-ci", false)]);
        let r = evaluate(&o, &policy(&["p.json"]));
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings[0].code, CODE_DEAD_SCAN_ROOT);
        assert!(r.findings[0].detail.contains("resolves to NOTHING"));
        // The remedy must name the ratchet interaction, else the reader tries to
        // delete the root and gets blocked with no explanation.
        assert!(r.findings[0].detail.contains("anti-narrowing ratchet"));
    }

    /// FORWARD DECLARATIONS: `app`/`base` are declared before the reorg creates them.
    /// Flagging these would punish declaring a destination ahead of the move.
    #[test]
    fn forward_declared_absent_path_is_green() {
        let o = observed(vec![root("m.json", "scan_roots", "app", false)]);
        let mut p = policy(&["m.json"]);
        p.forward_declarations.insert(
            "m.json::scan_roots::app".to_owned(),
            ForwardDeclaration {
                value: "app".to_owned(),
                reason: "ADR-0562 destination; created when the first product app lands".to_owned(),
            },
        );
        let r = evaluate(&o, &p);
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
    }

    /// A forward declaration that LANDED must be retired, or it becomes a permanent
    /// unaudited exemption that would hide the path dying later.
    #[test]
    fn landed_forward_declaration_must_be_retired() {
        let o = observed(vec![root("m.json", "scan_roots", "app", true)]);
        let mut p = policy(&["m.json"]);
        p.forward_declarations.insert(
            "m.json::scan_roots::app".to_owned(),
            ForwardDeclaration {
                value: "app".to_owned(),
                reason: "not yet created".to_owned(),
            },
        );
        let r = evaluate(&o, &p);
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings[0].code, CODE_FORWARD_DECLARATION_LANDED);
    }

    #[test]
    fn baselined_dead_root_is_tolerated_and_counted() {
        let o = observed(vec![root("p.json", "roots", "bin", false)]);
        let mut p = policy(&["p.json"]);
        p.baselined_dead_roots
            .insert("p.json::roots::bin".to_owned());
        let r = evaluate(&o, &p);
        assert_eq!(r.verdict, Verdict::Green);
        assert_eq!(r.dead_tolerated, 1);
    }

    #[test]
    fn absent_optional_local_root_is_green_without_becoming_dead_root_debt() {
        let o = observed(vec![root("p.json", "/scan/roots", ".codex", false)]);
        let mut p = policy(&["p.json"]);
        p.optional_local_roots.insert(
            "p.json::/scan/roots::.codex".to_owned(),
            ForwardDeclaration {
                value: ".codex".to_owned(),
                reason: "ignored machine-local runtime overlay".to_owned(),
            },
        );
        let r = evaluate(&o, &p);
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
        assert_eq!(r.dead_tolerated, 0);
    }

    #[test]
    fn stale_or_malformed_optional_local_root_is_red() {
        let o = observed(vec![root("p.json", "/scan/roots", ".codex", false)]);
        let mut p = policy(&["p.json"]);
        p.optional_local_roots.insert(
            "p.json::/scan/roots::.cursor".to_owned(),
            ForwardDeclaration {
                value: ".cursor".to_owned(),
                reason: String::new(),
            },
        );
        let r = evaluate(&o, &p);
        assert_eq!(r.verdict, Verdict::Red);
        assert!(
            r.findings
                .iter()
                .any(|finding| finding.code == CODE_INVALID_OPTIONAL_LOCAL_ROOT)
        );
    }

    #[test]
    fn absent_retired_root_tombstone_is_green_and_reappearance_is_red() {
        let key = "p.json::/scan/roots::cloud";
        let mut p = policy(&["p.json"]);
        p.retired_root_tombstones.insert(
            key.to_owned(),
            ForwardDeclaration {
                value: "cloud".to_owned(),
                reason: "root deliberately deleted".to_owned(),
            },
        );

        let absent = evaluate(
            &observed(vec![root("p.json", "/scan/roots", "cloud", false)]),
            &p,
        );
        assert_eq!(absent.verdict, Verdict::Green, "{:?}", absent.findings);
        assert_eq!(absent.retired_tombstones, 1);

        let reappeared = evaluate(
            &observed(vec![root("p.json", "/scan/roots", "cloud", true)]),
            &p,
        );
        assert_eq!(reappeared.verdict, Verdict::Red);
        assert!(reappeared
            .findings
            .iter()
            .any(|finding| finding.code == CODE_RETIRED_ROOT_REAPPEARED));
    }

    #[test]
    fn stale_or_overlapping_retired_root_tombstone_is_red() {
        let o = observed(vec![root("p.json", "/scan/roots", "cloud", false)]);
        let mut p = policy(&["p.json"]);
        let key = "p.json::/scan/roots::cloud".to_owned();
        p.retired_root_tombstones.insert(
            key.clone(),
            ForwardDeclaration {
                value: "cloud".to_owned(),
                reason: "root deliberately deleted".to_owned(),
            },
        );
        p.baselined_dead_roots.insert(key);
        let r = evaluate(&o, &p);
        assert!(r
            .findings
            .iter()
            .any(|finding| finding.code == CODE_INVALID_RETIRED_ROOT_TOMBSTONE));
    }

    /// COMPLETENESS: the property that stops THIS gate going vacuous. A new gate
    /// policy declaring roots must be registered, or it escapes silently.
    #[test]
    fn unregistered_policy_file_fails_closed() {
        let o = observed(vec![root("brand-new-gate.json", "roots", "libs", true)]);
        let r = evaluate(&o, &policy(&["p.json"]));
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings[0].code, CODE_UNREGISTERED_POLICY_FILE);
    }

    #[test]
    fn exempt_policy_file_with_reason_is_accepted() {
        let o = observed(vec![root("fixture.json", "roots", "nowhere", false)]);
        let mut p = policy(&[]);
        p.exempt_policy_files.insert(
            "fixture.json".to_owned(),
            "test fixture, not a live gate".to_owned(),
        );
        // Exempt from COMPLETENESS, but its roots are still evaluated — exemption is
        // about registration, not about licensing dead roots.
        let r = evaluate(&o, &p);
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings[0].code, CODE_DEAD_SCAN_ROOT);
    }

    /// The same literal value in two different gates is tracked separately: one
    /// gate's baselined dead root must not license another's.
    #[test]
    fn baseline_is_scoped_per_policy_file() {
        let o = observed(vec![
            root("a.json", "roots", "gone", false),
            root("b.json", "roots", "gone", false),
        ]);
        let mut p = policy(&["a.json", "b.json"]);
        p.baselined_dead_roots
            .insert("a.json::roots::gone".to_owned());
        let r = evaluate(&o, &p);
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].subject.starts_with("b.json"));
    }

    #[test]
    fn implausible_corpus_fails_rather_than_reporting_clean() {
        let o = observed(vec![]);
        let mut p = policy(&[]);
        p.min_expected_roots = 100;
        let r = evaluate(&o, &p);
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings[0].code, CODE_IMPLAUSIBLE_CORPUS);
    }

    #[test]
    fn findings_are_deterministically_ordered() {
        let o = observed(vec![
            root("z.json", "roots", "gone", false),
            root("a.json", "roots", "gone", false),
        ]);
        let r = evaluate(&o, &policy(&["a.json", "z.json"]));
        let subjects: Vec<&str> = r.findings.iter().map(|f| f.subject.as_str()).collect();
        assert_eq!(subjects, vec!["a.json::roots::gone", "z.json::roots::gone"]);
    }

    #[test]
    fn every_emitted_code_is_registered() {
        let o = Observed {
            roots: vec![
                root("p.json", "roots", "dead", false),
                root("p.json", "roots", "landed", true),
                root("p.json", "roots", "reappeared", true),
            ],
            policy_files_with_roots: ["p.json".to_owned(), "unregistered.json".to_owned()]
                .into_iter()
                .collect(),
        };
        let mut p = policy(&["p.json"]);
        p.min_expected_roots = 100;
        p.forward_declarations.insert(
            "p.json::roots::landed".to_owned(),
            ForwardDeclaration {
                value: "landed".to_owned(),
                reason: "x".to_owned(),
            },
        );
        p.optional_local_roots.insert(
            "p.json::roots::missing-optional".to_owned(),
            ForwardDeclaration {
                value: "missing-optional".to_owned(),
                reason: "intentionally invalid aggregate reachability fixture".to_owned(),
            },
        );
        p.retired_root_tombstones.insert(
            "p.json::roots::reappeared".to_owned(),
            ForwardDeclaration {
                value: "reappeared".to_owned(),
                reason: "intentionally reappeared aggregate reachability fixture".to_owned(),
            },
        );
        p.retired_root_tombstones.insert(
            "p.json::roots::missing-tombstone".to_owned(),
            ForwardDeclaration {
                value: "missing-tombstone".to_owned(),
                reason: "intentionally invalid aggregate reachability fixture".to_owned(),
            },
        );
        let r = evaluate(&o, &p);
        for f in &r.findings {
            assert!(
                VIOLATION_CODES.contains(&f.code.as_str()),
                "unregistered {}",
                f.code
            );
        }
        let codes: BTreeSet<&str> = r.findings.iter().map(|f| f.code.as_str()).collect();
        assert_eq!(
            codes.len(),
            VIOLATION_CODES.len(),
            "all codes reachable: {codes:?}"
        );
    }
}
