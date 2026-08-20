// governance-check-ontology-projection-coverage LIVE-TREE gate.
//
// Authority: specs/microservices/manifest-schema.json (which lists `ontology_projections` in its
// top-level `required` array and names this crate as its validator), carried into the live apex by
// ADR-0701's ADR-145 residual. See `_authority_note` in the policy JSON: the apex prose truncates
// before Invariant 3, so the schema is what makes this doctrine live law rather than a delete
// candidate.
//
// The unit suite inside src/lib.rs proves the kernel correct on hand-written fixtures. It says
// nothing about this repository, and until this file existed nothing did: the crate's only Cargo
// consumer was marketplace/facade/dev-cli, which no workflow invokes, so the doctrine had never
// produced a verdict about the tree it governs.
//
// The kernel is pure; this is the CALLER that walks the real repository and hands it observations
// as DATA. Walk failures are ERRORS, never omitted observations: a manifest dropped from the census
// because its contents failed to read would quietly shrink every frozen set below, and a shrink
// reads as repair.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use check_ontology_projection_coverage::{
    CANONICAL_ENTITY_OWNERS, ManifestDocument, StrictReport, validate_strict,
};

const POLICY_PATH: &str =
    "governance/check/ontology-projection-coverage/ontology-projection-coverage-policy.json";
const MAX_SCANNED_BYTES: u64 = 4_194_304;

struct Policy {
    min_tracked_files: usize,
    min_manifests: usize,
    min_owner_manifests: usize,
    min_projection_entries: usize,
    frozen_strict_findings: BTreeSet<String>,
    frozen_owner_manifests: BTreeMap<String, BTreeSet<String>>,
    frozen_unmatched_owners: BTreeSet<String>,
}

struct Observed {
    report: StrictReport,
    tracked_files: usize,
    manifests: usize,
    /// Manifests that merely CONTAIN an `ontology_projections` key, empty list included. Reported
    /// for the census only; the kernel's own `manifests_with_projections` counts non-empty lists.
    manifests_declaring_the_key: usize,
    /// `<manifest path>::<kernel summary>` for every strict finding.
    strict_findings: BTreeSet<String>,
    /// Canonical-entity owner name -> the manifests that DECLARE it. The subject binding itself,
    /// frozen because it decides which manifests the doctrine reaches at all.
    owner_manifests: BTreeMap<String, BTreeSet<String>>,
    /// Owner names no tracked manifest declares.
    unmatched_owners: BTreeSet<String>,
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
    let string_set = |key: &str| -> BTreeSet<String> {
        doc[key]
            .as_array()
            .unwrap_or_else(|| panic!("policy field {key} missing or not an array"))
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .unwrap_or_else(|| panic!("policy field {key} holds a non-string"))
                    .to_owned()
            })
            .collect()
    };
    let frozen_owner_manifests = doc["frozen_owner_manifests"]
        .as_object()
        .expect("policy field frozen_owner_manifests missing or not an object")
        .iter()
        .map(|(owner, paths)| {
            let set = paths
                .as_array()
                .unwrap_or_else(|| panic!("frozen_owner_manifests[{owner}] is not an array"))
                .iter()
                .map(|entry| {
                    entry
                        .as_str()
                        .unwrap_or_else(|| {
                            panic!("frozen_owner_manifests[{owner}] holds a non-string")
                        })
                        .to_owned()
                })
                .collect();
            (owner.clone(), set)
        })
        .collect();
    Policy {
        min_tracked_files: number("min_tracked_files"),
        min_manifests: number("min_manifests"),
        min_owner_manifests: number("min_owner_manifests"),
        min_projection_entries: number("min_projection_entries"),
        frozen_strict_findings: string_set("frozen_strict_findings"),
        frozen_owner_manifests,
        frozen_unmatched_owners: string_set("frozen_unmatched_owners"),
    }
}

/// The tracked file list, from git — the same corpus boundary every other live gate here uses.
///
/// Walking the working tree instead would measure a different corpus than CI does the moment an
/// ignored `manifest.json` exists on disk (a vendored node_modules manifest, a build scratch dir),
/// and with these sets pinned by equality that is a red gate CI cannot reproduce.
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

