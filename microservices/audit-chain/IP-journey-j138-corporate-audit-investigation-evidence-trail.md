---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j138-audit-chain-investigation-evidence-trail
journey_id: j138-corporate-audit-fraud-investigation-via-pattern-detection
microservice: audit-chain
role: investigation-evidence-trail
status: draft
date: 2026-05-20
authority_tier: 3
owner_team: axis-audit-chain + axis-internal-audit
parallel_work_compatibility: extends j137 evidence-bundler with investigation event classes
related_adrs: [ADR-0028, ADR-0310, ADR-0307, ADR-0311, ADR-0145]
depends_on:
  - microservices/audit-chain/IP-journey-j137-corporate-internal-audit-sox-controls-test-evidence-bundler.md
---

# IP-journey-j138-audit-chain-investigation-evidence-trail — Audit-chain: investigation event classes + prosecution-grade evidence trail

## Goal

Extend the audit-chain event-class registry with 30+ new event
classes specific to investigation cases. The audit chain must
produce a prosecution-grade evidence trail (signed, Merkle-rooted,
chain-of-custody-preserved) for the work-tenant portion of an
investigation that can be admitted in court.

## Data model

New event classes registered:

- `DetectionSignalEmitted`
- `DetectionSignalSealed`
- `InvestigationOpenRequested`
- `InvestigationCaseCreated`
- `InvestigationCedarPermitDraftAllocated`
- `InvestigationCedarPermitCoSigned`
- `InvestigationCedarPermitActivated`
- `InvestigationEvidencePullRequested`
- `InvestigationPaymentsGraphExported`
- `InvestigationMailCorrespondenceRead`
- `InvestigationMessengerCorrespondenceRead`
- `InvestigationPublicRecordsLookup`
- `InvestigationFindingFiled`
- `InvestigationFindingResolved`
- `InvestigationFindingEscalated`
- `InvestigationActionRequested`
- `InvestigationActionDualControlSigned`
- `VendorSuspended`
- `VendorUnsuspended`
- `InvoiceFrozen`
- `InvoiceUnfrozen`
- `PrincipalRoleSuspended`
- `PrincipalRoleUnsuspended`
- `HRReportingNotified`
- `SubpoenaPreparationRequested`
- `OutsideCounselEngaged`
- `InvestigationHandoffToExternal`
- `InvestigationCaseClosed`
- `InvestigationCaseExpired`
- `PersonalTenantReadDenied` (per-event seal, separate from j137)
- `ChainOfCustodyTransferred`
- `EvidencePackForProsecutionAssembled`

## Schema mapping

The `audit_chain.leaves` table (existing) accepts new classes; no
schema change required. The investigation-specific Merkle proofs
are computed per-case (via the j137 EvidencePackAssembler primitive
with `investigation_case_id` filter).

New tables for prosecution-grade evidence:

```sql
CREATE TABLE audit_chain.prosecution_evidence_packs (
  pack_id TEXT PRIMARY KEY,
  investigation_case_id TEXT NOT NULL,
  tenant_id TEXT NOT NULL,
  merkle_root TEXT NOT NULL,
  leaf_count INTEGER NOT NULL,
  chain_of_custody_certificate TEXT NOT NULL,
  outside_counsel_principal TEXT,
  assembled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  delivered_to_counsel_at TIMESTAMPTZ,
  delivered_to_court_at TIMESTAMPTZ,
  signature_id TEXT NOT NULL,
  chair_signature_id TEXT NOT NULL,
  counsel_signature_id TEXT
);
```

## API surface (gRPC)

```protobuf
service InvestigationEvidenceTrail {
  rpc AssembleProsecutionPack (AssembleProsecutionPackRequest) returns (AssembleProsecutionPackResponse);
  rpc DeliverToCounsel (DeliverToCounselRequest) returns (DeliverToCounselResponse);
  rpc GenerateChainOfCustodyCertificate (GenerateChainOfCustodyCertificateRequest) returns (GenerateChainOfCustodyCertificateResponse);
}
```

## Cedar policy

```cedar
@id("audit-chain-assemble-prosecution-pack-v1")
permit (
  principal,
  action == Action::"audit_chain.assemble_prosecution_pack",
  resource is InvestigationCase
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  resource.status in ["REMEDIATION", "EXTERNAL"] &&
  resource.director_signature != null &&
  resource.chair_signature != null
};

@id("audit-chain-deliver-to-counsel-v1")
permit (
  principal,
  action == Action::"audit_chain.deliver_to_counsel",
  resource is ProsecutionEvidencePack
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  context.outside_counsel_engaged == true
};
```

## Implementation notes

### Chain-of-custody certificate

