// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` to assert
// invariants under the `cfg(test)` exemption (production code is Tier 1).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_check_adr_citation::{AdrCitationDocument, validate_adr_citations};
use oya_check_brand_residue::{BrandResidueDocument, validate_brand_residue};
use oya_check_glossary_vocabulary::{
    GlossaryVocabularyWarning, GlossaryVocabularyWarningKind, GlossaryVocabularyWarningSource,
    IgnoredUppercaseWord, VocabularyDocument,
    validate_glossary_vocabulary_hygiene_with_baseline_and_ignored_words,
    validate_glossary_vocabulary_hygiene_with_ignored_words,
};
use oya_check_license_policy::LicensePolicy;
use oya_check_mobile_native::{
    MobileNativeDiscoveryMarker, MobileNativeManifest, MobileNativePolicy,
    MobileNativeProductRecord, validate_mobile_native,
};
use oya_check_vendor_recency::{
    VendorContractRecencyPolicy, VendorContractRecord, validate_vendor_contract_recency,
};
use oya_foundry_api_semver_domain::validate_api_semver;
use oya_foundry_cargo_prefix_domain::{CargoPrefixMember, validate_cargo_prefix};

mod active_artifact_contract_gate;
mod api_contract_registry;
mod architecture_map_emit_gate;
mod architecture_plane_gates;
mod catalog_contract_gates;
mod catalog_registry;
mod cedar_fragment_coverage_gate;
mod codeview_read_surface_gates;
mod command_output;
mod command_process;
mod commands;
mod cross_axis_contracts;
mod cross_tenant_access_gates;
mod data_class_gates;
mod date_utils;
mod documentation_gates;
mod foundation_audit_gates;
mod foundation_fixture;
mod foundry_capability_schema_gates;
mod foundry_eval_gates;
mod glossary_cross_doc_gates;
mod governance_gates;
mod json_scan;
mod openapi_rest_route_parity_gate;
mod path_format;
mod placeholder_debt_gates;
mod quality_lane_gates;
mod runbook_gates;
mod scalability_gates;
mod scalar_parse;
mod supply_chain_gates;
mod team_ownership_gates;
mod typescript_workspace_gates;
mod workspace_manifest;
mod yaml_scan;

pub(crate) use active_artifact_contract_gate::{
    parse_active_artifact_contract_validate_args, validate_active_artifact_contract_gate,
};
pub(crate) use api_contract_registry::{is_api_contract_metadata_path, read_api_contract_records};
pub(crate) use architecture_map_emit_gate::{
    emit_architecture_map_gate, parse_architecture_map_emit_args,
};
pub(crate) use architecture_plane_gates::{
    parse_planes_validate_args, parse_wave_integration_validate_args, validate_planes_gate,
    validate_wave_integration_gate,
};
pub(crate) use catalog_contract_gates::{
    parse_cohesion_validate_args, parse_slo_coverage_validate_args, validate_cohesion_gate,
    validate_slo_coverage_gate,
};
pub(crate) use catalog_registry::read_catalog_records;
pub(crate) use cedar_fragment_coverage_gate::{
    parse_cedar_fragment_coverage_validate_args, validate_cedar_fragment_coverage_gate,
};
pub(crate) use codeview_read_surface_gates::{
    parse_codeview_read_surface_validate_args, validate_codeview_read_surface_gate,
};
pub(crate) use cross_axis_contracts::read_cross_axis_contracts;
pub(crate) use cross_tenant_access_gates::{
    parse_cross_tenant_access_fuzz_validate_args, validate_cross_tenant_access_fuzz_gate,
};
pub(crate) use data_class_gates::{parse_data_class_validate_args, validate_data_class_gate};
pub(crate) use date_utils::{
    current_epoch_days, current_epoch_days_i64, parse_yyyy_mm_dd_to_epoch_days,
};
pub(crate) use documentation_gates::{
    parse_doc_catalog_validate_args, parse_documentation_system_validate_args,
    parse_readme_doc_coverage_validate_args, validate_doc_catalog_gate,
    validate_documentation_system_gate, validate_readme_doc_coverage_gate,
};
pub(crate) use foundation_audit_gates::{
    parse_audit_chain_replay_validate_args, parse_foundation_bypass_validate_args,
    parse_pr_traceability_validate_args, validate_audit_chain_replay_gate,
    validate_foundation_bypass_gate, validate_pr_traceability_gate,
};
pub(crate) use foundry_capability_schema_gates::{
    parse_foundry_capability_schema_validate_args, validate_foundry_capability_schema_gate,
};
pub(crate) use foundry_eval_gates::{parse_foundry_eval_validate_args, validate_foundry_eval_gate};
pub(crate) use glossary_cross_doc_gates::{
    parse_glossary_coverage_validate_args, validate_glossary_coverage_gate,
};
pub(crate) use governance_gates::{
    parse_authority_cohesion_validate_args, parse_claim_ceiling_validate_args,
    parse_plane_class_validate_args, validate_authority_cohesion_gate, validate_claim_ceiling_gate,
    validate_plane_class_gate,
};
pub(crate) use json_scan::{
    extract_json_array_for_key, extract_json_object_entries, extract_json_object_for_key,
    extract_json_objects, find_matching_json_delimiter, json_field_has_non_empty_value,
    parse_json_string_array_field, parse_json_string_field, parse_json_string_value,
    quoted_json_len,
};
pub(crate) use openapi_rest_route_parity_gate::{
    parse_openapi_rest_route_parity_validate_args, validate_openapi_rest_route_parity_gate,
};
pub(crate) use path_format::slash_path;
pub(crate) use placeholder_debt_gates::{
    parse_placeholder_debt_validate_args, validate_placeholder_debt_gate,
};
pub(crate) use quality_lane_gates::{
    parse_quality_lanes_validate_args, validate_quality_lanes_gate,
};
pub(crate) use runbook_gates::{
    parse_runbook_freshness_validate_args, parse_runbook_index_validate_args,
    validate_runbook_freshness_gate, validate_runbook_index_gate,
};
pub(crate) use scalability_gates::{
    parse_benchmark_validate_args, parse_perf_budget_validate_args,
    parse_shardability_validate_args, parse_statelessness_validate_args, validate_benchmark_gate,
    validate_perf_budget_gate, validate_shardability_gate, validate_statelessness_gate,
};
pub(crate) use scalar_parse::{
    clean_scalar_value, insert_scalar_field, parse_bool_field, parse_u8_percent, parse_u32_field,
    parse_u64_field, required_field, required_scalar, scalar_value,
};
pub(crate) use supply_chain_gates::{
    parse_release_evidence_pack_validate_args, parse_release_supply_chain_validate_args,
    parse_supply_chain_validate_args, release_supply_chain_phase_name,
    validate_release_evidence_pack_gate, validate_release_supply_chain_gate,
    validate_supply_chain_gate,
};
pub(crate) use team_ownership_gates::{
    list_team_ids, parse_codeowners_mirror_validate_args, parse_raci_team_coverage_validate_args,
    validate_codeowners_mirror_gate, validate_raci_team_coverage_gate,
};
pub(crate) use typescript_workspace_gates::{
    parse_typescript_workspace_validate_args, validate_typescript_workspace_gate,
};
pub(crate) use workspace_manifest::{
    read_package_license, read_package_name, read_workspace_member_crate_ids,
    read_workspace_member_paths,
};
pub(crate) use yaml_scan::{clean_yaml_value, parse_yaml_inline_values};

