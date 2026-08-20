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
//! ## The mirror defect: a LIVE root nobody declared
//!
//! Everything above is about a declared root that stopped resolving. The mirror case
//! is strictly worse, and this gate was blind to it until 2026-08-19: a root that
//! resolves perfectly well and that the gate never names. Dead roots at least leave a
//! trace in the policy that someone can read; an undeclared root leaves nothing at
//! all, and there is no ratchet interaction to make it awkward — the gate is simply
//! quiet about a part of the tree, forever, and every one of its green runs is honest
//! about a corpus that is not the corpus.
//!
//! It is the same shrink, arriving from the other side. A reorg move can blind a gate
//! two ways: by emptying a root it declares (dead_scan_root) or by materialising a
//! root it never declared (this code). `embedded-asset-hermeticity` declares sixteen
//! scan roots, of which thirteen are live, and never names nineteen other live roots —
//! among them `comms/`, a registered capability holding 24 crate directories. A
//! dangling embedded asset planted in comms/ leaves that gate GREEN, mutation-proven
//! twice. Nothing in the fleet reported it, because nothing was looking for the
//! ABSENCE of a declaration.
//!
//! The fix is the one the root `Cargo.toml` already made one level up: stop
//! enumerating instances, describe the SHAPE. Its member array replaced 24
//! per-capability globs with four (`*/core/*`, `*/ports/*`, `*/adapters/*`,
//! `*/facade/*`) precisely because "the array was a mutex again, one level up, which
//! is why every reorg move serialized on this file". The gates never followed —
//! `layer-dependency-acyclicity` still carries the 31-glob per-capability form the
//! workspace retired, and is missing the registered `policy` capability entirely.
//! This code does not do the widening (that is per-gate work); it makes the gap
//! VISIBLE and blocking, so the widening cannot be deferred silently.
//!
//! Two judgements make it a gate rather than a nag:
//!
//! - WHICH ROOTS EXIST is not this gate's opinion. It is
//!   `governance/capability-registry.json`, the ADR-0562 CLOSED registry, intersected
//!   with what is actually on disk — plus legacy roots (`oya/`, `libs/`, `tools/`,
//!   `infra/`) that predate the registry and are therefore carried explicitly, each
//!   with a written deletion condition, exactly as the root `Cargo.toml` carries its
//!   legacy globs. Deriving the universe is the whole point: a capability that lands
//!   is expected everywhere BY CONSTRUCTION, with no edit to this file.
//! - WHICH SITES OWE THE FULL SET is derived, not hand-listed. A declaration site
//!   (one array, keyed by policy file + JSON pointer) is ROOT-ENUMERATING when its own
//!   vocabulary is already top-level roots: at least [`ROOT_ENUMERATION_MIN_HITS`] of
//!   its entries are root references into the live universe. That is what separates
//!   `embedded-asset-hermeticity`'s `scan_roots` (thirteen live roots named, nineteen
//!   missing) from `operator-secret-rbac`'s `manifest_paths` (three specific YAML
//!   files, which owe nothing) with no per-gate list to maintain and no threshold
//!   applied to a site that never spoke in roots at all.
//!
//! SCOPE LIMIT, stated so it is not silently lost: the universe is the registry plus
//! carried legacy roots. Top-level directories outside ADR-0562's closed root
//! vocabulary — `templates/`, `scripts/`, `specs/`, `docs/`, `benchmarks/` and the
//! rest — are content roots, not capability roots, and are NOT expected of every
//! site here. `automation-language-policy` not scanning `templates/` for shell is a
//! real exposure and it is out of this code's reach; it belongs to that gate's own
//! widening.
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
/// A root-enumerating declaration site omits a root that is LIVE in the governed
/// universe. The gate is quiet about a real part of the tree and its green is honest
/// about a corpus that is not the corpus. The mirror of `dead_scan_root`.
pub const CODE_UNDECLARED_LIVE_ROOT: &str = "undeclared_live_root";

pub const VIOLATION_CODES: [&str; 5] = [
    CODE_DEAD_SCAN_ROOT,
    CODE_FORWARD_DECLARATION_LANDED,
    CODE_UNREGISTERED_POLICY_FILE,
    CODE_IMPLAUSIBLE_CORPUS,
    CODE_UNDECLARED_LIVE_ROOT,
];

