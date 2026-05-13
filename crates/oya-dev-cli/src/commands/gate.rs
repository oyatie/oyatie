use std::process::ExitCode;

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut args = args.into_iter();
    match (args.next().as_deref(), args.next().as_deref()) {
        // Grounding note for exception-ledger audits:
        // "foundation-bypass" is Oyatie's documented engineering-platform
        // domain term for tracked, expirable gate exceptions, not a code
        // recovery path. This command is fail-closed: malformed, duplicate,
        // missing-ledger, zero-window, or expired records return FAILURE.
        // An explicitly present ledger with zero records means no exception
        // exists.
        // Tested by oya-tooling-cli-dev-runtime::gate_cli and
        // oya-foundry-bypass-kernel::foundation_bypass.
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
                            "ADR citation validation passed: {documents} documents, {citations} citations, {allowed_pack_adrs} pack ADRs"
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
        (Some("validate"), Some("constitution-cite-coverage")) => {
            match crate::parse_constitution_cite_validate_args(args.collect()) {
                Ok(args) => match crate::validate_constitution_cite_gate(args) {
                    Ok(documents) => {
                        println!(
                            "constitution cite coverage validation passed: {documents} documents"
                        );
                        ExitCode::SUCCESS
                    }
                    Err(message) => {
                        eprintln!("constitution cite coverage validation failed: {message}");
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
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}
