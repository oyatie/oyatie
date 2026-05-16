use std::process::ExitCode;

mod architecture_boundaries;
mod deployment_ops_contract;
mod milestone_audit;
mod product_index;
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
        // oya-foundry-bypass-domain::foundation_bypass.
        // active-artifact-contract: ADR-0069 v3.0.0 vertical enforcement loop.
        // Validates that every row in registry/artifact-capabilities-registry.json
        // (a) has its `artifact_path` HEAD-tracked per `git ls-files`, and
        // (b) has a unique `artifact_id`.
        // Optionally emits an evidence bundle and one graph-edge artifact per
        // full-consensus-planner-v3 amendments #4/#9/#10.
        (Some("validate"), Some("active-artifact-contract")) => {
            match crate::parse_active_artifact_contract_validate_args(args.collect()) {
                Ok(args) => match crate::validate_active_artifact_contract_gate(args) {
                    Ok(report) => {
                        println!(
                            "active-artifact-contract validation passed: {} rows, {} HEAD-tracked, {} graph edges, {} ms",
                            report.rows_seen,
                            report.head_tracked_count,
                            report.graph_edges.len(),
                            report.validation_duration_ms
                        );
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
                    Ok((capabilities, mcp_contracts, schemas)) => {
                        println!(
                            "foundry capability schema validation passed: {capabilities} capabilities, {mcp_contracts} mcp contracts, {schemas} schemas"
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
        (Some("validate"), Some("product-index")) => product_index::run(args.collect()),
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
        // `oya-foundry-fitness-retired-vocabulary`. Kernel:
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
        // `.github/workflows/*.yml`. Catches the silent-bypass class
        // where the protection lists a context name that no
        // workflow posts; GitHub then waits forever for a check_run
        // that never arrives. Lane id:
        // `oya-foundry-fitness-protection-context-match`. Kernel:
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
        // pre-push git hook. Lane id: `oya-foundry-fitness-pre-push`.
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
        // changeset-state-monotonicity: ADR-0110 wave-A IP-001. Asserts every
        // row in registry/vcs/changeset-event-log.json obeys the
        // non-decreasing-subsequence invariant on the advancing-state
        // axis, has a unique dedup_key, and terminates at most once.
        // Lane: oya-foundry-fitness-changeset-state-monotonicity.
        (Some("validate"), Some("changeset-state-monotonicity")) => {
            match crate::parse_changeset_state_monotonicity_validate_args(args.collect()) {
                Ok(args) => match crate::validate_changeset_state_monotonicity_gate(args) {
                    Ok(report) => {
                        println!(
                            "changeset-state-monotonicity validation passed: {} events checked, terminal={}",
                            report.events_checked,
                            report
                                .terminal_state
                                .map(|s| s.as_wire().to_string())
                                .unwrap_or_else(|| "none".to_string())
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("changeset-state-monotonicity validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        // changeset-state-enum-closed: ADR-0110 wave-A IP-001. Asserts every
        // emitted `to_state` (and `from_state`, when present) is in the
        // closed 13-value ChangesetState enum. Distinct lane from the
        // monotonicity check so a typo surfaces with the right name.
        // Lane: oya-foundry-fitness-changeset-state-enum-closed.
        (Some("validate"), Some("changeset-state-enum-closed")) => {
            match crate::parse_changeset_state_enum_closed_validate_args(args.collect()) {
                Ok(args) => match crate::validate_changeset_state_enum_closed_gate(args) {
                    Ok(report) => {
                        println!(
                            "changeset-state-enum-closed validation passed: {} events checked, {} distinct states observed",
                            report.events_checked, report.distinct_states_seen
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("changeset-state-enum-closed validation failed: {message}");
                        ExitCode::FAILURE
                    }
                },
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}
