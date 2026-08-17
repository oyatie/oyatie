// cloud-ci-slo-coverage live-corpus gate. Runs the producer `--face slo-coverage`, then asserts
// the gate verdict matches the current registry catalog corpus. ADR-0083 Tier-3: integration tests
// assert with unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use ci_slo_coverage::{
    CODE_CATALOG_CENSUS_DROP_UNATTRIBUTED, Verdict, evaluate, evaluate_catalog_census,
    evaluate_keyed,
};
use serde_json::Value;

const REQUIRED_SLO_LINKED_CLOUD_MANIFESTS: [&str; 6] = [
    "intelligence/manifest.json",
    "k8s/managed-cluster-lifecycle/manifest.json",
    "k8s/managed-control-plane-host/manifest.json",
    "k8s/managed-sla-observability/manifest.json",
    "k8s/managed-tenant-quota/manifest.json",
    "tenancy/manifest.json",
];

/// FALSE-GREEN CENSUS PIN: the producer must be shown to have actually enumerated the catalog.
/// A broken collector returns zero rows, finds zero violations, and would otherwise report a
/// clean pass — so the observed row count is pinned by EQUALITY and any move must be re-frozen,
/// with an attribution, in the change that caused it. The rule itself is
/// [`ci_slo_coverage::evaluate_catalog_census`], which documents why equality and not a floor;
/// the history that proves the point is below.
///
/// THE HISTORY. This was `MIN_SLO_CATALOG_ROWS`, a floor, and it went stale three times in the
/// same way — because it was a floor on a term whose legitimate direction is DOWN:
///
///   783 rows existed when the value 776 was set — 7 rows of slack. The ProviderAuthPort
///   consolidation deleted 10 rows, leaving 773, and moved the floor to 766 to preserve exactly
///   that same 7-row margin.
///
///   2026-07-31: the floor was left at 766 while the corpus fell to 755, so this gate was RED on
///   dev. The convention was not followed by the removals that landed after it:
///     #1451 chore(payments): delete the closed oya/payments crate cluster  -20 rows
///     #1413 chore(libs): delete the orphaned oya-gen-microservice-manifests-app  -1 row
///   Each deletion is legitimate and reviewed; none lowered the floor in the same change. They
///   were able to merge that way because this gate produced NO VERDICT at the time — the
///   artifact-storage-quota outage failed `producer-regen`, and every `needs:` leg was SKIPPED
///   rather than run. A skipped gate is not a passing gate, and this is what that distinction
///   costs: a born-blocking floor sat stale behind a lane nobody was reading.
///
///   2026-08-01: two later, reviewed consolidations again removed live catalog rows without
///   carrying the floor in the same change:
///     #1485 refactor(intelligence): collapse three provider adapters into one  -2 rows
///     #1483 fix(ci): retire eight duplicate dark lifecycle catalog rows        -8 rows
///   745 rows existed then; the floor moved to 738 to preserve the same 7-row margin.
///
/// The floor's own closing note asked for an INDEPENDENT enumeration to remove the manual step.
/// That would have removed the staleness by removing the guard's independence — a count derived
/// from the same tree the producer walks cannot contradict the producer. Equality removes the
/// staleness instead: a pin that must match exactly cannot silently drift, because drift IS the
/// failure, and it is reported in the change that causes it rather than discovered a wave later.
///
/// ATTRIBUTIONS (append one line per move; never move this number without one):
///   2026-08-01  745 -> floor 738   (see history above; the last move made as a floor)
///   2026-08-09  738 -> pin 749     re-measure at the floor -> pin conversion. GROWTH, not a
///                                  correction: the corpus was 745 when the floor last moved and
///                                  the producer enumerates 749 rows on dev today, which an
///                                  independent count of registry/catalog/*.yaml confirms. The
///                                  floor saw neither the four arrivals nor the eleven rows of
///                                  slack it was sitting on — 738 against 749 means the producer
///                                  could have silently lost eleven rows and still passed. That
///                                  slack is what this pin removes.
///   2026-08-09  749 -> pin 750     BASE MOVE, not a change of what this branch enumerates. The
///                                  one added row is named: crate_id `port-engine-kernel`,
///                                  source_path `registry/catalog/port-engine-kernel.yaml`,
///                                  introduced by dev commit 885794461 (PR #1621, port-engine W0
///                                  skeleton, ADR-0637 D1). `git diff --name-only
///                                  origin/dev...HEAD -- registry/catalog/port-engine-kernel.yaml`
///                                  is EMPTY, so this branch does not contribute it.
///
///                                  WHICH OF THE TWO IT WAS, since the gate says a count cannot
///                                  tell a legitimate arrival from a widened enumeration: it was
///                                  an ARRIVAL, and the discriminator is a control rather than an
///                                  argument. The SAME producer binary, built from this branch's
///                                  head, was run twice over the same source tree with only the
///                                  `--scm-facts` face swapped — the pre-rebase face gives 749
///                                  rows and the regenerated face gives 750, added 1, removed 0.
///                                  Holding the predicate fixed and moving only the corpus isolates
///                                  the delta to the corpus. Independently: the enumerating
///                                  producer is ci/facade/artifact-inventory-registry, which this
///                                  branch does not touch at all; the slo-coverage edits in this
///                                  branch are to the census RULE
///                                  (`evaluate_catalog_census`), never to the walk.
///
///                                  WHY IT WAS MISSED until CI, recorded because the failure mode
///                                  outlived the number: `scm-facts.generated.json` is gitignored
///                                  and locally MATERIALIZED, so a lane that rebases without
///                                  regenerating it keeps enumerating the pre-rebase corpus. The
///                                  local gate ran GREEN at 749 against a face materialized hours
///                                  before the rebase, and CI — which regenerates it — read 750.
///                                  Regenerate the face after any rebase before trusting a census
///                                  pin locally; a green local census gate proves nothing about a
///                                  stale face.
///
///   2026-08-10  750 -> pin 757     Re-measured after the kernel and os SLO-home retargets
///                                  added seven catalog rows. This updates only the exact live
///                                  corpus pin; the kernel/os manifest fallback contract remains
///                                  unchanged.
///   2026-08-11  757 -> pin 759     Live catalog grew by two rows (integ/ci babysit tip); pin
///                                  matches measured census only — no contract change.
///   2026-08-11  759 -> pin 761     After merge(dev) absorbing #1926 os/harness + cloud-os
///                                  residual, live catalog grew by two rows; re-freeze only.
///   2026-08-11  761 -> pin 762     BASE MOVE from #1647: registry/catalog/check-apex-gist-
///                                  integrity.yaml (apex-gist-integrity designed-ahead row).
///                                  Tracked catalog yamls 762->763; enumerated face rows +1.
///   2026-08-11  762 -> pin 773     BASE MOVE from #1934: +4 ci-controller-* and +7 port-engine
///                                  W0-B catalog rows (face enumerated 773). Keep dual-home
///                                  oya/ci-controller until lock/baseline tip-free cleanup.
///   2026-08-17  773 -> pin 769     RETIREMENT: remove the four deleted-crate catalog rows
///                                  oya-cloud-os-{cluster-mgmt,kubernetes,secrets,trustd}-domain;
///                                  the same producer now enumerates exactly four fewer rows.
///   2026-08-14  769 -> pin 770     Owned ADR-0535 actuator row added: registry/catalog/
///                                  ci-rust-toolchain-bump-proposer.yaml (the rust toolchain
///                                  bump proposer crate). Independent of the retirement above:
///                                  dev removed four rows (773 -> 769) while this branch added
///                                  one (773 -> 774), so the merged face enumerates 769 + 1.
const SLO_CATALOG_CENSUS: usize = 770;

