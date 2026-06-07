---
doc_class: LocalizationPack
pack_id: EU-PACK-1
version: "1.0.0"
status: Draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0248
  - ADR-0251
  - ADR-0263
  - ADR-0304
  - ADR-0316
citing_authority_url:
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R2554
  - https://eur-lex.europa.eu/eli/reg_del/2024/1772/oj
  - https://eur-lex.europa.eu/eli/reg_del/2024/1773/oj/eng/pdf
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1774
  - https://eur-lex.europa.eu/eli/reg/2024/2956/oj
  - https://eur-lex.europa.eu/eli/reg_del/2025/301/oj/eng
  - https://eur-lex.europa.eu/legal-content/en/TXT/?uri=CELEX:32025R0302
---

# EU-PACK-1 DORA Operational Resilience

## Purpose

This document defines the EU-PACK-1 financial-sector operational resilience layer.
It applies when an Oyatie tenant is a financial entity in DORA scope.
It also applies when an Oyatie tenant supports an in-scope financial entity as an ICT service provider.
It does not turn a non-financial tenant into a financial entity.
It does not provide a legal opinion on whether a tenant is in DORA scope.
It provides the platform controls needed once the tenant's sector profile says DORA applies.
It makes ICT risk ownership explicit.
It makes third-party ICT dependencies inventoryable.
It makes major ICT-related incident reporting evidence collectable.
It makes threat-led penetration testing supportable without weakening tenant isolation.
It binds DORA evidence to ADR-0263 audit events.
It binds DORA policy decisions to Cedar gates.
It binds DORA register-of-information data to typed pack deltas.
It keeps DORA operational data separate from generic security telemetry.
It keeps competent-authority reporting workflows tenant-owned.
It keeps Oyatie-generated evidence immutable after submission freeze.
It keeps every DORA assertion traceable to a control owner, timestamp, and source event.

## Scope

EU-PACK-1 activates this document only for financial-sector tenant profiles.
Financial-sector tenant profiles include credit institutions.
Financial-sector tenant profiles include payment institutions.
Financial-sector tenant profiles include electronic-money institutions.
Financial-sector tenant profiles include investment firms.
Financial-sector tenant profiles include trading venues.
Financial-sector tenant profiles include central securities depositories.
Financial-sector tenant profiles include central counterparties.
Financial-sector tenant profiles include insurance and reinsurance undertakings.
Financial-sector tenant profiles include insurance intermediaries when the tenant profile selects DORA.
Financial-sector tenant profiles include crypto-asset service provider profiles where Union law places them in DORA scope.
Financial-sector tenant profiles include ICT third-party provider profiles that serve in-scope entities.
The pack does not assume every fintech customer is in scope.
The pack requires the tenant to declare `dora_applicability_status`.
The pack requires the tenant to declare competent authority routing metadata.
The pack requires the tenant to declare whether simplified ICT risk management is claimed.
The pack requires the tenant to declare critical or important functions.
The pack requires the tenant to declare ICT third-party service providers.
The pack requires the tenant to declare outsourcing chain depth for critical or important functions.
The pack requires the tenant to declare incident reporting contacts.
The pack requires the tenant to declare TLPT eligibility and testing authority context.
The pack blocks production DORA mode if the sector profile is unknown.
The pack blocks production DORA mode if competent authority routing is absent.
The pack blocks production DORA mode if no accountable management-body owner is recorded.
The pack blocks production DORA mode if ICT asset identification is stale.
The pack blocks production DORA mode if critical-function mapping is incomplete.
The pack blocks production DORA mode if incident classification thresholds are not configured.
The pack blocks production DORA mode if the register-of-information export is not testable.
The pack blocks production DORA mode if third-party concentration risk has no owner.
The pack blocks production DORA mode if TLPT artifacts are stored in general logs.
The pack allows read-only assessment mode before full DORA activation.
The pack allows sandbox rehearsals of incident reports.
The pack allows dry-run register exports.
The pack allows internal audit review before competent-authority submission.

## Authority Citations

| Authority | Official URL | Pack use |
|---|---|---|
| DORA Regulation (EU) 2022/2554 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R2554 | Primary operational-resilience authority for the financial sector. |
| DORA incident classification RTS, Commission Delegated Regulation (EU) 2024/1772 | https://eur-lex.europa.eu/eli/reg_del/2024/1772/oj | Classification criteria and materiality thresholds for ICT incidents and cyber threats. |
| DORA ICT third-party contractual policy RTS, Commission Delegated Regulation (EU) 2024/1773 | https://eur-lex.europa.eu/eli/reg_del/2024/1773/oj/eng/pdf | Policy content for contractual arrangements supporting critical or important functions. |
| DORA ICT risk management RTS, Commission Delegated Regulation (EU) 2024/1774 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1774 | ICT risk management tools, methods, processes, policies, and simplified framework detail. |
| Register of information ITS, Commission Implementing Regulation (EU) 2024/2956 | https://eur-lex.europa.eu/eli/reg/2024/2956/oj | Standard templates for ICT third-party service provider contractual arrangements. |
| Incident reporting RTS, Commission Delegated Regulation (EU) 2025/301 | https://eur-lex.europa.eu/eli/reg_del/2025/301/oj/eng | Content and time limits for initial, intermediate, and final major incident reports. |
| Incident reporting ITS, Commission Implementing Regulation (EU) 2025/302 | https://eur-lex.europa.eu/legal-content/en/TXT/?uri=CELEX:32025R0302 | Standard forms, templates, and procedures for major ICT incident reports and significant cyber threat notifications. |

## DORA Article Binding

| DORA article | Operational obligation | EU-PACK-1 binding |
|---|---|---|
| Article 5 | Governance and organisation. | Management-body accountability is a required tenant profile field. |
| Article 6 | ICT risk management framework. | ICT control framework records must exist before DORA production activation. |
| Article 8 | Identification. | ICT assets, functions, data stores, third parties, and dependencies must be inventoryable. |
| Article 17 | ICT-related incident management process. | Incident intake, classification, escalation, and evidence freeze workflows must be active. |
| Article 19 | Reporting of major ICT-related incidents. | Report package construction must preserve initial, intermediate, and final report state. |
| Article 20 | Harmonised reporting content and templates. | API exports must support template-backed report generation. |
| Article 24 | Digital operational resilience testing. | Test evidence must be linked to critical functions and remediation owners. |
| Article 26 | Advanced testing with TLPT. | TLPT artifacts must use restricted evidence vaults and approved tester scopes. |
| Article 28 | ICT third-party risk management. | Register-of-information source data must cover all ICT contractual arrangements. |
| Article 31 | Critical ICT third-party provider designation. | Provider criticality metadata must not be collapsed into procurement labels. |

