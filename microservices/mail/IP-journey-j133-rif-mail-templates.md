---
doc_class: Implementation-Plan
ip_id: IP-journey-j133-rif-mail-templates
journey_ref: docs/user-journeys/j133-hr-conducts-layoff-with-dignity-and-compliance/
status: draft
date: 2026-05-20
microservice: mail
related_adrs: [ADR-0311, ADR-0244, ADR-0263, ADR-0292]
---

# IP — Mail's role in j133 RIF mail cascade

## Scope

Mail composes and sends per-jurisdiction termination notices, severance summary statements,
reference letter delivery, outplacement enrollment invitations, and OWBPA mutual-release offers.
All templates reviewed by Naomi (legal) AND HR communications sensitivity-reviewer. DKIM signed; per-send audit-chain sealed.

## Acceptance criteria

1. 7 RIF-specific templates registered with per-jurisdiction localization:
   - termination-notice (per-jurisdiction)
   - severance-summary
   - cobra-notice (US-AUS only)
   - outplacement-enrollment-invite
   - cohort-channel-invitation
   - owbpa-mutual-release-offer (US-AUS ≥40 only)
   - reference-letter-delivery
2. Per-template Cedar permit gates.
3. Per-jurisdiction statute citation in template.
4. Inclusive-language scrubber (sensitivity layer).
5. SLO: P95 send ≤ 700ms; sustained 50/sec.

## Atomic deliverables

| Step | Change | Verification |
|---|---|---|
| 1 | Register `termination-notice` (4 jurisdiction variants × 4 languages) | Template-store passes |
| 2 | Register `severance-summary` template | passes |
| 3 | Register `cobra-notice` template (US-AUS only) | passes |
| 4 | Register `outplacement-enrollment-invite` template | T-301 sub-step passes |
| 5 | Register `cohort-channel-invitation` template | T-401 sub-step passes |
| 6 | Register `owbpa-mutual-release-offer` template (US-AUS ≥40 only) | T-203 + T-903 passes |
| 7 | Register `reference-letter-delivery` template | T-601 passes |
| 8 | Implement per-template Cedar permits | Cedar tests pass |
| 9 | Implement inclusive-language scrubber + sensitivity-layer pass | scrubber tests pass |
| 10 | Wire audit-chain emit per send (8 audit-event classes) | Registry green |

## Templates (key variables)

### termination-notice (jurisdiction-aware)

Variables:
```
{employee_first_name}
{employee_last_name}
{employee_id}
{tenure_years}
{last_working_day}
{severance_amount_localized}
{notice_period}
{outplacement_url}
{cohort_channel_url}
{personal_tenant_continuity_assurance_text}
{labor_law_anchor_citation}  // per-jurisdiction
{hr_contact_email}
{hr_contact_phone}
{works_council_contact}  // DE-BER only
```

### severance-summary

Variables:
```
{base_severance}
{notice_period_pay}
{cobra_weeks}  // US-AUS only
{gratuity}  // IN-BLR only
{equity_grant_vested_summary}
{equity_grant_unvested_forfeit}
{disbursement_date_localized}
{disbursement_rail_human_readable}
{final_paycheck_clauses}  // per-jurisdiction
```

### owbpa-mutual-release-offer (US-AUS ≥40)

Variables:
```
{owbpa_consider_window_days}  // 21
{owbpa_revoke_window_days}  // 7
{mutual_release_text_link}
{enhanced_severance_amount}  // if signed
{signing_options}
```

## Cedar permits

```cedar
// b2b.mail.send_termination.cedar
permit (
  principal,
  action == Action::"b2b.mail.send_termination",
  resource is MailMessage
) when {
  resource.template_id == "termination-notice" &&
  resource.template_legal_review_passed == true &&
  resource.template_sensitivity_review_passed == true &&
  context.tenant.compliance_pack_active(resource.jurisdiction_pack) &&
  context.audit_session_open == true
};
```

```cedar
// b2b.mail.send_severance_summary.cedar
permit (
  principal,
  action == Action::"b2b.mail.send_severance_summary",
  resource is MailMessage
) when {
  resource.template_id == "severance-summary" &&
  resource.severance_computed_complete == true
};
```

## Per-jurisdiction statute citation

| Jurisdiction | Statute citation in template |
|---|---|
| US-AUS | "Pursuant to your at-will employment, the WARN Act 1988 (where applicable), and Texas Payday Law" |
| DE-BER | "gemäß § 1a Kündigungsschutzgesetz (KSchG) und § 17 KSchG Massenentlassung" |
| KR-SEO | "근로기준법 제24조에 따른" (per LSA §24) |
| IN-BLR | "Pursuant to Section 25F of the Industrial Disputes Act 1947" |

## Dependencies

- **template-store** (8 templates × 4 languages)
- **identity** (recipient resolution)
- **compliance** (per-jurisdiction pack + sensitivity scrubber)
- **drive** (severance-statement PDF reference)
- **audit-chain** (EmitSealed)
- **legal-review-workflow** (out-of-band; templates pre-approved)