fn producer_command(root: &Path, producer_bin: Option<&str>) -> Result<Command, String> {
    if let Some(bin) = producer_bin {
        let bin = resolve_producer_binary(root, bin)?;
        Ok(Command::new(bin))
    } else {
        Err("OYA_CI_PRODUCER_BIN is required for hermetic Buck2 gate execution".to_owned())
    }
}

fn resolve_producer_binary(root: &Path, value: &str) -> Result<PathBuf, String> {
    ci_path_resolver_adapters::resolve_cargo_test_binary(root, std::ffi::OsStr::new(value))
}

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let mut command = producer_command(root, producer_bin.as_deref()).expect("producer command");

    let output = command
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run producer binary");

    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

/// The issue #993 cloud substrate service manifests, at their capability-first homes.
/// This was a `read_dir("cloud")` walk until the ADR-0562 rehomes emptied `cloud/`, at which point
/// the walk silently found 2 of 21 — a scan that shrinks to nothing is a false green, so the set is
/// now named explicitly and `manifest_missing` below fails closed on any entry that stops resolving.
///
/// Wave-2 absorb: the deleted cloud-kernel service-manifest pin had no forever service-manifest
/// successor, while the OS pin retains its capability-first home:
/// `cloud/cloud-os/manifest.json` → `os/manifest.json` (#1926 / #1839). Cite the forever path
/// here; `cloud_manifest_paths` may accept the transitional source only until destination bytes
/// land (fail closed when neither resolves). Do not re-list hub enumerations — named pins,
/// not a dual-truth root table.
const CLOUD_SUBSTRATE_MANIFESTS: [&str; 20] = [
    "billing/manifest.json",
    "billing/tax/manifest.json",
    "cell/cell-lifecycle/manifest.json",
    "cell/cell-rebalancer/manifest.json",
    "os/manifest.json",
    "data/cloud-data/manifest.json",
    "iac/manifest.json",
    "iam/cloud-iam/manifest.json",
    "intelligence/manifest.json",
    "k8s/managed-cluster-lifecycle/manifest.json",
    "k8s/managed-control-plane-host/manifest.json",
    "k8s/managed-sla-observability/manifest.json",
    "k8s/managed-tenant-quota/manifest.json",
    "k8s/manifest.json",
    "network/dns/manifest.json",
    "network/manifest.json",
    "secrets/kms/manifest.json",
    "secrets/manifest.json",
    "storage/manifest.json",
    "tenancy/manifest.json",
];