## Financial-Sector Activation Model

Activation key: `eu_pack.dora.enabled`.
Activation mode: `assessment`, `production`, `incident_drill`, or `tlpt_window`.
Default mode: disabled unless tenant sector metadata activates it.
Assessment mode permits inventory gathering.
Assessment mode permits control-gap scoring.
Assessment mode permits report template rehearsal.
Assessment mode does not permit competent-authority submission claims.
Production mode requires accountable owner evidence.
Production mode requires control framework evidence.
Production mode requires third-party register readiness.
Production mode requires incident reporting contact readiness.
Production mode requires retention rules for report evidence.
Incident drill mode permits simulated major-incident clocks.
Incident drill mode writes drill-marked audit events.
Incident drill mode must never overwrite production incident records.
TLPT window mode opens bounded evidence vaults.
TLPT window mode requires explicit start and end timestamps.
TLPT window mode requires approved target scope.
TLPT window mode requires separation between testers and production operators.
TLPT window mode requires post-test closure evidence.

## ICT Risk Governance

The management body owns the DORA governance decision record.
The management body record must name an accountable executive.
The accountable executive record must include role, delegation boundary, and review cadence.
The tenant must record ICT risk tolerance.
The tenant must record how ICT risk tolerance maps to customer-facing service objectives.
The tenant must record critical or important functions.
The tenant must record owner teams for critical or important functions.
The tenant must record continuity requirements for critical or important functions.
The tenant must record recovery-time and recovery-point expectations.
The tenant must record upstream dependencies for critical or important functions.
The tenant must record downstream dependencies for critical or important functions.
The tenant must record evidence for risk acceptance.
Risk acceptance cannot be indefinite.
Risk acceptance cannot be anonymous.
Risk acceptance cannot cover missing incident reporting contacts.
Risk acceptance cannot cover unknown critical function ownership.
Risk acceptance cannot cover unclassified third-party ICT arrangements.
Risk acceptance must expire.
Risk acceptance must be revalidated after material architecture change.
Risk acceptance must be revalidated after a major incident.
Risk acceptance must be revalidated after a third-party provider criticality change.
Risk acceptance must be revalidated after supervisory finding import.

## ICT Asset Identification

Article 8 identification starts from the service graph.
The service graph must identify runtime cells.
The service graph must identify data stores.
The service graph must identify message buses.
The service graph must identify identity providers.
The service graph must identify cryptographic key stores.
The service graph must identify logging and monitoring systems.
The service graph must identify backup systems.
The service graph must identify recovery environments.
The service graph must identify external APIs.
The service graph must identify operator access paths.
The service graph must identify deployment pipelines.
The service graph must identify firmware or device dependencies when relevant.
The service graph must identify subcontracted ICT services.
Every ICT asset must have an owner.
Every ICT asset must have a criticality rating.
Every ICT asset must have a data-classification label.
Every ICT asset must have a dependency direction.
Every ICT asset must have a jurisdiction profile.
Every ICT asset must have a last-reviewed timestamp.
Every ICT asset supporting a critical or important function must have continuity metadata.
Every ICT asset supporting a critical or important function must have restoration evidence.
Every ICT asset supporting a critical or important function must have monitoring coverage.
Every ICT asset supporting a critical or important function must have incident routing.
Unknown ICT assets block production DORA mode.
Unowned ICT assets block production DORA mode.
Unmapped critical functions block production DORA mode.
Inventory older than the tenant-defined stale threshold blocks release.
Inventory older than the pack maximum stale threshold blocks release even if tenant policy is looser.

## ICT Incident Management

Incident intake must distinguish ICT incidents from generic business incidents.
Incident intake must distinguish cybersecurity events from operational availability events.
Incident intake must distinguish service degradation from total outage.
Incident intake must distinguish customer-impacting incidents from internal-only incidents.
Incident intake must record detection time.
Incident intake must record awareness time.
Incident intake must record classification time.
Incident intake must record who classified the incident.
Incident intake must record affected critical or important functions.
Incident intake must record affected clients, counterparties, or users when known.
Incident intake must record affected data categories.
Incident intake must record affected geographic scope.
Incident intake must record third-party provider involvement.
Incident intake must record whether a cyber threat notification is voluntary.
Incident intake must record whether competent-authority notification is required.
Incident classification must apply DORA materiality criteria from the active threshold profile.
Incident classification must preserve threshold evidence.
Incident classification must preserve unknown values.
Incident classification must not downgrade major status without reason evidence.
Incident classification must not ignore third-party outage evidence.
Incident classification must not delay the report clock while waiting for perfect root cause.
Incident escalation must notify the DORA incident owner.
Incident escalation must notify legal or compliance contacts configured by the tenant.
Incident escalation must notify communications contacts when customer impact is likely.
Incident escalation must notify data-protection contacts when personal data may be involved.
Incident escalation must notify third-party risk contacts when provider service is implicated.
Incident escalation must preserve handoff timestamps.
Incident report packages must be immutable after each report submission state.
Initial report packages must capture known facts and unknowns.
Intermediate report packages must capture updated impact, remediation, and continuity status.
Final report packages must capture root cause, corrective measures, and lessons learned.
Report packages must preserve template version.
Report packages must preserve competent-authority endpoint metadata.
Report packages must preserve approver identity.
Report packages must preserve export hash.
Report packages must preserve submission timestamp when supplied by the tenant.
Report packages must preserve evidence links but not duplicate sensitive payloads into low-trust logs.

## Major Incident Reporting Clock

The pack records four different clock values.
Clock one is event start time.
Clock two is detection time.
Clock three is awareness time.
Clock four is major classification time.
The initial report timer uses the regulation and RTS profile selected by the tenant.
The implementation default records whether initial notification was prepared within four hours of major classification.
The implementation default records whether initial notification was prepared within twenty-four hours of awareness.
The intermediate report timer records the active regulatory due time.
The final report timer records the active regulatory due time.
Timer evidence is advisory until the tenant submits through its competent-authority route.
Timer evidence must never be edited after freeze.
Timer evidence must include timezone and clock source.
Timer evidence must flag clock-source disagreement.
Timer evidence must flag missing awareness time.
Timer evidence must flag missing classification time.
Timer evidence must flag after-the-fact classification edits.
Timer evidence must flag report package mutation after approval.

## Third-Party Register of Information