pub fn run_cli_from_env() -> ExitCode {
    let mut raw_args = std::env::args();
    let program = raw_args.next().unwrap_or_default();
    let args = raw_args.collect::<Vec<_>>();
    if Path::new(&program)
        .file_stem()
        .and_then(|stem| stem.to_str())
        == Some("repoctl")
    {
        return commands::repoctl::run(args, &usage());
    }

    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("demo") => commands::demo::run(args.collect(), &usage()),
        Some("check") => commands::check::run(args.collect(), &usage()),
        Some("dev") => commands::dev::run(args.collect(), &usage()),
        Some("doc") => commands::doc::run(args.collect(), &usage()),
        Some("repoctl") => commands::repoctl::run(args.collect(), &usage()),
        Some("catalog") => commands::catalog::run(args.collect(), &usage()),
        Some("gate") => commands::gate::run(args.collect(), &usage()),
        Some("vcs") => commands::vcs::run(args.collect(), &usage()),
        Some("verify") => commands::verify::run(args.collect(), &usage()),
        _ => {
            eprintln!("{}", usage());
            ExitCode::from(2)
        }
    }
}

pub(crate) fn usage() -> String {
    "Usage: oya demo [--audit-ledger <path>] [--evidence-store <path>] [--run-ledger <path>] [--step-ledger <path>] [--outbox-store <path>] [--secret-store <path>]\n       oya check <architecture|bounded-context|supply-chain|semver|documentation|statelessness|shardability|perf-budget|benchmark> [check-specific args]\n       oya dev check [--check-script <scripts/check.sh>] [--format <text|json>]\n       oya doc rustdoc [--target-dir <target/oya-rustdoc-check>] [--rustdoc <path>] [--cargo <path>] [--format <text|json>] [--keep-target-dir]\n       oya doc openapi [--contracts-dir <contracts>] [--spec <docs/SPEC.md>] [--contracts-mirror <docs/machine-readable/contracts.json>] [--runtime-bindings <registry/openapi/runtime-bindings.tsv>] [--schema-bindings <registry/openapi/schema-bindings.tsv>] [--runtime-root <.>] [--format <text|json>]\n       oya doc mdbook [--site-dir <docs/site>] [--format <text|json>]\n       oya doc adr-index [--decisions-dir <docs/decisions>] [--index <docs/ADR-INDEX.md>] [--machine <docs/machine-readable/decisions.json>] [--write] [--format <text|json>]\n       oya doc inventory [--repo-root <.>] [--workspace <Cargo.toml>] [--crate-registry <registry/catalog>] [--doc-catalog <docs/machine-readable/catalog.json>] [--contracts-dir <contracts>] [--products-dir <docs/products>] [--capabilities-dir <product-control/capabilities>] [--out <docs/machine-readable/documentation-inventory.json>] [--write] [--format <text|json>]\n       repoctl pre-push [--check-script <scripts/check.sh>] [--format <text|json>] [--verify-contract]\n       oya catalog validate [--workspace <Cargo.toml>] [--registry <registry/catalog>]"
        .to_string()
        + "\n       oya vcs [--format <text|json>] [--policy <observe|warn|enforce>] [--evidence-command <shell-command>] <claim|work|verify|done|status|symbols|queue|watch|promote> [vcs-specific args]"
        + "\n       oya gate validate foundation-bypass [--ledger <registry/foundation-bypasses>] [--now-epoch-days <days>]"
        + "\n       oya gate validate audit-chain-replay [--shards-dir <registry/audit-chain/shards>]"
        + "\n       oya gate validate foundry-capability-schema [--capabilities-dir <product-control/capabilities>]"
        + "\n       oya gate validate foundry-eval [--capabilities-dir <product-control/capabilities>]"
        + "\n       oya gate validate cross-tenant-access-fuzz"
        + "\n       oya gate validate adr-citation [--docs-dir <docs>] [--decisions-dir <docs/decisions>] [--inheritance-registry <registry/adr/inherited-bominal-adrs.yaml>]"
        + "\n       oya gate validate brand-residue [--docs-dir <docs>]"
        + "\n       oya gate validate api-semver [--contracts-dir <contracts>]"
        + "\n       oya gate validate supply-chain [--registry <registry/catalog>] [--deny <deny.toml>] [--check-script <scripts/check.sh>] [--adr0039-script <scripts/supply-chain-adr0039.sh>] [--workflows-dir <.github/workflows>] [--release-images <registry/release/images.yaml>] [--branch-protection <.github/branch-protection.yaml>] [--admission-policy <infra/kyverno/policies/require-signed-images.yaml>] [--require-adr0039-evidence]"
        + "\n       oya gate validate release-supply-chain [--release-images <registry/release/images.yaml>] [--evidence-dir <registry/release/supply-chain>] [--phase <pre-release|release>]"
        + "\n       oya gate validate release-evidence-pack [--manifest <registry/release/evidence-packs.tsv>] [--compliance <docs/machine-readable/compliance.json>] [--require-records]"
        + "\n       oya gate validate typescript-workspace --lane <typecheck|test> [--repo-root <.>]"
        + "\n       oya gate validate pr-traceability [--pr-body <docs/templates/pull-request-template.md>] [--require-code-review|--forbid-code-review]"
        + "\n       oya gate validate authority-cohesion [--docs-dir <docs>]"
        + "\n       oya gate validate cargo-prefix [--workspace <Cargo.toml>] [--prefix <oya->]"
        + "\n       oya gate validate claim-ceiling [--registry <registry/catalog>]"
        + "\n       oya gate validate codeview-read-surface [--spec <specs/cross-cutting/codeview-read-surface.json>]"
        + "\n       oya gate validate cohesion [--workspace <Cargo.toml>] [--registry <registry/catalog>] [--contracts <docs/machine-readable/contracts.json>]"
        + "\n       oya gate validate codeowners-mirror [--codeowners <.github/CODEOWNERS>] [--teams-dir <docs/teams>]"
        + "\n       oya gate validate statelessness [--workspace-root <.>] [--allow-empty]"
        + "\n       oya gate validate shardability [--migrations-dir <migrations>] [--allow-empty]"
        + "\n       oya gate validate perf-budget [--plans-dir <.omc/plans/milestones>] [--allow-empty]"
        + "\n       oya gate validate benchmark [--prds-dir <docs/prds>] [--products-dir <docs/products>] [--competitor <name>] [--allow-empty]"
        + "\n       oya gate validate data-class [--workspace <Cargo.toml>] [--legacy <registry/data-class/legacy-unannotated-fields.tsv>]"
        + "\n       oya gate validate doc-catalog [--docs-dir <docs>] [--catalog <docs/machine-readable/catalog.json>]"
        + "\n       oya gate validate documentation-system [--documentation <docs/DOCUMENTATION.md>] [--pipeline <registry/docs/pipeline.tsv>] [--check-script <scripts/check.sh>] [--wiki-quickref <docs/wiki/quickref/README.md>] [--repo-root <.>]"
        + "\n       oya gate validate glossary-cross-doc-coverage [--docs-dir <docs>] [--glossary <docs/GLOSSARY.md>] [--machine <docs/machine-readable/glossary.json>]"
        + "\n       oya gate validate glossary-vocabulary [--docs-dir <docs>] [--glossary <docs/GLOSSARY.md>] [--baseline <registry/glossary-vocabulary/warning-baseline.tsv>] [--ignored-uppercase-words <registry/glossary-vocabulary/ignored-uppercase-words.tsv>] [--write-baseline <path>] [--write-warning-report <path>]"
        + "\n       oya gate validate placeholder-debt [--docs-dir <docs>] [--registry <registry/placeholder-debt/registry.tsv>] [--write-registry <path>] [--write-report <path>]"
        + "\n       oya gate validate quality-lanes [--registry <registry/quality/lanes.yaml>] [--ci-lanes <docs/standards/ci-lanes.md>] [--check-script <scripts/check.sh>] [--teams-dir <docs/teams>]"
        + "\n       oya gate validate license-policy [--workspace <Cargo.toml>]"
        + "\n       oya gate validate vendor-contract-recency [--ledger <docs/VENDOR-PARTNER-LEDGER.md>] [--today <YYYY-MM-DD>] [--renewal-window-days <90>]"
        + "\n       oya gate validate planes --all [--repo-root <.>]"
        + "\n       oya gate validate wave-integration --milestone <M02> [--manifest <.omc/plans/M01-M03-parallelization-manifest.md>] [--phases-dir <.omc/plans/milestones/M02-substrate/phases>]"
        + "\n       oya gate validate mobile-native [--manifest <registry/mobile-native/products.tsv>] [--repo-root <.>]"
        + "\n       oya gate validate plane-class [--registry <registry/catalog>] [--baseline <registry/catalog>] [--reviewed-change <crate-id>]"
        + "\n       oya gate validate raci-team-coverage [--teams-dir <docs/teams>] [--raci <docs/RACI-OWNERSHIP.md>] [--codeowners <.github/CODEOWNERS>]"
        + "\n       oya gate validate readme-doc-coverage [--docs-dir <docs>] [--catalog <docs/machine-readable/catalog.json>]"
        + "\n       oya gate validate runbook-index-resolves [--docs-dir <docs>]"
        + "\n       oya gate validate runbook-freshness [--runbooks-dir <docs/runbooks>] [--today <YYYY-MM-DD>]"
        + "\n       oya gate validate slo-coverage [--registry <registry/catalog>]"
        + "\n       oya gate validate architecture-boundaries [--repo-root <.>] [--registry <registry/catalog>] [--self-test]"
        + "\n       oya gate run-all [--include-deferred]"
        + "\n       oya verify [--include-deferred]   # local-developer fold of `gate run-all`; canonical pre-push entry"
}

