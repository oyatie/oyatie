// cloud-ci-slo-coverage live-corpus gate. Runs the producer `--face slo-coverage`, then asserts
// the gate verdict matches the current registry catalog corpus. ADR-0083 Tier-3: integration tests
// assert with unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use ci_slo_coverage::{Verdict, evaluate, evaluate_keyed};
use serde_json::Value;

const REQUIRED_SLO_LINKED_CLOUD_MANIFESTS: [&str; 6] = [
    "intelligence/manifest.json",
    "cloud/managed-k8s-cluster-lifecycle/manifest.json",
    "cloud/managed-k8s-control-plane-host/manifest.json",
    "cloud/managed-k8s-sla-observability/manifest.json",
    "cloud/managed-k8s-tenant-quota/manifest.json",
    "cloud/tenancy/manifest.json",
];

/// FALSE-GREEN FLOOR: the producer must be shown to have actually enumerated the
/// catalog. A broken collector returns zero rows, finds zero violations, and would
/// otherwise report a clean pass — so an implausibly small corpus is a gate failure,
/// never coverage.
///
/// This is a floor on COLLECTION, not a ratchet on catalog SIZE. It therefore has to
/// be lowered deliberately whenever rows are legitimately removed, and the lowering
/// belongs in the same change as the removal so the two are reviewed together.
///
/// 783 rows existed when the previous value of 776 was set — 7 rows of slack. The
/// ProviderAuthPort consolidation deleted 10 rows, leaving 773, and moved the floor to
/// 766 to preserve exactly that same 7-row margin.
///
/// 2026-07-31: the floor was left at 766 while the corpus fell to 755, so this gate was
/// RED on dev. The convention above was not followed by the removals that landed after it:
///   #1451 chore(payments): delete the closed oya/payments crate cluster  -20 rows
///   #1413 chore(libs): delete the orphaned oya-gen-microservice-manifests-app  -1 row
/// Each deletion is legitimate and reviewed; none lowered the floor in the same change.
/// They were able to merge that way because this gate produced NO VERDICT at the time —
/// the artifact-storage-quota outage failed `producer-regen`, and every `needs:` leg was
/// SKIPPED rather than run. A skipped gate is not a passing gate, and this is what that
/// distinction costs: a born-blocking floor sat stale behind a lane nobody was reading.
///
/// 2026-08-01: two later, reviewed consolidations again removed live catalog rows without
/// carrying this floor in the same change:
///   #1485 refactor(intelligence): collapse three provider adapters into one  -2 rows
///   #1483 fix(ci): retire eight duplicate dark lifecycle catalog rows        -8 rows
///
/// 745 rows exist today. The floor moves to 738 to preserve the same 7-row margin rather
/// than silently loosening or tightening the guard.
///
/// NOTE for the next removal: this constant has now gone stale three times in the same way.
/// A floor that must be hand-lowered on every legitimate deletion is a staleness surface;
/// deriving it from an INDEPENDENT enumeration of the catalog files on disk (with a small
/// absolute floor purely to catch a both-are-zero scan) would keep the empty-scan
/// protection while removing the manual step. That is a separate reviewed change, not a
/// side effect of un-reddening dev.
const MIN_SLO_CATALOG_ROWS: usize = 738;

fn producer_command(root: &Path, producer_bin: Option<&str>) -> Result<Command, String> {
    if let Some(bin) = producer_bin {
        let bin = if Path::new(bin).is_absolute() {
            PathBuf::from(bin)
        } else {
            root.join(bin)
        };
        Ok(Command::new(bin))
    } else {
        Err("OYA_CI_PRODUCER_BIN is required for hermetic Buck2 gate execution".to_owned())
    }
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
    let scm_facts = root
        .join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
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

fn cloud_manifest_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(root.join("cloud")).expect("read cloud directory") {
        let entry = entry.expect("read cloud child");
        let manifest = entry.path().join("manifest.json");
        if manifest.is_file() {
            paths.push(manifest);
        }
    }
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
    assert!(
        rows.len() >= MIN_SLO_CATALOG_ROWS,
        "the slo-coverage face should enumerate at least {MIN_SLO_CATALOG_ROWS} current catalog rows, got {}",
        rows.len()
    );

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
        manifest_count >= 21,
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
