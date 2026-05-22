---
doc_class: Implementation-Plan-Journey-Slice
journey_id: j142
microservice: mail
status: draft
date: 2026-05-20
authority_tier: 3
intern_buildable: true
adr_anchors: [ADR-0145, ADR-0244, ADR-0311]
---

# mail — IP slice for j142 (work-mail demotion + cross-tenant separation packet)

## Scope

1. **`Mailbox.Demote`** (work-tenant): block outbound, enable inbound auto-forward for 30d, install auto-reply.
2. **Cross-tenant separation-packet send**: from work-Mail to Chris's personal-Mail; carries 4 PDFs.
3. **Personal-tenant inbound receive** with `cross_tenant=true` banner.
4. **Auto-forward** of new inbound during 30d read-only window to personal-Mail (opt-in at hire time, default-on).

## API surface

```proto
service Mailbox {
  rpc Demote(DemoteRequest) returns (DemoteResponse);
  rpc Revoke(RevokeRequest) returns (RevokeResponse);
  rpc ConfigureAutoReply(AutoReplyRequest) returns (AutoReplyResponse);
  rpc ConfigureForwarding(ForwardingRequest) returns (ForwardingResponse);
}

service OutboundMail {
  rpc Send(SendRequest) returns (SendResponse);
  // Cross-tenant: same RPC but envelope carries source_tenant_id + dest_tenant_id
}

service InboundMail {
  rpc DeliverCrossTenant(DeliverCrossTenantRequest) returns (DeliverCrossTenantResponse);
}
```

## Domain model additions

```
struct Mailbox {
  principal: PrincipalId,
  tenant_id: TenantId,
  state: MailboxState,
  outbound_blocked: bool,
  inbound_forwarding: Option<ForwardingConfig>,
  auto_reply: Option<AutoReplyConfig>,
  demotion_expiry: Option<Instant>,
}

enum MailboxState { Active, Demoted, FullyRevoked }

struct ForwardingConfig {
  destination_address: EmailAddress,    // e.g., chris.volkov@<personal-tenant>
  banner_required: bool,                // "via former-employer-mail"
  retention_after_forwarding: Duration, // typically Zero (don't keep)
}
```

## Implementation tasks

### T1 — `Mailbox.Demote`

- Validate caller's Cedar permit `b2b.mail.demote`.
- Set `outbound_blocked=true`, `demotion_expiry=now+30d`.
- Idempotent.
- Audit: `MailReadOnlyDemoted{principal, tenant, expiry}`.

### T2 — `ConfigureAutoReply`

- Set `auto_reply` config (body, signature, exception_addresses for HR).
- Audit: `AutoReplyConfigured`.

### T3 — `ConfigureForwarding`

- Validate destination address is verified (personal-tenant mail address verified during onboarding).
- Set `inbound_forwarding`.
- Audit: `InboundForwardingConfigured{destination, retention}`.

### T4 — `OutboundMail.Send` cross-tenant variant

- Detect `dest_address` is in a different tenant than `from_address`.
- Build `cross_tenant_envelope{source_tenant, dest_tenant, purpose}`.
- Cedar check source: `b2b.mail.outbound.cross_tenant.{purpose}`.
- gRPC to dest-tenant `InboundMail.DeliverCrossTenant`.
- Audit double-seal: source emits `MailSentCrossTenant`; dest emits `MailDeliveredCrossTenant`.

### T5 — `InboundMail.DeliverCrossTenant` (personal-tenant side)

- Cedar check: `b2c.mail.inbound.from_known_employer_cross_tenant` — context evaluates whether the sender is a known prior counterparty.
- If PERMIT: place in inbox with `cross_tenant_banner="from your former employer"`.
- If DENY: route to quarantine; user can release.
- Audit: `MailDeliveredCrossTenant` or `MailQuarantinedCrossTenant`.

### T6 — Forward-on-receive (during 30d window)

- When inbound arrives at demoted mailbox:
  - If `inbound_forwarding` set: forward to destination with banner.
  - Audit: `MailAutoForwarded{from_addr, to_personal_addr}`.
  - Original message still persisted at work-tenant for read-only viewing.

