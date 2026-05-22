# IP-002 Healthcare Integration Cedar Default Deny

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-002-cedar-default-deny.md
Batch: Batch C healthcare-integration IP deepening
Status: implementation-plan
Owner: axis-healthcare-integration + council-security
Primary layer: governance
Primary policy file: microservices/healthcare-integration/policy/clinical-interoperability-authorization.cedar
Supporting policy file: microservices/healthcare-integration/policy/auditor-scope.cedar
Supporting policy file: microservices/healthcare-integration/policy/ci-scope.cedar
Supporting policy file: microservices/healthcare-integration/policy/data-residency.md
Supporting policy file: microservices/healthcare-integration/policy/emergency-services-bypass.cedar
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md
Repo references: microservices/healthcare-integration/ARCHITECTURE.md
Repo references: microservices/healthcare-integration/manifest.json
Repo references: microservices/healthcare-integration/threat-model.md
Repo references: microservices/healthcare-integration/compliance.md
Repo references: microservices/healthcare-integration/policies/local-breakglass-access-control.cedar
Repo references: microservices/healthcare-integration/policies/local-patient-consent-sync.cedar
Repo references: microservices/healthcare-integration/policies/local-fhir-exchange-consent.cedar
Repo references: microservices/healthcare-integration/policies/local-hipaa-audit-completeness.cedar
Repo references: microservices/healthcare-integration/policies/local-hl7-ingest-source-scope.cedar
Repo references: microservices/healthcare-integration/policies/local-phi-delivery-authorization.cedar

## Objective
- Turn clinical interoperability into a default-deny policy surface instead of a connector allowlist.
- Require Cedar authorization before source-system access, storage writes, event publication, workflow transitions, and evidence export.
- Keep policy evaluation caller-side library first per the capability records and ADR-0246.
- Keep OpenBao credential sidecar checks outside Cedar secrets while requiring credential lease posture in context per ADR-0296.
- Keep abuse controls connected to healthcare safety paths per ADR-0297 without blocking clean emergency services use.
- Keep Cedar fragment rollout soakable and reversible per ADR-0294.
- Keep policy effects tied to tenant scope per ADR-0244.
- Keep DealSet obligations explicit per ADR-0314.
- Keep industry-leader coverage explicit per ADR-0321.

## Authorization Context
- context.tenant_id is required.
- context.principal_id is required.
- context.audience_type is required.
- context.cell_tier is required.
- context.home_cell is required.
- context.jurisdiction_code is required.
- context.purpose is required.
- context.data_class is required.
- context.audit_event_class is required.
- context.policy_evaluation_mode is required.
- context.provider_credential_mode is required.
- context.trace_context is required.
- context.idempotency_key is required for commands.
- context.workflow_run_id is required for async workers.
- context.deal_set_id is required when commercial routing or marketplace settlement applies.
- context.ontology_object_type is required when read/write projection is involved.
- context.ontology_object_version is required when read/write projection is involved.
- context.source_system_ref is required for provider adapter access.
- context.emergency_attestation is required for emergency services bypass.
- context.bot_score is optional but must be honored when present.
- context.fragment_version is required for policy soak and rollback.
- context.policy_pack_ids is required for compliance-sensitive operations.
- context.residency_label is required before export or replication.
- context.break_glass_justification is required for break-glass mutation or read elevation.

## Principal Classes
- HEALTHCARE_OPERATOR can invoke treatment, consent, referral, and operations actions when tenant and purpose match.
- TENANT_ADMIN can configure integration settings but cannot read PHI by default.
- AUDITOR can read evidence and policy decisions but cannot mutate clinical state.
- SUPPORT_OPERATOR can inspect operational evidence but cannot export PHI without tenant-granted policy.
- AUTOMATED_WORKER can run workflow-bound imports, routes, replays, and reconciliations with idempotency and workflow_run_id.
- EMERGENCY_SERVICES can receive limited emergency permit only with jurisdiction_registered attestation and audit class.
- CI_SYSTEM can validate policy fragments and contract fixtures, never access live PHI.
- PROVIDER_ADAPTER is never a human principal and cannot mint its own tenant authority.