The DORA register is not a procurement convenience table.
The DORA register is a regulatory dataset for ICT contractual arrangements.
The DORA register must include direct ICT third-party providers.
The DORA register must include subcontractor chains where required.
The DORA register must link providers to contractual arrangements.
The DORA register must link contractual arrangements to ICT services.
The DORA register must link ICT services to supported functions.
The DORA register must identify critical or important function support.
The DORA register must identify provider legal name.
The DORA register must identify provider country.
The DORA register must identify provider identifier where available.
The DORA register must identify arrangement reference numbers.
The DORA register must identify service start dates.
The DORA register must identify service end dates where applicable.
The DORA register must identify notice periods where applicable.
The DORA register must identify substitutability.
The DORA register must identify exit strategy status.
The DORA register must identify audit and access rights.
The DORA register must identify data location.
The DORA register must identify processing location.
The DORA register must identify concentration risk flags.
The DORA register must identify chain rank for subcontractors.
The DORA register must identify whether the provider supports a critical or important function.
The DORA register must support entity level, sub-consolidated level, and consolidated level views when the tenant config requires them.
The DORA register export must preserve schema version.
The DORA register export must preserve validation errors.
The DORA register export must preserve source record hashes.
The DORA register export must preserve approval state.
The DORA register export must not expose unrelated tenant records.
The DORA register export must not collapse EU and non-EU service locations.
The DORA register export must not treat generic vendor risk records as sufficient.
The DORA register export must not omit internal group ICT service providers when the tenant marks them reportable.

## Threat-Led Penetration Testing

TLPT mode is a controlled resilience-testing state.
TLPT mode is not a blanket exception to security policy.
TLPT mode is not a reason to weaken tenant isolation.
TLPT mode is not a reason to suppress ADR-0263 audit emission.
TLPT mode requires an approved test authority contact.
TLPT mode requires scope boundaries.
TLPT mode requires start and end dates.
TLPT mode requires target function identifiers.
TLPT mode requires rules of engagement.
TLPT mode requires test-team identity evidence.
TLPT mode requires white-team contact evidence.
TLPT mode requires incident handling carve-out rules.
TLPT mode requires safe-word or stop condition records.
TLPT mode requires data-handling constraints.
TLPT mode requires evidence vault segregation.
TLPT mode requires post-test remediation owner mapping.
TLPT mode requires finding severity mapping.
TLPT mode requires retest requirements for critical findings.
TLPT mode requires closure approval.
TLPT mode emits test-specific audit events.
TLPT mode never writes exploit detail into general-purpose analytics.
TLPT mode never grants unrestricted production access.
TLPT mode never permits cross-tenant testing.
TLPT mode never permits persistence outside approved target scope.
TLPT mode never lets a tester self-approve closure.
TLPT mode must be revoked automatically at the scheduled end time.
TLPT mode must be revoked manually if the stop condition fires.
TLPT mode must be reviewed after any unexpected customer impact.

## Activated Cedar Policies

Policy family: `eu.dora.activation`.
Policy family: `eu.dora.ict_risk`.
Policy family: `eu.dora.asset_identification`.
Policy family: `eu.dora.incident_reporting`.
Policy family: `eu.dora.third_party_register`.
Policy family: `eu.dora.tlpt`.
Policy family: `eu.dora.evidence_freeze`.
Policy `eu.dora.activation.require_sector_profile` denies production mode without financial-sector applicability.
Policy `eu.dora.activation.require_authority_route` denies production mode without competent-authority metadata.
Policy `eu.dora.activation.require_management_owner` denies production mode without accountable owner.
Policy `eu.dora.ict_risk.require_framework_record` denies release if the ICT risk framework has no current review.
Policy `eu.dora.ict_risk.require_tolerance_owner` denies release if risk tolerance is orphaned.
Policy `eu.dora.ict_risk.expire_acceptance` denies stale risk acceptance.
Policy `eu.dora.ict_risk.block_unknown_critical_function` denies activation if a critical function is unowned.
Policy `eu.dora.asset_identification.require_asset_owner` denies release for unowned ICT assets.
Policy `eu.dora.asset_identification.require_dependency_map` denies release for unmapped critical dependencies.
Policy `eu.dora.asset_identification.require_location` denies release if critical-function ICT asset location is unknown.
Policy `eu.dora.incident_reporting.require_clock_values` denies report approval if awareness or classification clocks are missing.
Policy `eu.dora.incident_reporting.freeze_report` denies mutation of approved report packages.
Policy `eu.dora.incident_reporting.block_template_mismatch` denies export when report template version is unsupported.
Policy `eu.dora.incident_reporting.require_major_threshold_evidence` denies major-incident downgrade without evidence.
Policy `eu.dora.third_party_register.require_arrangement_reference` denies register export for missing arrangement references.
Policy `eu.dora.third_party_register.require_function_link` denies register export when ICT services are not linked to functions.
Policy `eu.dora.third_party_register.require_subcontractor_rank` denies export when critical service subcontractor ranks are missing.
Policy `eu.dora.third_party_register.require_exit_strategy` denies production for critical providers without exit strategy status.
Policy `eu.dora.tlpt.require_scope` denies TLPT mode without approved target scope.
Policy `eu.dora.tlpt.require_vault` denies TLPT mode without restricted evidence vault.
Policy `eu.dora.tlpt.expire_window` denies tester capability after approved end time.
Policy `eu.dora.tlpt.prevent_self_closure` denies closure if tester and approver are the same principal.
Policy `eu.dora.evidence_freeze.prevent_post_submission_edit` denies modification after report state freeze.
Policy `eu.dora.evidence_freeze.require_hash` denies submission bundle without export hash.

## Data Model Deltas

