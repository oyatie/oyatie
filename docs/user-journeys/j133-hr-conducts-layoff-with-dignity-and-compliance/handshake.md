---
doc_class: User-Journey-Handshake
journey_id: j133-hr-conducts-layoff-with-dignity-and-compliance
status: draft
date: 2026-05-20
related_adrs: [ADR-0311, ADR-0244, ADR-0263, ADR-0246, ADR-0247]
µservices_touched: [workflow-engine, mail, messenger, payments, finops-portal, identity, tenancy, community, drive, compliance]
---

# j133 — Handshake: 10-µservice RIF cascade

## Phase 0 — Pre-RIF state

- Marcus approved 4% workforce reduction at T-14 days via finops-portal.
- Workflow-Engine holds `rif-event-v3` workflow definition; pinned to compliance packs per jurisdiction.
- Compliance pack registry: pack-us-warn-act, pack-eu-anti-discrimination-baseline, pack-eu-works-council-baseline, pack-de-kschg-baseline, pack-kr-labor-standards-act-amendment, pack-in-industrial-disputes-act, pack-us-litigation-hold-baseline.
- Tenancy holds works-council recipient lists for DE-BER + labor-management-council for KR-SEO.
- Audit-chain Merkle root current.

## Phase 1 — Pre-announcement (T-14 to T-1 day)

### Sequence (selection + analysis)

```
Priya  api-gw  workflow-engine  intelligence  compliance  tenancy  audit-chain
  │      │           │                │             │           │            │
  │POST  │           │                │             │           │            │
  │ rif/ │           │                │             │           │            │
  │plan  │           │                │             │           │            │
  ├─────►│           │                │             │           │            │
  │      │ Cedar     │                │             │           │            │
  │      ├──────────►│                │             │           │            │
  │      │           │ compose select │             │           │            │
  │      │           │ run disparate  │             │           │            │
  │      │           │ impact         │             │           │            │
  │      │           ├───────────────►│             │           │            │
  │      │           │ DEI result     │             │           │            │
  │      │           │◄───────────────┤             │           │            │
  │      │           │ resolve pack   │             │           │            │
  │      │           ├──────────────────────────────►│           │            │
  │      │           │ resolve sub-tenants           │           │            │
  │      │           ├─────────────────────────────────────────►│            │
  │      │           │ emit sealed   │             │           │            │
  │      │           ├──────────────────────────────────────────────────────►│
  │      │           │ notify works-council (DE-BER, T-7d)     │            │
  │      │           ├────────────────────────────────────────►│            │
  │ 200  │           │                │             │           │            │
  │◄─────┤           │                │             │           │            │
```

### Per-step table

| Step | Caller | Callee | RPC | Schema | Cedar permit | Audit event | Failure-mode |
|---|---|---|---|---|---|---|---|
| 1.1 | Priya | api-gw | POST /api/v1/hr/rif/plan | rif-plan-req | b2b.hr.rif_plan | n/a | gateway down → banner |
| 1.2 | api-gw | workflow-engine | gRPC PlanRif | rif-plan-spec | b2b.hr.rif_plan | RifPlanned | wf-engine degraded → queue |
| 1.3 | workflow-engine | intelligence | gRPC RunDisparateImpact | disparate-impact-req | b2b.intelligence.disparate_impact.run | RifDisparateImpactAnalysisCompleted | scorer down → halt; alert |
| 1.4 | workflow-engine | compliance | gRPC ResolveCompliancePack ×4 | resolve-pack-req | b2b.compliance.overlay_resolve | PackResolved ×4 | pack missing → halt |
| 1.5 | workflow-engine | tenancy | gRPC ResolveSubTenant ×4 | resolve-sub-tenant | (internal) | TenantScopeResolved ×4 | tenancy degraded → fail-closed |
| 1.6 | workflow-engine | tenancy | gRPC WorksCouncilNotify (DE-BER, T-7d) | works-council-notify | b2b.tenancy.works_council_notify | WorksCouncilNotified | notification fail → escalate |
| 1.7 | workflow-engine | audit-chain | gRPC EmitSealed | audit-event-sealed | (internal) | RifPlanned + pre-announce events | audit degraded → WAL |

