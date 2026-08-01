use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use check_release_pack::{
    ComplianceRegulatorRef, ReleaseEvidencePackManifest, ReleaseEvidencePackPolicy,
    ReleaseEvidencePackRecord, validate_release_evidence_packs,
};
use check_supply_chain::{
    ImagePromotionRecord, ImagePromotionTier, ImagePromotionVerifier, ReleaseArtifact,
    ReleaseSupplyChainEvidence, SupplyChainEvidence, SupplyChainRecord,
    validate_image_promotion_pipeline, validate_pre_release_supply_chain,
    validate_release_supply_chain, validate_supply_chain,
};
use oya_governance_gate_catalog_domain::all_canonical_commands_rendered;

use crate::{
    clean_scalar_value, extract_json_array_for_key, extract_json_object_for_key,
    find_matching_json_delimiter, insert_scalar_field, next_arg, parse_bool_field,
    parse_json_string_value, parse_u32_cell_field, parse_u64_field, quoted_json_len,
    read_catalog_records, required_field, scalar_value, usage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SupplyChainValidateArgs {
    registry_dir: PathBuf,
    deny_config_path: PathBuf,
    /// Optional test-only override for the wired-commands corpus. When
    /// `None` (production default) the kernel sources its wired-commands
    /// catalog from `oya-governance-gate-catalog-domain` per the .sh-removal
    /// chain IP-C. When `Some(path)`, the CLI reads the path verbatim —
    /// used by the integration-test fixtures in
    /// `tests/gate_cli.rs` to exercise rejection paths.
    check_script_path: Option<PathBuf>,
    adr0039_script_path: PathBuf,
    adr0039_rust_path: PathBuf,
    workflows_dir: PathBuf,
    release_images_path: PathBuf,
    branch_protection_path: PathBuf,
    admission_policy_path: PathBuf,
    require_adr0039_evidence: bool,
}

pub(crate) fn parse_supply_chain_validate_args(
    args: Vec<String>,
) -> Result<SupplyChainValidateArgs, String> {
    let mut parsed = SupplyChainValidateArgs {
        registry_dir: PathBuf::from("registry/catalog"),
        deny_config_path: PathBuf::from("deny.toml"),
        check_script_path: None,
        adr0039_script_path: PathBuf::from("scripts/supply-chain-adr0039.sh"),
        adr0039_rust_path: PathBuf::from("marketplace/facade/dev-cli/src/commands/supply_chain.rs"),
        workflows_dir: PathBuf::from(".github/workflows"),
        release_images_path: PathBuf::from("registry/release/images.yaml"),
        branch_protection_path: PathBuf::from(".github/branch-protection.yaml"),
        admission_policy_path: PathBuf::from("infra/kyverno/policies/require-signed-images.yaml"),
        require_adr0039_evidence: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--require-adr0039-evidence" => parsed.require_adr0039_evidence = true,
            "--registry" => parsed.registry_dir = PathBuf::from(next_arg(&mut iter)?),
            "--deny" => parsed.deny_config_path = PathBuf::from(next_arg(&mut iter)?),
            "--check-script" => {
                parsed.check_script_path = Some(PathBuf::from(next_arg(&mut iter)?))
            }
            "--adr0039-script" => parsed.adr0039_script_path = PathBuf::from(next_arg(&mut iter)?),
            "--adr0039-rust" => parsed.adr0039_rust_path = PathBuf::from(next_arg(&mut iter)?),
            "--workflows-dir" => parsed.workflows_dir = PathBuf::from(next_arg(&mut iter)?),
            "--release-images" => parsed.release_images_path = PathBuf::from(next_arg(&mut iter)?),
            "--branch-protection" => {
                parsed.branch_protection_path = PathBuf::from(next_arg(&mut iter)?)
            }
            "--admission-policy" => {
                parsed.admission_policy_path = PathBuf::from(next_arg(&mut iter)?)
            }
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_supply_chain_gate(
    args: SupplyChainValidateArgs,
) -> Result<(usize, usize), String> {
    let records = read_supply_chain_records(&args.registry_dir)?;
    // Canonical catalog replaces the legacy `scripts/check.sh` file read
    // (audit `evidence/audits/shell-python-replacement-audit-2026-05-15.md`
    // row B-1, .sh-removal chain IP-C). The catalog substring-matches the
    // `cargo deny check` / `cargo audit` tokens the supply-chain kernel
    // historically required to find inside the script body.
    // Test-only override: `--check-script <path>` swaps the canonical
    // catalog for the file body, so integration-test fixtures can
    // exercise rejection paths against synthetic wired-command surfaces.
    let wired_commands = match args.check_script_path.as_ref() {
        Some(path) => fs::read_to_string(path).map_err(|error| {
            format!(
                "supply chain check script unreadable {}: {error}",
                path.display()
            )
        })?,
        None => all_canonical_commands_rendered(),
    };
    let workflow_text = read_workflow_text(&args.workflows_dir)?;
    let adr0039_script = read_optional_text(&args.adr0039_script_path)?;
    let adr0039_rust = read_optional_text(&args.adr0039_rust_path)?;
    let supply_chain_text =
        format!("{wired_commands}\n{workflow_text}\n{adr0039_script}\n{adr0039_rust}");
    let sbom_spdx_wired = sbom_spdx_wired(&supply_chain_text);
    let sbom_cyclonedx_wired = sbom_cyclonedx_wired(&supply_chain_text);
    let evidence = SupplyChainEvidence {
        deny_config_present: args.deny_config_path.is_file(),
        cargo_deny_check_wired: wired_commands.contains("cargo deny check"),
        cargo_audit_check_wired: wired_commands.contains("cargo audit"),
        third_party_actions_pinned: third_party_actions_are_pinned(&args.workflows_dir)?,
        require_adr0039_evidence: args.require_adr0039_evidence,
        release_manifest_present: args.release_images_path.is_file(),
        release_images_declared: release_images_declared(&args.release_images_path)?,
        release_empty_scope_declared: release_image_manifest_empty_scope_declared(
            &args.release_images_path,
        )?,
        trivy_release_scan_wired: trivy_container_scan_wired(&supply_chain_text),
        trivy_filesystem_scan_wired: trivy_filesystem_scan_wired(&supply_chain_text),
        trivy_container_scan_wired: trivy_container_scan_wired(&supply_chain_text),
        trivy_iac_scan_wired: trivy_iac_scan_wired(&supply_chain_text),
        trivy_dependency_scan_wired: trivy_dependency_scan_wired(&supply_chain_text),
        cosign_release_signing_wired: cosign_release_signing_wired(&supply_chain_text),
        cosign_rekor_verification_wired: cosign_rekor_verification_wired(&supply_chain_text),
        sbom_dual_format_wired: sbom_spdx_wired && sbom_cyclonedx_wired,
        sbom_spdx_wired,
        sbom_cyclonedx_wired,
        provenance_attestation_wired: provenance_attestation_wired(&supply_chain_text),
        signed_commit_policy_wired: signed_commit_policy_wired(&args.branch_protection_path)?,
        admission_policy_wired: admission_policy_wired(&args.admission_policy_path)?,
    };
    let report = validate_supply_chain(records, evidence)
        .map_err(|error| format!("supply chain invalid: {error:?}"))?;
    Ok((report.records_checked, report.source_only_records))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReleaseSupplyChainValidateArgs {
    release_images_path: PathBuf,
    evidence_dir: PathBuf,
    phase: ReleaseSupplyChainPhase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReleaseSupplyChainPhase {
    PreRelease,
    Release,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReleaseSupplyChainGateReport {
    pub(crate) artifacts: usize,
    pub(crate) evidence: usize,
    pub(crate) phase: ReleaseSupplyChainPhase,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImagePromotionValidateArgs {
    promotion_dir: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImagePromotionGateReport {
    pub(crate) artifacts: usize,
    pub(crate) promotion_records: usize,
    pub(crate) kubewarden_verifier_records: usize,
    pub(crate) kyverno_verifier_records: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReleaseArtifactManifest {
    artifacts: Vec<ReleaseArtifact>,
    empty_scope_declared: bool,
}

pub(crate) fn parse_release_supply_chain_validate_args(
    args: Vec<String>,
) -> Result<ReleaseSupplyChainValidateArgs, String> {
    let mut parsed = ReleaseSupplyChainValidateArgs {
        release_images_path: PathBuf::from("registry/release/images.yaml"),
        evidence_dir: PathBuf::from("registry/release/supply-chain"),
        phase: ReleaseSupplyChainPhase::Release,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--release-images" => parsed.release_images_path = PathBuf::from(next_arg(&mut iter)?),
            "--evidence-dir" => parsed.evidence_dir = PathBuf::from(next_arg(&mut iter)?),
            "--phase" => parsed.phase = parse_release_supply_chain_phase(&next_arg(&mut iter)?)?,
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_release_supply_chain_gate(
    args: ReleaseSupplyChainValidateArgs,
) -> Result<ReleaseSupplyChainGateReport, String> {
    let manifest = read_release_artifact_manifest(&args.release_images_path)?;
    let evidence = read_release_supply_chain_evidence_records(
        &args.evidence_dir,
        matches!(args.phase, ReleaseSupplyChainPhase::Release),
    )?;
    let report = match args.phase {
        ReleaseSupplyChainPhase::PreRelease => validate_pre_release_supply_chain(
            manifest.artifacts,
            evidence,
            manifest.empty_scope_declared,
        ),
        ReleaseSupplyChainPhase::Release => {
            validate_release_supply_chain(manifest.artifacts, evidence)
        }
    }
    .map_err(|error| format!("release supply chain invalid: {error:?}"))?;
    Ok(ReleaseSupplyChainGateReport {
        artifacts: report.artifacts_checked,
        evidence: report.evidence_records_checked,
        phase: args.phase,
    })
}

pub(crate) fn parse_image_promotion_validate_args(
    args: Vec<String>,
) -> Result<ImagePromotionValidateArgs, String> {
    let mut parsed = ImagePromotionValidateArgs {
        promotion_dir: PathBuf::from("registry/release/image-promotions"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--promotion-dir" => parsed.promotion_dir = PathBuf::from(next_arg(&mut iter)?),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_image_promotion_gate(
    args: ImagePromotionValidateArgs,
) -> Result<ImagePromotionGateReport, String> {
    let report =
        validate_image_promotion_pipeline(read_image_promotion_records(&args.promotion_dir)?)
            .map_err(|error| format!("image promotion invalid: {error:?}"))?;
    Ok(ImagePromotionGateReport {
        artifacts: report.artifacts_checked,
        promotion_records: report.promotion_records_checked,
        kubewarden_verifier_records: report.kubewarden_verifier_records,
        kyverno_verifier_records: report.kyverno_verifier_records,
    })
}

fn parse_release_supply_chain_phase(value: &str) -> Result<ReleaseSupplyChainPhase, String> {
    match value {
        "pre-release" => Ok(ReleaseSupplyChainPhase::PreRelease),
        "release" => Ok(ReleaseSupplyChainPhase::Release),
        _ => Err(usage()),
    }
}

pub(crate) fn release_supply_chain_phase_name(phase: ReleaseSupplyChainPhase) -> &'static str {
    match phase {
        ReleaseSupplyChainPhase::PreRelease => "pre-release",
        ReleaseSupplyChainPhase::Release => "release",
    }
}

fn read_release_artifact_manifest(
    release_images_path: &Path,
) -> Result<ReleaseArtifactManifest, String> {
    let contents = fs::read_to_string(release_images_path).map_err(|error| {
        format!(
            "release image manifest unreadable {}: {error}",
            release_images_path.display()
        )
    })?;
    let mut artifacts = Vec::new();
    let mut release_state = String::new();
    let mut empty_scope_rationale = String::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix('#') {
            if let Some(value) = scalar_value(comment.trim(), "release_state") {
                release_state = value;
            } else if let Some(value) = scalar_value(comment.trim(), "empty_scope_rationale") {
                empty_scope_rationale = value;
            }
            continue;
        }
        if trimmed == "images:" || trimmed == "images: []" {
            continue;
        }
        let ref_value = if let Some(value) = trimmed.strip_prefix("- ref:") {
            Some(value)
        } else if let Some(value) = trimmed.strip_prefix("ref:") {
            Some(value)
        } else if trimmed.starts_with("- ") && trimmed.contains('@') {
            Some(trimmed.trim_start_matches("- "))
        } else {
            None
        };
        if let Some(value) = ref_value {
            artifacts.push(ReleaseArtifact {
                artifact_ref: clean_scalar_value(value),
            });
        }
    }
    Ok(ReleaseArtifactManifest {
        empty_scope_declared: artifacts.is_empty()
            && release_state == "pre-release"
            && !empty_scope_rationale.trim().is_empty(),
        artifacts,
    })
}

fn read_release_supply_chain_evidence_records(
    evidence_dir: &Path,
    required: bool,
) -> Result<Vec<ReleaseSupplyChainEvidence>, String> {
    if !evidence_dir.is_dir() {
        if required {
            return Err(format!(
                "release supply-chain evidence directory missing: {}",
                evidence_dir.display()
            ));
        }
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(evidence_dir)
        .map_err(|error| format!("release supply-chain evidence directory unreadable: {error}"))?
    {
        let entry = entry.map_err(|error| format!("release evidence entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir()
            || !matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("yaml") | Some("yml")
            )
        {
            continue;
        }
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("release evidence unreadable {}: {error}", path.display()))?;
        records.push(parse_release_supply_chain_evidence(&path, &contents)?);
    }
    records.sort_by(|left, right| left.artifact_ref.cmp(&right.artifact_ref));
    Ok(records)
}

fn parse_release_supply_chain_evidence(
    path: &Path,
    contents: &str,
) -> Result<ReleaseSupplyChainEvidence, String> {
    let mut fields = BTreeMap::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        insert_scalar_field(path, &mut fields, trimmed)?;
    }
    Ok(ReleaseSupplyChainEvidence {
        artifact_ref: required_field(path, &fields, "artifact_ref")?,
        artifact_digest: required_field(path, &fields, "artifact_digest")?,
        release_version: required_field(path, &fields, "release_version")?,
        source_revision: required_field(path, &fields, "source_revision")?,
        sbom_spdx_ref: required_field(path, &fields, "sbom_spdx_ref")?,
        sbom_cyclonedx_ref: required_field(path, &fields, "sbom_cyclonedx_ref")?,
        cosign_signature_ref: required_field(path, &fields, "cosign_signature_ref")?,
        cosign_certificate_ref: required_field(path, &fields, "cosign_certificate_ref")?,
        rekor_log_index: parse_u64_field(
            &required_field(path, &fields, "rekor_log_index")?,
            "rekor_log_index",
        )?,
        trivy_filesystem_scan_ref: required_field(path, &fields, "trivy_filesystem_scan_ref")?,
        trivy_container_scan_ref: required_field(path, &fields, "trivy_container_scan_ref")?,
        trivy_iac_scan_ref: required_field(path, &fields, "trivy_iac_scan_ref")?,
        trivy_dependency_scan_ref: required_field(path, &fields, "trivy_dependency_scan_ref")?,
        provenance_attestation_ref: required_field(path, &fields, "provenance_attestation_ref")?,
        audit_event_type: required_field(path, &fields, "audit_event_type")?,
        attestor: required_field(path, &fields, "attestor")?,
        high_critical_findings_open: parse_u64_field(
            &required_field(path, &fields, "high_critical_findings_open")?,
            "high_critical_findings_open",
        )?,
        signed: parse_bool_field(path, "signed", &required_field(path, &fields, "signed")?)?,
    })
}

fn read_image_promotion_records(promotion_dir: &Path) -> Result<Vec<ImagePromotionRecord>, String> {
    if !promotion_dir.is_dir() {
        return Err(format!(
            "image promotion directory missing: {}",
            promotion_dir.display()
        ));
    }
    let mut records = Vec::new();
    for entry in fs::read_dir(promotion_dir)
        .map_err(|error| format!("image promotion directory unreadable: {error}"))?
    {
        let entry = entry.map_err(|error| format!("image promotion entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir()
            || !matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("yaml") | Some("yml")
            )
        {
            continue;
        }
        let contents = fs::read_to_string(&path).map_err(|error| {
            format!(
                "image promotion record unreadable {}: {error}",
                path.display()
            )
        })?;
        records.push(parse_image_promotion_record(&path, &contents)?);
    }
    records.sort_by(|left, right| {
        left.artifact_digest
            .cmp(&right.artifact_digest)
            .then(left.tier.cmp(&right.tier))
    });
    Ok(records)
}

fn parse_image_promotion_record(
    path: &Path,
    contents: &str,
) -> Result<ImagePromotionRecord, String> {
    let mut fields = BTreeMap::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        insert_scalar_field(path, &mut fields, trimmed)?;
    }
    Ok(ImagePromotionRecord {
        artifact_ref: required_field(path, &fields, "artifact_ref")?,
        artifact_digest: required_field(path, &fields, "artifact_digest")?,
        tier: parse_image_promotion_tier(path, &required_field(path, &fields, "tier")?)?,
        cosign_identity: required_field(path, &fields, "cosign_identity")?,
        verifier: parse_image_promotion_verifier(
            path,
            &required_field(path, &fields, "verifier")?,
        )?,
        verifier_ref: required_field(path, &fields, "verifier_ref")?,
        provenance_attestation_ref: required_field(path, &fields, "provenance_attestation_ref")?,
        runner_kill_switch_ref: required_field(path, &fields, "runner_kill_switch_ref")?,
        audit_event_type: required_field(path, &fields, "audit_event_type")?,
        signed: parse_bool_field(path, "signed", &required_field(path, &fields, "signed")?)?,
    })
}

fn parse_image_promotion_tier(path: &Path, value: &str) -> Result<ImagePromotionTier, String> {
    match value {
        "dev" => Ok(ImagePromotionTier::Dev),
        "staging" => Ok(ImagePromotionTier::Staging),
        "prod" | "production" => Ok(ImagePromotionTier::Prod),
        _ => Err(format!(
            "{}: image promotion tier must be one of dev, staging, prod: {value}",
            path.display()
        )),
    }
}

fn parse_image_promotion_verifier(
    path: &Path,
    value: &str,
) -> Result<ImagePromotionVerifier, String> {
    match value {
        "kubewarden" => Ok(ImagePromotionVerifier::Kubewarden),
        "kyverno" => Ok(ImagePromotionVerifier::Kyverno),
        _ => Err(format!(
            "{}: image promotion verifier must be kubewarden or kyverno: {value}",
            path.display()
        )),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReleaseEvidencePackValidateArgs {
    manifest_path: PathBuf,
    compliance_matrix_path: PathBuf,
    require_records: bool,
}

pub(crate) fn parse_release_evidence_pack_validate_args(
    args: Vec<String>,
) -> Result<ReleaseEvidencePackValidateArgs, String> {
    let mut parsed = ReleaseEvidencePackValidateArgs {
        manifest_path: PathBuf::from("registry/release/evidence-packs.tsv"),
        compliance_matrix_path: PathBuf::from("docs/machine-readable/compliance.json"),
        require_records: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--manifest" => parsed.manifest_path = PathBuf::from(next_arg(&mut iter)?),
            "--compliance" => parsed.compliance_matrix_path = PathBuf::from(next_arg(&mut iter)?),
            "--require-records" => parsed.require_records = true,
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_release_evidence_pack_gate(
    args: ReleaseEvidencePackValidateArgs,
) -> Result<(usize, usize, usize), String> {
    let (manifest, records) = read_release_evidence_pack_manifest(&args.manifest_path)?;
    let known_regulators = read_compliance_regulator_refs(&args.compliance_matrix_path)?;
    let policy = if args.require_records {
        ReleaseEvidencePackPolicy::release_blocking_sla()
    } else {
        ReleaseEvidencePackPolicy::compliance_matrix_sla()
    };
    let report = validate_release_evidence_packs(manifest, records, known_regulators, policy)
        .map_err(|error| format!("release evidence pack invalid: {error:?}"))?;
    Ok((
        report.known_regulators_checked,
        report.records_checked,
        report.published_records_checked,
    ))
}

fn read_release_evidence_pack_manifest(
    manifest_path: &Path,
) -> Result<(ReleaseEvidencePackManifest, Vec<ReleaseEvidencePackRecord>), String> {
    let contents = fs::read_to_string(manifest_path).map_err(|error| {
        format!(
            "release evidence-pack manifest unreadable {}: {error}",
            manifest_path.display()
        )
    })?;
    let mut release_version = None;
    let mut empty_scope_rationale = None;
    let mut seen_header = false;
    let mut records = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix('#') {
            if let Some(value) = scalar_value(comment.trim(), "release_version") {
                release_version = Some(value);
            } else if let Some(value) = scalar_value(comment.trim(), "empty_scope_rationale") {
                empty_scope_rationale = Some(value);
            }
            continue;
        }
        if trimmed.starts_with("regulator\t") {
            seen_header = true;
            continue;
        }
        if !seen_header {
            return Err(format!(
                "{}: expected regulator TSV header before records",
                manifest_path.display()
            ));
        }
        records.push(parse_release_evidence_pack_row(manifest_path, trimmed)?);
    }
    if !seen_header {
        return Err(format!(
            "{}: missing regulator TSV header",
            manifest_path.display()
        ));
    }
    Ok((
        ReleaseEvidencePackManifest {
            release_version: release_version.unwrap_or_default(),
            empty_scope_rationale: empty_scope_rationale.unwrap_or_default(),
        },
        records,
    ))
}

fn parse_release_evidence_pack_row(
    manifest_path: &Path,
    row: &str,
) -> Result<ReleaseEvidencePackRecord, String> {
    let cells = row
        .split('\t')
        .map(|cell| cell.trim().trim_matches('`').to_string())
        .collect::<Vec<_>>();
    if cells.len() != 18 {
        return Err(format!(
            "{}: release evidence-pack row must have 18 TSV columns: {row}",
            manifest_path.display()
        ));
    }
    Ok(ReleaseEvidencePackRecord {
        regulator: cells[0].clone(),
        region: cells[1].clone(),
        pack_id: cells[2].clone(),
        release_version: cells[3].clone(),
        audit_cycle: cells[4].clone(),
        coverage_window_start: cells[5].clone(),
        coverage_window_end: cells[6].clone(),
        owner_team: cells[7].clone(),
        evidence_pack_ref: cells[8].clone(),
        cosign_attestation_ref: cells[9].clone(),
        audit_event_id: cells[10].clone(),
        requested_at_epoch_minutes: parse_u64_field(&cells[11], "requested_at_epoch_minutes")?,
        regenerated_at_epoch_minutes: parse_u64_field(&cells[12], "regenerated_at_epoch_minutes")?,
        controls_mapped: parse_u32_cell_field(manifest_path, "controls_mapped", &cells[13])?,
        evidence_links: parse_u32_cell_field(manifest_path, "evidence_links", &cells[14])?,
        trust_portal_mirror_regenerated: parse_bool_field(
            manifest_path,
            "trust_portal_mirror_regenerated",
            &cells[15],
        )?,
        regulator_notification_sent: parse_bool_field(
            manifest_path,
            "regulator_notification_sent",
            &cells[16],
        )?,
        status: cells[17].clone(),
    })
}

fn read_compliance_regulator_refs(path: &Path) -> Result<Vec<ComplianceRegulatorRef>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "machine-readable compliance matrix unreadable {}: {error}",
            path.display()
        )
    })?;
    let regulators_object = extract_json_object_for_key(&contents, "regulators_per_region")
        .ok_or_else(|| "compliance matrix missing regulators_per_region".to_string())?;
    let mut refs = BTreeMap::new();
    for regulator in collect_json_array_string_values(regulators_object) {
        refs.insert(regulator.clone(), ComplianceRegulatorRef { regulator });
    }
    let standards_array = extract_json_array_for_key(&contents, "cross_regional_standards")
        .ok_or_else(|| "compliance matrix missing cross_regional_standards".to_string())?;
    for regulator in json_string_values_in_array(standards_array) {
        refs.insert(regulator.clone(), ComplianceRegulatorRef { regulator });
    }
    Ok(refs.into_values().collect())
}

fn collect_json_array_string_values(contents: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = contents;
    while let Some(array_start_index) = rest.find('[') {
        let array = &rest[array_start_index..];
        let Some(array_end_index) = find_matching_json_delimiter(array, '[', ']') else {
            break;
        };
        values.extend(json_string_values_in_array(&array[1..array_end_index]));
        rest = &array[array_end_index + 1..];
    }
    values
}

fn json_string_values_in_array(array: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = array;
    while let Some(quote_index) = rest.find('"') {
        let value_start = &rest[quote_index..];
        let Some(value) = parse_json_string_value(value_start) else {
            break;
        };
        if !value.trim().is_empty() {
            values.push(value);
        }
        let Some(consumed) = quoted_json_len(value_start) else {
            break;
        };
        rest = &value_start[consumed..];
    }
    values
}

fn read_supply_chain_records(registry_dir: &Path) -> Result<Vec<SupplyChainRecord>, String> {
    read_catalog_records(registry_dir)?
        .into_iter()
        .map(|record| {
            Ok(SupplyChainRecord {
                subject: record.crate_id.value,
                attestation: supply_chain_attestation_id(record.supply_chain.value),
            })
        })
        .collect()
}

fn supply_chain_attestation_id(
    attestation: intelligence_catalog_domain::SupplyChainAttestation,
) -> String {
    match attestation {
        intelligence_catalog_domain::SupplyChainAttestation::SourceOnly => "source-only",
        intelligence_catalog_domain::SupplyChainAttestation::LicenseChecked => {
            "license-checked"
        }
        intelligence_catalog_domain::SupplyChainAttestation::Sbom => "sbom",
        intelligence_catalog_domain::SupplyChainAttestation::SignedProvenance => {
            "signed-provenance"
        }
    }
    .into()
}

fn third_party_actions_are_pinned(workflows_dir: &Path) -> Result<bool, String> {
    for line in read_workflow_lines(workflows_dir)? {
        let Some(action_ref) = workflow_action_ref(&line) else {
            continue;
        };
        if action_ref.starts_with("./") || action_ref.starts_with("docker://") {
            continue;
        }
        let Some((owner_repo, reference)) = action_ref.rsplit_once('@') else {
            return Ok(false);
        };
        if owner_repo.starts_with("actions/") {
            continue;
        }
        if reference.len() != 40 || !reference.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn read_workflow_text(workflows_dir: &Path) -> Result<String, String> {
    Ok(read_workflow_lines(workflows_dir)?.join("\n"))
}

fn read_optional_text(path: &Path) -> Result<String, String> {
    if !path.is_file() {
        return Ok(String::new());
    }
    fs::read_to_string(path).map_err(|error| {
        format!(
            "optional evidence file unreadable {}: {error}",
            path.display()
        )
    })
}

fn read_workflow_lines(workflows_dir: &Path) -> Result<Vec<String>, String> {
    if !workflows_dir.exists() {
        return Ok(Vec::new());
    }
    let mut lines = Vec::new();
    for entry in fs::read_dir(workflows_dir)
        .map_err(|error| format!("workflows directory unreadable: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("workflows directory entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if !matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("yaml") | Some("yml")
        ) {
            continue;
        }
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("workflow unreadable {}: {error}", path.display()))?;
        lines.extend(contents.lines().map(str::to_string));
    }
    Ok(lines)
}

fn workflow_action_ref(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let value = trimmed
        .strip_prefix("-")
        .map(str::trim)
        .unwrap_or(trimmed)
        .strip_prefix("uses:")?
        .trim();
    Some(value.trim_matches('"').trim_matches('\''))
}

fn release_images_declared(release_images_path: &Path) -> Result<bool, String> {
    if !release_images_path.is_file() {
        return Ok(false);
    }
    let contents = fs::read_to_string(release_images_path).map_err(|error| {
        format!(
            "release image manifest unreadable {}: {error}",
            release_images_path.display()
        )
    })?;
    Ok(contents.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "images:" {
            return false;
        }
        trimmed.starts_with("- ")
            || trimmed.starts_with("name:")
            || trimmed.starts_with("ref:")
            || trimmed.starts_with("image:")
    }))
}

fn release_image_manifest_empty_scope_declared(release_images_path: &Path) -> Result<bool, String> {
    if !release_images_path.is_file() {
        return Ok(false);
    }
    let manifest = read_release_artifact_manifest(release_images_path)?;
    Ok(manifest.empty_scope_declared)
}

fn trivy_filesystem_scan_wired(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("trivy fs")
        && text.contains("--severity high,critical")
        && text.contains("--exit-code 1")
}

fn trivy_container_scan_wired(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("trivy image")
        && text.contains("--severity high,critical")
        && text.contains("--exit-code 1")
}

fn trivy_iac_scan_wired(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("trivy config")
        && text.contains("--severity high,critical")
        && text.contains("--exit-code 1")
}

fn trivy_dependency_scan_wired(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("trivy fs") && text.contains("--scanners vuln,secret,license")
}

fn cosign_release_signing_wired(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("cosign sign")
}

fn cosign_rekor_verification_wired(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("rekor") && text.contains("cosign verify")
}

fn sbom_spdx_wired(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("spdx 2.3") || text.contains("spdx-json") || text.contains(".spdx")
}

fn sbom_cyclonedx_wired(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("cyclonedx 1.5")
        || text.contains("--format cyclonedx")
        || text.contains(".cyclonedx")
}

fn provenance_attestation_wired(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("cosign attest") || text.contains("slsa")
}

fn signed_commit_policy_wired(branch_protection_path: &Path) -> Result<bool, String> {
    if !branch_protection_path.is_file() {
        return Ok(false);
    }
    let contents = fs::read_to_string(branch_protection_path).map_err(|error| {
        format!(
            "branch-protection policy unreadable {}: {error}",
            branch_protection_path.display()
        )
    })?;
    let contents = contents.to_ascii_lowercase();
    Ok(contents.contains("require_signed_commits: true")
        && contents.contains("require_signed_tags: true"))
}

fn admission_policy_wired(admission_policy_path: &Path) -> Result<bool, String> {
    if !admission_policy_path.is_file() {
        return Ok(false);
    }
    let contents = fs::read_to_string(admission_policy_path).map_err(|error| {
        format!(
            "admission policy unreadable {}: {error}",
            admission_policy_path.display()
        )
    })?;
    let contents = contents.to_ascii_lowercase();
    Ok(contents.contains("verifyimages")
        && contents.contains("keyless")
        && contents.contains("rekor"))
}