pub(crate) fn path_has_component(path: &Path, component: &str) -> bool {
    path.components()
        .any(|path_component| path_component.as_os_str().to_str() == Some(component))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GlossaryVocabularyValidateArgs {
    docs_dir: PathBuf,
    glossary_path: PathBuf,
    baseline_path: PathBuf,
    ignored_uppercase_words_path: PathBuf,
    write_baseline_path: Option<PathBuf>,
    warning_report_path: Option<PathBuf>,
}

fn parse_glossary_vocabulary_validate_args(
    args: Vec<String>,
) -> Result<GlossaryVocabularyValidateArgs, String> {
    let mut parsed = GlossaryVocabularyValidateArgs {
        docs_dir: PathBuf::from("docs"),
        glossary_path: PathBuf::from("docs/GLOSSARY.md"),
        baseline_path: PathBuf::from("registry/glossary-vocabulary/warning-baseline.tsv"),
        ignored_uppercase_words_path: PathBuf::from(
            "registry/glossary-vocabulary/ignored-uppercase-words.tsv",
        ),
        write_baseline_path: None,
        warning_report_path: None,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--docs-dir" => parsed.docs_dir = PathBuf::from(path),
            "--glossary" => parsed.glossary_path = PathBuf::from(path),
            "--baseline" => parsed.baseline_path = PathBuf::from(path),
            "--ignored-uppercase-words" => {
                parsed.ignored_uppercase_words_path = PathBuf::from(path)
            }
            "--write-baseline" => parsed.write_baseline_path = Some(PathBuf::from(path)),
            "--write-warning-report" => parsed.warning_report_path = Some(PathBuf::from(path)),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn validate_glossary_vocabulary_gate(
    args: GlossaryVocabularyValidateArgs,
) -> Result<(usize, usize, usize), String> {
    let documents = read_markdown_vocabulary_documents(&args.docs_dir)?;
    let allowed_acronyms = read_glossary_acronyms(&args.glossary_path)?;
    let ignored_uppercase_words = read_ignored_uppercase_words(&args.ignored_uppercase_words_path)?;
    let report = if let Some(path) = &args.write_baseline_path {
        let report = validate_glossary_vocabulary_hygiene_with_ignored_words(
            documents,
            allowed_acronyms,
            ignored_uppercase_words,
        )
        .map_err(|error| format!("glossary vocabulary invalid: {error:?}"))?;
        write_glossary_warning_baseline(path, &report.warnings)?;
        report
    } else {
        let baseline_warnings = read_glossary_warning_baseline(&args.baseline_path)?;
        validate_glossary_vocabulary_hygiene_with_baseline_and_ignored_words(
            documents,
            allowed_acronyms,
            ignored_uppercase_words,
            baseline_warnings,
        )
        .map_err(|error| format!("glossary vocabulary invalid: {error:?}"))?
    };
    if let Some(path) = &args.warning_report_path {
        write_glossary_warning_report(path, &report.warning_sources)?;
    }
    Ok((
        report.documents_checked,
        report.casing_warnings,
        report.uncited_acronym_warnings,
    ))
}

fn read_markdown_vocabulary_documents(docs_dir: &Path) -> Result<Vec<VocabularyDocument>, String> {
    let mut documents = Vec::new();
    collect_markdown_vocabulary_documents(docs_dir, docs_dir, &mut documents)?;
    if documents.is_empty() {
        Err(format!(
            "docs directory contains no markdown files: {}",
            docs_dir.display()
        ))
    } else {
        Ok(documents)
    }
}

fn collect_markdown_vocabulary_documents(
    root: &Path,
    current: &Path,
    documents: &mut Vec<VocabularyDocument>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("glossary vocabulary docs directory unreadable: {error}"))?
    {
        let entry = entry.map_err(|error| {
            format!("glossary vocabulary docs directory entry unreadable: {error}")
        })?;
        let path = entry.path();
        if path_has_component(&path, "raw") || path_has_component(&path, "machine-readable") {
            continue;
        }
        if path.is_dir() {
            collect_markdown_vocabulary_documents(root, &path, documents)?;
            continue;
        }
        if !path.is_file()
            || path.extension().and_then(|extension| extension.to_str()) != Some("md")
        {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("glossary vocabulary doc path not under docs dir: {error}"))?;
        let normalized_path = format!("docs/{}", slash_path(relative));
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("glossary vocabulary doc unreadable: {error}"))?;
        documents.push(VocabularyDocument {
            forensic_allowed: glossary_vocabulary_forensic_path(&normalized_path),
            path: normalized_path,
            contents,
        });
    }
    Ok(())
}

fn glossary_vocabulary_forensic_path(path: &str) -> bool {
    matches!(
        path,
        "docs/GLOSSARY.md"
            | "docs/ADR-INDEX.md"
            | "docs/fitness-lanes/glossary-vocabulary.md"
            | "docs/MISTAKES-LEDGER.md"
            | "docs/ADR-CONSOLIDATION-PLAN.md"
            | "docs/ADR-LEGACY-REGRESSION-MAPPING.md"
            | "docs/CHANGELOG.md"
            | "docs/RISK-REGISTER.md"
            | "docs/decisions/ADR-0052-inventory-grit-cutover.md"
            | "docs/teams/README.md"
            | "docs/teams/tactical-first-vertical-pilot/CHARTER.md"
    ) || path.starts_with("docs/decisions/ADR-0016-")
        || path.starts_with("docs/decisions/ADR-0018-")
        || path.starts_with("docs/plans/M-CC-01-cutover/")
        || path == "docs/plans/cutover-cross-cutting-amendments-2026-05-12.md"
        || path == "docs/plans/rename-plan-v4-clean-arch-2026-05-13.md"
        || path.starts_with("docs/specs/deep-dive-oyatie-sst-consolidation")
        || path.starts_with("docs/specs/deep-dive-trace-oyatie-sst-consolidation")
        || path.starts_with("docs/decisions/specs/deep-dive-oyatie-sst-consolidation")
        || path.starts_with("docs/decisions/specs/deep-dive-trace-oyatie-sst-consolidation")
}

fn read_glossary_warning_baseline(path: &Path) -> Result<Vec<GlossaryVocabularyWarning>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "glossary warning baseline unreadable {}: {error}",
            path.display()
        )
    })?;
    let mut warnings = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((kind, token)) = trimmed.split_once('\t') else {
            return Err(format!(
                "glossary warning baseline {}:{} must be '<kind>\\t<token>'",
                path.display(),
                index + 1
            ));
        };
        let kind = GlossaryVocabularyWarningKind::parse(kind.trim()).ok_or_else(|| {
            format!(
                "glossary warning baseline {}:{} has unknown warning kind '{}'",
                path.display(),
                index + 1,
                kind.trim()
            )
        })?;
        let token = token.trim();
        if token.is_empty() {
            return Err(format!(
                "glossary warning baseline {}:{} has empty token",
                path.display(),
                index + 1
            ));
        }
        warnings.push(GlossaryVocabularyWarning {
            kind,
            token: token.to_string(),
        });
    }
    Ok(warnings)
}