### Cedar permit (key fragment for disparate-impact gate)

```cedar
// b2b.intelligence.disparate_impact.run.cedar
permit (
  principal,
  action == Action::"b2b.intelligence.disparate_impact.run",
  resource is RifSelection
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  context.tenant.compliance_pack_active("pack-us-title-vii-baseline") &&
  resource.selections.count <= 1000 &&
  context.audit_session_open == true
};
```

## Phase 2 — Announcement day cascade (T+0 day, jurisdiction-staggered)

### Per-employee sequence

```
Priya  api-gw  workflow-engine  messenger  mail  payments  finops  identity  community  audit-chain
  │      │           │             │         │       │         │         │           │            │
  │POST  │           │             │         │       │         │         │           │            │
  │ rif/ │           │             │         │       │         │         │           │            │
  │exec  │           │             │         │       │         │         │           │            │
  ├─────►│           │             │         │       │         │         │           │            │
  │      │ Cedar     │             │         │       │         │         │           │            │
  │      ├──────────►│             │         │       │         │         │           │            │
  │      │           │ for each    │         │       │         │         │           │            │
  │      │           │ employee:   │         │       │         │         │           │            │
  │      │           │  start 1:1  │         │       │         │         │           │            │
  │      │           ├────────────►│         │       │         │         │           │            │
  │      │           │  send mail  │         │       │         │         │           │            │
  │      │           ├─────────────────────►│        │         │         │           │            │
  │      │           │  compute sev│         │       │         │         │           │            │
  │      │           ├──────────────────────────────►│         │         │           │            │
  │      │           │ schedule pay│         │       │         │         │           │            │
  │      │           ├──────────────────────────────────────────►│         │           │            │
  │      │           │ outplace enroll      │       │         │         │           │            │
  │      │           ├──────────────────────────────────────────────────────────────►│            │
  │      │           │ cohort channel       │       │         │         │           │            │
  │      │           ├──────────────────────────────────────────────────────────────►│            │
  │      │           │ revoke session       │       │         │         │           │            │
  │      │           ├──────────────────────────────────────────────────►│           │            │
  │      │           │ emit sealed ×17/emp  │       │         │         │           │            │
  │      │           ├──────────────────────────────────────────────────────────────────────────►│
  │ 200  │           │             │         │       │         │         │           │            │
  │◄─────┤           │             │         │       │         │         │           │            │
```

### Per-step table (annotated by Cedar permit)

