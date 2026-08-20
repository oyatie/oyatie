// governance-check-layered-architecture-discipline LIVE-TREE gate
// (ADR-0148 Cilium L3/L4 + Istio Ambient L7, ADR-0182 gateway vs mesh, ADR-0183 Cedar vs Kyverno,
// ADR-0184 Valkey vs Memcached).
//
// The `#[cfg(test)] mod tests` inside src/lib.rs proves the kernel correct on hand-written
// manifests. It says nothing about this repository, and until this file existed nothing did. The
// crate's only Cargo consumer is marketplace/facade/dev-cli, which no workflow invokes — and that
// runner would find almost nothing even if something did invoke it, because it enumerates
// `cloud/*/`, `oya/*/` and `microservices/*/` only, and the capability reorg (ADR-0562) moved the
// manifests out from under all three roots. Those fixture tests stay exactly where they are; this
// target is ADDED beside them, never in place of them.
//
// The kernel is pure; this is the CALLER that walks the real repository and hands it observations
// as DATA. Walk failures are ERRORS, never omitted observations: a manifest dropped from the
// census because its contents failed to read would quietly shrink the frozen maps, and a shrink
// reads as repair.
//
// IDENTITY IS DECLARED, NOT INFERRED FROM THE PATH. Every violation the kernel emits is keyed on
// the µservice name the caller supplies, and 30 of the 89 tracked manifests declare a
// `"microservice"` that differs from their directory leaf. gateway/manifest.json declares
// "api-gateway" — precisely the name ViolationKind::NorthSouthOnlyMisplaced exempts, and precisely
// the name the Wave-3-I deferral ledger's `api-gateway GatewayAndMeshConflict` row is keyed on.
// Keying on the directory leaf would manufacture a violation against the one service the rule
// excuses AND un-suppress ledger rows that are keyed on declared names. That was found the hard
// way in PR #2156; this file reads `"microservice"` out of the JSON and never looks at the
// directory name.
//
// ONE DOCUMENT PER DECLARED IDENTITY, not one per file. This is the kernel's own documented
// calling convention — `ManifestDocument`'s doc comment says callers pass "the concatenated text
// plus the µservice name", because a µservice is described by several files. Three identities
// today are spread over more than one manifest: `calendar` (app/ and oya/, a live duplicate),
// `developer-sdk` and `plugin-app-store` (a base manifest plus five regulatory pack overlays
// each). Auditing per FILE reports ten MeshTierUnderclaimed violations against pack overlays that
// carry no `mesh_layering` block because their base manifest already declares `cilium_l4: true`
// and `ambient_ztunnel: true` — ten findings whose only honest repair is "audit the µservice, not
// the fragment". Grouping first removes them.
//
// WHAT GROUPING COSTS, stated rather than papered over: concatenation can mask an under-claim (one
// fragment declares the tier for all of them) and can manufacture a conflict (two fragments each
// individually consistent, contradicting each other once merged). Both are properties of treating
// a µservice as the unit, which is what the doctrine is about; neither is hypothetical-free. Today
// it affects exactly the three identities named above, and the census below prints every
// contributing path so which files were merged is never a guess.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use check_layered_architecture_discipline::{
    LayeredArchitectureViolation, ManifestDocument, audit_all_violations,
};

const POLICY_PATH: &str = "governance/check/layered-architecture-discipline/\
                           layered-architecture-discipline-policy.json";
const DEFERRAL_LEDGER: &str =
    "registry/layered-architecture-discipline/wave-3-i-deferred-manifest-violations.tsv";
const MAX_SCANNED_BYTES: u64 = 4_194_304;

struct Policy {
    min_tracked_files: usize,
    min_manifest_documents: usize,
    min_declared_identities: usize,
    frozen_active_violations: BTreeMap<String, usize>,
    frozen_deferred_violations: BTreeMap<String, usize>,
    frozen_manifests_without_declared_identity: BTreeSet<String>,
}