fn read_ignored_uppercase_words(path: &Path) -> Result<Vec<IgnoredUppercaseWord>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "glossary ignored uppercase words unreadable {}: {error}",
            path.display()
        )
    })?;
    let mut words = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((token, rationale)) = trimmed.split_once('\t') else {
            return Err(format!(
                "glossary ignored uppercase words {}:{} must be '<token>\\t<rationale>'",
                path.display(),
                index + 1
            ));
        };
        let token = token.trim();
        let rationale = rationale.trim();
        if token.is_empty() || rationale.is_empty() {
            return Err(format!(
                "glossary ignored uppercase words {}:{} has empty token or rationale",
                path.display(),
                index + 1
            ));
        }
        words.push(IgnoredUppercaseWord {
            token: token.to_string(),
            rationale: rationale.to_string(),
        });
    }
    Ok(words)
}

fn write_glossary_warning_baseline(
    path: &Path,
    warnings: &[GlossaryVocabularyWarning],
) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "glossary warning baseline directory unwritable {}: {error}",
                parent.display()
            )
        })?;
    }
    let mut warnings = warnings.to_vec();
    warnings.sort();
    let mut contents =
        "# Oyatie glossary vocabulary warning baseline. Format: <kind>\\t<token>\n".to_string();
    for warning in warnings {
        contents.push_str(&warning.id());
        contents.push('\n');
    }
    fs::write(path, contents).map_err(|error| {
        format!(
            "glossary warning baseline unwritable {}: {error}",
            path.display()
        )
    })
}

