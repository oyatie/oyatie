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
//! Not every declared path that is absent is a defect. Three classes:
//!
//! - COVERAGE-BEARING (`roots`, `scan_roots`, `crate_root_globs`, ...) — a dead entry
//!   means the gate scans less than it claims. THIS is the defect.
//! - VOCABULARY (`exclude_prefixes`, `allowed_paths`, `forbidden_prefixes`) — a dead
//!   entry excludes or permits nothing. Stale, not blinding. Out of scope here.
//! - FORWARD — declared deliberately BEFORE the path exists. `module-membership`
//!   lists `app`, `base` and `policy` as allowed homes so the reorg can land there
//!   without a policy edit. Flagging those would punish exactly the good practice of
//!   declaring a destination ahead of the move.
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

pub const VIOLATION_CODES: [&str; 4] = [
    CODE_DEAD_SCAN_ROOT,
    CODE_FORWARD_DECLARATION_LANDED,
    CODE_UNREGISTERED_POLICY_FILE,
    CODE_IMPLAUSIBLE_CORPUS,
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
        if policy.registered_policy_files.contains(file) || policy.exempt_policy_files.contains_key(file) {
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

    for r in &observed.roots {
        let key = root_key(r);
        if r.resolves {
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
        p.baselined_dead_roots.insert("p.json::roots::bin".to_owned());
        let r = evaluate(&o, &p);
        assert_eq!(r.verdict, Verdict::Green);
        assert_eq!(r.dead_tolerated, 1);
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
        p.exempt_policy_files
            .insert("fixture.json".to_owned(), "test fixture, not a live gate".to_owned());
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
        p.baselined_dead_roots.insert("a.json::roots::gone".to_owned());
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
        let r = evaluate(&o, &p);
        for f in &r.findings {
            assert!(VIOLATION_CODES.contains(&f.code.as_str()), "unregistered {}", f.code);
        }
        let codes: BTreeSet<&str> = r.findings.iter().map(|f| f.code.as_str()).collect();
        assert_eq!(codes.len(), VIOLATION_CODES.len(), "all codes reachable: {codes:?}");
    }
}