struct Observed {
    manifest_documents: usize,
    tracked_files: usize,
    /// Declared µservice identity -> every tracked manifest that declares it, in path order.
    identities: BTreeMap<String, Vec<String>>,
    /// Tracked `manifest.json` files that declare no `"microservice"` identity, and are therefore
    /// outside the subject. Frozen so the exclusion is a reviewable list rather than a silent
    /// hole: deleting the `"microservice"` key from a manifest would otherwise be a way to walk
    /// straight out of this gate.
    without_declared_identity: BTreeSet<String>,
    /// `<declared µservice>::<ViolationKind>` -> multiplicity, for violations the Wave-3-I
    /// deferral ledger does NOT cover.
    active: BTreeMap<String, usize>,
    /// The same key shape, for violations the ledger DOES cover. Frozen too, so a ledger row
    /// cannot silently start absorbing a violation it was never reviewed for.
    deferred: BTreeMap<String, usize>,
    /// Every violation, retained whole, for the human-readable report only.
    violations: Vec<LayeredArchitectureViolation>,
    /// Ledger rows that currently suppress nothing. Reported as evidence, never asserted on.
    unused_deferrals: Vec<(String, String)>,
}

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(POLICY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (the dir holding {POLICY_PATH})");
}

fn load_policy(root: &Path) -> Policy {
    let raw = std::fs::read_to_string(root.join(POLICY_PATH)).expect("read policy");
    let doc: serde_json::Value = serde_json::from_str(&raw).expect("policy parses");
    let number = |key: &str| -> usize {
        usize::try_from(
            doc[key]
                .as_u64()
                .unwrap_or_else(|| panic!("policy field {key} missing or not a number")),
        )
        .expect("policy number fits usize")
    };
    let map = |key: &'static str| -> BTreeMap<String, usize> {
        doc[key]
            .as_object()
            .unwrap_or_else(|| panic!("policy field {key} missing or not an object"))
            .iter()
            .map(|(entry, value)| {
                let count = usize::try_from(
                    value
                        .as_u64()
                        .unwrap_or_else(|| panic!("{key}[{entry}] is not a number")),
                )
                .expect("count fits usize");
                (entry.clone(), count)
            })
            .collect()
    };
    Policy {
        min_tracked_files: number("min_tracked_files"),
        min_manifest_documents: number("min_manifest_documents"),
        min_declared_identities: number("min_declared_identities"),
        frozen_active_violations: map("frozen_active_violations"),
        frozen_deferred_violations: map("frozen_deferred_violations"),
        frozen_manifests_without_declared_identity:
            doc["frozen_manifests_without_declared_identity"]
                .as_array()
                .expect(
                    "policy field frozen_manifests_without_declared_identity missing or not array",
                )
                .iter()
                .map(|entry| {
                    entry
                        .as_str()
                        .expect("excluded manifest path is a string")
                        .to_owned()
                })
                .collect(),
    }
}

/// The tracked file list, from git — the same corpus boundary every other live gate here uses.
///
/// Walking the working tree instead would measure a different corpus than CI does the moment an
/// ignored `manifest.json` exists on disk, and with the maps pinned by equality that is a red gate
/// CI cannot reproduce.
fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|e| format!("git ls-files failed to start: {e}"))?;
    if !out.status.success() {
        return Err(format!("git ls-files exited with {}", out.status));
    }
    let text =
        String::from_utf8(out.stdout).map_err(|e| format!("git ls-files output not UTF-8: {e}"))?;
    Ok(text
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Basename match, never a suffix match. `ends_with("manifest.json")` would also swallow
/// `freshness-manifest.json`, `mirror-manifest.json`, `archive-manifest.json` and
/// `client-manifest.json` — four tracked files that are a dependency mirror index, an advisory
/// mirror index, an IaC archive index and a frontend asset manifest respectively. None is a
/// µservice manifest, and `client-manifest.json` is the subject of a DIFFERENT gate
/// (check-client-stack-discipline), so admitting them would have this gate rule on documents it
/// does not understand.
fn is_microservice_manifest_path(relative: &str) -> bool {
    relative == "manifest.json" || relative.ends_with("/manifest.json")
}

fn read_tracked(root: &Path, relative: &str) -> Result<Option<String>, String> {
    let path = root.join(relative);
    // Every failure below is an ERROR, never an omitted observation.
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("metadata {relative} failed: {e} (tracked but unreadable)"))?;
    if !metadata.is_file() {
        return Ok(None); // a tracked symlink to a directory carries no manifest text
    }
    if metadata.len() > MAX_SCANNED_BYTES {
        return Err(format!(
            "{relative} is {} bytes, over the {MAX_SCANNED_BYTES}-byte scan cap — raise the cap \
             deliberately rather than dropping the manifest from the census",
            metadata.len()
        ));
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("read {relative} failed: {e}"))?;
    Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
}

