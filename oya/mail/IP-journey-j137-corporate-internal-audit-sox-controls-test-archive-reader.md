---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j137-mail-archive-reader
journey_id: j137-corporate-internal-audit-sox-controls-test
microservice: mail
role: archive-reader
status: draft
date: 2026-05-20
authority_tier: 3
owner_team: axis-mail + axis-internal-audit
parallel_work_compatibility: independent of messenger-IP; depends on identity B2B_INTERNAL_AUDIT resolver
related_adrs: [ADR-0311, ADR-0313, ADR-0243, ADR-0244, ADR-0028, ADR-0263, ADR-0145, ADR-0310]
related_journey_artifacts:
  - docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/handshake.md (Phase 2)
  - docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/schemas/sox-audit-sample-request.json
  - docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/schemas/sox-control-evidence-bundle.json
depends_on:
  - microservices/identity/IP-journey-j137-corporate-internal-audit-sox-controls-test-permit-resolver.md
  - microservices/audit-chain/IP-journey-j137-corporate-internal-audit-sox-controls-test-evidence-bundler.md
---

# IP-journey-j137-mail-archive-reader — Mail: tenant-scoped archive read for SOX internal audit

## Goal

Implement the `mail.read_tenant_archive` surface that a
B2B_INTERNAL_AUDIT principal can use to pull email threads from the
work-tenant Mail archive. Mirror of messenger's archive-reader but
adapted for Mail's IMAP-class persistence model + ECPA 1986 §2701
employer-stored-communications doctrine (which says employer can
lawfully access work email; oyatie scopes this via Cedar so the
default-deny still holds for personal-tenant principals).

## Data model

| Object | Storage | Schema | TTL |
|---|---|---|---|
| `MailThread` (tenant-owned) | Postgres `mail.archive_threads` partitioned by `tenant_id, year_month` | per-tenant Avro v2 | 7y SOX retention |
| `MailMessage` | Postgres `mail.archive_messages` (one row per message) | per-tenant Avro v2 | 7y |
| `MailAttachment` (encrypted) | Object storage `mail-archive-attachments` with envelope keys | binary | 7y |
| `MailParticipantClassMap` | Postgres `mail.participant_class_view` materialized from identity | view | refreshed on principal-change |
| `MailAuditReadAttempt` | Postgres `mail_audit_read_attempts` | per-read row | 7y |

## Schema mapping

Postgres archive schema:

```sql
CREATE TABLE mail.archive_threads (
  thread_id UUID PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  cell_tier INTEGER NOT NULL,
  participants JSONB NOT NULL,           -- [{principal, class}]
  classification_window_start TIMESTAMPTZ NOT NULL,
  classification_window_end   TIMESTAMPTZ NOT NULL,
  message_count INTEGER NOT NULL,
  subject_encrypted BYTEA NOT NULL,
  body_root_hash TEXT NOT NULL,
  pack_set TEXT[] NOT NULL,
  sealed_at TIMESTAMPTZ NOT NULL,
  trace_id TEXT NOT NULL
) PARTITION BY LIST (tenant_id);

CREATE TABLE mail.archive_messages (
  message_id UUID PRIMARY KEY,
  thread_id UUID NOT NULL REFERENCES mail.archive_threads(thread_id),
  tenant_id TEXT NOT NULL,
  from_principal TEXT NOT NULL,
  from_principal_class TEXT NOT NULL,
  to_principals JSONB NOT NULL,
  cc_principals JSONB,
  bcc_principals JSONB,
  sent_at TIMESTAMPTZ NOT NULL,
  subject_encrypted BYTEA NOT NULL,
  body_encrypted BYTEA NOT NULL,
  body_key_envelope BYTEA NOT NULL,
  attachment_refs TEXT[],
  merkle_leaf_hash TEXT NOT NULL
) PARTITION BY LIST (tenant_id);

CREATE INDEX idx_mail_thread_tenant_window ON mail.archive_threads(tenant_id, classification_window_start, classification_window_end);
CREATE INDEX idx_mail_msg_from ON mail.archive_messages(tenant_id, from_principal, sent_at DESC);
```

Audit-read-attempts table mirrors messenger pattern; same shape.

## API surface (gRPC)

```protobuf
syntax = "proto3";
package oyatie.mail.audit.v1;

service MailArchive {
  rpc ReadTenantArchive (ReadTenantArchiveRequest) returns (ReadTenantArchiveResponse);
  rpc ReadCorrelatedPrincipals (ReadCorrelatedPrincipalsRequest) returns (ReadCorrelatedPrincipalsResponse);
}

message ReadTenantArchiveRequest {
  string audit_case_id = 1;
  string tenant_id = 2;
  ParticipantFilter participants = 3;
  KeywordFilter keywords = 4;
  TimeWindow window = 5;
  string requestor_principal = 6;
  string permit_batch_ref = 7;
  string trace_id = 8;
  bool include_attachments = 9;  // Cedar gates attachments separately
}

message ReadTenantArchiveResponse {
  repeated MailThreadEvidence threads = 1;
  PersonalTenantDenySummary personal_tenant_deny_summary = 2;
  string audit_seal_id = 3;
  google.protobuf.Timestamp evaluated_at = 4;
}

message MailThreadEvidence {
  string thread_id = 1;
  string tenant_id = 2;
  string subject_decrypted = 3;
  repeated MailMessageEvidence messages = 4;
  repeated string attachment_refs = 5;
  string merkle_leaf_hash = 6;
}
```