## Cedar permits

| Permit | Granted to | Purpose |
|---|---|---|
| `b2b.mail.demote` | workflow-engine | Demote work mailbox |
| `b2b.mail.revoke` | workflow-engine | Final revocation at T+30d |
| `b2b.mail.outbound.cross_tenant.mandated_layoff_communication` | workflow-engine | Send separation packet |
| `b2c.mail.inbound.from_known_employer_cross_tenant` | personal-tenant default | Accept compliance-mandated email |

## Audit emissions

- `MailReadOnlyDemoted`, `MailFullyRevoked`
- `AutoReplyConfigured`, `InboundForwardingConfigured`
- `MailSentCrossTenant`, `MailDeliveredCrossTenant`, `MailQuarantinedCrossTenant`
- `MailAutoForwarded`

## Performance

- `Demote` p99 ≤ 50ms.
- `OutboundMail.Send` cross-tenant p99 ≤ 5s (delivery to dest mailbox).
- `InboundMail.DeliverCrossTenant` p99 ≤ 2s.

## Acceptance criteria

- [ ] Mailbox demote blocks outbound immediately; allows inbound read for 30d.
- [ ] Cross-tenant separation packet delivers within 60s SLA (per integration B.1).
- [ ] Personal-Mail receives with `cross_tenant_banner` rendered.
- [ ] Auto-forward of new inbound during 30d window works with banner.
- [ ] Anti-leak: work-tenant cannot query personal-Mail content (B.2).

## Out of scope

- Personal-Mail content rendering (UX layer).
- Connect-mediated COBRA/ERISA submission (Connect µservice).
- Spam filtering policy authoring (lives in compliance).

## Completion expansion — j142 mail IP rigor pass