/// How many live-universe root references a declaration site must already contain
/// before it is held to the FULL universe.
///
/// Two, not one. One is a coincidence: `canonical-json`'s `governed_roots` is
/// `["specs", "governance"]`, and `governance` alone being a registered meta
/// directory does not make that array a statement about the tree's root vocabulary.
/// Two or more entries drawn from the live universe is a site whose vocabulary IS the
/// root list, and a root list that is missing roots is the defect. Raising this
/// further would start excusing real enumerations: `cli_package_authority` names
/// exactly three (`os`, `infra`, `tools`) and a `-cli` package can be born under any
/// capability, so it owes the full set.
pub const ROOT_ENUMERATION_MIN_HITS: usize = 2;

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
    /// One entry per DECLARATION SITE (`<policy_file>::<json_pointer>`), holding the
    /// set of top-level roots that site references. Sites are the unit here, not
    /// files: `rust-first-automation-policy.json` carries five separate `roots`
    /// arrays with genuinely different scopes, and holding the file to one universe
    /// would either excuse the broad one or drown the narrow ones.
    pub declaration_sites: BTreeMap<String, BTreeSet<String>>,
    /// The governed root universe as observed on disk: the ADR-0562 closed registry
    /// intersected with what exists, plus the explicitly carried legacy roots.
    pub live_roots: BTreeSet<String>,
}