/// The DECLARED µservice identity, or `None` when the document declares none.
///
/// Parsed as JSON rather than grepped: `"microservice": "api-gateway"` quoted inside a prose field
/// would satisfy a substring scan while declaring nothing.
fn declared_identity(contents: &str) -> Option<String> {
    let doc: serde_json::Value = serde_json::from_str(contents).ok()?;
    let name = doc.get("microservice")?.as_str()?.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// The Wave-3-I deferral ledger, parsed with the SAME rules the only existing runner uses
/// (marketplace/facade/dev-cli/src/layered_architecture_gates.rs): three tab-separated non-empty
/// fields, `#` comments and blank lines skipped, duplicates refused. A malformed ledger is an
/// ERROR — a ledger that fails to parse must not degrade into "nothing is deferred", because that
/// would flip reviewed deferrals into blocking findings on a lane that touched none of them.
fn load_deferrals(root: &Path) -> Result<BTreeSet<(String, String)>, String> {
    let raw = std::fs::read_to_string(root.join(DEFERRAL_LEDGER))
        .map_err(|e| format!("deferral ledger unreadable {DEFERRAL_LEDGER}: {e}"))?;
    let mut out = BTreeSet::new();
    for (index, line) in raw.lines().enumerate() {
        let line_no = index + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 3 {
            return Err(format!(
                "{DEFERRAL_LEDGER}:{line_no} row must be \
                 <microservice><tab><violation_kind><tab><reason>"
            ));
        }
        if fields.iter().any(|field| field.trim().is_empty()) {
            return Err(format!(
                "{DEFERRAL_LEDGER}:{line_no} row fields must all be non-empty; a blank reason is \
                 an unexplained mute, not a deferral"
            ));
        }
        if !out.insert((fields[0].trim().to_owned(), fields[1].trim().to_owned())) {
            return Err(format!(
                "{DEFERRAL_LEDGER}:{line_no} duplicate deferral for {}/{}",
                fields[0], fields[1]
            ));
        }
    }
    Ok(out)
}

fn key(violation: &LayeredArchitectureViolation) -> String {
    format!("{}::{:?}", violation.microservice, violation.kind)
}

/// What the manifest walk produced, before the kernel is asked anything.
struct Corpus {
    docs: Vec<ManifestDocument>,
    identities: BTreeMap<String, Vec<String>>,
    without_declared_identity: BTreeSet<String>,
    manifest_documents: usize,
}

/// Group every tracked manifest by the identity it declares, and hand the kernel one document per
/// identity carrying the concatenated text of all of them.
fn corpus(root: &Path, tracked: &[String]) -> Result<Corpus, String> {
    let mut identities: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut bodies: BTreeMap<String, String> = BTreeMap::new();
    let mut without_declared_identity = BTreeSet::new();
    let mut manifest_documents = 0usize;

    for relative in tracked {
        if !is_microservice_manifest_path(relative) {
            continue;
        }
        let Some(contents) = read_tracked(root, relative)? else {
            continue;
        };
        manifest_documents += 1;
        let Some(identity) = declared_identity(&contents) else {
            without_declared_identity.insert(relative.clone());
            continue;
        };
        identities
            .entry(identity.clone())
            .or_default()
            .push(relative.clone());
        let body = bodies.entry(identity).or_default();
        body.push_str(&contents);
        body.push('\n');
    }

    let docs = identities
        .iter()
        .map(|(identity, paths)| ManifestDocument {
            microservice: identity.clone(),
            path: paths.join(" + "),
            contents: bodies[identity].clone(),
        })
        .collect();
    Ok(Corpus {
        docs,
        identities,
        without_declared_identity,
        manifest_documents,
    })
}

fn observe(root: &Path) -> Result<Observed, String> {
    let tracked = tracked_files(root)?;
    let deferrals = load_deferrals(root)?;
    let corpus = corpus(root, &tracked)?;

    let (_, violations) = audit_all_violations(corpus.docs);

    let mut active: BTreeMap<String, usize> = BTreeMap::new();
    let mut deferred: BTreeMap<String, usize> = BTreeMap::new();
    let mut used_deferrals: BTreeSet<(String, String)> = BTreeSet::new();
    for violation in &violations {
        let ledger_key = (
            violation.microservice.clone(),
            format!("{:?}", violation.kind),
        );
        if deferrals.contains(&ledger_key) {
            used_deferrals.insert(ledger_key);
            *deferred.entry(key(violation)).or_default() += 1;
        } else {
            *active.entry(key(violation)).or_default() += 1;
        }
    }
    let unused_deferrals = deferrals.difference(&used_deferrals).cloned().collect();

    Ok(Observed {
        manifest_documents: corpus.manifest_documents,
        tracked_files: tracked.len(),
        identities: corpus.identities,
        without_declared_identity: corpus.without_declared_identity,
        active,
        deferred,
        violations,
        unused_deferrals,
    })
}

/// The live walk, done ONCE for the whole binary: it is a pure function of the tree, so every test
/// re-walking it would recompute the same answer over the whole tracked file list.
fn live() -> &'static (Policy, Observed) {
    static LIVE: OnceLock<(Policy, Observed)> = OnceLock::new();
    LIVE.get_or_init(|| {
        let root = repo_root();
        let policy = load_policy(&root);
        let observed = observe(&root).expect("live walk");
        (policy, observed)
    })
}

