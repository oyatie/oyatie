//! `oya gate validate korea-localization-evidence` runner.
//!
//! This gate closes a false-green gap between FD-001 planning closure and the
//! KR pack exit bar: planning says the pack is required, but delivery evidence
//! must also prove which pack surfaces are covered without claiming the pack is
//! active before signed tenant/regulatory evidence exists.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::{slash_path, usage};

const PACK_STATUS: &str = "planning-closed-foundational";
const ACTIVATION_CLAIM: &str = "not-active";

const REQUIRED_KR_PACK_SURFACES: &[&str] = &[
    "pack_manifest",
    "regulatory_bindings",
    "cedar_policy_fragments",
    "workflow_templates",
    "typst_document_templates",
    "messenger_mail_community_localization",
    "tenant_rbac_operating_flows",
    "audit_chain_evidence",
    "data_residency_and_privacy_controls",
    "import_export_migration_paths",
    "operational_runbooks_and_slos",
    "ops_control_center_localization_runbooks_and_escalation_flows",
];

const REQUIRED_EVIDENCE_HEADINGS: &[&str] = &[
    "## Evidence",
    "## KR Pack Responsibilities",
    "## Non-Claims",
    "## Exit Blockers",
    "## Acceptance Commands",
];

const FORBIDDEN_EVIDENCE_MARKERS: &[&str] = &[
    "tbd",
    "todo",
    "fixme",
    "placeholder",
    "stub",
    "thin scaffold",
];