## Observability

| Metric | Type | Labels |
|---|---|---|
| `oya_mail_send_total` | counter | template_id, jurisdiction |
| `oya_mail_send_latency_ms` | histogram | template_id |
| `oya_mail_scrubber_reject_total` | counter | reason |
| `oya_mail_sensitivity_reject_total` | counter | reason |

## SLOs

- P50 send: 280ms; P95: 700ms; P99: 1.4s
- Sustained: 50/sec
- 100% legal-reviewed templates only

## Failure modes

| Failure | Recovery |
|---|---|
| Template not legal-reviewed | Reject send; banner |
| Sensitivity-scrubber FAIL | Reject; offer remediation inline |
| Recipient mailbox unreachable | Retry with backoff; if 7-day fail, manual escalation |
| Audit-chain degraded | Local WAL |

## Test gates

- T-101 (termination mail send)
- T-203 + T-903 (OWBPA template)
- T-301 (outplacement invite)
- T-401 (cohort invite)
- T-601 (ref-letter delivery)
- T-901 (per-jurisdiction citation)

## Notes

- Per ADR-0263, every send emits typed audit event.
- Per ADR-0292 accessibility, all RIF templates are screen-reader-friendly + plain-text alternative.
- Per ADR-0311, the mail goes to the address the employee provided (work or personal); marcus-tenant does NOT pierce employee's personal Mail without consent.
- Termination templates include the personal-tenant-continuity-assurance text prominently.

— end of IP —

## Completion expansion — j133 mail IP rigor pass

Journey context: 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade.
Service role: work-mail archive, notification cascade, and personal-mail refusal boundary.
Mapped services in this journey: workflow-engine, mail, messenger, payments, finops-portal, identity, tenancy, community, drive, compliance.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0313, ADR-0317, ADR-0320.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in mail, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in mail, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in mail, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in mail, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving mail and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in mail, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in mail, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in mail, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in mail, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in mail, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving mail and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in mail, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in mail, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in mail, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in mail, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in mail, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving mail and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in mail, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in mail, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in mail, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in mail, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 018: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 019: in mail, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 019: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 019: add pack-overlay coverage proving mail and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 019: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 020: in mail, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 020: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 020: add unit coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 020: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 02: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 021: in mail, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 021: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 021: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 021: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 022: in mail, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 022: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 022: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 022: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 023: in mail, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 023: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 023: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 023: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 024: in mail, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 024: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 024: add replay coverage proving mail and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 024: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 025: in mail, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 025: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 025: add load coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 025: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 026: in mail, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 026: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 026: add chaos coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 026: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 027: in mail, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 027: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 027: add negative authorization coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 027: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 028: in mail, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 028: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 028: add multi-region coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 028: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 029: in mail, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 029: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 029: add pack-overlay coverage proving mail and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 029: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 030: in mail, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 030: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 030: add unit coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 030: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 03: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 031: in mail, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 031: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 031: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 031: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 032: in mail, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 032: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 032: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 032: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 033: in mail, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 033: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 033: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 033: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 034: in mail, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 034: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 034: add replay coverage proving mail and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 034: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 035: in mail, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 035: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 035: add load coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 035: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 036: in mail, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 036: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 036: add chaos coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 036: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 037: in mail, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 037: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 037: add negative authorization coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 037: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 038: in mail, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 038: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 038: add multi-region coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 038: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 039: in mail, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 039: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 039: add pack-overlay coverage proving mail and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 039: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 040: in mail, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 040: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 040: add unit coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 040: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 04: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 041: in mail, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 041: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 041: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 041: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 042: in mail, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 042: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 042: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 042: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 043: in mail, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 043: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 043: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 043: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 044: in mail, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 044: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 044: add replay coverage proving mail and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 044: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 045: in mail, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 045: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 045: add load coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 045: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 046: in mail, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 046: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 046: add chaos coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 046: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 047: in mail, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 047: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 047: add negative authorization coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 047: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 048: in mail, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 048: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 048: add multi-region coverage proving mail and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 048: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 049: in mail, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 049: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 049: add pack-overlay coverage proving mail and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 049: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 050: in mail, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 050: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 050: add unit coverage proving mail and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 050: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 05: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 051: in mail, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 051: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 051: add property coverage proving mail and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 051: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 052: in mail, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 052: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 052: add contract coverage proving mail and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 052: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 053: in mail, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 053: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 053: add integration coverage proving mail and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 053: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 054: in mail, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 054: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 054: add replay coverage proving mail and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 054: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 055: in mail, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 055: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 055: add load coverage proving mail and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 055: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 056: in mail, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 056: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 056: add chaos coverage proving mail and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 056: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 057: in mail, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 057: mail MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 057: add negative authorization coverage proving mail and community agree on contract version, Cedar decision, audit event class, and replay cursor.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-journey-j133-rif-mail-templates.md` matched `SLO, multi-region, payment`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/mail/IP-journey-j133-rif-mail-templates.md` matched `finops, emission`; anchors `microservices/mail/manifest.json, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