fn write_glossary_warning_report(
    path: &Path,
    sources: &[GlossaryVocabularyWarningSource],
) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "glossary warning report directory unwritable {}: {error}",
                parent.display()
            )
        })?;
    }
    let mut sources = sources.to_vec();
    sources.sort();
    let mut contents =
        "# Oyatie glossary vocabulary warning source report. Format: <kind>\\t<token>\\t<path>\n"
            .to_string();
    for source in sources {
        contents.push_str(source.warning.kind.as_str());
        contents.push('\t');
        contents.push_str(&source.warning.token);
        contents.push('\t');
        contents.push_str(&source.path);
        contents.push('\n');
    }
    fs::write(path, contents).map_err(|error| {
        format!(
            "glossary warning report unwritable {}: {error}",
            path.display()
        )
    })
}

fn read_glossary_acronyms(path: &Path) -> Result<Vec<String>, String> {
    let contents =
        fs::read_to_string(path).map_err(|error| format!("glossary unreadable: {error}"))?;
    let mut in_acronym_index = false;
    let mut acronyms = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## 10.") {
            in_acronym_index = true;
            continue;
        }
        if in_acronym_index && trimmed.starts_with("## 11.") {
            break;
        }
        if !in_acronym_index || !trimmed.starts_with('|') || trimmed.contains("---") {
            continue;
        }
        let cells = trimmed.split('|').collect::<Vec<_>>();
        let Some(acronym_cell) = cells.get(1) else {
            continue;
        };
        if acronym_cell.trim() == "Acronym" {
            continue;
        }
        acronyms.extend(extract_acronym_tokens(acronym_cell));
    }
    if acronyms.is_empty() {
        Err(format!(
            "glossary acronym index empty or missing: {}",
            path.display()
        ))
    } else {
        Ok(acronyms)
    }
}

fn extract_acronym_tokens(value: &str) -> Vec<String> {
    value
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || character == '-' || character == '/')
        })
        .flat_map(|part| part.split('/'))
        .map(str::trim)
        .filter(|part| {
            part.len() >= 2
                && part.chars().all(|character| {
                    character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-'
                })
        })
        .map(str::to_string)
        .collect()
}