Each prosecution pack carries a chain-of-custody certificate
documenting:

- Investigation case ID + creation provenance.
- Cedar permit batch and dual-control approval.
- Every read (with Cedar decision + audit-seal-id).
- Every action (with dual-control + audit-seal-id).
- Sam's signature + Audrey's co-signature + outside-counsel signature.
- Merkle root + leaf count + per-source root hashes.

The certificate is generated as a signed PDF + companion JSON.

### Personal-tenant deny inclusion

Personal-tenant deny events are included in the prosecution pack as
COUNTS + class labels. This shows the court that the investigation
respected the boundary and that any further evidence MUST come via
court warrant (ADR-0312). The deny events themselves are part of
the evidence trail because they demonstrate due-process compliance.

### Court-admission readiness

The pack format follows Federal Rule of Evidence 901 (authenticity)
+ 902 (self-authenticating documents) + 1006 (summaries) standards.
The Merkle proofs allow any leaf to be independently verified
against the sealed root.

## Performance budget

- `AssembleProsecutionPack` p95 ≤ 60s for ≤500-leaf packs.
- `DeliverToCounsel` p95 ≤ 5s.

## Test plan

See integration-test-plan.md §5, §6.

Unit tests:
- `test_prosecution_pack_assembly`
- `test_chain_of_custody_certificate_complete`
- `test_personal_tenant_denies_included_as_counts`
- `test_pack_immutability_post_seal`
- `test_court_admissibility_certificate_format`

## Build sequence

1. Register new event classes in audit-chain.
2. Schema migration `audit_chain-2026-q3-add-prosecution-packs`.
3. Cedar policies.
4. Chain-of-custody certificate generator.
5. Counsel-delivery channel (encrypted signed URL).
6. gRPC services.
7. Tests.

## Acceptance gates

- All tests PASS.
- Cedar lint clean.
- Schema migration applied.
- Mock-court-verification PASS.
- Code review: axis-audit-chain + axis-internal-audit + axis-legal.

## Operational notes

- Owner: axis-audit-chain.
- Pager: `oya-audit-chain-investigation`.
- Dashboards: `prosecution-pack-assembly-rate`,
  `chain-of-custody-cert-gen-latency`.

## Compliance / packs

- `pack-fed-rules-of-evidence-901-902-1006`.
- `pack-us-sox-404 + pack-fcpa-1977 + pack-eu-whistleblower-2019-1937`.

## Cross-microservice port declaration

Per ADR-0145, `InvestigationEvidenceTrail` in
`oyatie.audit_chain.investigation.v1`.

## Roll-out plan

Same five-phase rollout.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Chain-of-custody cert incomplete | CRITICAL | Required-field validation + lane test |
| Pack mutated after seal | CRITICAL | append-only + immutability test |
| Counsel delivery URL leak | HIGH | TTL + single-use + audit-sealed fetch |
| Court rejects on evidentiary technicality | MEDIUM | Mock-court-verification ahead of real cases |

## Definition of done

- All three gRPC services live behind flag.
- All tests PASS.
- AcmeWire prosecution pack assembled correctly.
- Mock counsel + mock court verification PASS.
- Chain-of-custody cert format matches FRE 901/902/1006 standards.
- Personal-tenant deny counts visible in pack; no personal-tenant
  content present.

## Completion expansion — j138 audit-chain IP rigor pass

