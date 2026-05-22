---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j138-mail-targeted-correspondence-pull
journey_id: j138-corporate-audit-fraud-investigation-via-pattern-detection
microservice: mail
role: targeted-correspondence-pull
status: draft
date: 2026-05-20
authority_tier: 3
owner_team: axis-mail + axis-internal-audit
parallel_work_compatibility: extends j137 archive-reader with investigation-mode filters
related_adrs: [ADR-0311, ADR-0310, ADR-0307, ADR-0028, ADR-0145]
depends_on:
  - microservices/mail/IP-journey-j137-corporate-internal-audit-sox-controls-test-archive-reader.md
---

# IP-journey-j138-mail-targeted-correspondence-pull — Mail: investigation-mode targeted thread pull

## Goal

Extend the mail.MailArchive surface to support targeted-pull semantics
needed for fraud investigations: thread filtering by subject keyword,
participant principal, vendor reference, and date window — while
maintaining the personal-tenant deny invariant per ADR-0311.

## Data model

Inherits from j137 IP. New schema fields:

- `mail.archive_threads.investigation_tags` (TEXT[]) — auto-tags
  applied when a thread is correlated to an investigation case.
- `mail.archive_threads.vendor_refs` (TEXT[]) — auto-extracted vendor
  references from subject/body for fast investigation filter.

## Schema mapping

```sql
ALTER TABLE mail.archive_threads
  ADD COLUMN investigation_tags TEXT[] DEFAULT '{}',
  ADD COLUMN vendor_refs TEXT[] DEFAULT '{}';

CREATE INDEX idx_mail_thread_vendor_refs ON mail.archive_threads USING GIN(vendor_refs);
CREATE INDEX idx_mail_thread_investigation_tags ON mail.archive_threads USING GIN(investigation_tags);
```

## API surface (gRPC)

```protobuf
service MailArchive {
  // existing from j137
  rpc ReadTenantArchive (ReadTenantArchiveRequest) returns (ReadTenantArchiveResponse);
  // new for j138
  rpc ReadInvestigationCorrespondence (ReadInvestigationCorrespondenceRequest) returns (ReadInvestigationCorrespondenceResponse);
}

message ReadInvestigationCorrespondenceRequest {
  string investigation_case_id = 1;
  string tenant_id = 2;
  repeated string vendor_refs = 3;
  repeated string subject_principals = 4;
  repeated string keywords = 5;
  TimeWindow window = 6;
  string requestor_principal = 7;
  string permit_batch_ref = 8;
  bool include_attachments = 9;
}

message ReadInvestigationCorrespondenceResponse {
  repeated MailThreadEvidence threads = 1;
  PersonalTenantDenySummary personal_tenant_deny_summary = 2;
  string audit_seal_id = 3;
  uint32 total_thread_count = 4;
  uint32 total_message_count = 5;
}
```

## Cedar policy

```cedar
@id("mail-read-investigation-correspondence-v1")
permit (
  principal,
  action == Action::"mail.read_investigation_correspondence",
  resource is MailThread
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  principal.investigation_case_id != null &&
  resource.tenant_id == principal.permit_scope.tenant_id &&
  resource.classification_window.intersects(principal.permit_scope.window)
};

@id("mail-investigation-personal-tenant-deny-v1")
forbid (
  principal,
  action == Action::"mail.read_investigation_correspondence",
  resource is MailThread
) when {
  resource.tenant_id != principal.permit_scope.tenant_id ||
  resource.from_principal_class == "personal_tenant_owned"
};
```

## Integration contracts

### Upstream

- `workflow-engine.investigation_orchestrator`.
- `ops-dashboard.audit-pane`.

### Downstream

- `identity` for principal context + classification.
- `audit-chain.SealLeaf`.
- `cloud-secrets.GetEnvelopeKey` for decryption.

## Implementation notes

### Vendor-ref extraction

Background job extracts vendor references from message subject/body
using regex against `payments.vendors.vendor_id` namespace. Indexes
maintained nightly + incrementally on new mail receipt.

### Investigation tag application

When an investigation case is created, the workflow-engine sends a
`tag-correspondence` request that asynchronously tags matching
threads. Tags are computed via:

- `vendor_refs` ∩ investigation.target_vendor_refs
- `participants` ∩ investigation.target_principals
- `subject` matches investigation.keyword_set

### Targeted-pull semantics

Unlike j137's broad sample-pull, j138's investigation pull is
targeted. Filters are AND-composed; result set typically 10x smaller
than SOX audit pulls.

### Personal-tenant invariant

Same as j137 — personal-tenant from_principal_class triggers deny.
Count-only return.

## Performance budget

- `ReadInvestigationCorrespondence` p95 ≤ 1.5s for filter sets ≤ 100
  threads.

## Test plan

See integration-test-plan.md §4.2.

Unit tests:
- `test_investigation_filter_correctness`
- `test_vendor_ref_extraction_accurate`
- `test_personal_tenant_from_denies_even_in_investigation`
- `test_investigation_tag_application_async`

## Build sequence

1. Schema migration `mail-2026-q3-add-investigation-tags-vendor-refs`.
2. Background vendor-ref-extraction job.
3. Cedar policy.
4. gRPC service.
5. Investigation-tag-application worker.
6. Tests.

## Acceptance gates

- All tests PASS.
- Cedar lint clean.
- Schema migration applied.
- Code review: axis-mail + axis-internal-audit.

## Operational notes

- Owner: axis-mail.
- Pager: `oya-mail-investigation`.
- Dashboards: `mail-investigation-read-rate`,
  `vendor-ref-extraction-latency`.

## Compliance / packs