## Resource Classes
- FhirResource belongs to tenant, source_system_ref, data_class=fhir_resource, and ontology object version.
- Hl7Message belongs to tenant, facility source, interface id, message control id, and data_class=hl7_message.
- BreakGlassEvent belongs to tenant, patient scope, emergency attestation, expiry, reviewer, and audit event class.
- ClinicalConsent belongs to tenant, patient, purpose, pack, consent source, status, and version.
- ReferralRecord belongs to sending tenant, receiving tenant when permitted, purpose, care context, and audit evidence.
- ProviderDirectoryRecord belongs to tenant or network scope and must not grant cross-tenant clinical data access by itself.
- AuditEvidence belongs to tenant, principal, policy decision id, audit event class, and digest.
- CredentialLease belongs to tenant, provider adapter, OpenBao lease ref, TTL, and rotation epoch.

## Default Deny Rules
- Deny when tenant_id is absent.
- Deny when principal_id is absent.
- Deny when audience_type is absent.
- Deny when data_class is absent.
- Deny when purpose is absent.
- Deny when policy_evaluation_mode is not caller_side_library_first.
- Deny when provider access is requested and provider_credential_mode is not openbao_sidecar_ttl_60s.
- Deny when source_system_ref tenant differs from authenticated tenant.
- Deny when ontology object tenant differs from authenticated tenant.
- Deny when workflow_run_id tenant differs from authenticated tenant.
- Deny when deal_set_id tenant differs from authenticated tenant.
- Deny when support operator requests PHI export without tenant permit.
- Deny when auditor requests mutation.
- Deny when automated worker lacks idempotency key.
- Deny when emergency services lacks jurisdiction_registered attestation.
- Deny when bot_score is high and audience_type is not EMERGENCY_SERVICES.
- Deny when residency_label blocks export or replication.
- Deny when pack resolver reports conflict without higher-restriction fallback.
- Deny when policy fragment is outside soak window and not promoted.
- Deny when audit_event_class is missing for accepted mutation.

## Permit Families
- Permit fhir-read for HEALTHCARE_OPERATOR when tenant, purpose, consent, policy pack, and ontology version align.
- Permit hl7-route for AUTOMATED_WORKER when source_system_ref, workflow_run_id, idempotency_key, and credential lease align.
- Permit break-glass-authorize for HEALTHCARE_OPERATOR or EMERGENCY_SERVICES only with justification, expiry, and audit event class.
- Permit consent-sync for HEALTHCARE_OPERATOR or AUTOMATED_WORKER when clinical_consent data class and consent source are explicit.
- Permit ehr-provenance-seal for AUTOMATED_WORKER when source_system_ref, digest, and audit-chain target are explicit.
- Permit patient-match-review for HEALTHCARE_OPERATOR when candidate records are same-tenant or relationship-authorized.
- Permit auditor-scope evidence read for AUDITOR when evidence tenant matches and PHI export is not requested.
- Permit CI fragment validation for CI_SYSTEM against fixtures only.

## Competitor Displacement
- Redox displacement: Redox access patterns are connector-contract oriented; Oyatie requires Cedar context before any connector sees a payload.
- Rhapsody displacement: Rhapsody routes messages through channels; Oyatie denies route execution unless tenant, pack, audit, and credential context are complete.
- InterSystems IRIS for Health displacement: IRIS can embed rules near data; Oyatie keeps authorization as a shared Cedar gate outside source-specific storage.
- Lyniate/Corepoint displacement: Corepoint interface logic is not enough; Oyatie requires policy evidence for every transform and replay.
- Mirth Connect displacement: Mirth channel filters do not replace tenant policy; Oyatie treats channel success as post-policy adapter behavior.
- NextGate displacement: NextGate match permissions become Cedar-gated review actions, not implicit authority to merge identities.
- Health Catalyst displacement: Health Catalyst analytics extracts require policy-cleared evidence before aggregation, not after dashboard publication.
- Epic displacement: Epic app authorization remains source-system evidence and cannot authorize cross-Oyatie tenant action by itself.
- Cerner displacement: Cerner SMART or interface permissions remain adapter inputs; Oyatie Cedar remains the universal gate.
- Allscripts displacement: Allscripts connector access is subordinated to tenant and pack policy.
- Veeva displacement: Veeva regulated workflows inform compliance contexts but do not bypass default deny.