Journey context: payroll anomaly detection triggers case-managed vendor-payment fraud investigation.
Service role: Merkle-sealed evidence, deny-event trail, and ADR-0263 audit emission.
Mapped services in this journey: observability, payments, workflow-engine, mail, audit-chain, community.
ADR anchors: ADR-0244, ADR-0297, ADR-0299, ADR-0310, ADR-0311, ADR-0319.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in audit-chain, define the Cedar policy change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving audit-chain and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in audit-chain, define the OpenAPI 3.2.0 contract change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving audit-chain and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in audit-chain, define the AsyncAPI 3.1.0 event change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving audit-chain and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in audit-chain, define the proto3 port change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving audit-chain and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in audit-chain, define the Postgres/RLS storage change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving audit-chain and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in audit-chain, define the audit-chain emission change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving audit-chain and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in audit-chain, define the dashboard projection change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving audit-chain and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in audit-chain, define the runbook hook change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving audit-chain and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in audit-chain, define the integration fixture change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving audit-chain and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in audit-chain, define the domain model change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving audit-chain and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in audit-chain, define the Cedar policy change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving audit-chain and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in audit-chain, define the OpenAPI 3.2.0 contract change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving audit-chain and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in audit-chain, define the AsyncAPI 3.1.0 event change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving audit-chain and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in audit-chain, define the proto3 port change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving audit-chain and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in audit-chain, define the Postgres/RLS storage change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving audit-chain and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in audit-chain, define the audit-chain emission change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving audit-chain and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in audit-chain, define the dashboard projection change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving audit-chain and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in audit-chain, define the runbook hook change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving audit-chain and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 018: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 019: in audit-chain, define the integration fixture change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 019: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 019: add pack-overlay coverage proving audit-chain and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 019: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 020: in audit-chain, define the domain model change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 020: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 020: add unit coverage proving audit-chain and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 020: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 02: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 021: in audit-chain, define the Cedar policy change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 021: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 021: add property coverage proving audit-chain and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 021: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 022: in audit-chain, define the OpenAPI 3.2.0 contract change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 022: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 022: add contract coverage proving audit-chain and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 022: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 023: in audit-chain, define the AsyncAPI 3.1.0 event change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 023: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 023: add integration coverage proving audit-chain and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 023: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 024: in audit-chain, define the proto3 port change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 024: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 024: add replay coverage proving audit-chain and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 024: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 025: in audit-chain, define the Postgres/RLS storage change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 025: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 025: add load coverage proving audit-chain and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 025: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 026: in audit-chain, define the audit-chain emission change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 026: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 026: add chaos coverage proving audit-chain and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 026: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 027: in audit-chain, define the dashboard projection change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 027: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 027: add negative authorization coverage proving audit-chain and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 027: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 028: in audit-chain, define the runbook hook change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 028: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 028: add multi-region coverage proving audit-chain and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 028: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 029: in audit-chain, define the integration fixture change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 029: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 029: add pack-overlay coverage proving audit-chain and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 029: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 030: in audit-chain, define the domain model change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 030: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 030: add unit coverage proving audit-chain and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 030: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 03: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 031: in audit-chain, define the Cedar policy change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 031: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 031: add property coverage proving audit-chain and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 031: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 032: in audit-chain, define the OpenAPI 3.2.0 contract change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 032: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 032: add contract coverage proving audit-chain and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 032: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 033: in audit-chain, define the AsyncAPI 3.1.0 event change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 033: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 033: add integration coverage proving audit-chain and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 033: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 034: in audit-chain, define the proto3 port change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 034: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 034: add replay coverage proving audit-chain and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 034: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 035: in audit-chain, define the Postgres/RLS storage change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 035: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 035: add load coverage proving audit-chain and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 035: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 036: in audit-chain, define the audit-chain emission change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 036: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 036: add chaos coverage proving audit-chain and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 036: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 037: in audit-chain, define the dashboard projection change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 037: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 037: add negative authorization coverage proving audit-chain and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 037: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 038: in audit-chain, define the runbook hook change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 038: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 038: add multi-region coverage proving audit-chain and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 038: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 039: in audit-chain, define the integration fixture change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 039: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 039: add pack-overlay coverage proving audit-chain and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 039: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 040: in audit-chain, define the domain model change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 040: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 040: add unit coverage proving audit-chain and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 040: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 04: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 041: in audit-chain, define the Cedar policy change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 041: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 041: add property coverage proving audit-chain and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 041: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 042: in audit-chain, define the OpenAPI 3.2.0 contract change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 042: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 042: add contract coverage proving audit-chain and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 042: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 043: in audit-chain, define the AsyncAPI 3.1.0 event change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 043: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 043: add integration coverage proving audit-chain and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 043: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 044: in audit-chain, define the proto3 port change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 044: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 044: add replay coverage proving audit-chain and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 044: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 045: in audit-chain, define the Postgres/RLS storage change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 045: audit-chain MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 045: add load coverage proving audit-chain and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 045: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 046: in audit-chain, define the audit-chain emission change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.

## Wave 15 counterpart evidence note

This IP is checked against `microservices/audit-chain/competitor-parity-matrix.md` and `microservices/audit-chain/feature-parity-matrix-2026-05-20.md`, not against line count. For the `j138 corporate audit investigation evidence trail` slice, the relevant counterpart gap is AWS CloudTrail / Google Cloud Audit Logs / Microsoft Purview Audit parity for searchable immutable audit history, plus Oyatie's additional tenant-verifiable Merkle proof path. The GitHub-pinned root and key manifests from `policy/seal-integrity.md` SI-04 and SI-11 are the evidence channel this implementation must preserve; if the slice cannot publish or verify through that channel, it remains below the Wave 15 substance bar.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-journey-j138-corporate-audit-investigation-evidence-trail.md` matched `SLO, multi-region, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/audit-chain/IP-journey-j138-corporate-audit-investigation-evidence-trail.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/audit-chain/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