Entity: `DoraTenantProfile`.
Field: `tenant_id`.
Field: `dora_applicability_status`.
Field: `financial_entity_type`.
Field: `competent_authority_country`.
Field: `competent_authority_name`.
Field: `competent_authority_route_id`.
Field: `management_body_owner_principal`.
Field: `management_body_review_cadence`.
Field: `simplified_framework_claimed`.
Field: `dora_mode`.
Field: `last_activation_review_at`.
Field: `activation_blockers`.
Entity: `DoraIctRiskFramework`.
Field: `framework_id`.
Field: `tenant_id`.
Field: `risk_tolerance_statement_id`.
Field: `approved_by_principal`.
Field: `approved_at`.
Field: `next_review_due_at`.
Field: `critical_function_count`.
Field: `known_asset_count`.
Field: `unowned_asset_count`.
Field: `accepted_risk_count`.
Field: `expired_acceptance_count`.
Field: `framework_evidence_uri`.
Entity: `DoraCriticalFunction`.
Field: `function_id`.
Field: `tenant_id`.
Field: `function_name`.
Field: `function_owner_principal`.
Field: `criticality_level`.
Field: `customer_impact_profile`.
Field: `rto_target`.
Field: `rpo_target`.
Field: `minimum_service_level`.
Field: `linked_product_ids`.
Field: `linked_api_contracts`.
Field: `linked_data_store_ids`.
Field: `linked_ict_asset_ids`.
Field: `linked_provider_arrangements`.
Field: `last_mapping_review_at`.
Entity: `DoraIctAsset`.
Field: `asset_id`.
Field: `tenant_id`.
Field: `asset_type`.
Field: `asset_owner_principal`.
Field: `critical_function_support`.
Field: `data_classification`.
Field: `jurisdiction_profile`.
Field: `runtime_cell_id`.
Field: `monitoring_coverage_status`.
Field: `backup_coverage_status`.
Field: `recovery_test_status`.
Field: `last_inventory_seen_at`.
Field: `source_discovery_event_id`.
Entity: `DoraIncident`.
Field: `incident_id`.
Field: `tenant_id`.
Field: `incident_kind`.
Field: `event_started_at`.
Field: `detected_at`.
Field: `awareness_at`.
Field: `classified_at`.
Field: `classification_principal`.
Field: `major_incident_status`.
Field: `significant_cyber_threat_status`.
Field: `affected_function_ids`.
Field: `affected_asset_ids`.
Field: `affected_provider_ids`.
Field: `affected_client_estimate`.
Field: `geographic_scope`.
Field: `personal_data_involvement`.
Field: `current_report_stage`.
Field: `competent_authority_route_id`.
Entity: `DoraIncidentThresholdEvidence`.
Field: `threshold_evidence_id`.
Field: `incident_id`.
Field: `criteria_name`.
Field: `criteria_value`.
Field: `criteria_status`.
Field: `source_event_id`.
Field: `recorded_by_principal`.
Field: `recorded_at`.
Field: `confidence`.
Entity: `DoraIncidentReportPackage`.
Field: `report_package_id`.
Field: `incident_id`.
Field: `report_stage`.
Field: `template_version`.
Field: `prepared_at`.
Field: `approved_by_principal`.
Field: `approved_at`.
Field: `export_hash`.
Field: `submitted_at`.
Field: `submission_reference`.
Field: `freeze_state`.
Field: `unknown_fields`.
Field: `evidence_uri`.
Entity: `DoraIctThirdPartyArrangement`.
Field: `arrangement_id`.
Field: `tenant_id`.
Field: `arrangement_reference`.
Field: `provider_id`.
Field: `direct_provider_id`.
Field: `subcontractor_rank`.
Field: `ict_service_type`.
Field: `supports_critical_function`.
Field: `critical_function_ids`.
Field: `contract_start_date`.
Field: `contract_end_date`.
Field: `notice_period`.
Field: `audit_rights_status`.
Field: `access_rights_status`.
Field: `exit_strategy_status`.
Field: `substitutability_rating`.
Field: `data_location_country_codes`.
Field: `processing_location_country_codes`.
Field: `concentration_risk_group`.
Entity: `DoraIctThirdPartyProvider`.
Field: `provider_id`.
Field: `provider_legal_name`.
Field: `provider_identifier_type`.
Field: `provider_identifier_value`.
Field: `provider_country_code`.
Field: `ctpp_designation_status`.
Field: `group_provider_status`.
Field: `last_due_diligence_review_at`.
Field: `contract_policy_evidence_uri`.
Entity: `DoraRegisterExport`.
Field: `register_export_id`.
Field: `tenant_id`.
Field: `register_level`.
Field: `schema_version`.
Field: `source_snapshot_hash`.
Field: `validation_status`.
Field: `validation_error_count`.
Field: `prepared_by_principal`.
Field: `approved_by_principal`.
Field: `approved_at`.
Field: `export_uri`.
Field: `export_hash`.
Entity: `DoraTlptExercise`.
Field: `tlpt_id`.
Field: `tenant_id`.
Field: `test_authority_contact`.
Field: `target_function_ids`.
Field: `target_asset_ids`.
Field: `rules_of_engagement_uri`.
Field: `tester_principal_group`.
Field: `white_team_principal_group`.
Field: `approved_start_at`.
Field: `approved_end_at`.
Field: `actual_start_at`.
Field: `actual_end_at`.
Field: `stop_condition_status`.
Field: `vault_id`.
Field: `closure_approved_by_principal`.
Field: `closure_approved_at`.
Entity: `DoraTlptFinding`.
Field: `finding_id`.
Field: `tlpt_id`.
Field: `severity`.
Field: `affected_function_ids`.
Field: `affected_asset_ids`.
Field: `remediation_owner_principal`.
Field: `remediation_due_at`.
Field: `retest_required`.
Field: `retest_status`.
Field: `closure_evidence_uri`.

## API Contract Deltas