## Implementation Slices
- Slice 1: align clinical-interoperability-authorization.cedar with TenantScope required fields.
- Slice 2: split human clinical permits from automated worker permits.
- Slice 3: split auditor read-evidence permits from mutation permits.
- Slice 4: make emergency-services-bypass.cedar require tenant_id and emergency_attestation.
- Slice 5: make data-residency.md feed residency_label into Cedar context.
- Slice 6: make CI scope policy validate fixtures without live PHI access.
- Slice 7: add policy fragment version and soak window metadata per ADR-0294.
- Slice 8: add provider_credential_mode checks per ADR-0296.
- Slice 9: add bot_score deny path per ADR-0297.
- Slice 10: add DealSet context checks per ADR-0314.
- Slice 11: add ontology object type and version checks per ADR-0257.
- Slice 12: add API version context checks per ADR-0258 where policy depends on contract version.
- Slice 13: add audit_event_class mandatory checks per ADR-0263.
- Slice 14: add tests for each principal and resource family.
- Slice 15: add denial evidence fixture for each default-deny branch.
- Slice 16: add rollback fixture for prior fragment promotion.
- Slice 17: add local policy pack overlay fixtures.
- Slice 18: add emergency path chaos fixture.
- Slice 19: add patient match cross-tenant rejection fixture.
- Slice 20: add provenance-seal credential lease rejection fixture.

## Failure Modes
- Missing context field: deny and emit PHI-safe refusal.
- Policy fragment typo: CI scope blocks promotion before runtime.
- Fragment regression after soak: rollback to prior promoted fragment.
- Credential sidecar unavailable: deny provider adapter action.
- Audit-chain target unavailable: deny mutation and allow read-only evidence review.
- High bot score for non-emergency actor: deny with abuse-defence event.
- Emergency actor missing attestation: deny with emergency refusal event.
- Support operator overreach: deny and open access review workflow.
- Auditor mutation attempt: deny and emit policy violation evidence.
- Cross-tenant patient match attempt: deny unless relationship evidence exists.
- Residency conflict: deny export and open compliance remediation.
- DealSet mismatch: deny commercial route acceptance.
- Source-system privilege escalation: ignore source claim and deny if Oyatie context fails.
- Ontology version mismatch: deny write and require projection migration.
- API version mismatch: deny deprecated action after sunset policy applies.

## Tests and Evidence
- Test default deny for absent tenant_id.
- Test default deny for absent principal_id.
- Test default deny for absent data_class.
- Test default deny for absent purpose.
- Test default deny for absent provider credential lease.
- Test permit for fhir-read with valid HEALTHCARE_OPERATOR context.
- Test permit for hl7-route with valid AUTOMATED_WORKER context.
- Test permit for break-glass only with justification and expiry.
- Test permit for consent-sync only with clinical_consent data class.
- Test permit for ehr-provenance-seal only with source_system_ref.
- Test patient-match-review rejects cross-tenant match without relationship.
- Test auditor can read evidence and cannot mutate.
- Test support operator cannot export PHI by default.
- Test emergency services path still emits audit event.
- Test bot-score denial excludes emergency services.
- Test CI scope cannot access live PHI.
- Test fragment rollback uses prior promoted policy id.
- Test denial evidence includes policy decision id.
- Test denial evidence excludes raw PHI.
- Test DealSet mismatch denies commercial action.

## Acceptance Criteria
- AC-001: Default deny is the first policy posture for all clinical interoperability actions.
- AC-002: Cedar receives complete tenant context from IP-001.
- AC-003: Human, auditor, support, emergency, CI, adapter, and worker principals are distinct.
- AC-004: Emergency access is a scoped permit with attestation, not a bypass.
- AC-005: Provider credentials are represented only by sidecar lease posture.
- AC-006: Bot and abuse controls cannot silently suppress emergency flows.
- AC-007: Cross-tenant clinical access requires explicit relationship evidence.
- AC-008: Denials emit audit-ready, PHI-safe evidence.
- AC-009: Policy fragments are soakable and rollback-ready per ADR-0294.
- AC-010: DealSet obligations are policy-visible per ADR-0314.
- AC-011: ADR-0243 universal Cedar gate is satisfied.
- AC-012: ADR-0246 policy-engine substrate promotion is respected.
- AC-013: ADR-0296 credential sidecar expectations are respected.
- AC-014: ADR-0297 abuse-defence baseline is respected.
- AC-015: ADR-0321 industry-leader depth includes explicit benchmark displacement.
- AC-016: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst are explicitly displaced.
- AC-017: The IP remains implementable without editing ADR-0321.
- AC-018: The IP cites existing repo policy files.
- AC-019: The IP keeps policy mechanics outside provider adapter code.
- AC-020: Default-deny fixtures prove missing tenant, expired consent, unsupported audience, and cross-cell requests refuse before adapter access.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-002-cedar-default-deny.md:66` - - TENANT_ADMIN can configure integration settings but cannot read PHI by default.; `microservices/healthcare-integration/IP-002-cedar-default-deny.md:68` - - SUPPORT_OPERATOR can inspect operational evidence but cannot export PHI without tenant-granted policy..