/// Forever → transitional source still present on origin/dev until forever bytes land
/// (#1926 os). Drop the entry when the transitional path is burned and the forever path is on
/// trunk.
const CLOUD_SUBSTRATE_MANIFEST_FALLBACKS: &[(&str, &str)] =
    &[("os/manifest.json", "cloud/cloud-os/manifest.json")];

fn cloud_manifest_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut missing = Vec::new();
    for relative in CLOUD_SUBSTRATE_MANIFESTS {
        let forever = root.join(relative);
        if forever.is_file() {
            paths.push(forever);
            continue;
        }
        let transitional = CLOUD_SUBSTRATE_MANIFEST_FALLBACKS
            .iter()
            .find(|(forever_rel, _)| *forever_rel == relative)
            .map(|(_, transitional_rel)| root.join(transitional_rel));
        match transitional {
            Some(path) if path.is_file() => paths.push(path),
            _ => missing.push(relative),
        }
    }
    assert!(
        missing.is_empty(),
        "cloud substrate service manifests no longer resolve (rehomed without re-anchoring this \
         gate): {missing:?}"
    );
    paths.sort();
    paths
}

fn required_string<'a>(object: &'a Value, field: &str) -> Option<&'a str> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn validate_manifest_slo_contract(
    root: &Path,
    manifest_path: &Path,
    manifest: &Value,
) -> Vec<String> {
    let rel_manifest = manifest_path
        .strip_prefix(root)
        .expect("manifest under root")
        .display()
        .to_string();
    let mut findings = Vec::new();

    let Some(slos) = manifest.get("slos").and_then(Value::as_array) else {
        findings.push(format!("{rel_manifest}: missing array field `slos`"));
        return findings;
    };

    if slos.is_empty() {
        let Some(exemption) = manifest.get("slo_exemption").and_then(Value::as_object) else {
            findings.push(format!(
                "{rel_manifest}: empty `slos` must carry explicit `slo_exemption`"
            ));
            return findings;
        };

        for field in ["status", "owner", "rationale", "cutover_on", "evidence"] {
            if !exemption
                .get(field)
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
            {
                findings.push(format!(
                    "{rel_manifest}: `slo_exemption.{field}` must be non-empty"
                ));
            }
        }

        if !exemption
            .get("status")
            .and_then(Value::as_str)
            .is_some_and(|status| status == "live_exempted_no_runtime_sli")
        {
            findings.push(format!(
                "{rel_manifest}: `slo_exemption.status` must be live_exempted_no_runtime_sli"
            ));
        }

        if !exemption
            .get("rationale")
            .and_then(Value::as_str)
            .is_some_and(|rationale| {
                rationale.contains("must not claim production or hyperscaler-ready SLO coverage")
            })
        {
            findings.push(format!(
                "{rel_manifest}: `slo_exemption.rationale` must explicitly block production/hyperscaler SLO claims"
            ));
        }

        return findings;
    }

    if manifest.get("slo_exemption").is_some() {
        findings.push(format!(
            "{rel_manifest}: non-empty `slos` must not also carry `slo_exemption`"
        ));
    }

    for (index, slo) in slos.iter().enumerate() {
        let Some(slo_object) = slo.as_object() else {
            findings.push(format!("{rel_manifest}: slos[{index}] must be an object"));
            continue;
        };

        for field in ["name", "target", "sli", "file"] {
            if required_string(slo, field).is_none() {
                findings.push(format!(
                    "{rel_manifest}: slos[{index}].{field} must be non-empty"
                ));
            }
        }

        let Some(file) = slo_object
            .get("file")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
        else {
            continue;
        };

        let slo_path = Path::new(file);
        let has_parent_component = slo_path
            .components()
            .any(|component| matches!(component, Component::ParentDir));
        let has_microservices_component = slo_path.components().any(|component| {
            matches!(component, Component::Normal(part) if part.to_string_lossy() == "microservices")
        });

        if slo_path.is_absolute() {
            findings.push(format!(
                "{rel_manifest}: slos[{index}].file must be repo-relative, got absolute path `{file}`"
            ));
        }

        if has_parent_component {
            findings.push(format!(
                "{rel_manifest}: slos[{index}].file must not contain parent-directory components, got `{file}`"
            ));
        }

        if has_microservices_component {
            findings.push(format!(
                "{rel_manifest}: slos[{index}].file uses retired path `{file}`"
            ));
        }

        if !file.ends_with(".openslo.yaml") {
            findings.push(format!(
                "{rel_manifest}: slos[{index}].file must point at an OpenSLO yaml file, got `{file}`"
            ));
        }

        if !(file.contains("/observability/slos/")
            || (file.starts_with("cloud/") && file.contains("/slos/")))
        {
            findings.push(format!(
                "{rel_manifest}: slos[{index}].file must use a current observability/slos or cloud/*/slos path, got `{file}`"
            ));
        }

        if !slo_path.is_absolute() && !has_parent_component && !root.join(slo_path).is_file() {
            findings.push(format!(
                "{rel_manifest}: slos[{index}].file does not exist: `{file}`"
            ));
        }
    }

    findings
}