fn census(observed: &Observed) -> String {
    let mut out = format!(
        "census: {} tracked manifest.json over {} tracked files; {} declare a `microservice` \
         identity and group into {} audited µservices, {} declare none and are excluded; {} \
         violations ({} active keys, {} deferred keys), {} ledger rows suppressing nothing\n",
        observed.manifest_documents,
        observed.tracked_files,
        observed.manifest_documents - observed.without_declared_identity.len(),
        observed.identities.len(),
        observed.without_declared_identity.len(),
        observed.violations.len(),
        observed.active.len(),
        observed.deferred.len(),
        observed.unused_deferrals.len(),
    );
    out.push_str("  VIOLATIONS (identity :: kind — contributing manifests):\n");
    for violation in &observed.violations {
        out.push_str(&format!(
            "    {}::{:?} — {}\n",
            violation.microservice, violation.kind, violation.manifest_path
        ));
    }
    out.push_str("  IDENTITIES SPREAD OVER MORE THAN ONE MANIFEST:\n");
    for (identity, paths) in &observed.identities {
        if paths.len() > 1 {
            out.push_str(&format!("    {identity}: {}\n", paths.join(", ")));
        }
    }
    out.push_str("  LEDGER ROWS SUPPRESSING NOTHING:\n");
    for (identity, kind) in &observed.unused_deferrals {
        out.push_str(&format!("    {identity} {kind}\n"));
    }
    out
}

