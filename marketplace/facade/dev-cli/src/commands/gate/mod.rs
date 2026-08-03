use std::process::ExitCode;

mod architecture_boundaries;
mod board_masterplan_consistency;
mod deployment_ops_contract;
mod master_plan_completion_audit;
mod milestone_audit;
mod product_index;
mod product_prd_json;
// O7 (ADR-0360): content-addressed gate-result cache, wired into run-all behind
// OYA_GATE_CACHE=1 (Unenumerable-by-default => safe). Per-gate input declaration
// in lane_gate_inputs() is the incremental adoption path.
mod result_cache;
mod run_all;
mod stage0_application_shell_prereqs;

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    // `gate run-all` aggregator branch: replaces the legacy
    // `scripts/check.sh` per Wave 2 of the shell/python → Rust
    // replacement program (audit row B-1). Special-case before the
    // (verb, lane) match because the verb is `run-all`, not `validate`.
    if args.first().map(String::as_str) == Some("run-all") {
        let rest = args.into_iter().skip(1).collect::<Vec<_>>();
        return match run_all::parse_run_all_args(rest) {
            Ok(parsed) => run_all::run_all_gates(parsed, usage),
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(2)
            }
        };
    }
    let mut args = args.into_iter();
    match (args.next().as_deref(), args.next().as_deref()) {
        // Grounding note for exception-ledger audits:
        // "foundation-bypass" is Oyatie's documented engineering-platform
        // domain term for tracked, expirable gate exceptions, not a code
        // recovery path. This command is fail-closed: malformed, duplicate,
        // missing-ledger, zero-window, or expired records return FAILURE.
        // An explicitly present ledger with zero records means no exception
        // exists.
        // Tested by oya-dev-cli::gate_cli and
        // intelligence-bypass-domain::foundation_bypass.
        // active-artifact-contract: ADR-0069 v3.0.0 vertical enforcement loop.
        // Validates that every row in registry/artifact-capabilities-registry.json
        // satisfies R01-R07: tracked path, unique artifact_id, all 9
        // capabilities, and status-specific evidence/prerequisite/rationale
        // requirements.
        // Optionally emits an evidence bundle and one graph-edge artifact per
        // full-consensus-planner-v3 amendments #4/#9/#10.
        (Some("validate"), Some("active-artifact-contract")) => {
            match crate::parse_active_artifact_contract_validate_args(args.collect()) {
                Ok(args) => match crate::validate_active_artifact_contract_gate(args) {
                    Ok(report) => {
                        println!(
                            "active-artifact-contract validation passed: {} rows, {} HEAD-tracked, {} graph edges, {} warnings, {} ms",
                            report.rows_seen,
                            report.head_tracked_count,
                            report.graph_edges.len(),
                            report.warning_count(),
                            report.validation_duration_ms
                        );
                        if report.warning_count() > 0 {
                            println!(
                                "active-artifact-contract warnings: {}",
                                report.warning_summary()
                            );
                        }
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("active-artifact-contract validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // cedar-fragment-coverage: enforces C01..C04 from
        // registry/cedar-fragments.json. Closes drift between OpenAPI
        // contracts (cedar_fragments[] arrays), bounded-contexts.json
        // (cedar_fragments_planned[] arrays), and on-disk .cedar files.
        (Some("validate"), Some("cedar-fragment-coverage")) => {
            match crate::parse_cedar_fragment_coverage_validate_args(args.collect()) {
                Ok(args) => match crate::validate_cedar_fragment_coverage_gate(args) {
                    Ok(report) => {
                        println!(
                            "cedar-fragment-coverage validation passed: {} rows, {} openapi-refs, {} bc-refs, {} .cedar files, {} ms",
                            report.report.rows_seen,
                            report.report.openapi_references_seen,
                            report.report.bc_references_seen,
                            report.report.cedar_files_seen,
                            report.validation_duration_ms
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("cedar-fragment-coverage validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // architecture-map: emits registry/graph/architecture-map.json by
        // walking workspace Cargo.toml + registry/* + contracts/.
        // Visualization-as-code directive 2026-05-12.
        (Some("emit"), Some("architecture-map")) => {
            match crate::parse_architecture_map_emit_args(args.collect()) {
                Ok(args) => match crate::emit_architecture_map_gate(args) {
                    Ok(report) => {
                        println!(
                            "architecture-map emitted: {} nodes, {} edges, {} orphans, {} ms",
                            report.node_count,
                            report.edge_count,
                            report.orphan_count,
                            report.duration_ms
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("architecture-map emit failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // openapi-rest-route-parity: enforces that route constants in
        // crates/oya-*-rest/src/lib.rs stay 1:1 with paths in
        // contracts/*.openapi.yaml. Closes drift between REST handlers and
        // OpenAPI contracts.
        (Some("validate"), Some("openapi-rest-route-parity")) => {
            match crate::parse_openapi_rest_route_parity_validate_args(args.collect()) {
                Ok(args) => match crate::validate_openapi_rest_route_parity_gate(args) {
                    Ok(report) => {
                        println!(
                            "openapi-rest-route-parity validation passed: {} REST routes, {} OpenAPI paths, {} ms",
                            report.report.rest_route_count,
                            report.report.openapi_path_count,
                            report.validation_duration_ms
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("openapi-rest-route-parity validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("foundation-bypass")) => {
            match crate::parse_foundation_bypass_validate_args(args.collect()) {
                Ok(args) => match crate::validate_foundation_bypass_gate(args) {
                    Ok((records, open)) => {
                        println!(
                            "foundation gate exception ledger validation passed: {records} records, {open} open"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("foundation gate exception ledger validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("audit-chain-replay")) => {
            match crate::parse_audit_chain_replay_validate_args(args.collect()) {
                Ok(args) => match crate::validate_audit_chain_replay_gate(args) {
                    Ok((shards, events)) => {
                        println!(
                            "audit chain replay validation passed: {shards} shards, {events} events"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("audit chain replay validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("foundry-capability-schema")) => {
            match crate::parse_foundry_capability_schema_validate_args(args.collect()) {
                Ok(args) => match crate::validate_foundry_capability_schema_gate(args) {
                    Ok((capabilities, mcp_contracts, schemas, internal_records)) => {
                        println!(
                            "foundry capability schema validation passed: {capabilities} capabilities, {mcp_contracts} mcp contracts, {schemas} schemas, {internal_records} internal registry records"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("foundry capability schema validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("foundry-eval")) => {
            match crate::parse_foundry_eval_validate_args(args.collect()) {
                Ok(args) => match crate::validate_foundry_eval_gate(args) {
                    Ok((capabilities, cases, runs)) => {
                        println!(
                            "foundry eval validation passed: {capabilities} capabilities, {cases} cases, {runs} passing runs"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("foundry eval validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cross-tenant-access-fuzz")) => {
            match crate::parse_cross_tenant_access_fuzz_validate_args(args.collect()) {
                Ok(args) => match crate::validate_cross_tenant_access_fuzz_gate(args) {
                    Ok(cases) => {
                        println!("cross-tenant access fuzz validation passed: {cases} cases");
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("cross-tenant access fuzz validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("plane-class")) => {
            match crate::parse_plane_class_validate_args(args.collect()) {
                Ok(args) => match crate::validate_plane_class_gate(args) {
                    Ok((records, reviewed_changes)) => {
                        println!(
                            "plane class validation passed: {records} records, {reviewed_changes} reviewed changes"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("plane class validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("claim-ceiling")) => {
            match crate::parse_claim_ceiling_validate_args(args.collect()) {
                Ok(args) => match crate::validate_claim_ceiling_gate(args) {
                    Ok(records) => {
                        println!("claim ceiling validation passed: {records} records");
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("claim ceiling validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("adr-citation")) => {
            match crate::parse_adr_citation_validate_args(args.collect()) {
                Ok(args) => match crate::validate_adr_citation_gate(args) {
                    Ok((documents, citations, allowed_pack_adrs)) => {
                        println!(
                            "ADR citation validation passed: {documents} documents, {citations} citations, {allowed_pack_adrs} allowed ADRs"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("ADR citation validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("brand-residue")) => {
            match crate::parse_brand_residue_validate_args(args.collect()) {
                Ok(args) => match crate::validate_brand_residue_gate(args) {
                    Ok((documents, patterns)) => {
                        println!(
                            "brand residue validation passed: {documents} files, {patterns} transition patterns"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("brand residue validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("no-grouping")) => {
            match crate::parse_no_grouping_validate_args(args.collect()) {
                Ok(args) => match crate::validate_no_grouping_gate(args) {
                    Ok((checked, retiring)) => {
                        println!(
                            "no-grouping validation passed: {checked} grouping artifacts inspected ({retiring} deprecated retiring wrappers; flat-only per ADR-0362)"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("no-grouping validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("api-semver")) => {
            match crate::parse_api_semver_validate_args(args.collect()) {
                Ok(args) => match crate::validate_api_semver_gate(args) {
                    Ok((contracts, metadata)) => {
                        println!(
                            "API semver validation passed: {contracts} contracts, {metadata} metadata records"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("API semver validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("supply-chain")) => {
            match crate::parse_supply_chain_validate_args(args.collect()) {
                Ok(args) => match crate::validate_supply_chain_gate(args) {
                    Ok((records, source_only)) => {
                        println!(
                            "supply chain validation passed: {records} catalog records, {source_only} source-only attestations"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("supply chain validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("image-promotion")) => {
            match crate::parse_image_promotion_validate_args(args.collect()) {
                Ok(args) => match crate::validate_image_promotion_gate(args) {
                    Ok(report) => {
                        println!(
                            "image promotion validation passed: {} artifacts, {} promotion records, {} kubewarden verifier records, {} kyverno verifier records",
                            report.artifacts,
                            report.promotion_records,
                            report.kubewarden_verifier_records,
                            report.kyverno_verifier_records
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("image promotion validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("release-supply-chain")) => {
            match crate::parse_release_supply_chain_validate_args(args.collect()) {
                Ok(args) => match crate::validate_release_supply_chain_gate(args) {
                    Ok(report) => {
                        let artifacts = report.artifacts;
                        let evidence = report.evidence;
                        let phase = crate::release_supply_chain_phase_name(report.phase);
                        println!(
                            "release supply chain validation passed: {artifacts} artifacts, {evidence} evidence records, phase={phase}"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("release supply chain validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("release-evidence-pack")) => {
            match crate::parse_release_evidence_pack_validate_args(args.collect()) {
                Ok(args) => match crate::validate_release_evidence_pack_gate(args) {
                    Ok((known_regulators, records, published)) => {
                        println!(
                            "release evidence pack validation passed: {known_regulators} known regulators, {records} records, {published} published"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("release evidence pack validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("typescript-workspace")) => {
            match crate::parse_typescript_workspace_validate_args(args.collect()) {
                Ok(args) => match crate::validate_typescript_workspace_gate(args) {
                    Ok((lane, workspace_present, markers, scripts)) => {
                        println!(
                            "typescript workspace validation passed: lane={lane}, workspace_present={workspace_present}, {markers} markers, {scripts} scripts"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("typescript workspace validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("pr-traceability")) => {
            match crate::parse_pr_traceability_validate_args(args.collect()) {
                Ok(args) => match crate::validate_pr_traceability_gate(args) {
                    Ok((sections, code_review_present)) => {
                        println!(
                            "PR traceability validation passed: {sections} required sections, code_review_present={code_review_present}"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("PR traceability validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cargo-prefix")) => {
            match crate::parse_cargo_prefix_validate_args(args.collect()) {
                Ok(args) => match crate::validate_cargo_prefix_gate(args) {
                    Ok(members) => {
                        println!("cargo prefix validation passed: {members} workspace members");
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("cargo prefix validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("codeview-read-surface")) => {
            match crate::parse_codeview_read_surface_validate_args(args.collect()) {
                Ok(args) => match crate::validate_codeview_read_surface_gate(args) {
                    Ok(report) => {
                        println!(
                            "codeview read-surface validation passed: {} commands, {} compatibility binaries, {} provider env vars, {} rejected tokens",
                            report.commands_checked,
                            report.compatibility_binaries_checked,
                            report.provider_env_vars_checked,
                            report.rejected_tokens_checked
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("codeview read-surface validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("authority-cohesion")) => {
            match crate::parse_authority_cohesion_validate_args(args.collect()) {
                Ok(args) => match crate::validate_authority_cohesion_gate(args) {
                    Ok(documents) => {
                        println!("authority cohesion validation passed: {documents} documents");
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("authority cohesion validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("statelessness")) => {
            match crate::parse_statelessness_validate_args(args.collect()) {
                Ok(args) => match crate::validate_statelessness_gate(args) {
                    Ok(report) => {
                        println!(
                            "statelessness validation passed: {} files scanned, {} in scope",
                            report.files_checked, report.files_in_scope
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("statelessness validation failed:\n{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("shardability")) => {
            match crate::parse_shardability_validate_args(args.collect()) {
                Ok(args) => match crate::validate_shardability_gate(args) {
                    Ok(report) => {
                        println!(
                            "shardability validation passed: {} files scanned, {} tables ({} global)",
                            report.files_checked, report.tables_seen, report.tables_global
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("shardability validation failed:\n{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("perf-budget")) => {
            match crate::parse_perf_budget_validate_args(args.collect()) {
                Ok(args) => match crate::validate_perf_budget_gate(args) {
                    Ok(report) => {
                        println!(
                            "perf-budget validation passed: {} plans checked",
                            report.plans_checked
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("perf-budget validation failed:\n{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("benchmark")) => {
            match crate::parse_benchmark_validate_args(args.collect()) {
                Ok(args) => match crate::validate_benchmark_gate(args) {
                    Ok(report) => {
                        println!(
                            "benchmark validation passed: {} PRDs checked",
                            report.prds_checked
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("benchmark validation failed:\n{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("dependency-seam")) => {
            match crate::parse_dependency_seam_validate_args(args.collect()) {
                Ok(args) => match crate::validate_dependency_seam_gate(args) {
                    Ok(report) => {
                        let blocking = report.blocking_diagnostics().len();
                        println!(
                            "dependency-seam validation passed: {} subchecks, {} pass, {} report-only, {} skipped, {} fail, {} diagnostics, {} blocking",
                            report.subchecks.len(),
                            report.status_count(oya_check_dependency_seam::SubcheckStatus::Pass),
                            report.status_count(
                                oya_check_dependency_seam::SubcheckStatus::ReportOnly
                            ),
                            report.status_count(oya_check_dependency_seam::SubcheckStatus::Skipped),
                            report.status_count(oya_check_dependency_seam::SubcheckStatus::Fail),
                            report.diagnostic_count(),
                            blocking
                        );
                        if blocking == 0 {
                            ExitCode::SUCCESS
                        } else {
                            eprintln!(
                                "dependency-seam validation failed: {blocking} blocking diagnostics"
                            );
                            ExitCode::FAILURE
                        }
                    }
                    Err(message) => {
                        eprintln!("dependency-seam validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("dependency-blessed-allowlist")) => {
            match crate::parse_dependency_blessed_allowlist_args(args.collect()) {
                Ok(args) => match crate::validate_dependency_blessed_allowlist_gate(args) {
                    Ok(report) => {
                        for finding in &report.findings {
                            println!(
                                "dependency-blessed-allowlist: {} ({}) declares unblessed direct dependency `{}` in [{}]",
                                finding.crate_name,
                                finding.crate_path,
                                finding.dependency,
                                finding.table
                            );
                        }
                        let unblessed = report.unblessed_count();
                        let distinct = report.distinct_unblessed().len();
                        println!(
                            "dependency-blessed-allowlist scan: {} crates scanned, {} blessed deps, {} unblessed findings, {} distinct unblessed deps ({})",
                            report.crates_scanned,
                            report.blessed_count,
                            unblessed,
                            distinct,
                            if report.enforced {
                                "enforce"
                            } else {
                                "report-only"
                            }
                        );
                        if report.enforced && unblessed > 0 {
                            eprintln!(
                                "dependency-blessed-allowlist validation failed: {unblessed} unblessed direct dependencies"
                            );
                            ExitCode::FAILURE
                        } else {
                            ExitCode::SUCCESS
                        }
                    }
                    Err(message) => {
                        eprintln!("dependency-blessed-allowlist validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("http-stack")) => {
            match crate::parse_http_stack_validate_args(args.collect()) {
                Ok(args) => match crate::validate_http_stack_gate(args) {
                    Ok(report) => {
                        for finding in &report.findings {
                            match finding.kind {
                                crate::HttpStackFindingKind::Forbidden => eprintln!(
                                    "http-stack: {} ({}) declares FORBIDDEN HTTP framework `{}` in [{}] — only hyper (preferred) / axum (sanctioned) are allowed",
                                    finding.crate_name,
                                    finding.crate_path,
                                    finding.framework,
                                    finding.table
                                ),
                                crate::HttpStackFindingKind::UnjustifiedSanctioned => println!(
                                    "http-stack WARN: {} ({}) declares `{}` without a recorded justification in specs/http-stack-policy.json (justified_crates.{}) — prefer hyper (low-level default) or record a rationale",
                                    finding.crate_name,
                                    finding.crate_path,
                                    finding.framework,
                                    finding.framework
                                ),
                            }
                        }
                        let forbidden = report.forbidden_count();
                        let unjustified = report.unjustified_count();
                        println!(
                            "http-stack scan: {} crates scanned, {} use hyper, {} use axum, {} forbidden, {} unjustified-axum ({})",
                            report.crates_scanned,
                            report.hyper_crate_count,
                            report.axum_crate_count,
                            forbidden,
                            unjustified,
                            if report.enforced {
                                "enforce"
                            } else {
                                "report-only"
                            }
                        );
                        if report.enforced && forbidden > 0 {
                            eprintln!(
                                "http-stack validation failed: {forbidden} forbidden HTTP-framework dependencies"
                            );
                            ExitCode::FAILURE
                        } else {
                            ExitCode::SUCCESS
                        }
                    }
                    Err(message) => {
                        eprintln!("http-stack validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("workspace-topology")) => {
            match crate::parse_workspace_topology_validate_args(args.collect()) {
                Ok(args) => match crate::validate_workspace_topology_gate(args) {
                    Ok(report) => {
                        for finding in &report.findings {
                            eprintln!(
                                "workspace-topology {}: {}",
                                finding.rule.as_str(),
                                finding.detail
                            );
                        }
                        let count = report.findings.len();
                        println!(
                            "workspace-topology scan: {} members scanned, {} findings ({})",
                            report.members_scanned,
                            count,
                            if report.enforced {
                                "enforce"
                            } else {
                                "report-only"
                            }
                        );
                        if report.enforced && count > 0 {
                            eprintln!(
                                "workspace-topology validation failed: {count} topology violations"
                            );
                            ExitCode::FAILURE
                        } else {
                            ExitCode::SUCCESS
                        }
                    }
                    Err(message) => {
                        eprintln!("workspace-topology validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("freshness")) => {
            match crate::parse_freshness_gate_args(args.collect()) {
                Ok(args) => match crate::validate_freshness_gate(args) {
                    Ok(report) => {
                        println!(
                            "{}",
                            ci_generated_artifact_freshness::render_findings(&report.findings)
                        );
                        if report.is_green() {
                            ExitCode::SUCCESS
                        } else {
                            ExitCode::FAILURE
                        }
                    }
                    Err(message) => {
                        eprintln!("freshness validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("license-policy")) => {
            match crate::parse_license_policy_validate_args(args.collect()) {
                Ok(args) => match crate::validate_license_policy_gate(args) {
                    Ok(packages) => {
                        println!("license policy validation passed: {packages} packages");
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("license policy validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("vendor-contract-recency")) => {
            match crate::parse_vendor_contract_recency_validate_args(args.collect()) {
                Ok(args) => match crate::validate_vendor_contract_recency_gate(args) {
                    Ok((records, contracted, tasks)) => {
                        println!(
                            "vendor contract recency validation passed: {records} records, {contracted} contracted, {tasks} renewal tasks required"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("vendor contract recency validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // PR #143 Fix-M/N/R/S/T/U dispatch arms (1 strict + 11 advisory).
        (Some("validate"), Some("vendor-lockin-discipline")) => {
            match crate::parse_vendor_lockin_discipline_validate_args(args.collect()) {
                Ok(args) => match crate::validate_vendor_lockin_discipline_gate(args) {
                    Ok(report) => {
                        println!(
                            "vendor-lockin discipline validation passed: {} entries (Tier I={}, II={}, III={})",
                            report.entries_seen,
                            report.tier_i_count + report.tier_i_asterisk_count,
                            report.tier_ii_count + report.tier_ii_pre_count,
                            report.tier_iii_count
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("vendor-lockin discipline validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("authz-tier-discipline")) => {
            match crate::validate_authz_tier_discipline_gate(args.collect()) {
                Ok(report) => {
                    println!(
                        "authz-tier discipline advisory: {} cedar files + {} envoy files scanned; {} findings",
                        report.cedar_files_scanned,
                        report.envoy_files_scanned,
                        report.total_findings
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("authz-tier discipline validation error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("tenant-cost-labels-coverage")) => {
            match crate::validate_tenant_cost_labels_coverage_gate(args.collect()) {
                Ok(summary) => {
                    println!(
                        "tenant cost-labels coverage advisory: {} manifests scanned; {} findings",
                        summary.manifests_scanned, summary.findings
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("tenant cost-labels coverage error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("backup-retention-discipline")) => {
            match crate::validate_backup_retention_discipline_gate(args.collect()) {
                Ok(summary) => {
                    println!(
                        "backup retention discipline advisory: {} declarations scanned; {} findings",
                        summary.declarations_scanned, summary.findings
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("backup retention discipline error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("vector-store-discipline")) => {
            match crate::validate_vector_store_discipline_gate(args.collect()) {
                Ok(summary) => {
                    println!(
                        "vector store discipline advisory: {} records scanned; {} violations",
                        summary.records_scanned, summary.violations
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("vector store discipline error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("olap-tier-discipline")) => {
            match crate::validate_olap_tier_discipline_gate(args.collect()) {
                Ok(summary) => {
                    println!(
                        "olap tier discipline advisory: {} records scanned; {} violations",
                        summary.records_scanned, summary.violations
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("olap tier discipline error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("wasm-runtime-discipline")) => {
            match crate::validate_wasm_runtime_discipline_gate(args.collect()) {
                Ok(summary) => {
                    println!(
                        "wasm runtime discipline advisory: {} manifests scanned; {} violations",
                        summary.manifests_scanned, summary.violations
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("wasm runtime discipline error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("iac-tier-discipline")) => {
            match crate::validate_iac_tier_discipline_gate(args.collect()) {
                Ok(summary) => {
                    println!(
                        "iac tier discipline advisory: {} artifacts scanned; {} violations",
                        summary.artifacts_scanned, summary.violations
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("iac tier discipline error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-module-catalog")) => {
            match crate::parse_cloud_iac_module_catalog_validate_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_module_catalog_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-module-catalog validation passed: {} modules, {} files checked; manifest {}; catalog {}",
                            report.modules_checked,
                            report.files_checked,
                            report.manifest_path,
                            report.catalog_path
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-gitops-evidence")) => {
            match crate::parse_cloud_iac_gitops_evidence_validate_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_gitops_evidence_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-gitops-evidence validation passed: {} contexts, {} templates checked; manifest {}; templates root {}",
                            report.contexts_checked,
                            report.templates_checked,
                            report.manifest_path,
                            report.templates_root
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-helm-chart-signed-image-wiring")) => {
            match crate::parse_cloud_iac_helm_chart_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_helm_chart_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-helm-chart-signed-image-wiring validation passed: {} files, {} required lines checked; manifest {}; chart root {}",
                            report.files_checked,
                            report.required_lines_checked,
                            report.manifest_path,
                            report.chart_root_path
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-kubewarden-admission-policy")) => {
            match crate::parse_cloud_iac_kubewarden_admission_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_kubewarden_admission_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-kubewarden-admission-policy validation passed: {} policy files, {} required markers checked; manifest {}; kubewarden root {}; kyverno policy {}",
                            report.policy_files_checked,
                            report.required_markers_checked,
                            report.manifest_path,
                            report.kubewarden_root_path,
                            report.kyverno_policy_path
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-cell-topology")) => {
            match crate::parse_cloud_iac_cell_topology_validate_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_cell_topology_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-cell-topology validation passed: {} contexts, {} cells, {} module refs, {} files checked; manifest {}; topology {}; catalog {}",
                            report.contexts_checked,
                            report.cells_checked,
                            report.module_refs_checked,
                            report.files_checked,
                            report.manifest_path,
                            report.topology_path,
                            report.catalog_path
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-opentofu-validation")) => {
            match crate::parse_cloud_iac_opentofu_validation_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_opentofu_validation_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-opentofu-validation validation passed: {} modules, {} init runs, {} validate runs; manifest {}; catalog {}; modules root {}",
                            report.modules_checked,
                            report.init_runs,
                            report.validate_runs,
                            report.manifest_path,
                            report.catalog_path,
                            report.modules_root
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-module-provenance")) => {
            match crate::parse_cloud_iac_module_provenance_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_module_provenance_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-module-provenance validation passed: {} modules, {} files checked; manifest {}; catalog {}; provenance {}",
                            report.modules_checked,
                            report.files_checked,
                            report.manifest_path,
                            report.catalog_path,
                            report.provenance_path
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-module-provider-requirements")) => {
            match crate::parse_cloud_iac_module_provider_requirements_args(args.collect()) {
                Ok(parsed) => {
                    match crate::validate_cloud_iac_module_provider_requirements_gate(parsed) {
                        Ok(report) => {
                            println!(
                                "cloud-iac-module-provider-requirements validation passed: {} modules, {} provider requirements checked; manifest {}; catalog {}; readiness {}",
                                report.modules_checked,
                                report.provider_requirements_checked,
                                report.manifest_path,
                                report.catalog_path,
                                report.readiness_path
                            );
                            ExitCode::SUCCESS
                        }
                        Err(message) => {
                            eprintln!("{message}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-module-release-index")) => {
            match crate::parse_cloud_iac_module_release_index_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_module_release_index_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-module-release-index validation passed: {} modules, {} files checked; manifest {}; catalog {}; provenance {}; release index {}; archive manifest {}",
                            report.modules_checked,
                            report.files_checked,
                            report.manifest_path,
                            report.catalog_path,
                            report.provenance_path,
                            report.release_index_path,
                            report.archive_manifest_path
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-module-archive")) => {
            match crate::parse_cloud_iac_module_archive_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_module_archive_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-module-archive validation passed: {} modules, {} archives built, {} files archived; manifest {}; catalog {}; provenance {}; release index {}; archive manifest {}; output root {}",
                            report.modules_checked,
                            report.archives_built,
                            report.files_archived,
                            report.manifest_path,
                            report.catalog_path,
                            report.provenance_path,
                            report.release_index_path,
                            report.archive_manifest_path,
                            report.output_dir
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-module-registry-protocol")) => {
            match crate::parse_cloud_iac_module_registry_protocol_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_module_registry_protocol_gate(parsed)
                {
                    Ok(report) => {
                        println!(
                            "cloud-iac-module-registry-protocol validation passed: {} modules, {} versions responses, {} download responses checked; manifest {}; release index {}; archive manifest {}; protocol fixtures {}",
                            report.modules_checked,
                            report.versions_responses_checked,
                            report.download_responses_checked,
                            report.manifest_path,
                            report.release_index_path,
                            report.archive_manifest_path,
                            report.protocol_fixtures_path
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-provider-readiness")) => {
            match crate::parse_cloud_iac_provider_readiness_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_provider_readiness_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-provider-readiness validation passed: {} modules, {} provider families checked; manifest {}; catalog {}; readiness {}",
                            report.modules_checked,
                            report.provider_families_checked,
                            report.manifest_path,
                            report.catalog_path,
                            report.readiness_path
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-provider-lockfile")) => {
            match crate::parse_cloud_iac_provider_lockfile_args(args.collect()) {
                Ok(parsed) => match crate::validate_cloud_iac_provider_lockfile_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "cloud-iac-provider-lockfile validation passed: {} providers, {} platforms checked; manifest {}; readiness {}; lock root {}",
                            report.providers_checked,
                            report.platforms_checked,
                            report.manifest_path,
                            report.readiness_path,
                            report.lock_root_path
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cloud-iac-provider-signature-review")) => {
            match crate::parse_cloud_iac_provider_signature_review_args(args.collect()) {
                Ok(parsed) => {
                    match crate::validate_cloud_iac_provider_signature_review_gate(parsed) {
                        Ok(report) => {
                            println!(
                                "cloud-iac-provider-signature-review validation passed: {} providers, {} signer keys, {} platforms checked; manifest {}; review {}; lock root {}",
                                report.providers_checked,
                                report.signer_keys_checked,
                                report.platforms_checked,
                                report.manifest_path,
                                report.review_path,
                                report.lock_root_path
                            );
                            ExitCode::SUCCESS
                        }
                        Err(message) => {
                            eprintln!("{message}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("a11y-discipline")) => {
            match crate::validate_a11y_discipline_gate(args.collect()) {
                Ok(summary) => {
                    println!(
                        "a11y discipline advisory: {} surfaces scanned; {} gaps",
                        summary.surfaces_scanned, summary.gaps
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("a11y discipline error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("i18n-coverage")) => {
            match crate::validate_i18n_coverage_gate(args.collect()) {
                Ok(summary) => {
                    println!(
                        "i18n coverage advisory: {} surfaces scanned; {} gaps",
                        summary.surfaces_scanned, summary.gaps
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("i18n coverage error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("compliance-evidence-coverage")) => {
            match crate::validate_compliance_evidence_coverage_gate(args.collect()) {
                Ok(summary) => {
                    println!(
                        "compliance evidence coverage advisory: {} µservices scanned; {} gaps",
                        summary.microservices_scanned, summary.gaps
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("compliance evidence coverage error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("realtime-transport-tier")) => {
            match crate::validate_realtime_transport_tier_gate(args.collect()) {
                Ok(summary) => {
                    println!(
                        "realtime transport tier advisory: {} declarations scanned; {} gaps",
                        summary.declarations_scanned, summary.gaps
                    );
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("realtime transport tier error: {message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("planes")) => {
            match crate::parse_planes_validate_args(args.collect()) {
                Ok(args) => match crate::validate_planes_gate(args) {
                    Ok(report) => {
                        println!(
                            "architecture plane validation passed: {} planes, {} lanes",
                            report.planes_checked, report.lanes_checked
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("architecture plane validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("wave-integration")) => {
            match crate::parse_wave_integration_validate_args(args.collect()) {
                Ok(args) => match crate::validate_wave_integration_gate(args) {
                    Ok(report) => {
                        println!(
                            "wave integration validation passed: {} phases, {} dependencies",
                            report.phases_checked, report.dependencies_checked
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("wave integration validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("master-plan-completion")) => {
            match master_plan_completion_audit::parse_master_plan_completion_audit_args(
                args.collect(),
            ) {
                Ok(parsed) => {
                    match master_plan_completion_audit::audit_master_plan_completion(parsed) {
                        Ok(report) => {
                            println!(
                                "master-plan-completion validation passed: {} phases, {} implementation plans",
                                report.phases_checked, report.implementation_plans_checked
                            );
                            ExitCode::SUCCESS
                        }
                        Err(message) => {
                            eprintln!("master-plan-completion validation failed:\n{message}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("board-masterplan-consistency")) => {
            match board_masterplan_consistency::parse_board_masterplan_consistency_args(
                args.collect(),
            ) {
                Ok(parsed) => {
                    match board_masterplan_consistency::validate_board_masterplan_consistency(
                        parsed,
                    ) {
                        Ok(report) => {
                            println!(
                                "board-masterplan-consistency validation passed: {} masterplan deliverables, {} board deliverables",
                                report.masterplan_deliverables_checked,
                                report.board_deliverables_checked
                            );
                            ExitCode::SUCCESS
                        }
                        Err(message) => {
                            eprintln!("board-masterplan-consistency validation failed:\n{message}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("product-index")) => product_index::run(args.collect()),
        (Some("validate"), Some("product-prd-json")) => {
            match product_prd_json::parse_product_prd_json_validate_args(args.collect()) {
                Ok(parsed) => match product_prd_json::validate_product_prd_json_gate(parsed) {
                    Ok(report) => {
                        println!(
                            "product-prd-json validation passed: {} products, {} acceptance criteria, {} test refs, {} metrics, {} verification refs, {} planned-feature refs, {} root-hub links, {} ms",
                            report.products_checked,
                            report.acceptance_criteria_checked,
                            report.test_refs_checked,
                            report.metrics_checked,
                            report.verification_refs_checked,
                            report.planned_feature_refs_checked,
                            report.root_hub_links_checked,
                            report.validation_duration_ms
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("product-prd-json validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("stage0-prereqs")) => {
            match stage0_application_shell_prereqs::parse_stage0_prereqs_validate_args(
                args.collect(),
            ) {
                Ok(parsed) => {
                    match stage0_application_shell_prereqs::validate_stage0_prereqs_gate(parsed) {
                        Ok(report) => {
                            println!(
                                "stage0-prereqs validation passed: {} required paths, workspace_member_present={}, edition={}, rust-version={}",
                                report.required_paths_checked,
                                report.workspace_member_present,
                                report.package_edition,
                                report.package_rust_version
                            );
                            ExitCode::SUCCESS
                        }
                        Err(message) => {
                            eprintln!("stage0-prereqs validation failed: {message}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("mobile-native")) => {
            match crate::parse_mobile_native_validate_args(args.collect()) {
                Ok(args) => match crate::validate_mobile_native_gate(args) {
                    Ok((wave, products, markers, quality_records)) => {
                        println!(
                            "mobile native validation passed: current_wave={wave}, {products} native products, {markers} native project markers, {quality_records} quality records"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("mobile native validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("runbook-index-resolves")) => {
            match crate::parse_runbook_index_validate_args(args.collect()) {
                Ok(args) => match crate::validate_runbook_index_gate(args) {
                    Ok(runbooks) => {
                        println!("runbook index validation passed: {runbooks} indexed runbooks");
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("runbook index validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("runbook-freshness")) => {
            match crate::parse_runbook_freshness_validate_args(args.collect()) {
                Ok(args) => match crate::validate_runbook_freshness_gate(args) {
                    Ok((runbooks, scoped, unscoped)) => {
                        println!(
                            "runbook freshness validation passed: {runbooks} runbooks, {scoped} severity-scoped, {unscoped} unscoped"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("runbook freshness validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("data-class")) => {
            match crate::parse_data_class_validate_args(args.collect()) {
                Ok(args) => match crate::validate_data_class_gate(args) {
                    Ok((fields_checked, annotated_fields, legacy_unannotated_fields)) => {
                        println!(
                            "data class fitness validation passed: {fields_checked} fields checked, {annotated_fields} annotated, {legacy_unannotated_fields} legacy unannotated"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("data class fitness validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("slo-coverage")) => {
            match crate::parse_slo_coverage_validate_args(args.collect()) {
                Ok(args) => match crate::validate_slo_coverage_gate(args) {
                    Ok(records) => {
                        println!("slo coverage validation passed: {records} records");
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("slo coverage validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("cohesion")) => {
            match crate::parse_cohesion_validate_args(args.collect()) {
                Ok(args) => match crate::validate_cohesion_gate(args) {
                    Ok((contracts, implemented_sources)) => {
                        println!(
                            "cohesion validation passed: {contracts} contracts, {implemented_sources} implemented sources"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("cohesion validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("codeowners-mirror")) => {
            match crate::parse_codeowners_mirror_validate_args(args.collect()) {
                Ok(args) => match crate::validate_codeowners_mirror_gate(args) {
                    Ok((entries, owners)) => {
                        println!(
                            "codeowners mirror validation passed: {entries} entries, {owners} owners"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("codeowners mirror validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("doc-catalog")) => {
            match crate::parse_doc_catalog_validate_args(args.collect()) {
                Ok(args) => match crate::validate_doc_catalog_gate(args) {
                    Ok(documents) => {
                        println!("doc catalog validation passed: {documents} documents");
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("doc catalog validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("documentation-system")) => {
            match crate::parse_documentation_system_validate_args(args.collect()) {
                Ok(args) => match crate::validate_documentation_system_gate(args) {
                    Ok(report) => {
                        println!(
                            "documentation system validation passed: {} pipeline records, {} active, {} adoption-guard, {} tracked-deferred",
                            report.pipeline_records_checked,
                            report.active_records,
                            report.adoption_guard_records,
                            report.tracked_deferred_records
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("documentation system validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("glossary-cross-doc-coverage")) => {
            match crate::parse_glossary_coverage_validate_args(args.collect()) {
                Ok(args) => match crate::validate_glossary_coverage_gate(args) {
                    Ok((terms, cross_doc_terms)) => {
                        println!(
                            "glossary cross-doc coverage validation passed: {terms} terms, {cross_doc_terms} cross-doc terms"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("glossary cross-doc coverage validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("glossary-vocabulary")) => {
            match crate::parse_glossary_vocabulary_validate_args(args.collect()) {
                Ok(args) => match crate::validate_glossary_vocabulary_gate(args) {
                    Ok((documents, casing_warnings, acronym_warnings)) => {
                        println!(
                            "glossary vocabulary validation passed: {documents} documents, {casing_warnings} casing warnings, {acronym_warnings} uncited acronym warnings"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("glossary vocabulary validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("placeholder-debt")) => {
            match crate::parse_placeholder_debt_validate_args(args.collect()) {
                Ok(args) => match crate::validate_placeholder_debt_gate(args) {
                    Ok((documents, open_placeholders, tracked_records)) => {
                        println!(
                            "placeholder debt validation passed: {documents} documents, {open_placeholders} open placeholders, {tracked_records} registry records"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("placeholder debt validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("quality-lanes")) => {
            match crate::parse_quality_lanes_validate_args(args.collect()) {
                Ok(args) => match crate::validate_quality_lanes_gate(args) {
                    Ok((records, markdown_rows, active_commands, owner_teams)) => {
                        println!(
                            "quality lane validation passed: {records} registry records, {markdown_rows} markdown rows, {active_commands} active commands, {owner_teams} owner teams"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("quality lane validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("honest-claims")) => {
            match crate::parse_honest_claims_validate_args(args.collect()) {
                Ok(args) => match crate::validate_honest_claims_gate(args) {
                    Ok(report) => {
                        println!(
                            "honest-claims validation passed: {} documents, {} lines, {} implementation plans, {} dependency edges, {} serialization edges, {} global artifact writes, {} legacy missing split-rule rows",
                            report.documents_checked,
                            report.lines_checked,
                            report.plans_checked,
                            report.dependency_edges,
                            report.serialization_edges,
                            report.global_artifact_writes,
                            report.legacy_missing_split_rule
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("honest-claims validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("aspirational-enforcement")) => {
            match crate::parse_aspirational_enforcement_validate_args(args.collect()) {
                Ok(args) => match crate::validate_aspirational_enforcement_gate(args) {
                    Ok(report) => {
                        println!(
                            "aspirational-enforcement validation passed: {} documents, {} lines, {} binding mentions, {} check crates, {} workflow contexts, {} quality lane contexts, {} required contexts",
                            report.documents_checked,
                            report.lines_checked,
                            report.binding_mentions,
                            report.known_crates,
                            report.workflow_contexts,
                            report.quality_lane_contexts,
                            report.branch_required_contexts
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("aspirational-enforcement validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("banned-primitives")) => {
            match crate::parse_banned_primitives_validate_args(args.collect()) {
                Ok(args) => match crate::validate_banned_primitives_gate(args) {
                    Ok(report) => {
                        println!(
                            "banned-primitives validation passed: {} files, {} sources, {} fences, {} command-log records, {} usages, {} documented exceptions",
                            report.files_scanned,
                            report.sources_checked,
                            report.fences_checked,
                            report.command_log_records_checked,
                            report.usages_checked,
                            report.documented_exceptions
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("banned-primitives validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("hyperscaler-arch-invariants")) => {
            match crate::parse_hyperscaler_arch_invariants_validate_args(args.collect()) {
                Ok(args) => match crate::validate_hyperscaler_arch_invariants_gate(args) {
                    Ok(report) => {
                        println!(
                            "hyperscaler architecture invariant validation passed: {} invariants, {} services, {} planned lanes",
                            report.invariant_count, report.product_count, report.planned_lane_count
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!(
                            "hyperscaler architecture invariant validation failed: {message}"
                        );
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("hyperscaler-maturity-claims")) => {
            match crate::parse_hyperscaler_maturity_claims_validate_args(args.collect()) {
                Ok(args) => match crate::validate_hyperscaler_maturity_claims_gate(args) {
                    Ok(report) => {
                        println!(
                            "hyperscaler maturity claim governance validation passed: {} gates, {} workflow-studio competitors, claim_status={}",
                            report.gate_count, report.competitor_count, report.claim_status
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!(
                            "hyperscaler maturity claim governance validation failed: {message}"
                        );
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("platform-substrate-defaults")) => {
            match crate::parse_platform_substrate_defaults_args(args.collect()) {
                Ok(args) => match crate::validate_platform_substrate_defaults_gate(args) {
                    Ok(report) => {
                        println!(
                            "platform-substrate-defaults validation passed: {} workload-specific substrate rows, {} universal-default fields checked; architecture {}",
                            report.workload_specific_substrates_checked,
                            report.universal_default_fields_checked,
                            report.architecture_path
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("design-spec-maturity-claims")) => {
            match crate::parse_design_spec_maturity_claims_validate_args(args.collect()) {
                Ok(args) => match crate::validate_design_spec_maturity_claims_gate(args) {
                    Ok(report) => {
                        let evidence = report
                            .evidence_path
                            .as_ref()
                            .map(|path| format!(", evidence={}", path.display()))
                            .unwrap_or_default();
                        println!(
                            "design/spec maturity claim validation passed: {} services, {} surfaces, missing_count={}, design_claim_status={}, operational_claim_status={}, allowed_claim={:?}{}",
                            report.service_count,
                            report.surface_count,
                            report.missing_count,
                            report.design_claim_status,
                            report.operational_claim_status,
                            report.allowed_design_claim,
                            evidence
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("design/spec maturity claim validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("planning-ssot-coverage")) => {
            crate::planning_ssot_coverage_gate::run_planning_ssot_coverage(args.collect())
        }
        // ADR-0364 D2: completeness gate over planning_impact ADRs. FAILs ADRs
        // that declare deliverables but leave them/milestone incomplete; ADRs
        // without a deliverables field are advisory (backfill deferred to D7).
        (Some("validate"), Some("adr-planning-completeness")) => {
            crate::adr_planning_completeness_gate::run_adr_planning_completeness(args.collect())
        }
        // #6b: ADR supersession back-link integrity. FAILs if any
        // supersedes/superseded_by pair is one-directional (X supersedes Y but
        // Y does not back-link X, or vice versa). Pure link-reciprocity check
        // over ADR<->ADR edges (ADR-0083 Tier-3 panic-free).
        (Some("validate"), Some("adr-supersession-consistency")) => {
            crate::adr_supersession_consistency_gate::run_adr_supersession_consistency(
                args.collect(),
            )
        }
        // ADR-0364 D4: masterplan drift gate. Wraps `gen masterplan --check`:
        // the committed projection must equal the regenerated projection.
        (Some("validate"), Some("masterplan-drift")) => {
            crate::masterplan_drift_gate::run_masterplan_drift(args.collect())
        }
        (Some("validate"), Some("canonical-base-neutrality")) => {
            match crate::parse_canonical_base_neutrality_validate_args(args.collect()) {
                Ok(args) => match crate::validate_canonical_base_neutrality_gate(args) {
                    Ok(report) => {
                        let suffix = if report.self_test { " (self-test)" } else { "" };
                        println!(
                            "canonical-base-neutrality validation passed: {} files checked, 0 jurisdiction leaks{}",
                            report.files_checked, suffix
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("canonical-base-neutrality validation failed:\n{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("workspace-hygiene")) => {
            match crate::parse_workspace_hygiene_validate_args(args.collect()) {
                Ok(args) => match crate::validate_workspace_hygiene_gate(args) {
                    Ok(report) => {
                        println!(
                            "workspace hygiene validation passed: {} surfaces, {} roots scanned, {} findings, strict={}, cleaned={}",
                            report.surfaces_checked,
                            report.roots_scanned,
                            report.findings,
                            report.strict,
                            report.cleaned
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("workspace hygiene validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // loop-recovery-patterns: Rust-owned autonomous-Foundry harness lane.
        // Validates deterministic score_cards, concrete score-card inventory,
        // and registry/loop-recovery-patterns records linked to the
        // mistakes-ledger before `oya verify`/pre-push can pass.
        (Some("validate"), Some("loop-recovery-patterns")) => {
            match crate::parse_loop_recovery_patterns_validate_args(args.collect()) {
                Ok(args) => match crate::validate_loop_recovery_patterns_gate(args) {
                    Ok(report) => {
                        println!(
                            "loop-recovery-patterns validation passed: {} score-schema fields, {} score cards, {} score-card commands, {} patterns, {} active blockers, {} mistakes refs, {} anomaly signals",
                            report.score_card_schema_fields_checked,
                            report.score_cards_checked,
                            report.score_card_commands_executed,
                            report.patterns_checked,
                            report.active_blockers_checked,
                            report.mistakes_ledger_refs_checked,
                            report.anomaly_watch_signals_checked
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("loop-recovery-patterns validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        (Some("validate"), Some("raci-team-coverage")) => {
            match crate::parse_raci_team_coverage_validate_args(args.collect()) {
                Ok(args) => match crate::validate_raci_team_coverage_gate(args) {
                    Ok(teams) => {
                        println!("raci team coverage validation passed: {teams} teams");
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("raci team coverage validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // deployment-ops-contract: makes OpenTofu + root Makefile + ops portal
        // the only normal deployment surface, and tracks every infra/onprem
        // shell script for Rust migration.
        (Some("validate"), Some("deployment-ops-contract")) => {
            deployment_ops_contract::run(args.collect())
        }
        // milestone-audit: machine-readable replacement for ad-hoc shell/Markdown
        // milestone readiness reviews.
        (Some("validate"), Some("milestone-audit")) => milestone_audit::run(args.collect()),
        // `gate validate architecture-boundaries` — Wave 2 B-2 replacement
        // for scripts/check-architecture-boundaries.sh.
        (Some("validate"), Some("architecture-boundaries")) => {
            architecture_boundaries::run(args.collect())
        }
        (Some("validate"), Some("readme-doc-coverage")) => {
            match crate::parse_readme_doc_coverage_validate_args(args.collect()) {
                Ok(args) => match crate::validate_readme_doc_coverage_gate(args) {
                    Ok(documents) => {
                        println!("readme doc coverage validation passed: {documents} documents");
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("readme doc coverage validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // retired-vocabulary: enforces zero drift back to retired CLI
        // surfaces, retired crates, and retired script paths. The
        // registry at `registry/vocabulary/retired.yaml` is the
        // machine-readable record of every retirement decision; the
        // lane fails fast on any document that still mentions a
        // retired term. Lane id:
        // `oya-governance-retired-vocabulary`. Kernel:
        // `oya-check-retired-vocabulary` (port-in-kernel, ADR-0056).
        (Some("validate"), Some("retired-vocabulary")) => {
            match crate::parse_retired_vocabulary_validate_args(args.collect()) {
                Ok(args) => match crate::validate_retired_vocabulary_gate(args) {
                    Ok(report) => {
                        println!(
                            "retired-vocabulary validation passed: {} documents checked, \
                             {} retired terms enforced, 0 drift hits",
                            report.documents_checked, report.terms_checked
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("retired-vocabulary validation failed:\n{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // protection-context-match: enforces that every required
        // status-check context in `.github/branch-protection.yaml`
        // is the `name:` field of some workflow job in
        // `.github/workflows/*.yml`; when the optional live-contexts
        // JSON is supplied, also verifies live branch protection
        // requires exactly the same contexts. Catches both
        // silent-bypass classes: local config points at a workflow
        // job that does not exist, or GitHub live enforcement drifts
        // behind the canonical repo policy. Lane id:
        // `oya-governance-protection-context-match`. Kernel:
        // `oya-check-protection-context-match` (port-in-kernel,
        // ADR-0056).
        (Some("validate"), Some("protection-context-match")) => {
            match crate::parse_protection_context_match_validate_args(args.collect()) {
                Ok(args) => match crate::validate_protection_context_match_gate(args) {
                    Ok(report) => {
                        println!(
                            "protection-context-match validation passed: {} required contexts, \
                             {} workflow jobs indexed across {} workflows",
                            report.contexts_checked,
                            report.workflow_jobs_indexed,
                            report.workflows_indexed
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("protection-context-match validation failed:\n{message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // pre-push-contract: enforces the canonical `oya verify`
        // local-developer entry point is wired consistently across
        // Done-Definition, dev-CLI dispatch source, and the local
        // pre-push git hook. Lane id: `oya-governance-pre-push`.
        // Kernel: `oya-check-pre-push` (port-in-kernel, ADR-0056).
        (Some("validate"), Some("pre-push-contract")) => {
            match crate::parse_pre_push_contract_validate_args(args.collect()) {
                Ok(args) => match crate::validate_pre_push_contract_gate(args) {
                    Ok(report) => {
                        println!(
                            "pre-push-contract validation passed: command={}, \
                             native-verify-dispatch-token={}, verify-subcommand=wired, hook=wired",
                            report.canonical_command, report.native_verify_dispatch_token
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("pre-push-contract validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // PR #143 Fix-D — high-risk auto-decision refusal grounding.
        (Some("validate"), Some("high-risk-auto-decision-refusal")) => {
            crate::adr_0145_gates::run_high_risk_auto_decision_refusal(args.collect())
        }
        // PR #143 Fix-D — SLSA L3 evidence-grounded check.
        (Some("validate"), Some("slsa-l3-evidence-grounded")) => {
            crate::adr_0145_gates::run_slsa_l3_evidence_grounded(args.collect())
        }
        // ADR-0145 Invariant 2 — OTel trace propagation (DEFERRED/advisory).
        (Some("validate"), Some("otel-trace-propagation")) => {
            crate::adr_0145_gates::run_otel_trace_propagation(args.collect())
        }
        // ADR-0145 Invariant 3 — ontology projection coverage (strict).
        (Some("validate"), Some("ontology-projection-coverage")) => {
            crate::adr_0145_gates::run_ontology_projection_coverage(args.collect())
        }
        // ADR-0340 / CAPACITY-001 — per-µservice capacity_model manifest contract.
        (Some("validate"), Some("capacity-model-manifest")) => {
            crate::capacity_model_manifest_gate::run_capacity_model_manifest(args.collect(), usage)
        }
        // ADR-0145 Invariant 1 — audit-chain seal coverage (DEFERRED/advisory).
        (Some("validate"), Some("audit-chain-seal-coverage")) => {
            crate::adr_0145_gates::run_audit_chain_seal_coverage(args.collect())
        }
        // Tier-A hyperscaler pattern remediation gates (Fix-Agent-I,
        // 2026-05-18). Each is strict-mode (fail-closed).
        // ADR-0149 — Stripe/AWS Idempotency-Key header.
        (Some("validate"), Some("idempotency-key-coverage")) => {
            crate::tier_a_gates::run_idempotency_key_coverage(args.collect())
        }
        // ADR-0150 — AWS NextToken / Stripe cursor pagination.
        (Some("validate"), Some("cursor-pagination-coverage")) => {
            crate::tier_a_gates::run_cursor_pagination_coverage(args.collect())
        }
        // ADR-0152 — AWS Well-Architected RPO/RTO five-tier model.
        (Some("validate"), Some("rpo-rto-coverage")) => {
            crate::tier_a_gates::run_rpo_rto_coverage(args.collect())
        }
        // ADR-0151 — high-cardinality metric label discipline.
        (Some("validate"), Some("metric-cardinality")) => {
            crate::tier_a_gates::run_metric_cardinality(args.collect())
        }
        // ADR-0154 — AsyncAPI 3.1.0 event version field.
        (Some("validate"), Some("event-schema-versioning")) => {
            crate::tier_a_gates::run_event_schema_versioning(args.collect())
        }
        // ADR-0156 — canonical ULID id discipline.
        (Some("validate"), Some("id-discipline")) => {
            crate::tier_a_gates::run_id_discipline(args.collect())
        }
        // ADR-0146 + ADR-0039 — cosign + Trivy + SLSA L3 provenance.
        (Some("validate"), Some("image-signing-discipline")) => {
            crate::tier_a_gates::run_image_signing_discipline(args.collect())
        }
        // ADR-0148 / ADR-0182 / ADR-0183 / ADR-0184 — layered architecture
        // discipline (Cilium L3/L4 + Istio Ambient L7 zero overlap; gateway
        // vs mesh; Cedar vs Kyverno; Valkey vs Memcached).
        (Some("validate"), Some("layered-architecture-discipline")) => {
            crate::layered_architecture_gates::run_layered_architecture_discipline(args.collect())
        }
        // ADR-0185 — native-per-platform client stack discipline.
        (Some("validate"), Some("client-stack-discipline")) => {
            crate::layered_architecture_gates::run_client_stack_discipline(args.collect())
        }
        // ADR-0110 — changeset state machine: monotonicity invariant.
        // Asserts every changeset's event-log state sequence is a non-decreasing
        // subsequence of the 9-state advancing order. A backwards move or any
        // transition after a terminal-fail state is a fatal violation.
        (Some("validate"), Some("changeset-state-monotonicity")) => {
            match crate::changeset_state_gates::parse_changeset_state_monotonicity_args(
                args.collect(),
            ) {
                Ok(args) => {
                    match crate::changeset_state_gates::validate_changeset_state_monotonicity(args)
                    {
                        Ok(report) => {
                            if report.violations.is_empty() {
                                println!(
                                    "changeset-state-monotonicity validation passed: \
                                     {} events, {} changesets, 0 violations",
                                    report.events_checked, report.changesets_checked
                                );
                                ExitCode::SUCCESS
                            } else {
                                for v in &report.violations {
                                    eprintln!("changeset-state-monotonicity violation: {v}");
                                }
                                eprintln!(
                                    "changeset-state-monotonicity validation failed: \
                                     {} violation(s)",
                                    report.violations.len()
                                );
                                ExitCode::FAILURE
                            }
                        }
                        Err(message) => {
                            eprintln!("changeset-state-monotonicity validation failed: {message}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // ADR-0110 — changeset state machine: closed enum invariant.
        // Asserts every `to_state` in the event log is a member of the 12-value
        // closed status enum (9 advancing + 3 terminal-fail). Any unrecognised
        // state string is a fatal violation.
        (Some("validate"), Some("changeset-state-enum-closed")) => {
            match crate::changeset_state_gates::parse_changeset_state_enum_closed_args(
                args.collect(),
            ) {
                Ok(args) => {
                    match crate::changeset_state_gates::validate_changeset_state_enum_closed(args) {
                        Ok(report) => {
                            if report.violations.is_empty() {
                                println!(
                                    "changeset-state-enum-closed validation passed: \
                                     {} events, 0 violations",
                                    report.events_checked
                                );
                                ExitCode::SUCCESS
                            } else {
                                for v in &report.violations {
                                    eprintln!("changeset-state-enum-closed violation: {v}");
                                }
                                eprintln!(
                                    "changeset-state-enum-closed validation failed: \
                                     {} violation(s)",
                                    report.violations.len()
                                );
                                ExitCode::FAILURE
                            }
                        }
                        Err(message) => {
                            eprintln!("changeset-state-enum-closed validation failed: {message}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // ADR-0388: doc-axis convention enforcement. Validates ADR status
        // casing, shadow ideas older than 14 days, docs/ proliferation, and
        // catalog/manifest crate-claim consistency. Status-casing findings
        // are warnings (not errors) in default mode; pass --strict to
        // promote them to blocking errors.
        (Some("validate"), Some("doc-axis")) => {
            let rest: Vec<String> = args.collect();
            let strict = rest.iter().any(|a| a == "--strict");
            let repo_root = std::path::Path::new(".");
            match oya_check_doc_axis::validate(repo_root, strict) {
                Ok(report) => {
                    if report.warnings > 0 {
                        println!(
                            "doc-axis validation passed with {} warning(s): {} ADRs, {} ideas, {} docs entries, {} manifests checked",
                            report.warnings,
                            report.adrs_checked,
                            report.ideas_checked,
                            report.docs_files_checked,
                            report.manifests_checked,
                        );
                    } else {
                        println!(
                            "doc-axis validation passed: {} ADRs, {} ideas, {} docs entries, {} manifests checked",
                            report.adrs_checked,
                            report.ideas_checked,
                            report.docs_files_checked,
                            report.manifests_checked,
                        );
                    }
                    ExitCode::SUCCESS
                }
                Err(findings) => {
                    for f in &findings {
                        eprintln!(
                            "doc-axis violation [{}:{}]: {:?} — {}",
                            f.path,
                            f.line
                                .map(|l| l.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                            f.rule_violated,
                            f.suggested_fix,
                        );
                    }
                    eprintln!(
                        "doc-axis validation failed: {} blocking finding(s)",
                        findings.len()
                    );
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}