Endpoint: `GET /v1/eu/dora/profile`.
Purpose: return DORA activation status and blocking reasons.
Authorization: tenant compliance reader.
Response field: `dora_applicability_status`.
Response field: `financial_entity_type`.
Response field: `dora_mode`.
Response field: `production_blockers`.
Endpoint: `PATCH /v1/eu/dora/profile`.
Purpose: update tenant-declared DORA profile fields.
Authorization: tenant compliance administrator.
Validation: rejects production activation without competent-authority route.
Validation: rejects production activation without management-body owner.
Endpoint: `GET /v1/eu/dora/ict-risk-framework`.
Purpose: expose current ICT risk framework status.
Authorization: tenant compliance reader.
Response field: `framework_review_status`.
Response field: `risk_tolerance_status`.
Response field: `critical_function_mapping_status`.
Endpoint: `POST /v1/eu/dora/ict-risk-framework/review`.
Purpose: record management-body or delegated review evidence.
Authorization: tenant compliance administrator.
Validation: requires evidence URI and approver principal.
Endpoint: `GET /v1/eu/dora/critical-functions`.
Purpose: list critical or important functions.
Authorization: tenant compliance reader.
Filter: `criticality_level`.
Filter: `owner_principal`.
Filter: `mapping_status`.
Endpoint: `POST /v1/eu/dora/critical-functions`.
Purpose: create a critical-function record.
Authorization: tenant compliance administrator.
Validation: requires owner and continuity metadata.
Endpoint: `GET /v1/eu/dora/ict-assets`.
Purpose: list ICT assets supporting the tenant.
Authorization: tenant compliance reader.
Filter: `critical_function_support`.
Filter: `asset_owner_principal`.
Filter: `jurisdiction_profile`.
Filter: `monitoring_coverage_status`.
Endpoint: `PATCH /v1/eu/dora/ict-assets/{asset_id}`.
Purpose: update DORA-specific asset metadata.
Authorization: tenant compliance administrator.
Validation: rejects owner removal on critical-function assets.
Endpoint: `POST /v1/eu/dora/incidents`.
Purpose: open a DORA ICT incident record.
Authorization: incident manager.
Validation: requires detection time or unknown reason.
Validation: requires affected function assessment.
Endpoint: `PATCH /v1/eu/dora/incidents/{incident_id}/classify`.
Purpose: record DORA incident classification.
Authorization: incident manager plus compliance reviewer for downgrade.
Validation: requires threshold evidence.
Validation: blocks major-to-non-major downgrade without reason evidence.
Endpoint: `GET /v1/eu/dora/incidents/{incident_id}/clock`.
Purpose: return event, detection, awareness, and classification clocks.
Authorization: incident manager.
Response field: `initial_report_due_at`.
Response field: `intermediate_report_due_at`.
Response field: `final_report_due_at`.
Endpoint: `POST /v1/eu/dora/incidents/{incident_id}/reports`.
Purpose: build a report package for initial, intermediate, or final stage.
Authorization: incident manager.
Validation: requires template version.
Validation: requires unknown-field list if facts are incomplete.
Endpoint: `POST /v1/eu/dora/incidents/{incident_id}/reports/{report_package_id}/approve`.
Purpose: approve and freeze a report package.
Authorization: compliance approver.
Validation: writes export hash before freeze.
Endpoint: `POST /v1/eu/dora/incidents/{incident_id}/reports/{report_package_id}/submission-reference`.
Purpose: attach tenant-supplied submission receipt metadata.
Authorization: compliance approver.
Validation: cannot change report body after submission reference attachment.
Endpoint: `GET /v1/eu/dora/third-party-arrangements`.
Purpose: list ICT third-party arrangements.
Authorization: third-party risk reader.
Filter: `supports_critical_function`.
Filter: `provider_country_code`.
Filter: `concentration_risk_group`.
Filter: `exit_strategy_status`.
Endpoint: `POST /v1/eu/dora/third-party-arrangements`.
Purpose: create an ICT arrangement record for the DORA register.
Authorization: third-party risk administrator.
Validation: requires arrangement reference.
Validation: requires provider identity.
Validation: requires function link for critical services.
Endpoint: `POST /v1/eu/dora/register-exports`.
Purpose: prepare a register-of-information export.
Authorization: third-party risk administrator.
Validation: requires zero blocking errors.
Validation: records source snapshot hash.
Endpoint: `GET /v1/eu/dora/register-exports/{register_export_id}`.
Purpose: retrieve export state and validation errors.
Authorization: third-party risk reader.
Response field: `schema_version`.
Response field: `validation_status`.
Response field: `export_hash`.
Endpoint: `POST /v1/eu/dora/tlpt`.
Purpose: create a TLPT exercise record.
Authorization: resilience testing administrator.
Validation: requires approved scope and rules of engagement.
Validation: requires segregated evidence vault.
Endpoint: `POST /v1/eu/dora/tlpt/{tlpt_id}/start`.
Purpose: activate TLPT window.
Authorization: resilience testing approver.
Validation: denies start before approved window.
Validation: denies start if tester and approver separation is missing.
Endpoint: `POST /v1/eu/dora/tlpt/{tlpt_id}/stop`.
Purpose: revoke TLPT capabilities and record stop reason.
Authorization: white team or resilience testing approver.
Validation: writes stop audit event.
Endpoint: `POST /v1/eu/dora/tlpt/{tlpt_id}/findings`.
Purpose: record TLPT findings in restricted evidence vault.
Authorization: white team.
Validation: requires remediation owner for high and critical findings.
Endpoint: `POST /v1/eu/dora/tlpt/{tlpt_id}/close`.
Purpose: close exercise after remediation plan and evidence review.
Authorization: resilience testing approver.
Validation: denies tester self-approval.

## Audit Event Additions (per ADR-0263)