pub(crate) fn extract_first_backticked_value(value: &str) -> Option<String> {
    let (_, after_open) = value.split_once('`')?;
    let (inner, _) = after_open.split_once('`')?;
    Some(inner.to_string())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LicensePolicyValidateArgs {
    workspace_manifest_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AdrCitationValidateArgs {
    docs_dir: PathBuf,
    decisions_dir: PathBuf,
    inheritance_registry: PathBuf,
}

fn parse_adr_citation_validate_args(args: Vec<String>) -> Result<AdrCitationValidateArgs, String> {
    let mut parsed = AdrCitationValidateArgs {
        docs_dir: PathBuf::from("docs"),
        decisions_dir: PathBuf::from("docs/decisions"),
        inheritance_registry: PathBuf::from("registry/adr/inherited-bominal-adrs.yaml"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--docs-dir" => parsed.docs_dir = PathBuf::from(path),
            "--decisions-dir" => parsed.decisions_dir = PathBuf::from(path),
            "--inheritance-registry" => parsed.inheritance_registry = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn validate_adr_citation_gate(
    args: AdrCitationValidateArgs,
) -> Result<(usize, usize, usize), String> {
    let mut allowed_pack_adrs = read_pack_adr_ids(&args.decisions_dir)?;
    allowed_pack_adrs.extend(read_inherited_adr_ids(&args.inheritance_registry)?);
    allowed_pack_adrs.sort();
    allowed_pack_adrs.dedup();
    let allowed_count = allowed_pack_adrs.len();
    let documents = read_adr_citation_documents(&args.docs_dir)?;
    let report = validate_adr_citations(documents, allowed_pack_adrs)
        .map_err(|error| format!("ADR citations invalid: {error:?}"))?;
    Ok((
        report.documents_checked,
        report.citations_checked,
        allowed_count,
    ))
}

fn read_pack_adr_ids(decisions_dir: &Path) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(decisions_dir).map_err(|error| {
        format!(
            "ADR decisions directory unreadable {}: {error}",
            decisions_dir.display()
        )
    })?;
    let mut adrs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("ADR decisions entry unreadable: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.len() < 8 || !name.starts_with("ADR-") {
            continue;
        }
        let adr = &name[..8];
        if adr[4..].bytes().all(|byte| byte.is_ascii_digit()) {
            adrs.push(adr.to_string());
        }
    }
    adrs.sort();
    adrs.dedup();
    if adrs.is_empty() {
        Err(format!(
            "ADR decisions directory contains no ADR-NNNN markdown files: {}",
            decisions_dir.display()
        ))
    } else {
        Ok(adrs)
    }
}

fn read_inherited_adr_ids(registry_path: &Path) -> Result<Vec<String>, String> {
    if !registry_path.exists() {
        return Ok(Vec::new());
    }
    let contents = fs::read_to_string(registry_path).map_err(|error| {
        format!(
            "ADR inheritance registry unreadable {}: {error}",
            registry_path.display()
        )
    })?;
    let mut adrs = Vec::new();
    for (line_index, raw_line) in contents.lines().enumerate() {
        let trimmed = raw_line.trim();
        let Some(value) = trimmed
            .strip_prefix("- id:")
            .or_else(|| trimmed.strip_prefix("id:"))
        else {
            continue;
        };
        let adr = value
            .split('#')
            .next()
            .unwrap_or_default()
            .trim()
            .trim_matches('"')
            .trim_matches('\'');
        if !is_adr_id(adr) {
            return Err(format!(
                "ADR inheritance registry {} line {} has invalid id {:?}",
                registry_path.display(),
                line_index + 1,
                adr
            ));
        }
        adrs.push(adr.to_string());
    }
    adrs.sort();
    adrs.dedup();
    if adrs.is_empty() {
        Err(format!(
            "ADR inheritance registry contains no `id: ADR-NNNN` entries: {}",
            registry_path.display()
        ))
    } else {
        Ok(adrs)
    }
}

fn is_adr_id(value: &str) -> bool {
    value.len() == 8
        && value.starts_with("ADR-")
        && value[4..].bytes().all(|byte| byte.is_ascii_digit())
}

fn read_adr_citation_documents(docs_dir: &Path) -> Result<Vec<AdrCitationDocument>, String> {
    let mut documents = Vec::new();
    collect_adr_citation_documents(docs_dir, docs_dir, &mut documents)?;
    if documents.is_empty() {
        Err(format!(
            "ADR citation docs directory contains no markdown files: {}",
            docs_dir.display()
        ))
    } else {
        Ok(documents)
    }
}

fn collect_adr_citation_documents(
    root: &Path,
    current: &Path,
    documents: &mut Vec<AdrCitationDocument>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("ADR citation docs directory unreadable: {error}"))?
    {
        let entry = entry
            .map_err(|error| format!("ADR citation docs directory entry unreadable: {error}"))?;
        let path = entry.path();
        if path_has_component(&path, "raw") || path_has_component(&path, "machine-readable") {
            continue;
        }
        if path.is_dir() {
            collect_adr_citation_documents(root, &path, documents)?;
            continue;
        }
        if !path.is_file()
            || path.extension().and_then(|extension| extension.to_str()) != Some("md")
        {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("ADR citation doc path not under docs dir: {error}"))?;
        let normalized_path = format!("docs/{}", slash_path(relative));
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("ADR citation doc unreadable: {error}"))?;
        let forensic_allowed = adr_citation_forensic_path(&normalized_path);
        documents.push(AdrCitationDocument {
            path: normalized_path,
            contents,
            forensic_allowed,
        });
    }
    Ok(())
}

fn adr_citation_forensic_path(path: &str) -> bool {
    matches!(
        path,
        "docs/ADR-CONSOLIDATION-PLAN.md"
            | "docs/ADR-LEGACY-REGRESSION-MAPPING.md"
            | "docs/CONTRADICTION-LEDGER.md"
            | "docs/decisions/RETIRED.md"
    ) || path.ends_with("-LEGACY.md")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BrandResidueValidateArgs {
    docs_dir: PathBuf,
}

fn parse_brand_residue_validate_args(
    args: Vec<String>,
) -> Result<BrandResidueValidateArgs, String> {
    let mut parsed = BrandResidueValidateArgs {
        docs_dir: PathBuf::from("docs"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--docs-dir" => parsed.docs_dir = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn validate_brand_residue_gate(args: BrandResidueValidateArgs) -> Result<(usize, usize), String> {
    let documents = read_brand_residue_documents(&args.docs_dir)?;
    let report = validate_brand_residue(documents)
        .map_err(|error| format!("brand residue invalid: {error:?}"))?;
    Ok((report.documents_checked, report.patterns_checked))
}

fn read_brand_residue_documents(docs_dir: &Path) -> Result<Vec<BrandResidueDocument>, String> {
    let mut documents = Vec::new();
    collect_brand_residue_documents(docs_dir, docs_dir, &mut documents)?;
    if documents.is_empty() {
        Err(format!(
            "brand residue docs directory contains no markdown files: {}",
            docs_dir.display()
        ))
    } else {
        Ok(documents)
    }
}

fn collect_brand_residue_documents(
    root: &Path,
    current: &Path,
    documents: &mut Vec<BrandResidueDocument>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("brand residue directory unreadable: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("brand residue directory entry unreadable: {error}"))?;
        let path = entry.path();
        if brand_residue_excluded_path(&path) {
            continue;
        }
        if path.is_dir() {
            collect_brand_residue_documents(root, &path, documents)?;
            continue;
        }
        if !path.is_file()
            || path.extension().and_then(|extension| extension.to_str()) != Some("md")
        {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("brand residue path not under root: {error}"))?;
        let normalized_path = format!("docs/{}", slash_path(relative));
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("brand residue file unreadable: {error}"))?;
        documents.push(BrandResidueDocument {
            path: normalized_path,
            contents,
        });
    }
    Ok(())
}

fn brand_residue_excluded_path(path: &Path) -> bool {
    ["raw", "machine-readable"]
        .into_iter()
        .any(|component| path_has_component(path, component))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ApiSemverValidateArgs {
    contracts_dir: PathBuf,
}

fn parse_api_semver_validate_args(args: Vec<String>) -> Result<ApiSemverValidateArgs, String> {
    let mut parsed = ApiSemverValidateArgs {
        contracts_dir: PathBuf::from("contracts"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--contracts-dir" => parsed.contracts_dir = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn validate_api_semver_gate(args: ApiSemverValidateArgs) -> Result<(usize, usize), String> {
    let records = read_api_contract_records(&args.contracts_dir)?;
    let report =
        validate_api_semver(records).map_err(|error| format!("API semver invalid: {error:?}"))?;
    Ok((report.contracts_checked, report.metadata_checked))
}

pub(crate) fn next_arg(iter: &mut impl Iterator<Item = String>) -> Result<String, String> {
    iter.next().ok_or_else(usage)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CargoPrefixValidateArgs {
    workspace_manifest_path: PathBuf,
    expected_prefix: String,
}

fn parse_cargo_prefix_validate_args(args: Vec<String>) -> Result<CargoPrefixValidateArgs, String> {
    let mut parsed = CargoPrefixValidateArgs {
        workspace_manifest_path: PathBuf::from("Cargo.toml"),
        expected_prefix: "oya-".into(),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--workspace" => parsed.workspace_manifest_path = PathBuf::from(value),
            "--prefix" => parsed.expected_prefix = value,
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn validate_cargo_prefix_gate(args: CargoPrefixValidateArgs) -> Result<usize, String> {
    let members = read_workspace_cargo_prefix_members(&args.workspace_manifest_path)?;
    let report = validate_cargo_prefix(members, &args.expected_prefix)
        .map_err(|error| format!("workspace Cargo prefix invalid: {error:?}"))?;
    Ok(report.members_checked)
}

fn read_workspace_cargo_prefix_members(
    workspace_manifest_path: &Path,
) -> Result<Vec<CargoPrefixMember>, String> {
    let workspace_dir = workspace_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let mut members = Vec::new();
    for member_path in read_workspace_member_paths(workspace_manifest_path)? {
        let manifest_path = workspace_dir.join(&member_path).join("Cargo.toml");
        let package_name = read_package_name(&manifest_path)?;
        members.push(CargoPrefixMember {
            member_path,
            package_name,
        });
    }
    Ok(members)
}

fn parse_license_policy_validate_args(
    args: Vec<String>,
) -> Result<LicensePolicyValidateArgs, String> {
    let mut parsed = LicensePolicyValidateArgs {
        workspace_manifest_path: PathBuf::from("Cargo.toml"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--workspace" => parsed.workspace_manifest_path = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn validate_license_policy_gate(args: LicensePolicyValidateArgs) -> Result<usize, String> {
    let member_paths = read_workspace_member_paths(&args.workspace_manifest_path)?;
    let workspace_dir = args
        .workspace_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let policy = LicensePolicy::adr_0013_product_policy();
    for member_path in &member_paths {
        let manifest_path = workspace_dir.join(member_path).join("Cargo.toml");
        let license = read_package_license(&manifest_path)?;
        policy
            .validate_product_license(&license)
            .map_err(|error| format!("{}: {error:?}", manifest_path.display()))?;
    }
    Ok(member_paths.len())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MobileNativeValidateArgs {
    manifest_path: PathBuf,
    repo_root: PathBuf,
}

fn parse_mobile_native_validate_args(
    args: Vec<String>,
) -> Result<MobileNativeValidateArgs, String> {
    let mut parsed = MobileNativeValidateArgs {
        manifest_path: PathBuf::from("registry/mobile-native/products.tsv"),
        repo_root: PathBuf::from("."),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let value = next_arg(&mut iter)?;
        match flag.as_str() {
            "--manifest" => parsed.manifest_path = PathBuf::from(value),
            "--repo-root" => parsed.repo_root = PathBuf::from(value),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn validate_mobile_native_gate(
    args: MobileNativeValidateArgs,
) -> Result<(String, usize, usize, usize), String> {
    let (manifest, records) = read_mobile_native_manifest(&args.manifest_path)?;
    let markers = discover_mobile_native_markers(&args.repo_root)?;
    let report = validate_mobile_native(
        manifest,
        records,
        markers,
        MobileNativePolicy::adr_0051_quality_bar(),
    )
    .map_err(|error| format!("mobile native invalid: {error:?}"))?;
    Ok((
        report.current_wave,
        report.native_products_checked,
        report.native_markers_checked,
        report.quality_bar_records_checked,
    ))
}

fn read_mobile_native_manifest(
    manifest_path: &Path,
) -> Result<(MobileNativeManifest, Vec<MobileNativeProductRecord>), String> {
    let contents = fs::read_to_string(manifest_path).map_err(|error| {
        format!(
            "mobile native manifest unreadable {}: {error}",
            manifest_path.display()
        )
    })?;
    let mut current_wave = None;
    let mut empty_scope_rationale = None;
    let mut seen_header = false;
    let mut records = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix('#') {
            if let Some(value) = scalar_value(comment.trim(), "current_wave") {
                current_wave = Some(value);
            } else if let Some(value) = scalar_value(comment.trim(), "empty_scope_rationale") {
                empty_scope_rationale = Some(value);
            }
            continue;
        }
        if trimmed.starts_with("product_id\t") {
            seen_header = true;
            continue;
        }
        if !seen_header {
            return Err(format!(
                "{}: expected product_id TSV header before records",
                manifest_path.display()
            ));
        }
        records.push(parse_mobile_native_product_row(manifest_path, trimmed)?);
    }
    if !seen_header {
        return Err(format!(
            "{}: missing product_id TSV header",
            manifest_path.display()
        ));
    }
    Ok((
        MobileNativeManifest {
            current_wave: current_wave.unwrap_or_default(),
            empty_scope_rationale: empty_scope_rationale.unwrap_or_default(),
        },
        records,
    ))
}

fn parse_mobile_native_product_row(
    manifest_path: &Path,
    row: &str,
) -> Result<MobileNativeProductRecord, String> {
    let cells = row
        .split('\t')
        .map(|cell| cell.trim().trim_matches('`').to_string())
        .collect::<Vec<_>>();
    if cells.len() != 17 {
        return Err(format!(
            "{}: mobile native product row must have 17 TSV columns: {row}",
            manifest_path.display()
        ));
    }
    Ok(MobileNativeProductRecord {
        product_id: cells[0].clone(),
        axis: cells[1].clone(),
        status: cells[2].clone(),
        canonical_web_reference: cells[3].clone(),
        target_matrix_ref: cells[4].clone(),
        tech_stack_rationale_ref: cells[5].clone(),
        store_policy_ref: cells[6].clone(),
        store_policy_validator_passed: parse_bool_field(
            manifest_path,
            "store_policy_validator_passed",
            &cells[7],
        )?,
        accessibility_audit_ref: cells[8].clone(),
        accessibility_audit_passed: parse_bool_field(
            manifest_path,
            "accessibility_audit_passed",
            &cells[9],
        )?,
        capability_parity_ref: cells[10].clone(),
        capability_parity_passed: parse_bool_field(
            manifest_path,
            "capability_parity_passed",
            &cells[11],
        )?,
        sbom_ref: cells[12].clone(),
        native_binary_blobs_without_sbom: parse_u32_cell_field(
            manifest_path,
            "native_binary_blobs_without_sbom",
            &cells[13],
        )?,
        crash_free_sessions_bps: parse_optional_u32_cell(
            manifest_path,
            "crash_free_sessions_bps",
            &cells[14],
        )?,
        crash_free_regression_bps: parse_optional_u32_cell(
            manifest_path,
            "crash_free_regression_bps",
            &cells[15],
        )?,
        cold_start_p99_ms: parse_optional_u32_cell(manifest_path, "cold_start_p99_ms", &cells[16])?,
    })
}

pub(crate) fn parse_u32_cell_field(path: &Path, field: &str, value: &str) -> Result<u32, String> {
    value
        .parse::<u32>()
        .map_err(|_| format!("{}: {field} must be u32", path.display()))
}

fn parse_optional_u32_cell(path: &Path, field: &str, value: &str) -> Result<Option<u32>, String> {
    match optional_cell(value) {
        Some(value) => Ok(Some(parse_u32_cell_field(path, field, &value)?)),
        None => Ok(None),
    }
}

fn discover_mobile_native_markers(
    repo_root: &Path,
) -> Result<Vec<MobileNativeDiscoveryMarker>, String> {
    let mut markers = Vec::new();
    collect_mobile_native_markers(repo_root, repo_root, &mut markers)?;
    markers.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(markers)
}

fn collect_mobile_native_markers(
    repo_root: &Path,
    dir: &Path,
    markers: &mut Vec<MobileNativeDiscoveryMarker>,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)
        .map_err(|error| format!("mobile native marker directory unreadable: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("mobile native marker entry unreadable: {error}"))?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if path.is_dir() {
            if ignored_mobile_native_dir(&file_name) {
                continue;
            }
            if let Some(marker_kind) = mobile_native_marker_kind(&path) {
                markers.push(mobile_native_marker(repo_root, &path, marker_kind));
                continue;
            }
            collect_mobile_native_markers(repo_root, &path, markers)?;
        } else if let Some(marker_kind) = mobile_native_marker_kind(&path) {
            markers.push(mobile_native_marker(repo_root, &path, marker_kind));
        }
    }
    Ok(())
}

fn ignored_mobile_native_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".omx" | "target" | "docs" | "registry" | ".github" | "node_modules"
    )
}

fn mobile_native_marker_kind(path: &Path) -> Option<&'static str> {
    let file_name = path.file_name()?.to_str()?;
    if let Some(kind) = match file_name {
        "Podfile" => Some("podfile"),
        "Cartfile" => Some("cartfile"),
        "Package.swift" => Some("swift-package"),
        "AndroidManifest.xml" => Some("android-manifest"),
        "build.gradle" => Some("gradle-build"),
        "settings.gradle" => Some("gradle-settings"),
        "gradlew" => Some("gradle-wrapper"),
        "gradle.properties" => Some("gradle-properties"),
        _ => None,
    } {
        return Some(kind);
    }
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("xcodeproj") => Some("xcodeproj"),
        Some("xcworkspace") => Some("xcworkspace"),
        Some("swift") => Some("swift"),
        Some("kt") => Some("kotlin"),
        Some("kts") => Some("kotlin-script"),
        Some("gradle") => Some("gradle"),
        _ => None,
    }
}

fn mobile_native_marker(
    repo_root: &Path,
    path: &Path,
    marker_kind: &str,
) -> MobileNativeDiscoveryMarker {
    let relative = path.strip_prefix(repo_root).unwrap_or(path);
    MobileNativeDiscoveryMarker {
        path: relative.to_string_lossy().replace('\\', "/"),
        marker_kind: marker_kind.to_string(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VendorContractRecencyValidateArgs {
    ledger_path: PathBuf,
    today_epoch_days: i64,
    renewal_window_days: i64,
}

fn parse_vendor_contract_recency_validate_args(
    args: Vec<String>,
) -> Result<VendorContractRecencyValidateArgs, String> {
    let mut parsed = VendorContractRecencyValidateArgs {
        ledger_path: PathBuf::from("docs/VENDOR-PARTNER-LEDGER.md"),
        today_epoch_days: current_epoch_days_i64()?,
        renewal_window_days: 90,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--ledger" => parsed.ledger_path = PathBuf::from(value),
            "--today" => parsed.today_epoch_days = parse_yyyy_mm_dd_to_epoch_days(&value)?,
            "--renewal-window-days" => {
                parsed.renewal_window_days = value
                    .parse::<i64>()
                    .map_err(|_| "renewal-window-days must be signed integer".to_string())?;
            }
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn validate_vendor_contract_recency_gate(
    args: VendorContractRecencyValidateArgs,
) -> Result<(usize, usize, usize), String> {
    let records = read_vendor_contract_records(&args.ledger_path)?;
    let report = validate_vendor_contract_recency(
        records,
        args.today_epoch_days,
        VendorContractRecencyPolicy {
            renewal_window_days: args.renewal_window_days,
        },
    )
    .map_err(|error| format!("vendor contract recency invalid: {error:?}"))?;
    Ok((
        report.records_checked,
        report.contracted_records_checked,
        report.renewal_tasks_required_checked,
    ))
}

fn read_vendor_contract_records(ledger_path: &Path) -> Result<Vec<VendorContractRecord>, String> {
    let contents = fs::read_to_string(ledger_path)
        .map_err(|error| format!("vendor partner ledger unreadable: {error}"))?;
    let mut rows = Vec::new();
    let mut in_contract_table = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("| Contract ID |") {
            in_contract_table = true;
            continue;
        }
        if !in_contract_table {
            continue;
        }
        if !trimmed.starts_with('|') {
            if !rows.is_empty() {
                break;
            }
            continue;
        }
        if trimmed
            .trim_matches('|')
            .chars()
            .all(|ch| ch == '-' || ch == ':' || ch == '|' || ch.is_whitespace())
        {
            continue;
        }
        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().trim_matches('`').to_string())
            .collect::<Vec<_>>();
        if cells.len() < 6 {
            return Err(format!(
                "{}: contract recency row must have 6 columns: {trimmed}",
                ledger_path.display()
            ));
        }
        rows.push(VendorContractRecord {
            contract_id: cells[0].clone(),
            vendor: cells[1].clone(),
            status: cells[2].clone(),
            expiry_epoch_days: parse_optional_yyyy_mm_dd(&cells[3])?,
            renewal_task: optional_cell(&cells[4]),
            owner_team: cells[5].clone(),
        });
    }
    Ok(rows)
}

fn optional_cell(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || matches!(value, "n/a" | "N/A" | "none" | "None" | "-") {
        None
    } else {
        Some(value.to_string())
    }
}

fn parse_optional_yyyy_mm_dd(value: &str) -> Result<Option<i64>, String> {
    match optional_cell(value) {
        Some(value) => Ok(Some(parse_yyyy_mm_dd_to_epoch_days(&value)?)),
        None => Ok(None),
    }
}