const REQUIRED_FD001_EVIDENCE: &[RequiredFd001Evidence] = &[
    RequiredFd001Evidence {
        surface: "application",
        microservice: "application",
        evidence_file: "application.md",
    },
    RequiredFd001Evidence {
        surface: "messenger",
        microservice: "messenger",
        evidence_file: "messenger.md",
    },
    RequiredFd001Evidence {
        surface: "mail",
        microservice: "mail",
        evidence_file: "mail.md",
    },
    RequiredFd001Evidence {
        surface: "community",
        microservice: "community",
        evidence_file: "community.md",
    },
    RequiredFd001Evidence {
        surface: "cloud-iac",
        microservice: "cloud-iac",
        evidence_file: "cloud-iac.md",
    },
    RequiredFd001Evidence {
        surface: "cloud-k8s",
        microservice: "cloud-k8s",
        evidence_file: "cloud-k8s.md",
    },
    RequiredFd001Evidence {
        surface: "cloud-secrets",
        microservice: "cloud-secrets",
        evidence_file: "cloud-secrets.md",
    },
    RequiredFd001Evidence {
        surface: "ops-dashboard-control-center",
        microservice: "ops-dashboard-control-center",
        evidence_file: "ops-dashboard-control-center.md",
    },
    RequiredFd001Evidence {
        surface: "foundry",
        microservice: "foundry",
        evidence_file: "foundry.md",
    },
    RequiredFd001Evidence {
        surface: "workflow-engine",
        microservice: "workflow-engine",
        evidence_file: "workflow-engine.md",
    },
    RequiredFd001Evidence {
        surface: "workflow-studio",
        microservice: "workflow-studio",
        evidence_file: "workflow-studio.md",
    },
    RequiredFd001Evidence {
        surface: "ontology",
        microservice: "ontology",
        evidence_file: "ontology.md",
    },
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KoreaLocalizationEvidenceValidateArgs {
    repo_root: PathBuf,
    pack_overview_path: PathBuf,
    pack_manifest_path: PathBuf,
    corpus_lock_path: PathBuf,
    evidence_dir: PathBuf,
    emit_evidence_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KoreaLocalizationEvidenceReport {
    pub evidence_file_count: usize,
    pub fd001_surface_count: usize,
    pub kr_pack_surface_count: usize,
    pub pack_status: String,
    pub activation_claim: String,
    pub emitted_evidence_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RequiredFd001Evidence {
    surface: &'static str,
    microservice: &'static str,
    evidence_file: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EvidenceRecord {
    surface: String,
    microservice: String,
    evidence_path: String,
    source_paths: Vec<String>,
    kr_pack_surfaces: Vec<String>,
}

pub(crate) fn parse_korea_localization_evidence_validate_args(
    args: Vec<String>,
) -> Result<KoreaLocalizationEvidenceValidateArgs, String> {
    let mut parsed = KoreaLocalizationEvidenceValidateArgs {
        repo_root: PathBuf::from("."),
        pack_overview_path: PathBuf::from("docs/localization-packs/kr.md"),
        pack_manifest_path: PathBuf::from("docs/localization-packs/kr/pack.yaml"),
        corpus_lock_path: PathBuf::from("docs/localization-packs/kr/corpus.lock"),
        evidence_dir: PathBuf::from("docs/localization-packs/kr/evidence"),
        emit_evidence_path: None,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = PathBuf::from(path),
            "--pack-overview" => parsed.pack_overview_path = PathBuf::from(path),
            "--pack-manifest" => parsed.pack_manifest_path = PathBuf::from(path),
            "--corpus-lock" => parsed.corpus_lock_path = PathBuf::from(path),
            "--evidence-dir" => parsed.evidence_dir = PathBuf::from(path),
            "--emit-evidence" => parsed.emit_evidence_path = Some(PathBuf::from(path)),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_korea_localization_evidence_gate(
    args: KoreaLocalizationEvidenceValidateArgs,
) -> Result<KoreaLocalizationEvidenceReport, String> {
    let pack_overview_path = resolve_path(&args.repo_root, &args.pack_overview_path);
    let pack_manifest_path = resolve_path(&args.repo_root, &args.pack_manifest_path);
    let corpus_lock_path = resolve_path(&args.repo_root, &args.corpus_lock_path);
    let evidence_dir = resolve_path(&args.repo_root, &args.evidence_dir);

    let pack_overview = read_to_string(&pack_overview_path, "KR pack overview")?;
    validate_pack_overview(&pack_overview, &args.pack_manifest_path)?;

    let pack_manifest = read_to_string(&pack_manifest_path, "KR pack manifest")?;
    validate_pack_manifest(&pack_manifest)?;

    let corpus_lock = read_to_string(&corpus_lock_path, "KR corpus lock")?;
    validate_corpus_lock(&corpus_lock)?;

    if !evidence_dir.is_dir() {
        return Err(format!(
            "KR evidence directory missing: {}",
            evidence_dir.display()
        ));
    }

    let mut records = Vec::new();
    let mut seen_kr_pack_surfaces = BTreeSet::new();
    for required in REQUIRED_FD001_EVIDENCE {
        let record = validate_required_evidence(
            required,
            &args.repo_root,
            &args.evidence_dir,
            &evidence_dir,
        )?;
        for kr_surface in &record.kr_pack_surfaces {
            seen_kr_pack_surfaces.insert(kr_surface.clone());
        }
        records.push(record);
    }
    for required in REQUIRED_KR_PACK_SURFACES {
        if !seen_kr_pack_surfaces.contains(*required) {
            return Err(format!(
                "KR evidence pack does not cover required surface {required:?}"
            ));
        }
    }

    if let Some(evidence_path) = &args.emit_evidence_path {
        let evidence = build_evidence_bundle(&args, &records, &seen_kr_pack_surfaces);
        write_json(&resolve_path(&args.repo_root, evidence_path), &evidence)?;
    }

    Ok(KoreaLocalizationEvidenceReport {
        evidence_file_count: records.len(),
        fd001_surface_count: REQUIRED_FD001_EVIDENCE.len(),
        kr_pack_surface_count: seen_kr_pack_surfaces.len(),
        pack_status: PACK_STATUS.to_owned(),
        activation_claim: ACTIVATION_CLAIM.to_owned(),
        emitted_evidence_path: args.emit_evidence_path,
    })
}

fn validate_pack_overview(contents: &str, manifest_path: &Path) -> Result<(), String> {
    require_contains(contents, "pack_code: kr", "KR pack overview")?;
    require_contains(
        contents,
        "status: planning-closed-foundational",
        "KR pack overview",
    )?;
    require_contains(
        contents,
        &slash_path(manifest_path),
        "KR pack overview manifest pointer",
    )?;
    require_contains(
        contents,
        "docs/localization-packs/kr/evidence/<microservice>.md",
        "KR pack overview evidence policy",
    )?;
    require_contains(
        contents,
        "Substrate",
        "KR pack overview pack-neutral boundary",
    )?;
    Ok(())
}

fn validate_pack_manifest(contents: &str) -> Result<(), String> {
    require_contains(contents, "code: kr", "KR pack manifest")?;
    require_contains(
        contents,
        "status: planning-closed-foundational",
        "KR pack manifest",
    )?;
    require_contains(
        contents,
        "corpus_lock: \"kr/corpus.lock\"",
        "KR pack manifest",
    )?;
    require_contains(
        contents,
        "evidence_dir: \"kr/evidence/\"",
        "KR pack manifest",
    )?;
    for required in [
        "regulatory_bindings:",
        "retention-kr",
        "pipa-b2b",
        "microservices_in_scope:",
    ] {
        require_contains(contents, required, "KR pack manifest required section")?;
    }
    Ok(())
}

fn validate_corpus_lock(contents: &str) -> Result<(), String> {
    let value: Value = serde_json::from_str(contents)
        .map_err(|error| format!("KR corpus lock must be valid JSON: {error}"))?;
    require_json_string(&value, "pack_code", "kr", "KR corpus lock")?;
    require_json_string(&value, "status", "planning-closed", "KR corpus lock status")?;
    require_contains(
        contents,
        "\"active_promotion_requires_signed_attestation\": true",
        "KR corpus lock",
    )?;
    require_contains(
        contents,
        "\"disallowed_claim\"",
        "KR corpus lock claim control",
    )?;
    Ok(())
}

fn validate_required_evidence(
    required: &RequiredFd001Evidence,
    repo_root: &Path,
    evidence_dir_arg: &Path,
    evidence_dir: &Path,
) -> Result<EvidenceRecord, String> {
    let evidence_path = evidence_dir.join(required.evidence_file);
    let contents = read_to_string(&evidence_path, "KR evidence file")?;
    validate_evidence_markers(required, &contents)?;

    let source_paths = required_source_paths(required);
    for source_path in &source_paths {
        let absolute = resolve_path(repo_root, Path::new(source_path));
        if !absolute.is_file() {
            return Err(format!(
                "KR evidence {} cites missing source file {}",
                slash_path(&evidence_path),
                source_path
            ));
        }
        require_contains(
            &contents,
            source_path,
            &format!("KR evidence source citation for {}", required.surface),
        )?;
    }

    let kr_pack_surfaces = extract_kr_pack_surfaces(&contents);
    if kr_pack_surfaces.is_empty() {
        return Err(format!(
            "KR evidence {} must list at least one kr_pack_surface line",
            slash_path(&evidence_path)
        ));
    }
    let required_surface_set = REQUIRED_KR_PACK_SURFACES
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for kr_surface in &kr_pack_surfaces {
        if !required_surface_set.contains(kr_surface.as_str()) {
            return Err(format!(
                "KR evidence {} lists unknown kr_pack_surface {kr_surface:?}",
                slash_path(&evidence_path)
            ));
        }
    }

    Ok(EvidenceRecord {
        surface: required.surface.to_owned(),
        microservice: required.microservice.to_owned(),
        evidence_path: slash_path(&evidence_dir_arg.join(required.evidence_file)),
        source_paths,
        kr_pack_surfaces,
    })
}

fn validate_evidence_markers(
    required: &RequiredFd001Evidence,
    contents: &str,
) -> Result<(), String> {
    require_contains(contents, "pack_code: kr", "KR evidence front matter")?;
    require_contains(
        contents,
        &format!("fd001_surface: {}", required.surface),
        "KR evidence surface marker",
    )?;
    require_contains(
        contents,
        &format!("source_microservice: {}", required.microservice),
        "KR evidence microservice marker",
    )?;
    require_contains(
        contents,
        &format!("status: {PACK_STATUS}"),
        "KR evidence pack status",
    )?;
    require_contains(
        contents,
        &format!("activation_claim: {ACTIVATION_CLAIM}"),
        "KR evidence non-active marker",
    )?;
    for heading in REQUIRED_EVIDENCE_HEADINGS {
        require_contains(contents, heading, "KR evidence required heading")?;
    }
    require_contains(
        contents,
        "cargo run -q -p oya-dev-cli -- gate validate korea-localization-evidence",
        "KR evidence acceptance command",
    )?;
    require_contains(contents, "not active", "KR evidence non-claim")?;
    require_contains(contents, "no live tenant", "KR evidence tenant non-claim")?;
    let normalized = contents.to_ascii_lowercase();
    for forbidden in FORBIDDEN_EVIDENCE_MARKERS {
        if normalized.contains(forbidden) {
            return Err(format!(
                "KR evidence for {} contains forbidden marker {forbidden:?}",
                required.surface
            ));
        }
    }
    Ok(())
}

fn required_source_paths(required: &RequiredFd001Evidence) -> Vec<String> {
    vec![
        "docs/localization-packs/kr.md".to_owned(),
        "docs/localization-packs/kr/pack.yaml".to_owned(),
        "docs/localization-packs/kr/corpus.lock".to_owned(),
        format!("microservices/{}/manifest.json", required.microservice),
        format!("microservices/{}/PRD.md", required.microservice),
    ]
}

fn extract_kr_pack_surfaces(contents: &str) -> Vec<String> {
    let mut surfaces = BTreeSet::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        let Some(raw_value) = trimmed.strip_prefix("- kr_pack_surface:") else {
            continue;
        };
        let value = raw_value.trim();
        if !value.is_empty() {
            surfaces.insert(value.to_owned());
        }
    }
    surfaces.into_iter().collect()
}

fn build_evidence_bundle(
    args: &KoreaLocalizationEvidenceValidateArgs,
    records: &[EvidenceRecord],
    seen_kr_pack_surfaces: &BTreeSet<String>,
) -> Value {
    let coverage_by_kr_surface = REQUIRED_KR_PACK_SURFACES
        .iter()
        .map(|required| {
            let covering_files = records
                .iter()
                .filter(|record| {
                    record
                        .kr_pack_surfaces
                        .iter()
                        .any(|surface| surface == required)
                })
                .map(|record| record.evidence_path.clone())
                .collect::<Vec<_>>();
            ((*required).to_owned(), json!(covering_files))
        })
        .collect::<BTreeMap<_, _>>();
    json!({
        "schema_version": "oyatie.kr-localization-evidence.v1",
        "generated_by": "oya gate validate korea-localization-evidence",
        "pack_code": "kr",
        "pack_status": PACK_STATUS,
        "activation_claim": ACTIVATION_CLAIM,
        "non_claims": [
            "KR pack evidence is planning-closed-foundational, not active.",
            "This evidence does not prove live tenant readiness, signed regulatory attestation, or production-current legal interpretation."
        ],
        "inputs": {
            "repo_root": slash_path(&args.repo_root),
            "pack_overview": slash_path(&args.pack_overview_path),
            "pack_manifest": slash_path(&args.pack_manifest_path),
            "corpus_lock": slash_path(&args.corpus_lock_path),
            "evidence_dir": slash_path(&args.evidence_dir)
        },
        "fd001_evidence": records.iter().map(|record| json!({
            "surface": record.surface,
            "microservice": record.microservice,
            "evidence_path": record.evidence_path,
            "source_paths": record.source_paths,
            "kr_pack_surfaces": record.kr_pack_surfaces
        })).collect::<Vec<_>>(),
        "coverage": {
            "fd001_surface_count": REQUIRED_FD001_EVIDENCE.len(),
            "evidence_file_count": records.len(),
            "required_kr_pack_surface_count": REQUIRED_KR_PACK_SURFACES.len(),
            "covered_kr_pack_surface_count": seen_kr_pack_surfaces.len(),
            "by_kr_pack_surface": coverage_by_kr_surface
        },
        "acceptance_commands": [
            "cargo run -q -p oya-dev-cli -- gate validate korea-localization-evidence",
            "cargo run -q -p oya-dev-cli -- gate validate planning-closure",
            "cargo run -q -p oya-dev-cli -- gate validate canonical-base-neutrality"
        ]
    })
}

fn resolve_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn read_to_string(path: &Path, label: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("{label} unreadable {}: {error}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "KR evidence bundle directory unwritable {}: {error}",
                parent.display()
            )
        })?;
    }
    let serialized = serde_json::to_string_pretty(value)
        .map_err(|error| format!("KR evidence bundle not serializable: {error}"))?;
    fs::write(path, format!("{serialized}\n"))
        .map_err(|error| format!("KR evidence bundle unwritable {}: {error}", path.display()))
}

fn require_contains(contents: &str, needle: &str, context: &str) -> Result<(), String> {
    if contents.contains(needle) {
        Ok(())
    } else {
        Err(format!("{context} must contain {needle:?}"))
    }
}

fn require_json_string(
    value: &Value,
    field: &str,
    expected: &str,
    context: &str,
) -> Result<(), String> {
    let Some(actual) = value.get(field).and_then(Value::as_str) else {
        return Err(format!("{context} missing string field {field:?}"));
    };
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{context} field {field:?} must be {expected:?}, got {actual:?}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_sorted_pack_surface_markers() {
        let surfaces = extract_kr_pack_surfaces(
            "- kr_pack_surface: workflow_templates\n- kr_pack_surface: pack_manifest\n",
        );
        assert_eq!(
            surfaces,
            vec!["pack_manifest".to_owned(), "workflow_templates".to_owned()]
        );
    }

    #[test]
    fn rejects_unknown_pack_surface_marker() {
        let required = RequiredFd001Evidence {
            surface: "application",
            microservice: "application",
            evidence_file: "application.md",
        };
        let temp = std::env::temp_dir().join("oya-kr-evidence-unknown-surface-test");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("docs/localization-packs/kr/evidence")).expect("evidence dir");
        fs::create_dir_all(temp.join("docs/localization-packs/kr")).expect("pack dir");
        fs::create_dir_all(temp.join("microservices/application")).expect("microservice dir");
        fs::write(temp.join("docs/localization-packs/kr.md"), "pack_code: kr").expect("overview");
        fs::write(
            temp.join("docs/localization-packs/kr/pack.yaml"),
            "code: kr",
        )
        .expect("manifest");
        fs::write(temp.join("docs/localization-packs/kr/corpus.lock"), "{}").expect("corpus");
        fs::write(temp.join("microservices/application/manifest.json"), "{}").expect("manifest");
        fs::write(temp.join("microservices/application/PRD.md"), "# PRD").expect("prd");
        fs::write(
            temp.join("docs/localization-packs/kr/evidence/application.md"),
            "pack_code: kr\nfd001_surface: application\nsource_microservice: application\nstatus: planning-closed-foundational\nactivation_claim: not-active\n## Evidence\n## KR Pack Responsibilities\n## Non-Claims\nnot active; no live tenant\n## Exit Blockers\n## Acceptance Commands\ncargo run -q -p oya-dev-cli -- gate validate korea-localization-evidence\ndocs/localization-packs/kr.md\ndocs/localization-packs/kr/pack.yaml\ndocs/localization-packs/kr/corpus.lock\nmicroservices/application/manifest.json\nmicroservices/application/PRD.md\n- kr_pack_surface: not_real\n",
        )
        .expect("evidence");
        let error = validate_required_evidence(
            &required,
            &temp,
            Path::new("docs/localization-packs/kr/evidence"),
            &temp.join("docs/localization-packs/kr/evidence"),
        )
        .expect_err("unknown surface rejected");
        assert!(error.contains("unknown kr_pack_surface"));
        let _ = fs::remove_dir_all(temp);
    }
}
