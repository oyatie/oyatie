---
doc_class: ImplementationPlan
ip_id: IP-013-emergency-services-bypass
microservice: marketing-automation
bounded_contexts: [deliverability, suppression, chatflow, notification]
related_adrs: [ADR-0243, ADR-0244, ADR-0263, ADR-0297, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-security
tenant_class_aware: true
---

# IP-013: Emergency Services Bypass

## A. Problem

Marketing systems are normally consent-first and suppression-first. Some regulated emergency communications need a narrow exception, but the stamped IP treated "emergency services" as another benchmark phrase. The real gap is a bounded bypass that allows jurisdiction-registered emergency sends while still refusing ordinary marketing, bot traffic, and unsealed breakglass.

## B. Approach

Use `policy/emergency-services-bypass.cedar` and the existing emergency permit clause in `policy/campaign-journey-authorization.cedar`. The bypass is not a general send override. It applies only when `audience_type == "EMERGENCY_SERVICES"`, `emergency_attestation == "jurisdiction_registered"`, and `audit_event_class == "AbuseDefenceEmergencyServiceBypass"`.

## C. Deliverables

| Artifact | Change |
|---|---|
| `policy/emergency-services-bypass.cedar` | Keep a dedicated action and context requirements for emergency communications. |
| `policy/campaign-journey-authorization.cedar` | Ensure generic journey permit cannot be used as a hidden emergency bypass. |
| `runbooks/local-campaign-egress-hold.md` | Add breakglass release and post-incident review steps. |
| `dashboards/abuse-defence-outcomes.json` | Add emergency bypass count, denied bypass count, and missing attestation panels. |
| `tests/integration.rs` | Add Cedar fixture for audited emergency allow and unaudited emergency deny. |

## D. Implementation

1. Define allowed emergency action names and resource kinds; exclude campaign newsletter, nurture, ABM, and promotional sends.
2. Require jurisdiction registration reference, approving principal, expiry time, and incident id in Cedar context.
3. Gate bypass against suppression only for emergency-purpose messages; normal marketing suppression remains fail-closed.
4. Emit `AbuseDefenceEmergencyServiceBypass` with tenant, principal, jurisdiction, purpose, channel, and expiry.
5. Add alerting for any bypass attempted without matching incident record.
6. Add runbook step to revoke the bypass and replay denied/allowed sends after incident close.
7. Confirm demo_trial tenants cannot self-issue emergency bypass without platform approval.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app emergency`
- `buck2 build //:quality-lane-registry-authority-check # lane=policy-authorization --microservice marketing-automation`
- Manual policy review confirms no `permit(principal, action, resource)` emergency clause lacks audit-event and attestation predicates.

## F. Evidence

- Local policy: `policy/emergency-services-bypass.cedar`.
- Local policy: `policy/campaign-journey-authorization.cedar`.
- Local runbook: `runbooks/local-campaign-egress-hold.md`.
- Doctrine: ADR-0244 tenant scoping and ADR-0328 Big-8 substance discipline.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Transactional/emergency style messaging is separated from marketing workflows. |
| Adobe Marketo Engage | Operational sends cannot silently bypass smart-list suppression. |
| Mailchimp | Compliance exceptions are auditable rather than hidden campaign settings. |

## H. Local Traceability

- Policy file: `policy/emergency-services-bypass.cedar`.
- Policy file: `policy/campaign-journey-authorization.cedar`.
- Audience type: `EMERGENCY_SERVICES`.
- Required attestation: `jurisdiction_registered`.
- Required event: `AbuseDefenceEmergencyServiceBypass`.
- Excluded surface: newsletter.
- Excluded surface: nurture journey.
- Excluded surface: promotional campaign.
- Context field: `incident_id`.
- Context field: `jurisdiction`.
- Context field: `expiry_time`.
- Context field: `approving_principal`.
- Runbook: `local-campaign-egress-hold.md`.
- Dashboard: `dashboards/abuse-defence-outcomes.json`.
- Fixture: audited emergency allow.
- Fixture: unaudited emergency deny.
- Failure state: demo_trial self-issued bypass is a blocker.
- Failure state: bypass after expiry is a blocker.