- `pack-us-sox-404` + `pack-fcpa-1977` + `pack-eu-whistleblower-2019-1937`.
- ECPA §2701 alignment maintained.

## Cross-microservice port declaration

Per ADR-0145; `MailArchive.ReadInvestigationCorrespondence` in
`oyatie.mail.audit.v1`.

## Roll-out plan

Same five-phase rollout.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Vendor-ref extraction false-positive | MED | Confidence-scored extraction + whitelist |
| Tag-application lag | LOW | SLA on async worker |
| Personal-tenant correlation false-positive | HIGH | Cedar gate + property test |
| Cross-investigation tag-leak | HIGH | Per-case tag namespace |

## Definition of done

- Service live behind flag.
- All tests PASS.
- AcmeWire investigation pulls correct 47-thread set.
- Personal-tenant deny holds in investigation mode.
- Tag-application worker runs reliably at scale.

## Completion expansion — j138 mail IP rigor pass

Journey context: payroll anomaly detection triggers case-managed vendor-payment fraud investigation.
Service role: work-mail archive, notification cascade, and personal-mail refusal boundary.
Mapped services in this journey: observability, payments, workflow-engine, mail, audit-chain, community.
ADR anchors: ADR-0244, ADR-0297, ADR-0299, ADR-0310, ADR-0311, ADR-0319.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in mail, define the Cedar policy change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in mail, define the OpenAPI 3.2.0 contract change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in mail, define the AsyncAPI 3.1.0 event change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in mail, define the proto3 port change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving mail and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in mail, define the Postgres/RLS storage change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in mail, define the audit-chain emission change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving mail and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in mail, define the dashboard projection change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in mail, define the runbook hook change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in mail, define the integration fixture change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in mail, define the domain model change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving mail and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in mail, define the Cedar policy change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in mail, define the OpenAPI 3.2.0 contract change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving mail and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in mail, define the AsyncAPI 3.1.0 event change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in mail, define the proto3 port change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in mail, define the Postgres/RLS storage change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in mail, define the audit-chain emission change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving mail and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in mail, define the dashboard projection change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in mail, define the runbook hook change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving mail and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 018: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 019: in mail, define the integration fixture change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 019: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 019: add pack-overlay coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 019: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 020: in mail, define the domain model change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 020: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 020: add unit coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 020: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 02: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 021: in mail, define the Cedar policy change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 021: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 021: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 021: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 022: in mail, define the OpenAPI 3.2.0 contract change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 022: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 022: add contract coverage proving mail and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 022: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 023: in mail, define the AsyncAPI 3.1.0 event change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 023: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 023: add integration coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 023: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 024: in mail, define the proto3 port change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 024: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 024: add replay coverage proving mail and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 024: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 025: in mail, define the Postgres/RLS storage change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 025: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 025: add load coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 025: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 026: in mail, define the audit-chain emission change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 026: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 026: add chaos coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 026: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 027: in mail, define the dashboard projection change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 027: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 027: add negative authorization coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 027: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 028: in mail, define the runbook hook change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 028: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 028: add multi-region coverage proving mail and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 028: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 029: in mail, define the integration fixture change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 029: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 029: add pack-overlay coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 029: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 030: in mail, define the domain model change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 030: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 030: add unit coverage proving mail and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 030: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 03: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 031: in mail, define the Cedar policy change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 031: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 031: add property coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 031: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 032: in mail, define the OpenAPI 3.2.0 contract change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 032: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 032: add contract coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 032: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 033: in mail, define the AsyncAPI 3.1.0 event change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 033: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 033: add integration coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 033: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 034: in mail, define the proto3 port change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 034: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 034: add replay coverage proving mail and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 034: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 035: in mail, define the Postgres/RLS storage change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 035: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 035: add load coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 035: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 036: in mail, define the audit-chain emission change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 036: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 036: add chaos coverage proving mail and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 036: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 037: in mail, define the dashboard projection change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 037: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 037: add negative authorization coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 037: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 038: in mail, define the runbook hook change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 038: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 038: add multi-region coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 038: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 039: in mail, define the integration fixture change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 039: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 039: add pack-overlay coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 039: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 040: in mail, define the domain model change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 040: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 040: add unit coverage proving mail and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 040: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 04: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 041: in mail, define the Cedar policy change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 041: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 041: add property coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 041: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 042: in mail, define the OpenAPI 3.2.0 contract change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 042: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 042: add contract coverage proving mail and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 042: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 043: in mail, define the AsyncAPI 3.1.0 event change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 043: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 043: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 043: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 044: in mail, define the proto3 port change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 044: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 044: add replay coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 044: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 045: in mail, define the Postgres/RLS storage change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 045: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0310 scope grants it; refusals are success states, not exceptions.
Verification 045: add load coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 045: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 046: in mail, define the audit-chain emission change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 046: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 046: add chaos coverage proving mail and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 046: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 047: in mail, define the dashboard projection change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 047: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 047: add negative authorization coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 047: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 048: in mail, define the runbook hook change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 048: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 048: add multi-region coverage proving mail and observability agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 048: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 049: in mail, define the integration fixture change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 049: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0297 scope grants it; refusals are success states, not exceptions.
Verification 049: add pack-overlay coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 049: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 050: in mail, define the domain model change for payroll anomaly detection triggers case-managed vendor-payment fraud investigation; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 050: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 050: add unit coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-journey-j138-corporate-audit-targeted-correspondence-pull.md` matched `SLO, multi-region, payment`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/mail/IP-journey-j138-corporate-audit-targeted-correspondence-pull.md` matched `emission`; anchors `microservices/mail/manifest.json, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