/// ANTI-VACUITY, asserted before any equality below is read.
///
/// A ratchet pinned by equality cannot distinguish "the manifests were repaired" from "the walk
/// collapsed"; both drive the observed maps toward empty. These floors are the machine oracle that
/// separates them. Every floor counts SUBJECT MATERIAL — tracked paths, manifest documents,
/// audited identities — never findings, so declaring `cilium_l4: true` on a manifest moves the
/// frozen map and leaves all three floors exactly where they are. No floor here can red on honest
/// progress.
///
/// There is deliberately NO floor on the violation counts and NONE on the deferral-ledger row
/// count. All three have zero as their intended destination, and a floor on a zero-target term
/// reds precisely when the work succeeds, which gets the guard deleted rather than the corpus
/// fixed.
#[test]
fn the_manifest_corpus_is_intact() {
    let (policy, observed) = live();
    assert!(
        observed.tracked_files >= policy.min_tracked_files,
        "git ls-files returned {} tracked paths, below the floor of {} — the corpus walk is broken \
         and every count below is meaningless\n{}",
        observed.tracked_files,
        policy.min_tracked_files,
        census(observed)
    );
    assert!(
        observed.manifest_documents >= policy.min_manifest_documents,
        "{} tracked manifest.json found, below the floor of {}. Service manifests do not disappear \
         in bulk; a drop here is a narrowed scan, and a narrowed scan reports a perfectly layered \
         fleet it never read\n{}",
        observed.manifest_documents,
        policy.min_manifest_documents,
        census(observed)
    );
    assert!(
        observed.identities.len() >= policy.min_declared_identities,
        "{} declared µservice identities, below the floor of {} — either the corpus shrank or \
         `\"microservice\"` stopped being the declaration key, and in the second case every \
         manifest silently left the subject\n{}",
        observed.identities.len(),
        policy.min_declared_identities,
        census(observed)
    );
}

fn drift(frozen: &BTreeMap<String, usize>, seen: &BTreeMap<String, usize>) -> Vec<String> {
    let keys: BTreeMap<&String, ()> = frozen
        .keys()
        .chain(seen.keys())
        .map(|entry| (entry, ()))
        .collect();
    keys.into_keys()
        .filter_map(|entry| {
            let observed = seen.get(entry).copied().unwrap_or(0);
            let want = frozen.get(entry).copied().unwrap_or(0);
            (observed != want).then(|| format!("  {entry}: observed {observed}, frozen {want}"))
        })
        .collect()
}

/// THE GATE: a SHRINK-ONLY, TWO-SIDED ratchet on the MAP of layering violations the Wave-3-I
/// deferral ledger does not cover.
///
/// Keys, not a count. `<declared µservice>::<ViolationKind>` names the service and the boundary it
/// crossed, and is the SAME key shape the deferral ledger uses, so the two mechanisms cannot
/// disagree about what a finding is. A count would tell a reviewer the number moved and nothing
/// about which service moved. The census printed on every failure carries the contributing
/// manifest paths, so attribution is never lost — it is simply not part of the key, because a
/// manifest that MOVES (and several have, through the ADR-0562 capability reorg) must not read as
/// one repair plus one regression.
///
/// The multiplicity is the value because GatewayAndMeshConflict has two independent detection
/// shapes — schema (`north_south_only` + `ambient_waypoint`) and helm annotation
/// (`gateway.networking.k8s.io/managed-by` + `istio.io/dataplane-mode: ambient`) — so one identity
/// can legitimately produce two of the same kind, and a bare set would hide the second appearing
/// or disappearing.
///
/// TWO-SIDED, over the UNION of both key sets. A new violation appears above its pin and blocks; a
/// repaired one falls below its pin and ALSO blocks, forcing the pin down in the same change so
/// the win is recorded.
#[test]
fn active_violations_equal_the_frozen_map() {
    let (policy, observed) = live();
    let drift = drift(&policy.frozen_active_violations, &observed.active);
    assert!(
        drift.is_empty(),
        "layered-architecture drift, per (declared µservice, kind). ABOVE the pin: a µservice \
         crossed a layer boundary — declare `mesh_layering.cilium_l4: true` and `ambient_ztunnel: \
         true` (ADR-0148), or drop the conflicting traffic direction (ADR-0182), or the \
         conflicting policy engine (ADR-0183) or cache backend (ADR-0184). BELOW the pin: lower \
         `frozen_active_violations` in THIS change so the win is recorded, or discover that the \
         scan narrowed and is reporting green over manifests it stopped reading. Re-derive by \
         RUNNING this gate and reading 'observed N' from these lines; never by arithmetic on the \
         old values:\n{}\n{}",
        drift.join("\n"),
        census(observed)
    );
}