| Step | Caller | Callee | RPC | Schema | Cedar permit | Audit event | Failure-mode |
|---|---|---|---|---|---|---|---|
| 2.1 | Priya | api-gw | POST /api/v1/hr/rif/execute | rif-execute-req | b2b.hr.rif_execute | n/a | n/a |
| 2.2 | api-gw | workflow-engine | gRPC ExecuteRif | rif-execute-spec | b2b.hr.rif_execute | RifExecutionStarted | wf degraded → queue |
| 2.3 | workflow-engine | messenger | gRPC StartManagerRif1on1 ×200 | manager-rif-dm-init | b2b.messenger.manager_rif_dm | ManagerRif1on1Started | messenger degraded → retry |
| 2.4 | workflow-engine | mail | gRPC SendTerminationNotice ×200 | mail-termination-template | b2b.mail.send_termination | TerminationMailSent | mail degraded → retry async |
| 2.5 | workflow-engine | finops-portal | gRPC ComputeSeverance ×200 | severance-compute-req | b2b.finops.severance_compute | SeveranceComputed | scorer down → halt employee cascade |
| 2.6 | workflow-engine | payments | gRPC ScheduleSeverancePayment ×200 | severance-payment-schedule | b2b.payments.severance_schedule | SeverancePaymentScheduled | payments degraded → defer per ADR-0028 |
| 2.7 | workflow-engine | community | gRPC EnrollOutplacement ×188 (excluding decliners) | outplacement-enroll-req | b2b.community.outplacement_enroll | OutplacementEnrolled | vendor unreachable → queue retry |
| 2.8 | workflow-engine | community | gRPC ProvisionCohortChannel | cohort-channel-provision | b2b.community.cohort_channel_provision | CohortChannelProvisioned | n/a |
| 2.9 | workflow-engine | community | gRPC EnrollCohortChannel ×200 | cohort-channel-enroll | b2b.community.cohort_channel_enroll | CohortChannelEnrolled | n/a |
| 2.10 | workflow-engine | identity | gRPC RevokeWorkTenantSession ×200 (scheduled per-jurisdiction last-day) | revoke-session-req | b2b.identity.session_revoke | WorkTenantSessionRevoked | identity degraded → retry |
| 2.11 | workflow-engine | drive | gRPC TransferTenantOwnedFiles ×200 | drive-transfer-req | b2b.drive.transfer_tenant_owned | TenantOwnedDriveTransferred | drive degraded → defer |
| 2.12 | workflow-engine | audit-chain | gRPC EmitSealed ×N (3400 total) | audit-event-sealed | (internal) | many | audit degraded → WAL |

### Cedar permit (key fragment for rif_execute)

```cedar
permit (
  principal == User::"priya-krishnan@marcus-tenant.hr",
  action == Action::"b2b.hr.rif_execute",
  resource is RifEvent
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  resource.approved_by in [User::"marcus-ceo@marcus-tenant"] &&
  resource.disparate_impact_analysis.verdict == "green" &&
  resource.per_jurisdiction_works_council_clearance.all_jurisdictions_cleared == true &&
  context.tenant.compliance_pack_active("pack-us-warn-act") &&
  context.tenant.compliance_pack_active("pack-eu-anti-discrimination-baseline") &&
  context.tenant.compliance_pack_active("pack-kr-labor-standards-act-amendment") &&
  context.tenant.compliance_pack_active("pack-in-industrial-disputes-act") &&
  context.audit_session_open == true
};
```

## Phase 3 — Severance disbursement (per-jurisdiction timing)