#[test]
fn manifest_slo_contract_rejects_missing_refs_and_retired_paths() {
    let root = repo_root();
    let manifest_path = root.join("cloud/example/manifest.json");
    let manifest = serde_json::json!({
        "slos": [{
            "name": "example-availability",
            "target": "99.9%",
            "sli": "availability",
            "file": "microservices/example/slos/availability.openslo.yaml"
        }]
    });

    let findings = validate_manifest_slo_contract(&root, &manifest_path, &manifest);
    assert!(
        findings
            .iter()
            .any(|finding| finding.contains("uses retired path")),
        "expected retired path finding, got {findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.contains("does not exist")),
        "expected missing file finding, got {findings:#?}"
    );
}

#[test]
fn manifest_slo_contract_rejects_absolute_and_parent_paths() {
    let root = repo_root();
    let manifest_path = root.join("cloud/example/manifest.json");
    let manifest = serde_json::json!({
        "slos": [
            {
                "name": "absolute",
                "target": "99.9%",
                "sli": "availability",
                "file": "/tmp/availability.openslo.yaml"
            },
            {
                "name": "parent",
                "target": "99.9%",
                "sli": "availability",
                "file": "cloud/example/../microservices/slos/availability.openslo.yaml"
            }
        ]
    });

    let findings = validate_manifest_slo_contract(&root, &manifest_path, &manifest);
    assert!(
        findings
            .iter()
            .any(|finding| finding.contains("must be repo-relative")),
        "expected absolute path finding, got {findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.contains("must not contain parent-directory components")),
        "expected parent component finding, got {findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.contains("uses retired path")),
        "expected normalized retired component finding, got {findings:#?}"
    );
}