/// The DEFERRAL LEDGER IS RATCHETED TOO, and this is the assertion that makes muting safe.
///
/// registry/layered-architecture-discipline/wave-3-i-deferred-manifest-violations.tsv suppresses
/// (µservice, kind) pairs whose repair belongs to a manifest owner this lane may not touch. It is
/// keyed on DECLARED names, which is the whole reason identity is read from the JSON above. But an
/// unratcheted mute is a hole: most of its rows currently suppress nothing, and any one of them
/// would silently absorb a brand-new violation the day that service regresses.
///
/// So the deferred set is frozen by the SAME two-sided equality as the active set. A ledger row
/// that starts covering something new blocks; a deferral whose subject was repaired blocks until
/// both the pin and the now-pointless ledger row come out together. Nothing is hidden by the
/// ledger — every deferred violation is enumerated in `frozen_deferred_violations` where a
/// reviewer reads it, which is the difference between a deferral and a disappearance.
///
/// The rows that suppress nothing are REPORTED in the census and deliberately NOT asserted on.
/// Striking them is a real cleanup, but it is the manifest owners' cleanup, and blocking on it
/// here would make this connection hostage to a burn-down it does not own.
#[test]
fn deferred_violations_equal_the_frozen_map() {
    let (policy, observed) = live();
    let drift = drift(&policy.frozen_deferred_violations, &observed.deferred);
    assert!(
        drift.is_empty(),
        "Wave-3-I deferral drift, per (declared µservice, kind). ABOVE the pin: a deferral row in \
         {DEFERRAL_LEDGER} has started suppressing a violation it was never reviewed for — the row \
         defers a specific known defect, it is not a standing licence for that (µservice, kind) \
         pair. BELOW the pin: the deferred defect was repaired; delete the pin here AND the \
         now-pointless row from the ledger in the same change. Re-derive by RUNNING this gate; \
         never by arithmetic on the old values:\n{}\n{}",
        drift.join("\n"),
        census(observed)
    );
}

/// The subject boundary is frozen, so leaving the subject is a reviewable act.
///
/// A manifest with no `"microservice"` key cannot be attributed to a µservice, so the kernel
/// cannot key a violation on it and the deferral ledger cannot key a deferral on it either. Those
/// documents are excluded — but the exclusion is a LIST, not a rule applied in silence, because
/// deleting one key from gateway/manifest.json would otherwise walk that manifest straight out of
/// this gate with nothing to show for it.
///
/// Two-sided as well: a manifest that GAINS a declared identity leaves this list and must be
/// struck here in the same change, which is also the change where it starts being audited.
#[test]
fn manifests_outside_the_subject_equal_the_frozen_set() {
    let (policy, observed) = live();
    let appeared: Vec<&String> = observed
        .without_declared_identity
        .difference(&policy.frozen_manifests_without_declared_identity)
        .collect();
    let vanished: Vec<&String> = policy
        .frozen_manifests_without_declared_identity
        .difference(&observed.without_declared_identity)
        .collect();
    assert!(
        appeared.is_empty() && vanished.is_empty(),
        "the set of manifests outside this gate's subject moved. NEWLY EXCLUDED (a tracked \
         manifest.json declares no `\"microservice\"`, so nothing audits it — add the declaration, \
         or list it in `frozen_manifests_without_declared_identity` and say why it is not a \
         µservice manifest): {appeared:?}. NO LONGER EXCLUDED (it gained a declared identity, \
         which is progress — strike it from that list in this same change, and expect its \
         violations to land in the active map): {vanished:?}\n{}",
        census(observed)
    );
}

