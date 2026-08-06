// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` to assert
// invariants under the `cfg(test)` exemption (production code is Tier 1).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use check_adr_citation::{AdrCitationDocument, validate_adr_citations};
use oya_check_brand_residue::{BrandResidueDocument, validate_brand_residue};
use check_glossary_vocabulary::{
    GlossaryVocabularyWarning, GlossaryVocabularyWarningKind, GlossaryVocabularyWarningSource,
    IgnoredUppercaseWord, VocabularyDocument,
    validate_glossary_vocabulary_hygiene_with_baseline_and_ignored_words,
    validate_glossary_vocabulary_hygiene_with_ignored_words,
};
use oya_check_license_policy::LicensePolicy;
use check_mobile_native::{
    MobileNativeDiscoveryMarker, MobileNativeManifest, MobileNativePolicy,
    MobileNativeProductRecord, validate_mobile_native,
};
use check_no_grouping::{GroupingArtifact, is_grouping_artifact, validate_no_grouping};
use check_vendor_lockin_discipline::{
    VendorLockinReport, parse_registry_json as parse_vendor_lockin_registry,
    validate_registry as validate_vendor_lockin_registry,
};
use check_vendor_recency::{
    VendorContractRecencyPolicy, VendorContractRecord, validate_vendor_contract_recency,
};
use intelligence_api_semver_domain::validate_api_semver;
use intelligence_cargo_prefix_domain::{CargoPrefixMember, validate_cargo_prefix};

mod active_artifact_contract_gate;
mod adr_0145_gates;
// ADR-0364 D2/D3/D4: generative ADR planning front-matter parser + the
// completeness and masterplan-drift gates.
mod adr_planning_completeness_gate;
mod adr_planning_frontmatter;
mod adr_supersession_consistency_gate;
mod api_contract_registry;
mod architecture_map_emit_gate;
mod architecture_plane_gates;
mod aspirational_enforcement_gate;
mod banned_primitives_gate;
mod canonical_base_neutrality_gate;
mod capacity_model_manifest_gate;
mod catalog_contract_gates;
mod catalog_registry;
mod cedar_fragment_coverage_gate;
mod changeset_state_gates;
mod cloud_iac_cell_topology_gate;
mod cloud_iac_gitops_evidence_gate;
mod cloud_iac_helm_chart_gate;
mod cloud_iac_kubewarden_admission_gate;
mod cloud_iac_module_archive_gate;
mod cloud_iac_module_catalog_gate;
mod cloud_iac_module_provenance_gate;
mod cloud_iac_module_provider_requirements_gate;
mod cloud_iac_module_registry_protocol_gate;
mod cloud_iac_module_release_index_gate;
mod cloud_iac_opentofu_validation_gate;
mod cloud_iac_provider_lockfile_gate;
mod cloud_iac_provider_readiness_gate;
mod cloud_iac_provider_signature_review_gate;
mod codeview_read_surface_gates;
mod command_output;
mod command_process;
mod commands;
mod cross_axis_contracts;
mod cross_tenant_access_gates;
mod data_class_gates;
mod date_utils;
mod dependency_blessed_allowlist_gate;
mod dependency_seam_gates;
mod design_spec_maturity_claims_gate;
mod documentation_gates;
mod fd001_manifest_workspace_alignment_gate;
mod foundation_audit_gates;
mod foundation_fixture;
mod foundry_capability_schema_gates;
mod foundry_eval_gates;
mod freshness_gate;
mod glossary_cross_doc_gates;
mod governance_advisory_lanes;
mod governance_gates;
mod honest_claims_gate;
mod http_stack_gate;
mod hyperscaler_arch_invariants_gate;
mod hyperscaler_maturity_claims_gate;
mod json_scan;
mod korea_localization_evidence_gate;
mod layered_architecture_gates;
mod loop_recovery_patterns_gate;
mod masterplan_drift_gate;
mod openapi_rest_route_parity_gate;
mod path_format;
mod placeholder_debt_gates;
mod planning_ssot_coverage_gate;
mod platform_substrate_defaults_gate;
mod pre_push_contract_gate;
mod protection_context_match_gate;
mod quality_lane_gates;
mod retired_vocabulary_gate;
mod runbook_gates;
mod scalability_gates;
mod scalar_parse;
mod supply_chain_gates;
mod team_ownership_gates;
mod tier_a_gates;
mod typescript_workspace_gates;
mod workspace_hygiene_gate;
mod workspace_manifest;
mod workspace_topology_gate;
mod yaml_scan;