Event class: `eu.dora.profile.activation_requested`.
Payload: `tenant_id`, `requested_mode`, `requested_by`, `financial_entity_type`, `authority_route_id`.
Retention: compliance pack audit retention.
Event class: `eu.dora.profile.activation_denied`.
Payload: `tenant_id`, `requested_mode`, `deny_policy`, `blocking_fields`.
Retention: compliance pack audit retention.
Event class: `eu.dora.profile.production_enabled`.
Payload: `tenant_id`, `enabled_by`, `authority_route_id`, `management_owner`, `framework_id`.
Retention: compliance pack audit retention.
Event class: `eu.dora.framework.review_recorded`.
Payload: `framework_id`, `tenant_id`, `approved_by`, `approved_at`, `next_review_due_at`.
Retention: compliance pack audit retention.
Event class: `eu.dora.framework.risk_acceptance_created`.
Payload: `tenant_id`, `risk_id`, `accepted_by`, `expires_at`, `affected_functions`.
Retention: compliance pack audit retention.
Event class: `eu.dora.framework.risk_acceptance_expired`.
Payload: `tenant_id`, `risk_id`, `expired_at`, `affected_functions`.
Retention: compliance pack audit retention.
Event class: `eu.dora.critical_function.created`.
Payload: `function_id`, `tenant_id`, `owner`, `criticality_level`.
Retention: compliance pack audit retention.
Event class: `eu.dora.critical_function.mapping_changed`.
Payload: `function_id`, `tenant_id`, `changed_by`, `linked_assets`, `linked_providers`.
Retention: compliance pack audit retention.
Event class: `eu.dora.critical_function.unowned_blocked`.
Payload: `tenant_id`, `function_id`, `deny_policy`, `requested_action`.
Retention: compliance pack audit retention.
Event class: `eu.dora.asset.discovered`.
Payload: `asset_id`, `tenant_id`, `asset_type`, `source_event_id`, `jurisdiction_profile`.
Retention: operational evidence retention.
Event class: `eu.dora.asset.owner_changed`.
Payload: `asset_id`, `tenant_id`, `old_owner`, `new_owner`, `changed_by`.
Retention: operational evidence retention.
Event class: `eu.dora.asset.inventory_stale`.
Payload: `tenant_id`, `asset_id`, `last_seen_at`, `stale_threshold`.
Retention: operational evidence retention.
Event class: `eu.dora.asset.critical_mapping_missing`.
Payload: `tenant_id`, `asset_id`, `critical_function_ids`, `deny_policy`.
Retention: operational evidence retention.
Event class: `eu.dora.incident.opened`.
Payload: `incident_id`, `tenant_id`, `incident_kind`, `detected_at`, `opened_by`.
Retention: incident evidence retention.
Event class: `eu.dora.incident.clock_recorded`.
Payload: `incident_id`, `event_started_at`, `detected_at`, `awareness_at`, `classified_at`.
Retention: incident evidence retention.
Event class: `eu.dora.incident.threshold_evidence_added`.
Payload: `incident_id`, `criteria_name`, `criteria_status`, `source_event_id`, `recorded_by`.
Retention: incident evidence retention.
Event class: `eu.dora.incident.classified_major`.
Payload: `incident_id`, `classified_by`, `classified_at`, `criteria_triggered`.
Retention: incident evidence retention.
Event class: `eu.dora.incident.downgrade_denied`.
Payload: `incident_id`, `requested_by`, `deny_policy`, `missing_evidence`.
Retention: incident evidence retention.
Event class: `eu.dora.incident.report_package_created`.
Payload: `report_package_id`, `incident_id`, `report_stage`, `template_version`.
Retention: incident evidence retention.
Event class: `eu.dora.incident.report_package_approved`.
Payload: `report_package_id`, `incident_id`, `approved_by`, `export_hash`.
Retention: incident evidence retention.
Event class: `eu.dora.incident.report_package_frozen`.
Payload: `report_package_id`, `incident_id`, `freeze_state`, `frozen_at`.
Retention: incident evidence retention.
Event class: `eu.dora.incident.report_mutation_denied`.
Payload: `report_package_id`, `incident_id`, `deny_policy`, `attempted_by`.
Retention: incident evidence retention.
Event class: `eu.dora.incident.submission_reference_attached`.
Payload: `report_package_id`, `incident_id`, `submission_reference`, `submitted_at`.
Retention: incident evidence retention.
Event class: `eu.dora.third_party.provider_created`.
Payload: `provider_id`, `tenant_id`, `provider_country_code`, `identifier_type`.
Retention: third-party risk retention.
Event class: `eu.dora.third_party.arrangement_created`.
Payload: `arrangement_id`, `tenant_id`, `provider_id`, `arrangement_reference`.
Retention: third-party risk retention.
Event class: `eu.dora.third_party.arrangement_critical_linked`.
Payload: `arrangement_id`, `tenant_id`, `critical_function_ids`, `linked_by`.
Retention: third-party risk retention.
Event class: `eu.dora.third_party.subcontractor_chain_changed`.
Payload: `arrangement_id`, `tenant_id`, `provider_id`, `rank`, `changed_by`.
Retention: third-party risk retention.
Event class: `eu.dora.third_party.exit_strategy_missing`.
Payload: `tenant_id`, `arrangement_id`, `provider_id`, `critical_function_ids`.
Retention: third-party risk retention.
Event class: `eu.dora.third_party.register_export_requested`.
Payload: `register_export_id`, `tenant_id`, `register_level`, `schema_version`, `requested_by`.
Retention: third-party risk retention.
Event class: `eu.dora.third_party.register_export_validated`.
Payload: `register_export_id`, `validation_status`, `validation_error_count`, `source_snapshot_hash`.
Retention: third-party risk retention.
Event class: `eu.dora.third_party.register_export_approved`.
Payload: `register_export_id`, `approved_by`, `export_hash`, `approved_at`.
Retention: third-party risk retention.
Event class: `eu.dora.third_party.register_export_denied`.
Payload: `register_export_id`, `deny_policy`, `blocking_errors`.
Retention: third-party risk retention.
Event class: `eu.dora.tlpt.created`.
Payload: `tlpt_id`, `tenant_id`, `target_function_ids`, `approved_start_at`, `approved_end_at`.
Retention: restricted resilience-test retention.
Event class: `eu.dora.tlpt.scope_approved`.
Payload: `tlpt_id`, `approved_by`, `target_asset_ids`, `rules_of_engagement_uri`.
Retention: restricted resilience-test retention.
Event class: `eu.dora.tlpt.start_denied`.
Payload: `tlpt_id`, `deny_policy`, `requested_by`, `blocking_fields`.
Retention: restricted resilience-test retention.
Event class: `eu.dora.tlpt.started`.
Payload: `tlpt_id`, `started_by`, `actual_start_at`, `vault_id`.
Retention: restricted resilience-test retention.
Event class: `eu.dora.tlpt.stop_condition_triggered`.
Payload: `tlpt_id`, `triggered_by`, `reason`, `triggered_at`.
Retention: restricted resilience-test retention.
Event class: `eu.dora.tlpt.stopped`.
Payload: `tlpt_id`, `stopped_by`, `actual_end_at`, `stop_reason`.
Retention: restricted resilience-test retention.
Event class: `eu.dora.tlpt.finding_recorded`.
Payload: `finding_id`, `tlpt_id`, `severity`, `affected_function_ids`, `remediation_owner`.
Retention: restricted resilience-test retention.
Event class: `eu.dora.tlpt.finding_retest_required`.
Payload: `finding_id`, `tlpt_id`, `severity`, `retest_due_at`.
Retention: restricted resilience-test retention.
Event class: `eu.dora.tlpt.closed`.
Payload: `tlpt_id`, `closed_by`, `closure_approved_at`, `remaining_risk_count`.
Retention: restricted resilience-test retention.
Event class: `eu.dora.tlpt.self_closure_denied`.
Payload: `tlpt_id`, `attempted_by`, `deny_policy`.
Retention: restricted resilience-test retention.

## Failure Modes specific to EU enforcement