Journey context: employee-side day-zero layoff with work revocation and personal continuity.
Service role: work-mail archive, notification cascade, and personal-mail refusal boundary.
Mapped services in this journey: identity, tenancy, workflow-engine, mail, meet, payments, messenger, drive.
ADR anchors: ADR-0244, ADR-0292, ADR-0299, ADR-0311, ADR-0317, ADR-0320.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in mail, define the Cedar policy change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in mail, define the OpenAPI 3.2.0 contract change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in mail, define the AsyncAPI 3.1.0 event change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in mail, define the proto3 port change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving mail and meet agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in mail, define the Postgres/RLS storage change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in mail, define the audit-chain emission change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in mail, define the dashboard projection change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in mail, define the runbook hook change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in mail, define the integration fixture change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in mail, define the domain model change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in mail, define the Cedar policy change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in mail, define the OpenAPI 3.2.0 contract change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving mail and meet agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in mail, define the AsyncAPI 3.1.0 event change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in mail, define the proto3 port change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in mail, define the Postgres/RLS storage change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in mail, define the audit-chain emission change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in mail, define the dashboard projection change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in mail, define the runbook hook change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 018: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 019: in mail, define the integration fixture change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 019: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 019: add pack-overlay coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 019: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 020: in mail, define the domain model change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 020: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 020: add unit coverage proving mail and meet agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 020: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 02: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 021: in mail, define the Cedar policy change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 021: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 021: add property coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 021: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 022: in mail, define the OpenAPI 3.2.0 contract change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 022: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 022: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 022: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 023: in mail, define the AsyncAPI 3.1.0 event change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 023: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 023: add integration coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 023: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 024: in mail, define the proto3 port change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 024: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 024: add replay coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 024: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 025: in mail, define the Postgres/RLS storage change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 025: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 025: add load coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 025: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 026: in mail, define the audit-chain emission change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 026: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 026: add chaos coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 026: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 027: in mail, define the dashboard projection change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 027: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 027: add negative authorization coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 027: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 028: in mail, define the runbook hook change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 028: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 028: add multi-region coverage proving mail and meet agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 028: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 029: in mail, define the integration fixture change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 029: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 029: add pack-overlay coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 029: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 030: in mail, define the domain model change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 030: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 030: add unit coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 030: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 03: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 031: in mail, define the Cedar policy change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 031: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 031: add property coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 031: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 032: in mail, define the OpenAPI 3.2.0 contract change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 032: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 032: add contract coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 032: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 033: in mail, define the AsyncAPI 3.1.0 event change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 033: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 033: add integration coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 033: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 034: in mail, define the proto3 port change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 034: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 034: add replay coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 034: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 035: in mail, define the Postgres/RLS storage change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 035: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 035: add load coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 035: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 036: in mail, define the audit-chain emission change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 036: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 036: add chaos coverage proving mail and meet agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 036: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 037: in mail, define the dashboard projection change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 037: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 037: add negative authorization coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 037: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 038: in mail, define the runbook hook change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 038: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 038: add multi-region coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 038: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 039: in mail, define the integration fixture change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 039: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 039: add pack-overlay coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 039: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 040: in mail, define the domain model change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 040: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 040: add unit coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 040: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 04: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 041: in mail, define the Cedar policy change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 041: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 041: add property coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 041: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 042: in mail, define the OpenAPI 3.2.0 contract change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 042: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 042: add contract coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 042: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 043: in mail, define the AsyncAPI 3.1.0 event change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 043: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 043: add integration coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 043: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 044: in mail, define the proto3 port change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 044: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 044: add replay coverage proving mail and meet agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 044: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 045: in mail, define the Postgres/RLS storage change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 045: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 045: add load coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 045: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 046: in mail, define the audit-chain emission change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 046: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 046: add chaos coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 046: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 047: in mail, define the dashboard projection change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 047: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 047: add negative authorization coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 047: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 048: in mail, define the runbook hook change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 048: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 048: add multi-region coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 048: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 049: in mail, define the integration fixture change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 049: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 049: add pack-overlay coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 049: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 050: in mail, define the domain model change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 050: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 050: add unit coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 050: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 05: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 051: in mail, define the Cedar policy change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 051: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 051: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 051: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 052: in mail, define the OpenAPI 3.2.0 contract change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 052: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 052: add contract coverage proving mail and meet agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 052: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 053: in mail, define the AsyncAPI 3.1.0 event change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 053: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 053: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 053: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 054: in mail, define the proto3 port change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 054: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 054: add replay coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 054: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 055: in mail, define the Postgres/RLS storage change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 055: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 055: add load coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 055: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 056: in mail, define the audit-chain emission change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 056: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 056: add chaos coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 056: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 057: in mail, define the dashboard projection change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 057: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 057: add negative authorization coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 057: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 058: in mail, define the runbook hook change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 058: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 058: add multi-region coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 058: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 059: in mail, define the integration fixture change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 059: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 059: add pack-overlay coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 059: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 060: in mail, define the domain model change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 060: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 060: add unit coverage proving mail and meet agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 060: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 06: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 061: in mail, define the Cedar policy change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 061: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 061: add property coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 061: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 062: in mail, define the OpenAPI 3.2.0 contract change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 062: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 062: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 062: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 063: in mail, define the AsyncAPI 3.1.0 event change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 063: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 063: add integration coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 063: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 064: in mail, define the proto3 port change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 064: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 064: add replay coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 064: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 065: in mail, define the Postgres/RLS storage change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 065: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 065: add load coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 065: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 066: in mail, define the audit-chain emission change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 066: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 066: add chaos coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 066: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 067: in mail, define the dashboard projection change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 067: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0292 scope grants it; refusals are success states, not exceptions.
Verification 067: add negative authorization coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 067: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 068: in mail, define the runbook hook change for employee-side day-zero layoff with work revocation and personal continuity; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 068: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 068: add multi-region coverage proving mail and meet agree on contract version, Cedar decision, audit event class, and replay cursor.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-journey-j142-work-mail-demotion-and-cross-tenant-packet.md` matched `p99, SLO, multi-region, payment`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/mail/IP-journey-j142-work-mail-demotion-and-cross-tenant-packet.md` matched `emission`; anchors `microservices/mail/manifest.json, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