pub(crate) use active_artifact_contract_gate::{
    parse_active_artifact_contract_validate_args, validate_active_artifact_contract_gate,
};
// Governance advisory lane re-exports (1 strict ref via local fn + 11 advisory).
pub(crate) use api_contract_registry::{is_api_contract_metadata_path, read_api_contract_records};
pub(crate) use architecture_map_emit_gate::{
    emit_architecture_map_gate, parse_architecture_map_emit_args,
};
pub(crate) use architecture_plane_gates::{
    parse_planes_validate_args, parse_wave_integration_validate_args, validate_planes_gate,
    validate_wave_integration_gate,
};
pub(crate) use aspirational_enforcement_gate::{
    parse_aspirational_enforcement_validate_args, validate_aspirational_enforcement_gate,
};
pub(crate) use banned_primitives_gate::{
    parse_banned_primitives_validate_args, validate_banned_primitives_gate,
};
pub(crate) use canonical_base_neutrality_gate::{
    parse_canonical_base_neutrality_validate_args, validate_canonical_base_neutrality_gate,
};
pub(crate) use catalog_contract_gates::{
    parse_cohesion_validate_args, parse_slo_coverage_validate_args, validate_cohesion_gate,
    validate_slo_coverage_gate,
};
pub(crate) use catalog_registry::read_catalog_records;
pub(crate) use cedar_fragment_coverage_gate::{
    parse_cedar_fragment_coverage_validate_args, validate_cedar_fragment_coverage_gate,
};
pub(crate) use cloud_iac_cell_topology_gate::{
    parse_cloud_iac_cell_topology_validate_args, validate_cloud_iac_cell_topology_gate,
};
pub(crate) use cloud_iac_gitops_evidence_gate::{
    parse_cloud_iac_gitops_evidence_validate_args, validate_cloud_iac_gitops_evidence_gate,
};
pub(crate) use cloud_iac_helm_chart_gate::{
    parse_cloud_iac_helm_chart_args, validate_cloud_iac_helm_chart_gate,
};
pub(crate) use cloud_iac_kubewarden_admission_gate::{
    parse_cloud_iac_kubewarden_admission_args, validate_cloud_iac_kubewarden_admission_gate,
};
pub(crate) use cloud_iac_module_archive_gate::{
    parse_cloud_iac_module_archive_args, validate_cloud_iac_module_archive_gate,
};
pub(crate) use cloud_iac_module_catalog_gate::{
    parse_cloud_iac_module_catalog_validate_args, validate_cloud_iac_module_catalog_gate,
};
pub(crate) use cloud_iac_module_provenance_gate::{
    parse_cloud_iac_module_provenance_args, validate_cloud_iac_module_provenance_gate,
};
pub(crate) use cloud_iac_module_provider_requirements_gate::{
    parse_cloud_iac_module_provider_requirements_args,
    validate_cloud_iac_module_provider_requirements_gate,
};
pub(crate) use cloud_iac_module_registry_protocol_gate::{
    parse_cloud_iac_module_registry_protocol_args, validate_cloud_iac_module_registry_protocol_gate,
};
pub(crate) use cloud_iac_module_release_index_gate::{
    parse_cloud_iac_module_release_index_args, validate_cloud_iac_module_release_index_gate,
};
pub(crate) use cloud_iac_opentofu_validation_gate::{
    parse_cloud_iac_opentofu_validation_args, validate_cloud_iac_opentofu_validation_gate,
};
pub(crate) use cloud_iac_provider_lockfile_gate::{
    parse_cloud_iac_provider_lockfile_args, validate_cloud_iac_provider_lockfile_gate,
};
pub(crate) use cloud_iac_provider_readiness_gate::{
    parse_cloud_iac_provider_readiness_args, validate_cloud_iac_provider_readiness_gate,
};
pub(crate) use cloud_iac_provider_signature_review_gate::{
    parse_cloud_iac_provider_signature_review_args,
    validate_cloud_iac_provider_signature_review_gate,
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
pub(crate) use dependency_blessed_allowlist_gate::{
    parse_dependency_blessed_allowlist_args, validate_dependency_blessed_allowlist_gate,
};
pub(crate) use dependency_seam_gates::{
    parse_dependency_seam_validate_args, validate_dependency_seam_gate,
};
pub(crate) use design_spec_maturity_claims_gate::{
    parse_design_spec_maturity_claims_validate_args, validate_design_spec_maturity_claims_gate,
};
pub(crate) use documentation_gates::{
    parse_doc_catalog_validate_args, parse_documentation_system_validate_args,
    parse_readme_doc_coverage_validate_args, validate_doc_catalog_gate,
    validate_documentation_system_gate, validate_readme_doc_coverage_gate,
};
pub(crate) use fd001_manifest_workspace_alignment_gate::{
    parse_fd001_manifest_workspace_alignment_validate_args,
    validate_fd001_manifest_workspace_alignment_gate,
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
pub(crate) use freshness_gate::{parse_freshness_gate_args, validate_freshness_gate};
pub(crate) use glossary_cross_doc_gates::{
    parse_glossary_coverage_validate_args, validate_glossary_coverage_gate,
};
pub(crate) use governance_advisory_lanes::{
    validate_a11y_discipline_gate, validate_authz_tier_discipline_gate,
    validate_backup_retention_discipline_gate, validate_compliance_evidence_coverage_gate,
    validate_i18n_coverage_gate, validate_iac_tier_discipline_gate,
    validate_olap_tier_discipline_gate, validate_realtime_transport_tier_gate,
    validate_tenant_cost_labels_coverage_gate, validate_vector_store_discipline_gate,
    validate_wasm_runtime_discipline_gate,
};
pub(crate) use governance_gates::{
    parse_authority_cohesion_validate_args, parse_claim_ceiling_validate_args,
    parse_plane_class_validate_args, validate_authority_cohesion_gate, validate_claim_ceiling_gate,
    validate_plane_class_gate,
};
pub(crate) use honest_claims_gate::{
    parse_honest_claims_validate_args, validate_honest_claims_gate,
};
pub(crate) use http_stack_gate::{
    HttpStackFindingKind, parse_http_stack_validate_args, validate_http_stack_gate,
};
pub(crate) use hyperscaler_arch_invariants_gate::{
    parse_hyperscaler_arch_invariants_validate_args, validate_hyperscaler_arch_invariants_gate,
};
pub(crate) use hyperscaler_maturity_claims_gate::{
    parse_hyperscaler_maturity_claims_validate_args, validate_hyperscaler_maturity_claims_gate,
};
pub(crate) use json_scan::{
    extract_json_array_for_key, extract_json_object_entries, extract_json_object_for_key,
    extract_json_objects, find_matching_json_delimiter, json_field_has_non_empty_value,
    parse_json_string_array_field, parse_json_string_field, parse_json_string_value,
    quoted_json_len,
};
pub(crate) use korea_localization_evidence_gate::{
    parse_korea_localization_evidence_validate_args, validate_korea_localization_evidence_gate,
};
pub(crate) use loop_recovery_patterns_gate::{
    parse_loop_recovery_patterns_validate_args, validate_loop_recovery_patterns_gate,
};
pub(crate) use openapi_rest_route_parity_gate::{
    parse_openapi_rest_route_parity_validate_args, validate_openapi_rest_route_parity_gate,
};
pub(crate) use path_format::slash_path;
pub(crate) use placeholder_debt_gates::{
    parse_placeholder_debt_validate_args, validate_placeholder_debt_gate,
};
pub(crate) use platform_substrate_defaults_gate::{
    parse_platform_substrate_defaults_args, validate_platform_substrate_defaults_gate,
};
pub(crate) use pre_push_contract_gate::{
    parse_pre_push_contract_validate_args, validate_pre_push_contract_gate,
};
pub(crate) use protection_context_match_gate::{
    parse_protection_context_match_validate_args, validate_protection_context_match_gate,
};
pub(crate) use quality_lane_gates::{
    parse_quality_lanes_validate_args, validate_quality_lanes_gate,
};
pub(crate) use retired_vocabulary_gate::{
    parse_retired_vocabulary_validate_args, validate_retired_vocabulary_gate,
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
    parse_image_promotion_validate_args, parse_release_evidence_pack_validate_args,
    parse_release_supply_chain_validate_args, parse_supply_chain_validate_args,
    release_supply_chain_phase_name, validate_image_promotion_gate,
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
pub(crate) use workspace_hygiene_gate::{
    WorkspaceHygieneValidateArgs, parse_workspace_hygiene_validate_args,
    validate_workspace_hygiene_gate,
};
pub(crate) use workspace_manifest::{
    read_package_license, read_package_name, read_workspace_member_crate_ids,
    read_workspace_member_paths,
};
pub(crate) use workspace_topology_gate::{
    WorkspaceTopologyRule, parse_workspace_topology_validate_args, validate_workspace_topology_gate,
};
pub(crate) use yaml_scan::{clean_yaml_value, parse_yaml_inline_values};

pub fn run_cli_from_env() -> ExitCode {
    let mut raw_args = std::env::args();
    let _program = raw_args.next().unwrap_or_default();
    let mut args = raw_args.collect::<Vec<_>>().into_iter();
    match args.next().as_deref() {
        Some("demo") => commands::demo::run(args.collect(), &usage()),
        Some("check") => commands::check::run(args.collect(), &usage()),
        Some("cleanup") => commands::cleanup::run(args.collect(), &usage()),
        Some("codex-thread-sweep") => {
            eprintln!(
                "oya codex-thread-sweep: RETIRED by ADR-0565/ADR-0363; review-thread automation belongs in cloud-ci/reviewer APIs."
            );
            ExitCode::from(2)
        }
        Some("doc") => commands::doc::run(args.collect(), &usage()),
        Some("gen") => commands::generate::run(args.collect(), &usage()),
        Some("catalog") => commands::catalog::run(args.collect(), &usage()),
        Some("gate") => commands::gate::run(args.collect(), &usage()),
        Some("lint") => commands::lint::run(args.collect(), &usage()),
        Some("plan") => commands::plan::run(args.collect(), &usage()),
        // ADR-0375 §Consequences retired `oya onprem` and `oya ops oci-*` (the
        // OCI/kubeadm/containerd/Istio-Envoy on-prem model superseded by Talos +
        // Cluster API + Argo CD). The Rust modules remain compiled (no follow-up
        // PR has dropped them yet) but the dispatch surfaces a typed RETIRED
        // exit so callers see a clear pointer instead of a file-not-found panic
        // when the deleted infra/onprem/* shell scripts are invoked.
        Some("onprem") => {
            eprintln!(
                "oya onprem: RETIRED by ADR-0375 (Talos + Cluster API + Argo CD fleet substrate)."
            );
            eprintln!(
                "  Replacement: bring-up via `infra/talos/installation-media/gen-media.sh` + "
            );
            eprintln!("  `infra/capi/init.sh` + spoke templates under `infra/capi/clusters/`.");
            ExitCode::from(2)
        }
        Some("ops") => {
            eprintln!("oya ops <oci-*|onprem-*>: RETIRED by ADR-0375 (the OCI/on-prem deployment");
            eprintln!("  model is superseded by CAPI/Talos/Argo CD). OpenTofu now owns only the");
            eprintln!("  Cloudflare edge (`infra/cloudflare`); the cluster fleet is declarative");
            eprintln!("  CAPI/Talos/Argo CD.");
            ExitCode::from(2)
        }
        Some("submit") => commands::submit::run(args.collect(), &usage()),
        Some("supply-chain") => commands::supply_chain::run(args.collect(), &usage()),
        Some("verify") => commands::verify::run(args.collect(), &usage()),
        Some("merge-queue") => commands::merge_queue::run(args.collect(), &usage()),
        _ => {
            eprintln!("{}", usage());
            ExitCode::from(2)
        }
    }
}

pub(crate) fn usage() -> String {
    "Usage: oya demo [--audit-ledger <path>] [--evidence-store <path>] [--run-ledger <path>] [--step-ledger <path>] [--outbox-store <path>] [--secret-store <path>]\n       oya verify [--ci-required] [--include-deferred] [--skip-fmt] [--skip-check] [--skip-clippy] [--skip-nextest] [--skip-gate-run-all]   # canonical local pre-push/pre-PR entry; --ci-required runs the full CI mirror\n       oya submit [--no-verify] [--push-only] [--draft] [--title <text>] [--body <text>]   # verify --ci-required → git push → open/extend PR\n       oya cleanup retired-and-renumber --plan <path> --renumber-map <path> [--apply]\n       oya supply-chain adr0039 [--manifest <registry/release/images.yaml>] [--artifacts-dir <artifacts/supply-chain>] [--dry-run] [--format <text|json>]\n       oya supply-chain install-trivy [--version <0.70.0>] [--install-dir </usr/local/bin>] [--dry-run] [--format <text|json>]\n       oya lint <proto|asyncapi|adr-shape|foundry-phase00-evidence> [lint-specific args]\n       oya check <architecture|bounded-context|supply-chain|semver|documentation|statelessness|shardability|perf-budget|benchmark> [check-specific args]\n       oya doc rustdoc [--target-dir <target/oya-rustdoc-check>] [--rustdoc <path>] [--cargo <path>] [--format <text|json>] [--keep-target-dir]\n       oya doc openapi [--contracts-dir <contracts>] [--spec <docs/SPEC.md>] [--contracts-mirror <docs/machine-readable/contracts.json>] [--runtime-bindings <registry/openapi/runtime-bindings.tsv>] [--schema-bindings <registry/openapi/schema-bindings.tsv>] [--runtime-root <.>] [--format <text|json>]\n       oya doc mdbook [--site-dir <docs/site>] [--format <text|json>]\n       oya doc adr-index [--decisions-dir <docs/decisions>] [--index <docs/ADR-INDEX.md>] [--machine <docs/machine-readable/decisions.json>] [--write] [--format <text|json>]\n       oya doc inventory [--repo-root <.>] [--workspace <Cargo.toml>] [--crate-registry <registry/catalog>] [--doc-catalog <docs/machine-readable/catalog.json>] [--contracts-dir <contracts>] [--products-dir <docs/products>] [--capabilities-dir <registry/capability-templates>] [--out <docs/machine-readable/documentation-inventory.json>] [--write] [--format <text|json>]\n       oya catalog validate [--workspace <Cargo.toml>] [--registry <registry/catalog>]"
        .to_string()
        + "\n       oya plan <next|claim|claim/next|reserve-id> [--master-plan <docs/machine-readable/masterplan.generated.json>] [--repo-root <.>] [--remote <origin>] [--deliverable <id>] [--id <ADR-NNNN|PRD-...>] [--claimant <agent-id>] [--lease-seconds <seconds>] [--recover-stale] [--recovery-reason <text>] [--dry-run] [--format <text|json>]   # ADR-0377 D2 + CI-009: remote git-ref CAS claims plus ADR/PRD id reservations under refs/heads/id-reservations/<id>"
        + "\n       oya gen board-sync [--master-plan <docs/machine-readable/masterplan.generated.json>] [--snapshot <docs/machine-readable/board-sync.generated.json>] [--claim-ref-snapshot <claims.json>] [--write|--check]   # ADR-0377 D3: masterplan deliverables + claim refs to GitHub issue/label projection"
        + "\n       oya gen masterplan [--decisions-dir <docs/decisions>] [--output <docs/machine-readable/masterplan.generated.json>] [--write|--check]   # ADR-0364 D3: generate the masterplan projection from planning_impact ADRs"
        // `oya onprem` / `oya ops oci-*` were RETIRED by ADR-0375; the dispatch
        // emits a RETIRED notice + exit 2. The subcommands are intentionally
        // absent from this usage string so new agents do not discover them.
        + "\n       oya vcs [--format <text|json>] [--policy <observe|warn|enforce>] [--evidence-command <shell-command>] <claim|work|verify|done|status|symbols|queue|watch|promote> [vcs-specific args]"
        + "\n       oya gate validate foundation-bypass [--ledger <registry/foundation-bypasses>] [--now-epoch-days <days>]"
        + "\n       oya gate validate audit-chain-replay [--shards-dir <registry/audit-chain/shards>]"
        + "\n       oya gate validate foundry-capability-schema [--capabilities-dir <registry/capability-templates>] [--internal-registry <registry/capabilities/foundry-internal.json>]"
        + "\n       oya gate validate foundry-eval [--capabilities-dir <registry/capability-templates>]"
        + "\n       oya gate validate cross-tenant-access-fuzz"
        + "\n       oya gate validate adr-citation [--docs-dir <docs>] [--decisions-dir <docs/decisions>] [--inheritance-registry <registry/adr/inherited-bominal-adrs.yaml>]"
        + "\n       oya gate validate brand-residue [--docs-dir <docs>]"
        + "\n       oya gate validate no-grouping [--microservices-dir <specs/microservices>]"
        + "\n       oya gate validate api-semver [--contracts-dir <contracts>]"
        + "\n       oya gate validate supply-chain [--registry <registry/catalog>] [--deny <deny.toml>] [--check-script <scripts/check.sh>] [--adr0039-script <scripts/supply-chain-adr0039.sh>] [--adr0039-rust <crates/oya-dev-cli/src/commands/supply_chain.rs>] [--workflows-dir <.github/workflows>] [--release-images <registry/release/images.yaml>] [--branch-protection <.github/branch-protection.yaml>] [--admission-policy <infra/kyverno/policies/require-signed-images.yaml>] [--require-adr0039-evidence]"
        + "\n       oya gate validate release-supply-chain [--release-images <registry/release/images.yaml>] [--evidence-dir <registry/release/supply-chain>] [--phase <pre-release|release>]"
        + "\n       oya gate validate image-promotion [--promotion-dir <registry/release/image-promotions>]"
        + "\n       oya gate validate release-evidence-pack [--manifest <registry/release/evidence-packs.tsv>] [--compliance <docs/machine-readable/compliance.json>] [--require-records]"
        + "\n       oya gate validate typescript-workspace --lane <typecheck|test> [--repo-root <.>]"
        + "\n       oya gate validate pr-traceability [--pr-title <title>] [--pr-body <docs/templates/pull-request-template.md>] [--require-code-review|--forbid-code-review]"
        + "\n       oya gate validate authority-cohesion [--docs-dir <docs>]"
        + "\n       oya gate validate cargo-prefix [--workspace <Cargo.toml>] [--prefix <oya->]"
        + "\n       oya gate validate claim-ceiling [--registry <registry/catalog>]"
        + "\n       oya gate validate codeview-read-surface [--spec <specs/codeview-read-surface.json>]"
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
        + "\n       oya gate validate loop-recovery-patterns [--agent-durable-goal <specs/agent-durable-goal.json>] [--score-cards <specs/score-cards.json>] [--patterns-dir <registry/loop-recovery-patterns>] [--mistakes-ledger <registry/mistakes-ledger.json>]"
        + "\n       oya gate validate pre-push-contract [--done-definition <docs/checklists/done-definition-checklist.md>] [--cli-dispatch-source <crates/oya-dev-cli/src/lib.rs>] [--hook-script <scripts/hooks/pre-push.sh>]"
        + "\n       oya gate validate freshness [--repo-root <.>]"
        + "\n       oya gate validate protection-context-match [--branch-protection <.github/branch-protection.yaml>] [--workflows-dir <.github/workflows>] [--branch <dev>] [--applied-branch-protection <infra/branch-protection/dev.json>] [--skip-applied-branch-protection] [--live-required-contexts <required_status_checks.json>]"
        + "\n       oya gate validate retired-vocabulary [--registry <registry/vocabulary/retired.yaml>] [--corpus-root <path>] (repeatable) [--exclude-root <path>] (repeatable)"
        + "\n       oya gate validate quality-lanes [--registry <registry/quality/lanes.yaml>] [--ci-lanes <docs/standards/ci-lanes.md>] [--check-script <scripts/check.sh>] [--teams-dir <docs/teams>]"
        + "\n       oya gate validate honest-claims [--clear-default-corpus] [--corpus-root <path>]... [--plans-dir <.omc/plans/milestones>]"
        + "\n       oya gate validate aspirational-enforcement [--clear-default-corpus] [--corpus-root <path>]... [--catalog-dir <registry/catalog>] [--workflows-dir <.github/workflows>] [--quality-lanes <registry/quality/lanes.yaml>] [--branch-protection <.github/branch-protection.yaml>] [--branch <dev>]"
        + "\n       oya gate validate banned-primitives [--repo-root <.>] [--clear-default-roots] [--root <path>]... [--command-log-root <path>]... [--require-command-log-corpus] [--known-rationale <id>]..."
        + "\n       oya gate validate design-spec-maturity-claims [--standard <specs/design-spec-maturity-claims.json>] [--microservices-root <microservices>] [--deferred-surfaces <registry/design-spec-maturity/wave-3-i-deferred-surfaces.tsv>] [--emit-evidence <evidence/design-spec-maturity/after-2026-05-18.json>]"
        + "\n       oya gate validate adr-planning-completeness [--decisions-dir <docs/decisions>]"
        + "\n       oya gate validate masterplan-drift [--decisions-dir <docs/decisions>] [--masterplan <docs/machine-readable/masterplan.generated.json>]"
        + "\n       oya gate validate canonical-base-neutrality [--repo-root <.>] [--root <path>]... [--exclude-root <path>]... [--self-test]"
        + "\n       oya gate validate hyperscaler-arch-invariants [--spec <specs/hyperscaler-architecture-invariants.json>]"
        + "\n       oya gate validate hyperscaler-maturity-claims [--gates <specs/hyperscaler-gates.json>] [--workflow-studio <specs/microservices/workflow-studio.json>] [--workflow <specs/microservices/workflow.json>] [--workspace-hygiene <specs/workspace-hygiene.json>] [--branch-protection <.github/branch-protection.yaml>] [--pr-review-workflow <.github/workflows/pr-review.yml>] [--ci-fix-loop-workflow <.github/workflows/ci-failure-fix-loop.yml>] [--gitops-vcs <specs/gitops-vcs-replacement.json>] [--merge-queue <specs/merge-queue-parked-pr.json>] [--iterative-fix-loop <specs/iterative-fix-loop.json>] [--ci-fix-loop-retry-budget <registry/ci-fix-loop-retry-budget.json>]"
        + "\n       oya gate validate platform-substrate-defaults [--architecture <specs/platform-architecture.json>]"
        + "\n       oya gate validate workspace-hygiene [--policy <specs/workspace-hygiene.json>] [--no-scan] [--strict] [--clean-build-artifacts] [--clean-temp-artifacts]"
        + "\n       oya gate validate cloud-iac-module-catalog [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--catalog <microservices/cloud-iac/tofu/modules/catalog.json>]"
        + "\n       oya gate validate cloud-iac-gitops-evidence [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--templates-root <microservices/cloud-iac/iac>]"
        + "\n       oya gate validate cloud-iac-helm-chart-signed-image-wiring [--repo-root <.>] [--manifest <cloud/cloud-iac/manifest.json>] [--chart-root <cloud/cloud-iac/iac/k8s/helm>]"
        + "\n       oya gate validate cloud-iac-kubewarden-admission-policy [--repo-root <.>] [--manifest <cloud/cloud-iac/manifest.json>] [--kubewarden-root <cloud/cloud-iac/iac/k8s/kubewarden>] [--kyverno-policy <infra/kyverno/policies/require-signed-images.yaml>]"
        + "\n       oya gate validate cloud-iac-cell-topology [--repo-root <.>] [--manifest <cloud/cloud-iac/manifest.json>] [--topology <cloud/cloud-iac/cell-topology/foundation.json>] [--catalog <cloud/cloud-iac/tofu/modules/catalog.json>]"
        + "\n       oya gate validate cloud-iac-opentofu-validation [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--catalog <microservices/cloud-iac/tofu/modules/catalog.json>] [--modules-root <microservices/cloud-iac/tofu/modules>] [--tofu-bin <tofu>] [--keep-temp]"
        + "\n       oya gate validate cloud-iac-module-provider-requirements [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--catalog <microservices/cloud-iac/tofu/modules/catalog.json>] [--readiness <microservices/cloud-iac/tofu/modules/provider-readiness.json>]"
        + "\n       oya gate validate cloud-iac-module-provenance [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--catalog <microservices/cloud-iac/tofu/modules/catalog.json>] [--provenance <microservices/cloud-iac/tofu/modules/provenance.json>]"
        + "\n       oya gate validate cloud-iac-module-release-index [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--catalog <microservices/cloud-iac/tofu/modules/catalog.json>] [--provenance <microservices/cloud-iac/tofu/modules/provenance.json>] [--release-index <microservices/cloud-iac/tofu/modules/release-index.json>] [--archive-manifest <microservices/cloud-iac/tofu/modules/archive-manifest.json>] [--provider-lock-root <microservices/cloud-iac/tofu/provider-locks/foundation>] [--provider-signature-review <microservices/cloud-iac/tofu/provider-locks/foundation/provider-signature-review.json>]"
        + "\n       oya gate validate cloud-iac-module-archive [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--catalog <microservices/cloud-iac/tofu/modules/catalog.json>] [--provenance <microservices/cloud-iac/tofu/modules/provenance.json>] [--release-index <microservices/cloud-iac/tofu/modules/release-index.json>] [--archive-manifest <microservices/cloud-iac/tofu/modules/archive-manifest.json>] [--out-dir <target/oya-cloud-iac/module-archives>]"
        + "\n       oya gate validate cloud-iac-module-registry-protocol [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--release-index <microservices/cloud-iac/tofu/modules/release-index.json>] [--archive-manifest <microservices/cloud-iac/tofu/modules/archive-manifest.json>] [--protocol-fixtures <microservices/cloud-iac/tofu/module-registry/protocol-fixtures.json>]"
        + "\n       oya gate validate cloud-iac-provider-readiness [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--catalog <microservices/cloud-iac/tofu/modules/catalog.json>] [--readiness <microservices/cloud-iac/tofu/modules/provider-readiness.json>]"
        + "\n       oya gate validate cloud-iac-provider-lockfile [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--readiness <microservices/cloud-iac/tofu/modules/provider-readiness.json>] [--lock-root <microservices/cloud-iac/tofu/provider-locks/foundation>]"
        + "\n       oya gate validate cloud-iac-provider-signature-review [--repo-root <.>] [--manifest <microservices/cloud-iac/manifest.json>] [--lock-root <microservices/cloud-iac/tofu/provider-locks/foundation>] [--review <microservices/cloud-iac/tofu/provider-locks/foundation/provider-signature-review.json>]"
        + "\n       oya gate validate dependency-seam [--repo-root <.>] [--registry <registry/dependency-rationales.json>] [--evidence <evidence/multispectrum/<change>.json>]... [--fixture-root <crates/oya-check-dependency-seam/tests/fixtures>] [--offline|--online-audit] [--severity <report-only|error>] [--emit-report <path>]"
        + "\n       oya gate validate dependency-blessed-allowlist [--repo-root <.>] [--allowlist <registry/dependency-blessed-allowlist.json>] [--report-only|--enforce] [--emit-report <path>]"
        + "\n       oya gate validate license-policy [--workspace <Cargo.toml>]"
        + "\n       oya gate validate vendor-lockin-discipline [--registry <registry/vendor-lockin-phaseout/index.json>] [--workspace <Cargo.toml>]"
        + "\n       oya gate validate vendor-contract-recency [--ledger <docs/VENDOR-PARTNER-LEDGER.md>] [--today <YYYY-MM-DD>] [--renewal-window-days <90>]"
        + "\n       oya gate validate planes --all [--repo-root <.>]"
        + "\n       oya gate validate wave-integration --milestone <M02> [--manifest <.omc/plans/M01-M03-parallelization-manifest.md>] [--phases-dir <.omc/plans/milestones/M02b-substrate/phases>]"
        + "\n       oya gate validate mobile-native [--manifest <registry/mobile-native/products.tsv>] [--repo-root <.>]"
        + "\n       oya gate validate plane-class [--registry <registry/catalog>] [--baseline <registry/catalog>] [--reviewed-change <crate-id>]"
        + "\n       oya gate validate raci-team-coverage [--teams-dir <docs/teams>] [--raci <docs/RACI-OWNERSHIP.md>] [--codeowners <.github/CODEOWNERS>]"
        + "\n       oya gate validate readme-doc-coverage [--docs-dir <docs>] [--catalog <docs/machine-readable/catalog.json>]"
        + "\n       oya gate validate runbook-index-resolves [--docs-dir <docs>]"
        + "\n       oya gate validate runbook-freshness [--runbooks-dir <docs/runbooks>] [--today <YYYY-MM-DD>]"
        + "\n       oya gate validate slo-coverage [--registry <registry/catalog>]"
        + "\n       oya gate validate capacity-model-manifest [--microservices-root <cloud|oya|microservices>]... [--manifest <path>]... [--require-tenant-class-deltas]"
        + "\n       oya gate validate architecture-boundaries [--repo-root <.>] [--registry <registry/catalog>] [--self-test]"
        + "\n       oya gate validate master-plan-completion [--master-plan <specs/masterplan.json>] [--evidence-dir <evidence/foundation>]..."
        + "\n       oya gate validate board-masterplan-consistency [--master-plan <docs/machine-readable/masterplan.generated.json>] [--board-snapshot <docs/machine-readable/board-sync.generated.json>]"
        + "\n       oya gate validate product-index [--products-readme <docs/products/README.md>] [--catalog <docs/machine-readable/catalog.json>]"
        + "\n       oya gate validate product-prd-json [--repo-root <.>] [--product <specs/products/<id>.json>]..."
        + "\n       oya gate validate stage0-prereqs [--repo-root <.>] [--self-test]"
        + "\n       oya gate validate deployment-ops-contract [--repo-root <.>] [--contract <specs/deployment-ops-contract.json>] [--makefile <Makefile>]"
        + "\n       oya gate validate milestone-audit [--repo-root <.>] [--audit <registry/milestone-audit/index.json>]"
        + "\n       oya gate run-all [--include-deferred] [--ci-required]"
        + "\n       oya verify [--include-deferred] [--ci-required]   # local-developer fold of `gate run-all`; canonical pre-push/pre-PR entry"
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
            | "docs/governance-lanes/glossary-vocabulary.md"
            | "docs/MISTAKES-LEDGER.md"
            | "docs/ADR-CONSOLIDATION-PLAN.md"
            | "docs/ADR-LEGACY-REGRESSION-MAPPING.md"
            | "docs/CHANGELOG.md"
            | "docs/RISK-REGISTER.md"
            | "docs/decisions/ADR-0709-general-live-apex.md"
            | "docs/teams/README.md"
            | "docs/teams/tactical-first-vertical-pilot/CHARTER.md"
    ) || path.starts_with("docs/decisions/ADR-0016-")
        || path.starts_with("docs/decisions/ADR-0018-")
        || path.starts_with("docs/decisions/ADR-")
        || path.starts_with("docs/plans/M01-foundation-cc-01-cutover/")
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
        || (path.starts_with("docs/architecture/")
            && (path.contains("audit")
                || path.contains("deep-dive")
                || path.contains("synthesis")
                || path.contains("adjudication")
                || path.contains("lessons-learned")
                || path.contains("executive-briefing")
                || path.contains("scorecard")))
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

struct NoGroupingValidateArgs {
    microservices_dir: PathBuf,
}

fn parse_no_grouping_validate_args(args: Vec<String>) -> Result<NoGroupingValidateArgs, String> {
    let mut parsed = NoGroupingValidateArgs {
        microservices_dir: PathBuf::from("specs/microservices"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--microservices-dir" => parsed.microservices_dir = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn validate_no_grouping_gate(args: NoGroupingValidateArgs) -> Result<(usize, usize), String> {
    let artifacts = read_grouping_artifacts(&args.microservices_dir)?;
    let report = validate_no_grouping(artifacts).map_err(|error| {
        format!("grouping artifact violates flat-only doctrine (ADR-0362): {error:?}")
    })?;
    Ok((report.artifacts_checked, report.retiring_wrappers))
}

// Discover grouping-shaped spec wrappers under `microservices_dir` and read each
// `_meta.status` / `_meta.retirement_ref`. Tolerates an absent directory (no
// microservice specs => no grouping artifacts => clean).
fn read_grouping_artifacts(dir: &Path) -> Result<Vec<GroupingArtifact>, String> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(error) => {
            return Err(format!(
                "no-grouping: directory unreadable {}: {error}",
                dir.display()
            ));
        }
    };
    for entry in entries {
        let entry = entry.map_err(|error| format!("no-grouping: dir entry unreadable: {error}"))?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !path.is_file() || !is_grouping_artifact(name) {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("no-grouping: read {} failed: {error}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|error| format!("no-grouping: parse {} failed: {error}", path.display()))?;
        let meta = value.get("_meta");
        let status = meta
            .and_then(|m| m.get("status"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);
        let has_retirement_ref = meta
            .and_then(|m| m.get("retirement_ref"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|s| !s.trim().is_empty());
        out.push(GroupingArtifact {
            file_name: name.to_string(),
            status,
            has_retirement_ref,
        });
    }
    Ok(out)
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
        // Superseded ADR bodies retain historical lineage (including retired
        // brand tokens such as `forgejo`); the deny-list prevents *new*
        // occurrences in live docs, not the rewriting of immutable history.
        // Exclude them here, in the caller's path/status filter, not in the
        // kernel (see libs/oya-check-brand-residue/src/lib.rs module docs).
        if is_superseded_adr_document(&contents) {
            continue;
        }
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

/// True when a doc is a Superseded ADR, detected from its YAML frontmatter
/// `status:` field. Superseded ADRs are immutable history and are excluded from
/// the brand-residue forbidden-token scan so the gate prevents new occurrences
/// without forcing a rewrite of retired lineage.
fn is_superseded_adr_document(contents: &str) -> bool {
    let mut in_frontmatter = false;
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim_end();
        if index == 0 {
            if trimmed == "---" {
                in_frontmatter = true;
                continue;
            }
            return false;
        }
        if !in_frontmatter {
            return false;
        }
        if trimmed == "---" {
            return false;
        }
        if let Some(value) = trimmed.strip_prefix("status:") {
            return value.trim().eq_ignore_ascii_case("superseded");
        }
    }
    false
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
pub(crate) struct VendorLockinDisciplineValidateArgs {
    pub(crate) registry_path: PathBuf,
    pub(crate) workspace_manifest_path: PathBuf,
}

pub(crate) fn parse_vendor_lockin_discipline_validate_args(
    args: Vec<String>,
) -> Result<VendorLockinDisciplineValidateArgs, String> {
    let mut parsed = VendorLockinDisciplineValidateArgs {
        registry_path: PathBuf::from("registry/vendor-lockin-phaseout/index.json"),
        workspace_manifest_path: PathBuf::from("Cargo.toml"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--registry" => parsed.registry_path = PathBuf::from(path),
            "--workspace" => parsed.workspace_manifest_path = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_vendor_lockin_discipline_gate(
    args: VendorLockinDisciplineValidateArgs,
) -> Result<VendorLockinReport, String> {
    let source = fs::read_to_string(&args.registry_path).map_err(|error| {
        format!(
            "vendor-lockin registry unreadable {}: {error}",
            args.registry_path.display()
        )
    })?;
    let entries = parse_vendor_lockin_registry(&source)
        .map_err(|error| format!("vendor-lockin registry parse failed: {error}"))?;
    let report = validate_vendor_lockin_registry(&entries)
        .map_err(|error| format!("vendor-lockin discipline violated: {error}"))?;

    // Second-impl rule for Tier II: every declared seam_adapter_trait that
    // points into the workspace MUST resolve to an existing workspace member;
    // and at least one of the registered seam_adapter_impls MUST also resolve.
    // This closes the gap where the JSON registry references a vendor seam
    // crate that has not actually landed.
    let workspace_members =
        read_workspace_member_paths(&args.workspace_manifest_path).map_err(|error| {
            format!("vendor-lockin workspace audit unable to read members: {error}")
        })?;
    let workspace_dir = args
        .workspace_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let member_set: std::collections::BTreeSet<String> = workspace_members
        .into_iter()
        .map(|member| {
            // Normalize against the workspace root so trait paths stored as
            // `crates/oya-foo` match `crates/oya-foo` entries from the
            // workspace manifest regardless of relative-path framing.
            let joined = workspace_dir.join(&member);
            joined
                .strip_prefix(workspace_dir)
                .map(slash_path_component)
                .unwrap_or_else(|_| slash_path_component(&joined))
        })
        .collect();
    for entry in entries.iter() {
        if !matches!(
            entry.tier,
            check_vendor_lockin_discipline::VendorTier::TierII
        ) {
            continue;
        }
        let Some(trait_ref) = entry.seam_adapter_trait.as_deref() else {
            continue;
        };
        if !trait_ref.starts_with("crates/") && !trait_ref.starts_with("microservices/") {
            continue;
        }
        if !member_set.contains(trait_ref) {
            return Err(format!(
                "vendor-lockin discipline violated: vendor {} declares seam_adapter_trait `{}` but no matching workspace member exists",
                entry.name, trait_ref
            ));
        }
        let any_impl_present = entry
            .seam_adapter_impls
            .iter()
            .filter(|impl_path| {
                impl_path.starts_with("crates/") || impl_path.starts_with("microservices/")
            })
            .any(|impl_path| {
                // Trim trailing parenthetical annotations like "(planned)".
                let normalized = impl_path
                    .split_whitespace()
                    .next()
                    .unwrap_or(impl_path.as_str());
                member_set.contains(normalized)
            });
        // Adopted Tier II MUST have at least one impl resolve. Pre-classified
        // Tier II is exempt above — it's a placeholder declaring future seam shape.
        if !any_impl_present
            && entry
                .seam_adapter_impls
                .iter()
                .any(|impl_path| impl_path.starts_with("crates/"))
        {
            return Err(format!(
                "vendor-lockin discipline violated: vendor {} declares workspace-rooted seam impls but none resolve to a workspace member",
                entry.name
            ));
        }
    }

    Ok(report)
}

fn slash_path_component(path: &Path) -> String {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
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