OpenAPI surface for the REST endpoint (used by the audit pane):

```yaml
openapi: 3.2.0
paths:
  /api/v1/mail/archive/audit-read:
    post:
      operationId: mailArchiveAuditRead
      security: [{ b2b_internal_audit_passkey: [] }]
      tags: [mail, internal-audit]
      requestBody:
        content:
          application/json:
            schema:
              $ref: "../schemas/sox-audit-sample-request.json"
      responses:
        "200":
          description: Tenant-scoped archive evidence returned.
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/MailArchiveAuditResponse"
        "403":
          description: Cedar policy denied the read.
```

## Cedar policy

```cedar
@id("mail-read-tenant-archive-v1")
permit (
  principal,
  action == Action::"mail.read_tenant_archive",
  resource is MailThread
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  principal.audit_case_id != null &&
  resource.tenant_id == principal.permit_scope.tenant_id &&
  resource.classification_window.intersects(principal.permit_scope.window) &&
  context.dual_control_approval_at != null &&
  context.audit_charter_active == true
};

@id("mail-personal-tenant-deny-v1")
forbid (
  principal,
  action == Action::"mail.read_tenant_archive",
  resource is MailThread
) when {
  resource.tenant_id != principal.permit_scope.tenant_id ||
  resource.from_principal_class == "personal_tenant_owned"
};

@id("mail-attachment-read-v1")
permit (
  principal,
  action == Action::"mail.read_attachment",
  resource is MailAttachment
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  resource.parent_thread.tenant_id == principal.permit_scope.tenant_id &&
  context.attachment_read_explicit_consent == true
};
```

## Integration contracts

### Upstream

- `workflow-engine.audit_sample_planner` (primary).
- `ops-dashboard-control-center.audit_pane`.

### Downstream

- `identity.B2BInternalAuditPrincipalResolver`.
- `audit-chain.SealLeaf`.
- `compliance.PackOverlayResolver`.
- `cloud-secrets.GetEnvelopeKey` for decryption.
- `observability` (OTLP).

### Cross-tenant invariants

- Partition-level filtering by `tenant_id` enforced in Postgres
  partition routing.
- Cedar deny on `from_principal_class == "personal_tenant_owned"`
  even if `to_principals` contains work-tenant principals (the FROM
  is the signal).

## Implementation notes

### ECPA §2701 alignment