#[test]
fn manifest_slo_contract_rejects_empty_slos_without_explicit_exemption() {
    let root = repo_root();
    let manifest_path = root.join("cloud/example/manifest.json");
    let manifest = serde_json::json!({ "slos": [] });

    let findings = validate_manifest_slo_contract(&root, &manifest_path, &manifest);
    assert_eq!(
        findings,
        vec![
            "cloud/example/manifest.json: empty `slos` must carry explicit `slo_exemption`"
                .to_owned()
        ]
    );
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_command(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(
        err.contains("OYA_CI_PRODUCER_BIN"),
        "error should name missing hermetic producer env, got {err}"
    );
}

#[test]
fn slo_coverage_verdict_matches_the_live_catalog() {
    let root = repo_root();
    let face = run_producer_face(&root, "slo-coverage");
    let rows = face["rows"].as_array().expect("slo-coverage face rows");
    if let Some(census) = evaluate_catalog_census(rows.len(), SLO_CATALOG_CENSUS) {
        panic!("[{}] {}", census.code, census.detail);
    }

    // The pin AS COMMITTED, against the LIVE producer, still rejects the false green it exists
    // for. The assertion above is the control: the live corpus sits exactly at the pin, so this
    // failure is the rule biting and not a mis-set number.
    let collapsed = evaluate_catalog_census(0, SLO_CATALOG_CENSUS)
        .expect("a zero-row enumeration must never read as coverage");
    assert_eq!(collapsed.code, CODE_CATALOG_CENSUS_DROP_UNATTRIBUTED);

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    eprintln!(
        "BORN-BLOCKING slo-coverage: catalog_records={} total_findings={} verdict={:?}",
        rows.len(),
        findings.len(),
        verdict
    );

    assert!(
        findings.is_empty(),
        "current catalog must carry explicit SLO rows for every record: {findings:?}"
    );
    assert_eq!(verdict, Verdict::Green);
}

#[test]
fn cloud_manifests_have_existing_slo_refs_or_explicit_non_claims() {
    let root = repo_root();
    let manifest_paths = cloud_manifest_paths(&root);
    let manifest_count = manifest_paths.len();
    assert!(
        manifest_count >= 20,
        "issue #993 coverage expects every current cloud/*/manifest.json; got {manifest_count}"
    );

    let mut findings = Vec::new();
    let mut non_empty_slo_manifests = 0usize;
    let mut exempted_manifests = 0usize;
    let mut linked_manifest_paths = BTreeSet::new();

    for manifest_path in manifest_paths {
        let manifest_text = fs::read_to_string(&manifest_path).expect("read cloud manifest");
        let manifest: Value = serde_json::from_str(&manifest_text).expect("manifest JSON");
        let rel_manifest = manifest_path
            .strip_prefix(&root)
            .expect("manifest under root")
            .display()
            .to_string();
        let slos = manifest
            .get("slos")
            .and_then(Value::as_array)
            .expect("slos array checked by validator");
        if slos.is_empty() {
            exempted_manifests += 1;
        } else {
            non_empty_slo_manifests += 1;
            linked_manifest_paths.insert(rel_manifest);
        }
        findings.extend(validate_manifest_slo_contract(
            &root,
            &manifest_path,
            &manifest,
        ));
    }

    assert!(
        non_empty_slo_manifests >= 6,
        "current cloud manifest corpus should keep at least the six existing services linked to OpenSLO files"
    );
    for required_manifest in REQUIRED_SLO_LINKED_CLOUD_MANIFESTS {
        assert!(
            linked_manifest_paths.contains(required_manifest),
            "existing SLO-linked cloud manifest lost its SLO refs: {required_manifest}"
        );
    }
    assert_eq!(
        non_empty_slo_manifests + exempted_manifests,
        manifest_count,
        "each cloud manifest must be either SLO-linked or explicitly exempted"
    );
    assert!(
        findings.is_empty(),
        "manifest SLO contract findings: {findings:#?}"
    );
}
