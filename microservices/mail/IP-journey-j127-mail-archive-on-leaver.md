---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j127-mail-archive-on-leaver
journey_id: j127-dual-tenant-identity-employee-resigns-and-keeps-personal
microservice: mail
role: mail-archive-on-leaver
status: draft
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0276-backup-portability-gdpr-art-20
  - ADR-0273-per-tenant-dkim-spf-dmarc
depends_on:
  - microservices/mail/IP-journey-j51-po-ingest-sender.md (baseline mail model)
date: 2026-05-20
owner_team: axis-mail + axis-hr
parallel_work_compatibility: Independent of j128 personal-tax-workflow Mail interactions
---

# IP-journey-j127-mail-archive-on-leaver — Mail µservice: archive leaver's mail under retention pack

## Goal

Implement mail surfaces that:

1. **`ArchiveAllForPrincipal`** — moves the leaver's inbox + sent + drafts
   to a tenant-archive bucket under the work tenant's retention pack
   (FedRAMP AU-2 = 7 years for Chen Aerospace).
2. **Auto-reply on receive** — incoming mail to `nadia.petrov@chen-aerospace.us`
   after revocation gets an auto-reply per the work-tenant's
   leaver-policy template (typical: "Nadia has left Chen Aerospace.
   Please direct queries to aleksandr.volkov@chen-aerospace.us").
3. **Per ADR-0273 DKIM** — auto-reply is DKIM-signed per the work
   tenant's domain key.

## Data model

```sql
-- Migration: 2026-05-20-001-mail-leaver-archive.sql

CREATE TABLE mail_leaver_archives (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id TEXT NOT NULL,
  principal_id TEXT NOT NULL,
  archive_bucket_uri TEXT NOT NULL,
  archived_count INT NOT NULL,
  archive_size_bytes BIGINT NOT NULL,
  retention_class TEXT NOT NULL,
  archive_completed_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL  -- per retention pack
);

CREATE TABLE mail_leaver_autoreply_config (
  tenant_id TEXT NOT NULL,
  principal_id TEXT NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT TRUE,
  reply_template TEXT NOT NULL,
  forward_to_principal_id TEXT,
  active_until TIMESTAMPTZ NOT NULL,  -- default +180 days
  PRIMARY KEY (tenant_id, principal_id)
);
```

## API surface

```protobuf
service MailOffboarding {
  rpc ArchiveAllForPrincipal (ArchiveAllForPrincipalRequest)
      returns (ArchiveAllForPrincipalResponse);
  rpc ConfigureAutoReply (ConfigureAutoReplyRequest)
      returns (ConfigureAutoReplyResponse);
}
```

## Files to author

| File | Purpose | Lines |
|---|---|---:|
| `microservices/mail/src/offboarding/archive_for_leaver.rs` | Archive sweep | ~280 |
| `microservices/mail/src/offboarding/autoreply.rs` | Auto-reply handler | ~220 |
| `microservices/mail/policy/offboarding-archive.cedar` | Cedar permit | ~30 |
| `microservices/mail/contracts/proto/offboarding.proto` | gRPC defs | ~100 |
| `microservices/mail/db/migrations/2026-05-20-001-mail-leaver-archive.sql` | DDL | ~50 |
| `microservices/mail/runbooks/leaver-mail-archive-incident.md` | Runbook | ~150 |
| `microservices/mail/tests/integration/offboarding_archive_test.rs` | Tests | ~360 |
| `microservices/mail/dashboards/leaver-archive-progress.json` | Grafana | ~80 |
| `microservices/mail/slos/leaver-archive-completion.openslo.yaml` | SLO | ~40 |

Total approximate: ~1,310 lines.

## Cedar fragments

```cedar
// offboarding-archive.cedar
permit (
  principal == Service::"workflow-engine",
  action == Action::"mail.archive_all_for_principal",
  resource is Tenant
) when {
  context.workflow_class == "offboarding"
};

// post-archive-mail-read.cedar — leaver cannot read archived mail
forbid (
  principal is User,
  action == Action::"mail.read_inbox",
  resource is User
) when {
  principal.id == resource.id &&
  resource.work_tenant_membership_status == "REVOKED"
};
```

## Integration contracts

| Contract | Direction | Notes |
|---|---|---|
| `audit-chain.EmitSealed` | mail → audit-chain | `MailArchivedForLeaver` |
| `comms-email.SendAutoReply` | mail → comms-email | Auto-reply transport |
| `s3.PutObject` | mail → s3-equivalent | Archive bucket write |

## Latency budget

- `ArchiveAllForPrincipal` (412 mails, ~80MB): ≤8s p99
- Per-mail archive: ≤25ms p99
- Auto-reply dispatch: ≤500ms p99

## Test plan

- Test A.3 — All 412 mails archived
- Test E.2 — Archive failure escalates (doesn't block cascade)

## Observability emissions

- `oya_mail_leaver_archive_total{tenant_id,outcome}`
- `oya_mail_leaver_autoreply_dispatched_total{tenant_id}`
- `oya_mail_leaver_archive_latency_ms`

## Acceptance criteria

- All 412 mails archived in test scenario.
- Auto-reply DKIM-valid per ADR-0273.
- Retention pack honored.

## Cross-references

- `docs/user-journeys/j127-*/handshake.md` §3.3
- ADR-0273 per-tenant DKIM

## Completion expansion — j127 mail IP rigor pass

Journey context: employee resignation where work access is revoked and personal tenant survives.
Service role: work-mail archive, notification cascade, and personal-mail refusal boundary.
Mapped services in this journey: identity, tenancy, messenger, mail, drive, workflow-engine.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0313, ADR-0317, ADR-0320.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in mail, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in mail, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in mail, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in mail, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in mail, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in mail, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in mail, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in mail, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in mail, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in mail, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in mail, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in mail, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in mail, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in mail, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in mail, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in mail, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in mail, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in mail, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 018: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 019: in mail, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 019: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 019: add pack-overlay coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 019: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 020: in mail, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 020: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 020: add unit coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 020: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 02: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 021: in mail, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 021: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 021: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 021: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 022: in mail, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 022: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 022: add contract coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 022: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 023: in mail, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 023: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 023: add integration coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 023: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 024: in mail, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 024: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 024: add replay coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 024: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 025: in mail, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 025: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 025: add load coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 025: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 026: in mail, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 026: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 026: add chaos coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 026: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 027: in mail, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 027: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 027: add negative authorization coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 027: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 028: in mail, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 028: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 028: add multi-region coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 028: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 029: in mail, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 029: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 029: add pack-overlay coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 029: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 030: in mail, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 030: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 030: add unit coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 030: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 03: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 031: in mail, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 031: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 031: add property coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 031: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 032: in mail, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 032: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 032: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 032: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 033: in mail, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 033: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 033: add integration coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 033: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 034: in mail, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 034: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 034: add replay coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 034: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 035: in mail, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 035: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 035: add load coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 035: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 036: in mail, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 036: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 036: add chaos coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 036: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 037: in mail, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 037: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 037: add negative authorization coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 037: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 038: in mail, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 038: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 038: add multi-region coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 038: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 039: in mail, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 039: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 039: add pack-overlay coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 039: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 040: in mail, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 040: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 040: add unit coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 040: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 04: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 041: in mail, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 041: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 041: add property coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 041: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 042: in mail, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 042: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 042: add contract coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 042: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 043: in mail, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 043: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 043: add integration coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 043: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 044: in mail, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 044: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 044: add replay coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 044: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 045: in mail, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 045: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 045: add load coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 045: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 046: in mail, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 046: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 046: add chaos coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 046: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 047: in mail, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 047: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 047: add negative authorization coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 047: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 048: in mail, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 048: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 048: add multi-region coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 048: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 049: in mail, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 049: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 049: add pack-overlay coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 049: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 050: in mail, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 050: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 050: add unit coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 050: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 05: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 051: in mail, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 051: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 051: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 051: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 052: in mail, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 052: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 052: add contract coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 052: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 053: in mail, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 053: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 053: add integration coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 053: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 054: in mail, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 054: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 054: add replay coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 054: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 055: in mail, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 055: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 055: add load coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 055: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 056: in mail, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 056: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 056: add chaos coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 056: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 057: in mail, define the dashboard projection change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 057: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 057: add negative authorization coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 057: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 058: in mail, define the runbook hook change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 058: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 058: add multi-region coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 058: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 059: in mail, define the integration fixture change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 059: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 059: add pack-overlay coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 059: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 060: in mail, define the domain model change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 060: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 060: add unit coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 060: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 06: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 061: in mail, define the Cedar policy change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 061: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 061: add property coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 061: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 062: in mail, define the OpenAPI 3.2.0 contract change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 062: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 062: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 062: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 063: in mail, define the AsyncAPI 3.1.0 event change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 063: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 063: add integration coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 063: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 064: in mail, define the proto3 port change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 064: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 064: add replay coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 064: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 065: in mail, define the Postgres/RLS storage change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 065: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 065: add load coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 065: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 066: in mail, define the audit-chain emission change for employee resignation where work access is revoked and personal tenant survives; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 066: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/mail/IP-journey-j127-mail-archive-on-leaver.md` matched `.proto`; contract files `microservices/mail/contracts/openapi/mail.yaml, microservices/mail/contracts/asyncapi/mail-events.yaml, microservices/mail/contracts/proto/mail.proto`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-journey-j127-mail-archive-on-leaver.md` matched `p99, SLO, multi-region`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/mail/IP-journey-j127-mail-archive-on-leaver.md` matched `emission`; anchors `microservices/mail/manifest.json, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