Failure mode: tenant enables DORA production mode without a financial-sector applicability record.
Enforcement risk: controls imply legal coverage without a declared regulated profile.
Pack response: deny production activation and emit `eu.dora.profile.activation_denied`.
Failure mode: management-body ownership is missing.
Enforcement risk: Article 5 governance evidence is incomplete.
Pack response: block production mode and require accountable owner.
Failure mode: ICT risk framework review expires.
Enforcement risk: Article 6 control framework becomes stale.
Pack response: deny release gates that rely on DORA readiness.
Failure mode: critical function has no asset mapping.
Enforcement risk: Article 8 identification is not operational.
Pack response: block DORA readiness claim and flag function.
Failure mode: incident classification waits for root cause.
Enforcement risk: report timer can be missed while perfect facts are unavailable.
Pack response: allow unknowns in report packages and preserve classification clock.
Failure mode: report package is edited after approval.
Enforcement risk: submission evidence becomes unverifiable.
Pack response: deny mutation and emit report mutation event.
Failure mode: major incident is downgraded without evidence.
Enforcement risk: under-reporting to competent authority.
Pack response: require compliance reviewer and threshold evidence.
Failure mode: third-party register omits subcontractor chain.
Enforcement risk: register-of-information export cannot support systemic-risk review.
Pack response: block export for critical service arrangements with missing rank data.
Failure mode: provider country is recorded as region only.
Enforcement risk: location and concentration analysis become unfit for supervisory use.
Pack response: require country code for provider and processing locations.
Failure mode: procurement vendor record is reused as DORA register source without contract mapping.
Enforcement risk: Article 28 register data is incomplete.
Pack response: deny register export until arrangement references and service links exist.
Failure mode: TLPT evidence enters general security logs.
Enforcement risk: sensitive test methods leak beyond need-to-know scope.
Pack response: require restricted evidence vault and deny low-trust export.
Failure mode: TLPT capability remains active after the approved window.
Enforcement risk: unbounded tester access.
Pack response: revoke capabilities automatically and emit stop event.
Failure mode: tester approves their own TLPT closure.
Enforcement risk: remediation evidence lacks independent review.
Pack response: deny closure.
Failure mode: DORA and GDPR incident workflows diverge.
Enforcement risk: ICT incident reports and personal-data breach assessments contradict each other.
Pack response: link incident records and require data-protection review when personal data may be involved.
Failure mode: competent-authority route is missing.
Enforcement risk: report package can be prepared but not operationally routed.
Pack response: block production activation and report approval.

## Worked Examples

### Example 1: Payment Institution Production Activation

Tenant profile declares `financial_entity_type=payment_institution`.
Tenant profile declares `dora_applicability_status=in_scope`.
Tenant profile declares competent authority country.
Tenant profile declares competent authority route.
Management body owner is set to the compliance executive principal.
ICT risk framework record is approved.
Critical functions are imported from service catalogue.
Critical function `payments-api-authorisation` has an owner.
Critical function `payments-api-authorisation` has RTO and RPO values.
Critical function `payments-api-authorisation` links to runtime cell `eu-west-payments-cell-01`.
Critical function `payments-api-authorisation` links to database `ledger-primary-eu`.
Critical function `payments-api-authorisation` links to provider arrangement `arr-cloud-eu-001`.
Third-party arrangement `arr-cloud-eu-001` has provider identity.
Third-party arrangement `arr-cloud-eu-001` has service type.
Third-party arrangement `arr-cloud-eu-001` has audit rights status.
Third-party arrangement `arr-cloud-eu-001` has exit strategy status.
Cedar evaluates `eu.dora.activation.require_sector_profile`.
Cedar permits the sector profile gate.
Cedar evaluates `eu.dora.activation.require_authority_route`.
Cedar permits the authority route gate.
Cedar evaluates `eu.dora.activation.require_management_owner`.
Cedar permits the owner gate.
Cedar evaluates `eu.dora.ict_risk.require_framework_record`.
Cedar permits the framework gate.
The pack enables production DORA mode.
Audit emits `eu.dora.profile.production_enabled`.
Stop condition is a DORA profile with zero production blockers.

### Example 2: Major ICT Incident Clock

Monitoring detects a payments API outage at 09:15 UTC.
Operations opens a DORA incident at 09:18 UTC.
Incident manager records awareness at 09:22 UTC.
Incident manager records initial unknown client impact.
Incident manager links the incident to critical function `payments-api-authorisation`.
Incident manager links provider arrangement `arr-cloud-eu-001`.
Threshold evidence records service unavailability.
Threshold evidence records affected client estimate as unknown.
Threshold evidence records geographic scope as multi-member-state.
Compliance reviewer classifies the incident as major at 09:40 UTC.
The report clock records major classification time.
The initial report package is created with known facts and unknown fields.
The report package lists unknown root cause.
The report package lists unknown final client count.
The report package lists temporary mitigation.
The report package records template version.
Approver approves the initial report package.
The package is frozen.
The export hash is recorded.
The tenant submits through its competent-authority channel.
The tenant attaches submission reference.
Audit emits report created, approved, frozen, and submission-reference events.
Stop condition is an immutable initial report package with submission metadata.

### Example 3: Intermediate and Final Report Continuity

The same incident remains unresolved at the intermediate report checkpoint.
Incident manager updates affected functions.
Incident manager updates customer-impact estimates.
Incident manager updates provider status.
Incident manager adds evidence from the cloud-provider status notice.
Incident manager adds business-continuity workaround evidence.
The intermediate report package is generated.
The package includes action plan.
The package includes expected restoration timeline.
The package includes unresolved questions.
Approver freezes the intermediate report package.
The final report is prepared after restoration.
Root cause analysis identifies dependency exhaustion in a provider-managed service.
Corrective action assigns capacity reservation review.
Corrective action assigns failover test enhancement.
Corrective action assigns provider contract review.
Final report package includes lessons learned.
Final report package includes remediation owners.
Final report package includes target closure dates.
Final report package is frozen.
Stop condition is all report packages immutable and cross-linked.

### Example 4: Register of Information Dry Run

Third-party risk administrator requests register export.
The export level is entity level.
The schema version is selected.
The source snapshot is built from provider and arrangement records.
Validation checks arrangement references.
Validation checks provider identifiers.
Validation checks critical-function links.
Validation checks service type.
Validation checks country code fields.
Validation checks subcontractor rank.
Validation checks exit strategy status.
One critical arrangement lacks exit strategy status.
Cedar denies export approval.
Audit emits `eu.dora.third_party.register_export_denied`.
Administrator updates the arrangement with exit strategy status.
Administrator reruns validation.
Validation returns zero blocking errors.
Approver approves the export.
The export hash is recorded.
Stop condition is an approved export with source snapshot hash and validation evidence.

### Example 5: TLPT Window