| Jurisdiction | Disbursement timing | Rail |
|---|---|---|
| US-AUS | Same day (Marcus's tenant elects floor) | ACH |
| KR-SEO | Within 14 days (LSA §36) | Wire |
| IN-BLR | 2 days from last working day | IMPS/RTGS |
| DE-BER | End of 8-week notice period | SEPA |

### Sequence

```
(durable timer fires)  workflow-engine  payments  finops-portal  audit-chain
                              │             │            │              │
                              │ disburse    │            │              │
                              ├────────────►│            │              │
                              │             │ update     │              │
                              │             │ budget     │              │
                              │             ├───────────►│              │
                              │ emit sealed │            │              │
                              ├─────────────────────────────────────────►│
```

## Phase 4 — Outplacement (cross-tenant)

### Sequence (per affected employee)

```
workflow-engine  community  connect  outplacement-vendor-X  mail  audit-chain
       │              │         │              │              │            │
       │ enroll      │         │              │              │            │
       ├────────────►│         │              │              │            │
       │             │ Connect:│              │              │            │
       │             │ cross-tenant invite    │              │            │
       │             ├────────►│              │              │            │
       │             │         │ accept       │              │            │
       │             │         │◄─────────────┤              │            │
       │             │ send mail with enrollment link        │            │
       │             ├──────────────────────────────────────►│            │
       │             │ emit sealed             │              │            │
       │             ├────────────────────────────────────────────────────►│
```

### Cedar permit

```cedar
permit (
  principal,
  action == Action::"b2b.community.outplacement_enroll",
  resource is OutplacementEnrollment
) when {
  principal.audience_type == "B2B_HR_ADMIN" &&
  resource.outplacement_vendor_tenant in principal.tenant.connect_trust_partners &&
  context.audit_session_open == true
};
```

## Phase 5 — Cohort channel provisioning

### Sequence

```
workflow-engine  community  identity  audit-chain
       │              │         │            │
       │ provision    │         │            │
       ├─────────────►│         │            │
       │              │ mint    │            │
       │              │ former-employer attestation
       │              ├────────►│            │
       │              │         │            │
       │              │ register 200 enrollment
       │              │ tokens  │            │
       │              ├────────►│            │
       │              │ emit sealed          │
       │              ├──────────────────────►│
```

### Critical: marcus-tenant CANNOT read the cohort channel

```cedar
// forbid-employer-cohort-channel-read.cedar
forbid (
  principal,
  action == Action::"b2c.community.channel_read",
  resource is CohortChannel
) when {
  resource.channel_type == "verified-former-employer" &&
  resource.formerly_tenant_id == principal.tenant_id &&
  !context.litigation_subpoena_active
};
```

Marcus's tenant principals are explicitly forbidden from reading the cohort channel even though they provisioned it. Only a litigation subpoena would unlock cross-tenant read.

## Phase 6 — Access revocation (per-jurisdiction last working day)

### Sequence

```
(durable timer fires)  workflow-engine  identity  drive  workplace-int  audit-chain
                              │             │         │           │              │
                              │ revoke      │         │           │              │
                              │ session     │         │           │              │
                              ├────────────►│         │           │              │
                              │ tenant binding revoke │           │              │
                              │             │ (passkey continues for personal)   │
                              │ transfer    │         │           │              │
                              │ work-Drive  │         │           │              │
                              ├────────────────────────►          │              │
                              │ deprovision workplace-int          │              │
                              ├──────────────────────────────────────►            │
                              │ emit sealed │         │           │              │
                              ├──────────────────────────────────────────────────►│
```

### Cedar permits

```cedar
permit (
  principal,
  action == Action::"b2b.identity.session_revoke",
  resource is WorkTenantSession
) when {
  principal == User::"oyatie:workflow-engine:internal:rif-cascade" &&
  resource.affected_employee.has_rif_termination_completed == true
};
```

```cedar
// preserve-personal-tenant-passkey-binding (forbid revoking personal-tenant binding even during rif)
forbid (
  principal,
  action == Action::"b2c.identity.session_revoke",
  resource is PersonalTenantSession
) when {
  context.action_source == "rif-cascade"
};
```

## Phase 7 — Reference letter on request

### Sequence

```
Former employee  api-gw  community  workflow-engine  mail  audit-chain
      │             │       │            │             │            │
      │ POST request│       │            │             │            │
      ├────────────►│       │            │             │            │
      │             │ Cedar │            │             │            │
      │             ├──────►│            │             │            │
      │             │       │ start ref letter wf      │            │
      │             │       ├────────────►             │            │
      │             │       │            │ generate    │            │
      │             │       │            │ ref letter  │            │
      │             │       │            │ send mail   │            │
      │             │       │            ├────────────►│            │
      │             │       │            │ emit sealed │            │
      │             │       │            ├─────────────────────────►│
      │ 200         │       │            │             │            │
      │◄────────────┤       │            │             │            │
```

## Phase 8 — Litigation hold (3-employee subset)

### Sequence

```
Naomi  api-gw  compliance  drive  workflow-engine  audit-chain
  │      │          │         │           │              │
  │ POST │          │         │           │              │
  │ hold │          │         │           │              │
  ├─────►│          │         │           │              │
  │      │ Cedar    │         │           │              │
  │      ├─────────►│         │           │              │
  │      │          │ mark    │           │              │
  │      │          │ hold    │           │              │
  │      │          ├────────►│           │              │
  │      │          │ suspend │           │              │
  │      │          │ retention scheduling             │
  │      │          ├──────────────────────►            │
  │      │          │ emit sealed         │              │
  │      │          ├─────────────────────────────────►│
  │ 200  │          │         │           │              │
  │◄─────┤          │         │           │              │
```

### Cedar permit

```cedar
permit (
  principal,
  action == Action::"b2b.compliance.litigation_hold_apply",
  resource is EmployeeRecord
) when {
  principal in [User::"naomi-legal@marcus-tenant.legal"] &&
  context.litigation_anticipated_documented == true &&
  context.tenant.compliance_pack_active("pack-us-litigation-hold-baseline") &&
  context.audit_session_open == true
};
```

## Cross-µservice audit-event class registry (j133)

| Class | Emitted by |
|---|---|
| RifPlanned | workflow-engine |
| RifDisparateImpactAnalysisCompleted | intelligence |
| PackResolved | compliance |
| TenantScopeResolved | tenancy |
| WorksCouncilNotified | tenancy |
| RifExecutionStarted | workflow-engine |
| ManagerRif1on1Started | messenger |
| TerminationMailSent | mail |
| SeveranceComputed | finops-portal |
| SeverancePaymentScheduled | payments |
| EmployeeFinalPayDisbursed | payments |
| OutplacementEnrolled | community |
| CohortChannelProvisioned | community |
| CohortChannelEnrolled | community |
| WorkTenantSessionRevoked | identity |
| TenantOwnedDriveTransferred | drive |
| ReferenceLetterGenerated | workflow-engine |
| LitigationHoldApplied | compliance |
| RifEmployeeCascadeAcknowledged | workflow-engine |
| RifCascadeCompleted | workflow-engine |

## SLOs (j133-specific composite)

| Phase | P50 | P95 | P99.9 |
|---|---:|---:|---:|
| 1 (planning, DEI, works-council) | 200ms-7d (per phase) | n/a | n/a |
| 2 (per-employee cascade) | 5min | 10min | 18min |
| 3 (disbursement) | <60min | <2h | <4h |
| 4 (outplacement enroll) | 800ms | 1.5s | 3s |
| 5 (cohort channel) | 1s | 2s | 4s |
| 6 (access revoke per employee) | 30s | 90s | 180s |

## Failure-mode catalog

| Failure | Detection | Recovery | Compensation |
|---|---|---|---|
| Disparate-impact analysis YELLOW/RED | Per-jurisdiction verdict | Re-balance selections; works-council re-consult | Re-run with adjusted list |
| Works-council declines proposed selections | §111 BetrVG objection event | Negotiate per §1 KSchG social-selection | Adjust list; re-notify |
| Payments rail down at disbursement time | rail-health-probe FAIL | Defer + retry with exponential backoff | EmployeeFinalPayDeferred event |
| Outplacement vendor unreachable | cross-tenant fail | Queue retry; manual fallback if persistent | OutplacementEnrollmentDeferred |
| Identity revocation fails for personal-tenant (should NEVER happen) | forbid clause triggers | Alarm + ops review; default-deny held | PersonalTenantRevokeAttempted (alert-only) |
| Cohort channel provisioning fails | provision RPC fail | Retry; if persistent, manual provisioning | CohortChannelProvisioningDeferred |
| Audit-chain degraded mid-cascade | seal-latency P99 spike | Local WAL per ADR-0028; flush on recovery | AuditEventDeferredLocal |
| Litigation-hold pack version mismatch | preflight FAIL | Halt hold; alert Naomi; resolve pack version | LitigationHoldDeferred |

— end of handshake —

## Completion expansion — j133 handshake rigor pass

Scope: 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade.
Persona: Priya Krishnan.
Services: workflow-engine + mail + messenger + payments + finops-portal + identity + tenancy + community + drive + compliance.
Applicable ADRs: ADR-0244, ADR-0299, ADR-0311, ADR-0313, ADR-0317, ADR-0320.
The expansion below is intentionally explicit so an intern can build and verify the journey without oral context.

Handshake step 001: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 002: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 003: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 004: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 005: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 006: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 007: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 008: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 009: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 010: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 011: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 012: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 013: workflow-engine invokes payments over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 014: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 015: identity publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 016: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 01: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 017: workflow-engine invokes community over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 018: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 019: compliance publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 020: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 021: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 022: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 023: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 024: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 025: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 026: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 027: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 028: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 029: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 030: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 031: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 032: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 02: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 033: workflow-engine invokes payments over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 034: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 035: identity publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 036: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 037: workflow-engine invokes community over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 038: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 039: compliance publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 040: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 041: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 042: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 043: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 044: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 045: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 046: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 047: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 048: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 03: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 049: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 050: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 051: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 052: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 053: workflow-engine invokes payments over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 054: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 055: identity publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 056: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 057: workflow-engine invokes community over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 058: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 059: compliance publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 060: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 061: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 062: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 063: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 064: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 04: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 065: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 066: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 067: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 068: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 069: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 070: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 071: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 072: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 073: workflow-engine invokes payments over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 074: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 075: identity publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 076: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 077: workflow-engine invokes community over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 078: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 079: compliance publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 080: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 05: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 081: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 082: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 083: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 084: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 085: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 086: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 087: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 088: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 089: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 090: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 091: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 092: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 093: workflow-engine invokes payments over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 094: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 095: identity publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 096: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 06: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 097: workflow-engine invokes community over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 098: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 099: compliance publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 100: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 101: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 102: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 103: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 104: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 105: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 106: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 107: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 108: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 109: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 110: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 111: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 112: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 07: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 113: workflow-engine invokes payments over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 114: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 115: identity publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 116: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 117: workflow-engine invokes community over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 118: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 119: compliance publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 120: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 121: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 122: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 123: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 124: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 125: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 126: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 127: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 128: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 08: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 129: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 130: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 131: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 132: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 133: workflow-engine invokes payments over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 134: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 135: identity publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 136: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 137: workflow-engine invokes community over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 138: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 139: compliance publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 140: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 141: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 142: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 143: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 144: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 09: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 145: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 146: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 147: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 148: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 149: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 150: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 151: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 152: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 153: workflow-engine invokes payments over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 154: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 155: identity publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 156: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 157: workflow-engine invokes community over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 158: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 159: compliance publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 160: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 10: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 161: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 162: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 163: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 164: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 165: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 166: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 167: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 168: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 169: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 170: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 171: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 172: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 173: workflow-engine invokes payments over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 174: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 175: identity publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 176: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 11: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 177: workflow-engine invokes community over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 178: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 179: compliance publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 180: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 181: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 182: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 183: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 184: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 185: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 186: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 187: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 188: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 189: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 190: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 191: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 192: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 12: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 193: workflow-engine invokes payments over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 194: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 195: identity publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 196: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 197: workflow-engine invokes community over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 198: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 199: compliance publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 200: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 201: workflow-engine invokes mail over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 202: ADR-0317 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 203: payments publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 204: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Handshake step 205: workflow-engine invokes identity over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 206: ADR-0311 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 207: community publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 208: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
Checkpoint 13: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Handshake step 209: workflow-engine invokes compliance over proto3 or REST, carrying tenant_id, actor_id, audience_type, purpose, idempotency_key, and traceparent.
Cedar permit 210: ADR-0244 requires default-deny first, explicit allow second, and forbidden personal-surface reads even when a work investigation is active.
Async event 211: mail publishes AsyncAPI 3.1.0 event with schema_version, pack_set, residency_cell, and replay cursor.
Compensation 212: if downstream verification fails, workflow-engine rolls back visible state, preserves immutable audit events, and records the refusal reason.