/// The top-level root a declared value REFERENCES, if it references one at all.
///
/// A value speaks in roots when its first segment is a literal directory name and it
/// either stops there (`comms`) or immediately widens (`comms/*/*`,
/// `oya/*/crates/oya-*`, `build/*/*/*`). A value whose second segment is another
/// literal is a specific PATH, not a root declaration —
/// `infra/external-secrets/clustersecretstore-openbao-oya.yaml`,
/// `.github/workflows`, `cloud/cloud-ci` and `oya/office/oya-*` all name one place
/// inside a root rather than the root itself. Getting this distinction right is what
/// keeps `operator-secret-rbac`'s three-YAML `manifest_paths` out of the universe
/// check instead of being told it owes a manifest per capability.
pub fn root_reference(value: &str) -> Option<&str> {
    let mut segments = value.split('/');
    let head = segments.next()?;
    if head.is_empty() || head.contains('*') || head.contains('?') {
        return None;
    }
    match segments.next() {
        None => Some(head),
        Some(next) if next.contains('*') || next.contains('?') => Some(head),
        Some(_) => None,
    }
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
    /// Frozen, shrink-only debt: `<site>::<root>` pairs a root-enumerating site omits
    /// today. A SET, never a count — a count cannot tell "comms/ got declared" from
    /// "the collector stopped seeing comms/".
    pub baselined_undeclared_live_roots: BTreeSet<String>,
    /// Roots removed from the expected universe, with a reason. Not an escape hatch
    /// for inconvenience: a root belongs here only when scanning it would be a
    /// category error, not when scanning it is merely unfinished work.
    pub universe_exclusions: BTreeMap<String, String>,
    /// False-green floor on the size of the governed universe. Without it, a registry
    /// that fails to parse yields an empty universe, every site trivially declares all
    /// zero of its roots, and the new code reports clean — the exact vacuity this
    /// crate exists to detect, reproduced inside it.
    pub min_expected_live_roots: usize,
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
    /// Sites held to the full universe, i.e. classified root-enumerating.
    pub enumerating_sites: usize,
    pub undeclared_tolerated: usize,
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

    // THE MIRROR DEFECT: a live root that a root-enumerating site never names.
    let universe: BTreeSet<&str> = observed
        .live_roots
        .iter()
        .map(String::as_str)
        .filter(|root| !policy.universe_exclusions.contains_key(*root))
        .collect();

    if universe.len() < policy.min_expected_live_roots {
        findings.push(Finding {
            code: CODE_IMPLAUSIBLE_CORPUS.to_owned(),
            subject: format!("{} live roots", universe.len()),
            detail: format!(
                "the governed root universe collapsed to {} entries, below the floor of {}; the \
                 capability registry did not load or the tree walk broke. An empty universe makes \
                 every site trivially complete, so this MUST fail rather than read as clean.",
                universe.len(),
                policy.min_expected_live_roots
            ),
        });
    }

    let mut enumerating_sites = 0usize;
    let mut undeclared_tolerated = 0usize;

    for (site, referenced) in &observed.declaration_sites {
        let hits = referenced
            .iter()
            .filter(|value| universe.contains(value.as_str()))
            .count();
        if hits < ROOT_ENUMERATION_MIN_HITS {
            continue; // this site does not speak in roots; it owes no universe
        }
        enumerating_sites += 1;
        for root in &universe {
            if referenced.contains(*root) {
                continue;
            }
            let key = format!("{site}::{root}");
            if policy.baselined_undeclared_live_roots.contains(&key) {
                undeclared_tolerated += 1;
                continue;
            }
            findings.push(Finding {
                code: CODE_UNDECLARED_LIVE_ROOT.to_owned(),
                subject: key,
                detail: format!(
                    "`{site}` enumerates top-level roots ({hits} of them live) but never names \
                     `{root}`, which EXISTS. The gate is silent about that root and stays GREEN \
                     over it — the mirror of a dead scan root, with no policy trace to notice. \
                     Declare it, or better, replace the enumeration with a SHAPE the way the root \
                     Cargo.toml replaced 24 per-capability globs with four; a hand-edited root \
                     list is a list that will be wrong the next time a capability lands."
                ),
            });
        }
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
        enumerating_sites,
        undeclared_tolerated,
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
            ..Observed::default()
        }
    }

    fn site(refs: &[&str]) -> BTreeSet<String> {
        refs.iter().map(|r| (*r).to_owned()).collect()
    }

    /// A site enumerating roots, against a three-root universe.
    fn enumerating(declared: &[&str]) -> Observed {
        Observed {
            declaration_sites: [("g.json::/scan_roots".to_owned(), site(declared))]
                .into_iter()
                .collect(),
            live_roots: site(&["comms", "iam", "storage"]),
            ..Observed::default()
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

    // ----- the mirror defect: a live root nobody declared -----

    /// THE CORE MIRROR DEFECT, in the exact shape the audit mutation-proved:
    /// `embedded-asset-hermeticity` names thirteen live roots and never names
    /// `comms/`, so a dangling embedded asset planted in comms/ leaves it GREEN.
    #[test]
    fn undeclared_live_root_fails_closed() {
        let r = evaluate(&enumerating(&["iam", "storage"]), &Policy::default());
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings[0].code, CODE_UNDECLARED_LIVE_ROOT);
        assert_eq!(r.findings[0].subject, "g.json::/scan_roots::comms");
        assert_eq!(r.enumerating_sites, 1);
        // The remedy must point at the SHAPE fix, not just "add comms" — otherwise
        // the reader closes this instance and leaves the generator in place.
        assert!(r.findings[0].detail.contains("SHAPE"));
    }

    #[test]
    fn fully_declared_enumerating_site_is_green() {
        let r = evaluate(
            &enumerating(&["comms", "iam", "storage"]),
            &Policy::default(),
        );
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
        assert_eq!(r.enumerating_sites, 1);
    }

    /// A glob site declares its root: `comms/*/*` names `comms` just as `comms` does.
    /// Without this, the SHAPE form this gate recommends would itself read as a gap.
    #[test]
    fn glob_form_counts_as_declaring_its_root() {
        let r = evaluate(
            &enumerating(&["comms/*/*", "iam/*/*", "storage/*/*"]),
            &Policy::default(),
        );
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
    }

    /// A site that never spoke in roots — `operator-secret-rbac`'s three specific
    /// YAML manifests — owes no universe. Holding it to one would make the finding
    /// unreadable and the gate would be tuned away rather than fixed.
    #[test]
    fn site_below_the_enumeration_threshold_owes_nothing() {
        let r = evaluate(&enumerating(&["iam"]), &Policy::default());
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
        assert_eq!(r.enumerating_sites, 0);
    }

    #[test]
    fn specific_paths_are_not_root_references() {
        assert_eq!(root_reference("comms"), Some("comms"));
        assert_eq!(root_reference("comms/*/*"), Some("comms"));
        assert_eq!(root_reference("oya/*/crates/oya-*"), Some("oya"));
        assert_eq!(root_reference("build/*/*/*"), Some("build"));
        assert_eq!(root_reference("cloud/cloud-ci"), None);
        assert_eq!(root_reference(".github/workflows"), None);
        assert_eq!(root_reference("oya/office/oya-*"), None);
        assert_eq!(root_reference("infra/external-secrets/store.yaml"), None);
        assert_eq!(root_reference("*/core/*"), None);
    }

    #[test]
    fn baselined_undeclared_root_is_tolerated_and_counted() {
        let mut p = Policy::default();
        p.baselined_undeclared_live_roots
            .insert("g.json::/scan_roots::comms".to_owned());
        let r = evaluate(&enumerating(&["iam", "storage"]), &p);
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
        assert_eq!(r.undeclared_tolerated, 1);
    }

    /// An excluded root leaves the universe entirely: it is not owed, and it is not
    /// counted toward the enumeration threshold either.
    #[test]
    fn universe_exclusion_removes_a_root_from_every_site() {
        let mut p = Policy::default();
        p.universe_exclusions
            .insert("comms".to_owned(), "vendored".to_owned());
        let r = evaluate(&enumerating(&["iam", "storage"]), &p);
        assert_eq!(r.verdict, Verdict::Green, "{:?}", r.findings);
    }

    /// The vacuity guard on the new code. A registry that fails to load yields an
    /// empty universe, under which EVERY site is trivially complete — that must be
    /// a gate failure, never a clean run.
    #[test]
    fn collapsed_universe_fails_rather_than_excusing_every_site() {
        let mut o = enumerating(&["iam", "storage"]);
        o.live_roots.clear();
        let p = Policy {
            min_expected_live_roots: 20,
            ..Policy::default()
        };
        let r = evaluate(&o, &p);
        assert_eq!(r.verdict, Verdict::Red);
        assert_eq!(r.findings[0].code, CODE_IMPLAUSIBLE_CORPUS);
        assert!(r.findings[0].detail.contains("trivially complete"));
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
            declaration_sites: [("p.json::/scan_roots".to_owned(), site(&["iam", "storage"]))]
                .into_iter()
                .collect(),
            live_roots: site(&["comms", "iam", "storage"]),
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