Resilience testing administrator creates TLPT exercise.
Target scope includes customer authentication and payment approval functions.
Target assets are mapped to EU runtime cells.
Rules of engagement are attached.
Evidence vault is assigned.
Tester group is recorded.
White-team group is recorded.
Approved start and end times are set.
Cedar checks target scope.
Cedar checks evidence vault.
Cedar checks tester and approver separation.
Cedar checks current time against approved window.
TLPT start is permitted.
Audit emits `eu.dora.tlpt.started`.
Tester records a high-severity finding in the restricted vault.
Finding owner is assigned to the identity-platform team.
Retest is marked required.
Stop condition triggers at scheduled end time.
Cedar revokes tester capabilities.
White team records closure after remediation plan approval.
Tester cannot approve closure.
Audit emits `eu.dora.tlpt.closed`.
Stop condition is a closed exercise with finding remediation owners.

### Example 6: Provider Concentration Review

Register data shows four critical functions use the same cloud provider group.
Provider group is outside the tenant's primary jurisdiction.
Two arrangements share the same regional service dependency.
Third-party risk administrator marks concentration risk group.
Risk framework review imports the concentration finding.
Management owner accepts temporary risk for ninety days.
Exit strategy owner is assigned.
Continuity test owner is assigned.
Cedar records expiring risk acceptance.
Release gate permits only because acceptance is current.
At expiry, Cedar denies readiness claim until risk is revalidated or remediated.
Stop condition is either remediated concentration or current accountable acceptance.

### Example 7: DORA and GDPR Incident Coordination

ICT incident affects availability of account-history APIs.
Incident manager records possible personal data exposure as unknown.
The DORA incident workflow opens.
The GDPR breach assessment workflow is linked.
Data-protection contact is notified.
DORA report package records ICT impact.
GDPR assessment records confidentiality assessment separately.
The two workflows share incident id but not legal conclusions.
The platform prevents inconsistent closure states.
DORA final report cannot claim no personal-data dimension until GDPR assessment is resolved or explicitly marked not applicable.
Stop condition is linked evidence without conflating report duties.

## Cross-References

EU pack overview: `packs/eu-localization/README.md`.
Regulatory matrix: `packs/eu-localization/regulatory-coverage.md`.
Data-residency controls: `packs/eu-localization/data-residency-and-cross-border.md`.
DSR workflows: `packs/eu-localization/dsr-and-portability.md`.
High-risk AI controls: `packs/eu-localization/high-risk-ai-systems.md`.
Audit baseline: ADR-0263.
Compliance-pack primitive: ADR-0251.
Cedar universal gate baseline: ADR-0243.
Tenant and sub-scope discipline: ADR-0242 and ADR-0244.
Cellular architecture baseline: ADR-0248.
Cross-jurisdiction conflict handling: ADR-0304.
Capability-tier overlay handling: ADR-0316.

## Operational Checklists

Checklist: DORA activation.
Item: confirm tenant financial-sector applicability.
Item: confirm competent authority country and route.
Item: confirm management-body owner.
Item: confirm ICT risk framework approval.
Item: confirm critical functions are owned.
Item: confirm critical functions map to ICT assets.
Item: confirm ICT assets have jurisdiction profile.
Item: confirm third-party ICT arrangements are recorded.
Item: confirm incident reporting contacts are present.
Item: confirm report templates are available.
Item: confirm TLPT mode is disabled unless a window is approved.
Checklist: ICT risk framework review.
Item: review risk tolerance statement.
Item: review critical function list.
Item: review ICT asset inventory.
Item: review continuity evidence.
Item: review third-party concentration risk.
Item: review open risk acceptances.
Item: review expired risk acceptances.
Item: review supervisory findings.
Item: review major incident lessons learned.
Item: record next review date.
Checklist: incident report package.
Item: record detection time.
Item: record awareness time.
Item: record classification time.
Item: record classifier principal.
Item: record threshold evidence.
Item: record affected functions.
Item: record affected providers.
Item: record unknown fields.
Item: select template version.
Item: approve and freeze package.
Item: attach submission reference after tenant submission.
Checklist: register export.
Item: confirm arrangement references.
Item: confirm provider identities.
Item: confirm direct provider and subcontractor ranks.
Item: confirm ICT service types.
Item: confirm critical-function links.
Item: confirm data and processing locations.
Item: confirm audit rights status.
Item: confirm exit strategy status.
Item: confirm schema version.
Item: record source snapshot hash.
Checklist: TLPT.
Item: approve target functions.
Item: approve target assets.
Item: attach rules of engagement.
Item: assign restricted vault.
Item: verify tester and approver separation.
Item: activate only within approved window.
Item: record findings with remediation owner.
Item: revoke capability at stop condition.
Item: close only after independent approval.

## Evidence Retention Rules

DORA profile evidence uses compliance pack retention.
ICT risk framework evidence uses compliance pack retention.
Critical-function mapping evidence uses compliance pack retention.
ICT asset inventory evidence uses operational evidence retention.
Incident report evidence uses incident evidence retention.
Incident submission metadata uses incident evidence retention.
Third-party register evidence uses third-party risk retention.
Register export hashes use third-party risk retention.
TLPT evidence uses restricted resilience-test retention.
TLPT exploit detail never enters general analytics retention.
Report packages cannot be purged before the tenant's regulatory retention policy allows it.
Register exports cannot be purged before the tenant's regulatory retention policy allows it.
Risk acceptance evidence cannot be purged while accepted risk is active.
Critical-function ownership history cannot be purged while the function is active.
Provider arrangement history cannot be purged while the arrangement supports a critical or important function.

## Release Gate Summary

Gate `dora-profile-ready` passes only with sector profile, authority route, and owner.
Gate `dora-risk-framework-ready` passes only with current framework approval.
Gate `dora-critical-functions-ready` passes only with owned and mapped critical functions.
Gate `dora-asset-inventory-ready` passes only with non-stale owned ICT assets.
Gate `dora-incident-ready` passes only with reporting contacts and templates.
Gate `dora-register-ready` passes only with valid provider arrangement data.
Gate `dora-tlpt-safe` passes only inside approved TLPT windows.
Gate `dora-evidence-freeze` passes only when frozen artifacts have hashes.
Gate failures must include policy id.
Gate failures must include remediation owner when known.
Gate failures must include affected tenant.
Gate failures must not include restricted TLPT details.

## Checkpoint

Checkpoint id: `eu-pack-dora-operational-resilience-2026-05-20`.
Authoring scope: `packs/eu-localization`.
Agent id: `codex-eu-localization-pack-w1`.
Evidence label: `eu_pack_docs:6`.
Status: drafted for pack review.
Stop condition: file exists, required sections exist, DORA scope is financial-sector specific, and line-count threshold is satisfied.