US ECPA 1986 §2701 lets an employer access employee work mail (the
employer is the "provider" of the email service). oyatie's
Cedar permit grammar codifies this: work-tenant mail is permit-able
by B2B_INTERNAL_AUDIT principal, but personal-tenant mail is NOT —
even when an employee sends from their personal mail TO a work
recipient. The work-tenant mail SERVER stores only the work side
of the conversation; the personal-tenant mail (sender's stored copy)
is not in the work tenant. If a personal-tenant principal SENT a
mail to a work-tenant recipient, the work-tenant copy of that mail
is in the work archive (and audit-readable) but the personal-tenant
sender's principal-class is `personal_tenant_owned` and the audit
gets only message metadata WITHIN the work-tenant copy — never
the personal-tenant sender's outbox.

### EU Whistleblower Directive 2019/1937 overlay

When a counterparty or recipient is EU-based, the
`pack-eu-whistleblower-2019-1937` overlay activates additional
context attributes that the Cedar policy checks. Specifically,
threads tagged as `potential_whistleblower_communication=true` by
the detection-substrate (ADR-0307) require a separate Cedar context
flag `whistleblower_review_authorized=true` to be readable — the
ordinary B2B_INTERNAL_AUDIT permit alone is not sufficient.

### Attachment encryption

Attachments are stored encrypted in object storage with envelope
keys per-tenant per-thread. The mail archive reader requests the
envelope key from `cloud-secrets` only after the Cedar permit
PERMITs the attachment read. If the Cedar permit DENIES, no key
is fetched and the attachment ref is omitted from the response.

### Performance budget

- `ReadTenantArchive` p95 ≤ 250ms for thread sets ≤ 50 messages.
- Attachment decryption p95 ≤ 100ms per attachment ≤ 10MB.
- End-to-end sample-pull contribution from mail: ≤ 1.5s p95.

### Brownout behaviour

If Postgres `mail.archive_*` is in brownout, return 503 with
`Retry-After`. workflow-engine pause-resumes.

## Test plan

See integration-test-plan.md §3.1, §3.2, §4.1–4.5.

Unit tests:
- `test_mail_cedar_permit_required`
- `test_mail_personal_tenant_from_denies`
- `test_mail_attachment_decryption_only_after_permit`
- `test_mail_whistleblower_pack_blocks_without_explicit_consent`
- `test_mail_audit_seal_per_read_attempt`
- `test_mail_classification_window_filter_correct`
- `test_mail_cross_tenant_partition_filter`

Property tests:
- Property: no personal-tenant principal id appears in response body.
- Property: every response has exactly one audit-seal-id.
- Property: every attachment ref appears only if Cedar permit allowed.

## Build sequence

1. Author Cedar policies.
2. Schema migration `mail-2026-q2-q3-add-participant-classes`.
3. Backfill `from_principal_class` column.
4. Implement gRPC service.
5. Wire Cedar gate.
6. Wire audit-chain seal + observability.
7. Add unit + property + integration tests.
8. Wire into workflow-engine.audit_sample_planner.

## Acceptance gates

- Unit + integration tests PASS.
- Cedar policy lint clean.
- Schema migration verified.
- Code review: axis-mail + axis-internal-audit.
- Multispectrum review v2.4.0 facets F1/F2/F3/M1/A1/A4/A5.

## Operational notes

- Owner: axis-mail (primary) + axis-internal-audit.
- Pager: `oya-mail-audit-reader`.
- Grafana dashboard: `mail-audit-reader`.
- Runbook: `microservices/mail/runbooks/audit-archive-reader.md`.

## Compliance and pack overlays

Same pack composition as messenger (see messenger IP §"Compliance
and pack overlays") plus ECPA-2701-alignment notes baked into the
policy fragment.

## Cross-microservice port declaration

`MailArchive` gRPC service in namespace `oyatie.mail.audit.v1` per
ADR-0145. Proto at `protos/mail-audit-v1.proto`.

## Roll-out plan

Same five phases as messenger archive-reader; coordinated rollout
so both services land together (workflow-engine.audit_sample_planner
depends on BOTH).

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Personal-tenant sender leaks via "to_principals" reverse-correlation | CRITICAL | Cedar deny on from_principal_class + property test |
| Attachment key envelope cache attack | HIGH | Per-read fresh key fetch; cache TTL ≤ 60s |
| EU-WB overlay misapplied | MEDIUM | Pack overlay reference tests |
| Postgres partition mis-routing | HIGH | Lane test for cross-tenant query patterns |
| BCC field leakage | HIGH | Cedar policy includes BCC in resource principal-class evaluation |

## Definition of done

- gRPC service in production behind feature flag.
- All tests PASS.
- Observability dashboard live.
- Runbook authored.
- The j137 audit-pane integration end-to-end test PASS with
  synthetic fixtures.
- Personal-tenant deny invariant holds across all integration tests.
- Cross-µservice port contract test PASS in the audit-pull pipeline.
- External-auditor verification path (PwC mock) succeeds end-to-end.

## Completion expansion — j137 mail IP rigor pass

Journey context: quarterly SOX 404 audit of work surfaces only.
Service role: work-mail archive, notification cascade, and personal-mail refusal boundary.
Mapped services in this journey: messenger, mail, workflow-engine, payments, audit-chain, ops-dashboard-control-center, identity, compliance.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0312, ADR-0313, ADR-0319.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in mail, define the Cedar policy change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in mail, define the OpenAPI 3.2.0 contract change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in mail, define the AsyncAPI 3.1.0 event change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in mail, define the proto3 port change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving mail and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in mail, define the Postgres/RLS storage change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving mail and ops-dashboard-control-center agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in mail, define the audit-chain emission change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in mail, define the dashboard projection change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving mail and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in mail, define the runbook hook change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in mail, define the integration fixture change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in mail, define the domain model change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in mail, define the Cedar policy change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in mail, define the OpenAPI 3.2.0 contract change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving mail and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in mail, define the AsyncAPI 3.1.0 event change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving mail and ops-dashboard-control-center agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in mail, define the proto3 port change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in mail, define the Postgres/RLS storage change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving mail and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in mail, define the audit-chain emission change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in mail, define the dashboard projection change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in mail, define the runbook hook change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/mail/IP-journey-j137-corporate-internal-audit-sox-controls-test-archive-reader.md` matched `openapi, .proto`; contract files `microservices/mail/contracts/openapi/mail.yaml, microservices/mail/contracts/asyncapi/mail-events.yaml, microservices/mail/contracts/proto/mail.proto`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-journey-j137-corporate-internal-audit-sox-controls-test-archive-reader.md` matched `SLO, multi-region, payment`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/mail/IP-journey-j137-corporate-internal-audit-sox-controls-test-archive-reader.md` matched `emission`; anchors `microservices/mail/manifest.json, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