/// The gate is DEMONSTRATED CAPABLE OF FAILING against REAL corpus text, not just fixtures.
///
/// A green ratchet proves nothing on its own: a caller that silently produced zero findings would
/// satisfy every assertion above by reporting a perfectly layered fleet. This plants two
/// independent defect shapes into a copy of a REAL tracked manifest's text and asserts the kernel
/// reddens — and asserts the new violation NAMES THE PLANTED KIND on the planted identity, so the
/// test cannot pass on a finding that was already there.
///
/// The plants are DECLARATIONS, not prose. Several kernels in this fleet are substring scanners,
/// and a plant whose comment describes the violation can satisfy the scanner with its own
/// description while the declaration does nothing — a probe that passes for the wrong reason. The
/// text injected below contains no English at all: it is `"north_south_only": true` and
/// `"ambient_waypoint": true` at structured-key positions, which is exactly what the detectors
/// read.
#[test]
fn planting_each_layer_breach_in_a_real_manifest_reddens_the_gate() {
    let root = repo_root();
    let tracked = tracked_files(&root).expect("git ls-files");

    // A conformant subject: a real, declared manifest that the live walk finds clean, so the
    // planted breach is unambiguously the thing that reddened it.
    let (subject, identity, body) = tracked
        .iter()
        .filter(|relative| is_microservice_manifest_path(relative))
        .filter_map(|relative| {
            let contents = read_tracked(&root, relative).ok().flatten()?;
            let identity = declared_identity(&contents)?;
            let (_, violations) = audit_all_violations(vec![ManifestDocument {
                microservice: identity.clone(),
                path: relative.clone(),
                contents: contents.clone(),
            }]);
            violations
                .is_empty()
                .then_some((relative.clone(), identity, contents))
        })
        .next()
        .expect(
            "no tracked manifest.json both declares an identity and passes the layering rules; \
             this gate has no clean subject to plant into",
        );

    let plant = |injected: &str| -> Vec<LayeredArchitectureViolation> {
        let contents = body.replacen('{', &format!("{{\n{injected}"), 1);
        assert_ne!(contents, body, "the plant did not modify {subject}");
        audit_all_violations(vec![ManifestDocument {
            microservice: identity.clone(),
            path: subject.clone(),
            contents,
        }])
        .1
    };

    // Shape 1 — ADR-0182: one workload claiming BOTH traffic directions.
    let both_directions = plant("  \"north_south_only\": true,\n  \"ambient_waypoint\": true,");
    assert!(
        both_directions.iter().any(|violation| {
            format!("{:?}", violation.kind) == "GatewayAndMeshConflict"
                && violation.microservice == identity
        }),
        "planting `north_south_only: true` + `ambient_waypoint: true` into the live manifest \
         {subject} produced no GatewayAndMeshConflict for {identity}; got {:?}",
        both_directions
            .iter()
            .map(|violation| format!("{:?}", violation.kind))
            .collect::<Vec<_>>()
    );

    // Shape 2 — ADR-0182: north-south ownership claimed by something that is not the api-gateway.
    // Guarded, because a clean subject that IS the api-gateway would legitimately not produce it.
    assert_ne!(
        identity, "api-gateway",
        "the clean subject picked for planting is the api-gateway itself, which the \
         NorthSouthOnlyMisplaced rule exempts by design; pick another subject rather than deleting \
         this half of the proof"
    );
    let misplaced = plant("  \"north_south_only\": true,");
    assert!(
        misplaced.iter().any(|violation| {
            format!("{:?}", violation.kind) == "NorthSouthOnlyMisplaced"
                && violation.microservice == identity
        }),
        "planting `north_south_only: true` into the live manifest {subject}, whose DECLARED \
         identity is {identity}, produced no NorthSouthOnlyMisplaced; got {:?}",
        misplaced
            .iter()
            .map(|violation| format!("{:?}", violation.kind))
            .collect::<Vec<_>>()
    );

    println!(
        "mutation proof: {subject} (declares {identity}) audits clean; \
         +north_south_only+ambient_waypoint -> {} violation(s) including GatewayAndMeshConflict; \
         +north_south_only alone -> {} violation(s) including NorthSouthOnlyMisplaced",
        both_directions.len(),
        misplaced.len()
    );
}

/// Evidence, always printed, so a reader can tell a repaired corpus from a collapsed walk without
/// re-running anything.
#[test]
fn live_census_is_reported() {
    let (_, observed) = live();
    println!("{}", census(observed));
    assert!(!observed.identities.is_empty());
}