/// Basename equality, never a suffix match.
///
/// `*manifest.json` as a git pathspec also matches `cargo-manifest.json`,
/// `codegen-manifest.json` and friends — four extra paths on this tree — which are not
/// microservice manifests and do not answer to `specs/microservices/manifest-schema.json`.
/// Admitting them would put permanent unrepairable entries into a corpus whose whole purpose is
/// to name microservices that own canonical entities.
fn is_microservice_manifest(relative: &str) -> bool {
    relative
        .rsplit('/')
        .next()
        .is_some_and(|leaf| leaf == "manifest.json")
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
    // LOSSY, never skipped: a manifest that is not valid UTF-8 must still reach the kernel, which
    // will report the JSON parse failure as a finding rather than have the caller hide the file.
    Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
}

/// The owning microservice is the manifest's own DECLARED name.
///
/// NOT the directory leaf. `CANONICAL_ENTITY_OWNERS` holds bare microservice names, and on this
/// tree leaf-keying is wrong in both directions: `audit/manifest.json` declares `audit-chain`
/// (an owner the leaf would lose) and `network/manifest.json` declares `cloud-network` (a
/// non-owner the leaf would invent). A manifest that declares no name yields the empty string,
/// which matches no owner — correct: the fixture manifests under
/// `libs/oya-check-dependency-seam/tests/fixtures/` are deliberately partial and own nothing.
fn declared_microservice(contents: &str) -> String {
    serde_json::from_str::<serde_json::Value>(contents)
        .ok()
        .and_then(|doc| {
            doc.get("microservice")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_default()
}

fn collect_manifests(root: &Path, tracked: &[String]) -> Result<Vec<ManifestDocument>, String> {
    let mut manifests = Vec::new();
    for relative in tracked {
        if !is_microservice_manifest(relative) {
            continue;
        }
        let Some(contents) = read_tracked(root, relative)? else {
            continue;
        };
        manifests.push(ManifestDocument {
            path: relative.clone(),
            microservice: declared_microservice(&contents),
            contents,
        });
    }
    Ok(manifests)
}

fn observe(root: &Path) -> Result<Observed, String> {
    let tracked = tracked_files(root)?;
    let manifests = collect_manifests(root, &tracked)?;

    let mut owner_manifests: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut manifests_declaring_the_key = 0usize;
    for manifest in &manifests {
        if manifest.contents.contains("\"ontology_projections\"") {
            manifests_declaring_the_key += 1;
        }
        if CANONICAL_ENTITY_OWNERS.contains(&manifest.microservice.as_str()) {
            owner_manifests
                .entry(manifest.microservice.clone())
                .or_default()
                .insert(manifest.path.clone());
        }
    }

    let unmatched_owners = CANONICAL_ENTITY_OWNERS
        .iter()
        .filter(|owner| !owner_manifests.contains_key(**owner))
        .map(|owner| (*owner).to_owned())
        .collect();

    let report = validate_strict(manifests.clone());
    let strict_findings = report
        .strict_findings
        .iter()
        .map(|finding| format!("{}::{}", finding.manifest_path, finding.summary))
        .collect();

    Ok(Observed {
        tracked_files: tracked.len(),
        manifests: manifests.len(),
        manifests_declaring_the_key,
        strict_findings,
        owner_manifests,
        unmatched_owners,
        report,
    })
}

/// The live walk, done ONCE for the whole binary: it is a pure function of the tree, so every test
/// re-walking it would recompute the same answer over ~14k tracked paths.
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
        "census: {} manifest.json over {} tracked files; {} carry an ontology_projections key, {} \
         declare a NON-EMPTY list, {} projection entries checked; {} manifests owned by {} of the \
         {} canonical-entity owners; {} strict findings\n",
        observed.manifests,
        observed.tracked_files,
        observed.manifests_declaring_the_key,
        observed.report.manifests_with_projections,
        observed.report.projections_checked,
        observed.report.manifests_owning_entities,
        observed.owner_manifests.len(),
        CANONICAL_ENTITY_OWNERS.len(),
        observed.report.strict_findings.len(),
    );
    for (owner, paths) in &observed.owner_manifests {
        out.push_str(&format!(
            "  {owner}: {}\n",
            paths.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    for owner in &observed.unmatched_owners {
        out.push_str(&format!("  {owner}: NO MANIFEST DECLARES THIS NAME\n"));
    }
    out
}

/// ANTI-VACUITY, asserted before any equality below is read.
///
/// Sets pinned by equality cannot distinguish "the corpus is disciplined" from "the walk
/// collapsed"; both drive the observed sets toward empty, and `frozen_strict_findings` is empty
/// TODAY, so without these floors a caller that read nothing would look identical to a clean tree.
/// Every floor counts SUBJECT manifests and projection entries, never findings, so repairing a
/// violation moves the frozen sets and leaves all four floors exactly where they are — no floor
/// here can red on honest progress.
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
        observed.manifests >= policy.min_manifests,
        "{} microservice manifests found, below the floor of {}. Manifests do not disappear in \
         bulk; a drop here is a narrowed scan, and a narrowed scan reports perfect projection \
         coverage it never read\n{}",
        observed.manifests,
        policy.min_manifests,
        census(observed)
    );
    assert!(
        observed.report.manifests_owning_entities >= policy.min_owner_manifests,
        "{} manifests declare a canonical-entity-owning microservice, below the floor of {} — the \
         only manifests this doctrine can ever find fault with went unscanned, so its zero \
         findings are not evidence\n{}",
        observed.report.manifests_owning_entities,
        policy.min_owner_manifests,
        census(observed)
    );
    assert!(
        observed.report.projections_checked >= policy.min_projection_entries,
        "{} ontology projection entries checked, below the floor of {} — the per-entry rules \
         (non-empty entity_name, non-empty projection_target_table, no duplicate entity) had \
         almost nothing to run against\n{}",
        observed.report.projections_checked,
        policy.min_projection_entries,
        census(observed)
    );
}

/// THE GATE: a TWO-SIDED, equality-pinned SET of strict-mode findings.
///
/// Keys, not a count. A count would tell a reviewer that the number moved and nothing about which
/// manifest moved; `<path>::<summary>` names the manifest and the rule it broke, and is reviewable
/// on its face.
///
/// The set is EMPTY today and empty is the correct pin, not a vacuity: all 13 owner manifests
/// declare at least one concrete projection, every entry names both an entity and a target table,
/// and no manifest fails to parse. Equality-pinned empty means born-blocking — the first manifest
/// to violate the doctrine reddens this gate rather than joining a tolerated backlog. Note this is
/// an equality pin and NOT a floor: a floor on a zero-target term would red on honest progress,
/// which is why none of the `min_*` values above touches a finding count.
#[test]
fn strict_findings_equal_the_frozen_set() {
    let (policy, observed) = live();
    let drift = set_drift(&policy.frozen_strict_findings, &observed.strict_findings);
    assert!(
        drift.is_empty(),
        "ontology-projection strict-finding drift. NEW (observed, not frozen): a microservice that \
         owns canonical entities stopped declaring a usable projection — fix the manifest against \
         specs/microservices/manifest-schema.json, or freeze the finding here with the reason. \
         STALE (frozen, not observed): the violation was repaired, so strike its line from \
         `frozen_strict_findings` in THIS change; or the walk narrowed and is reporting green over \
         manifests it stopped reading, which the floors above are there to catch:\n{}\n{}",
        drift.join("\n"),
        census(observed)
    );
}

/// The SUBJECT BINDING itself, frozen — the part of this gate that a reviewer should read first.
///
/// `CANONICAL_ENTITY_OWNERS` is a list of BARE microservice names, so how the caller derives a name
/// from a manifest decides which manifests the doctrine reaches at all. Freezing the resolved
/// owner -> manifests map makes that decision a reviewable diff instead of an invisible assumption,
/// and closes the specific failure the floors cannot see: a manifest that renames its declared
/// `"microservice"` out of the owner set removes itself from the subject while every count above
/// stays put. Here that reddens.
///
/// A MAP, not a set of owner names: `calendar` is declared by two manifests today, and a set keyed
/// on the owner alone would let one of them vanish unnoticed.
#[test]
fn the_owner_to_manifest_binding_equals_the_frozen_map() {
    let (policy, observed) = live();
    let owners: BTreeSet<&String> = policy
        .frozen_owner_manifests
        .keys()
        .chain(observed.owner_manifests.keys())
        .collect();
    let empty = BTreeSet::new();
    let drift: Vec<String> = owners
        .into_iter()
        .filter_map(|owner| {
            let seen = observed.owner_manifests.get(owner).unwrap_or(&empty);
            let want = policy.frozen_owner_manifests.get(owner).unwrap_or(&empty);
            (seen != want).then(|| {
                format!(
                    "  {owner}: observed [{}], frozen [{}]",
                    seen.iter().cloned().collect::<Vec<_>>().join(", "),
                    want.iter().cloned().collect::<Vec<_>>().join(", ")
                )
            })
        })
        .collect();
    assert!(
        drift.is_empty(),
        "canonical-entity owner binding drift, per owner name. This map is what makes the finding \
         set above mean anything: an owner whose manifests disappear here contributes zero \
         findings for the rest of time and no count notices. Re-derive by RUNNING this gate and \
         reading the 'observed [...]' lists; never by arithmetic on the old values. Remember the \
         binding is the manifest's DECLARED `\"microservice\"` value, not its directory leaf:\n{}\n{}",
        drift.join("\n"),
        census(observed)
    );
}

/// Owner names the doctrine declares but no manifest answers to.
///
/// A dangling entry in `CANONICAL_ENTITY_OWNERS` enforces nothing while reading as coverage. One
/// exists today — `network`, because `network/manifest.json` declares `cloud-network` — and it is
/// frozen by NAME so it stays attributable and cannot be joined by a second dangling entry
/// unnoticed. Two-sided: repairing it (rename the declared microservice, or drop the name from the
/// kernel's list) must strike the entry here in the same change.
#[test]
fn unmatched_canonical_entity_owners_equal_the_frozen_set() {
    let (policy, observed) = live();
    let drift = set_drift(&policy.frozen_unmatched_owners, &observed.unmatched_owners);
    assert!(
        drift.is_empty(),
        "dangling canonical-entity owner drift. NEW: a name in CANONICAL_ENTITY_OWNERS now matches \
         no tracked manifest, so that entity has silently stopped being enforced — usually because \
         a manifest renamed its declared `\"microservice\"`. STALE: the name now matches, so strike \
         it here in the same change:\n{}\n{}",
        drift.join("\n"),
        census(observed)
    );
}

fn set_drift(frozen: &BTreeSet<String>, observed: &BTreeSet<String>) -> Vec<String> {
    let mut drift: Vec<String> = observed
        .difference(frozen)
        .map(|key| format!("  NEW    {key}"))
        .collect();
    drift.extend(
        frozen
            .difference(observed)
            .map(|key| format!("  STALE  {key}")),
    );
    drift
}

/// The gate is DEMONSTRATED CAPABLE OF FAILING against the REAL corpus, not just fixtures.
///
/// A green equality over an EMPTY frozen set proves nothing on its own — a caller that silently
/// handed the kernel zero manifests would satisfy it by reporting a perfectly projected tree. This
/// takes a REAL owner manifest's text and injects each defect shape the doctrine exists to catch,
/// then asserts the finding names THAT manifest. The injections are structural JSON edits, not
/// prose: the strict kernel parses the document, so a comment describing a violation cannot
/// satisfy the rule the way a substring scanner would be satisfied by its own error message.
#[test]
fn injecting_each_defect_into_a_real_owner_manifest_reddens_the_gate() {
    let root = repo_root();
    let tracked = tracked_files(&root).expect("git ls-files");
    let manifests = collect_manifests(&root, &tracked).expect("collect manifests");

    let subject = manifests
        .iter()
        .find(|manifest| {
            CANONICAL_ENTITY_OWNERS.contains(&manifest.microservice.as_str())
                && serde_json::from_str::<serde_json::Value>(&manifest.contents)
                    .ok()
                    .and_then(|doc| {
                        doc.get("ontology_projections")
                            .and_then(serde_json::Value::as_array)
                            .map(|entries| !entries.is_empty())
                    })
                    .unwrap_or(false)
        })
        .expect(
            "no tracked manifest declares a canonical-entity-owning microservice with a non-empty \
             ontology_projections list; this doctrine has no live subject and must be deleted \
             rather than connected",
        );

    let baseline = validate_strict(vec![subject.clone()]);
    assert!(
        baseline.is_success(),
        "the manifest chosen as the mutation subject ({}) already has findings, so a rise in the \
         count would not attribute to the injection",
        subject.path
    );

    let mutate = |edit: &dyn Fn(&mut serde_json::Value)| -> StrictReport {
        let mut doc: serde_json::Value =
            serde_json::from_str(&subject.contents).expect("subject manifest parses");
        edit(&mut doc);
        validate_strict(vec![ManifestDocument {
            path: subject.path.clone(),
            microservice: subject.microservice.clone(),
            contents: serde_json::to_string(&doc).expect("mutated manifest serializes"),
        }])
    };

    // Defect 1 — an owner stops projecting: the projection list is emptied.
    let emptied = mutate(&|doc| {
        doc["ontology_projections"] = serde_json::json!([]);
    });
    assert_eq!(
        emptied.strict_findings.len(),
        1,
        "emptying ontology_projections in the live manifest {} did not produce exactly one finding",
        subject.path
    );
    assert!(
        emptied.strict_findings[0].manifest_path == subject.path
            && emptied.strict_findings[0].summary.contains("is empty"),
        "the finding does not name the injected defect in {}: {:?}",
        subject.path,
        emptied.strict_findings[0]
    );

    // Defect 2 — an owner drops the block entirely, which the schema lists as `required`.
    let removed = mutate(&|doc| {
        doc.as_object_mut()
            .expect("manifest is a JSON object")
            .remove("ontology_projections");
    });
    assert_eq!(
        removed.strict_findings.len(),
        1,
        "removing ontology_projections from the live manifest {} did not produce exactly one \
         finding",
        subject.path
    );
    assert_eq!(removed.strict_findings[0].manifest_path, subject.path);

    // Defect 3 — a projection entry names an entity but no target table, so nothing is projected.
    let untargeted = mutate(&|doc| {
        doc["ontology_projections"][0]["projection_target_table"] = serde_json::json!("");
    });
    assert!(
        untargeted
            .strict_findings
            .iter()
            .any(|finding| finding.manifest_path == subject.path
                && finding.summary.contains("empty projection_target_table")),
        "blanking a projection_target_table in the live manifest {} produced no matching finding: \
         {:?}",
        subject.path,
        untargeted.strict_findings
    );

    // Defect 4 — the same entity projected twice, so one target is silently dead.
    let duplicated = mutate(&|doc| {
        let first = doc["ontology_projections"][0].clone();
        doc["ontology_projections"]
            .as_array_mut()
            .expect("projections is an array")
            .push(first);
    });
    assert!(
        duplicated
            .strict_findings
            .iter()
            .any(|finding| finding.manifest_path == subject.path
                && finding.summary.contains("duplicate ontology projection")),
        "duplicating a projection entity in the live manifest {} produced no matching finding: {:?}",
        subject.path,
        duplicated.strict_findings
    );

    println!(
        "mutation proof on live manifest {} (declared microservice {:?}): baseline 0 findings; \
         emptied -> {}, removed -> {}, blank target -> {}, duplicate entity -> {}",
        subject.path,
        subject.microservice,
        emptied.strict_findings.len(),
        removed.strict_findings.len(),
        untargeted.strict_findings.len(),
        duplicated.strict_findings.len(),
    );
}

/// Evidence, always printed, so a reader can tell a disciplined corpus from a collapsed walk
/// without re-running anything.
#[test]
fn live_census_is_reported() {
    let (_, observed) = live();
    println!("{}", census(observed));
    assert!(observed.manifests > 0);
}
