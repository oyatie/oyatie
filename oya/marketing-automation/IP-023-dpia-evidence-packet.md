---
doc_class: ImplementationPlan
ip_id: IP-023-dpia-evidence-packet
microservice: marketing-automation
bounded_contexts: [consent-audience, behavioral-profile, email-tracking, attribution, segment, customer-analytics]
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-privacy
tenant_class_aware: true
---

# IP-023: DPIA Evidence Packet

## A. Problem

Marketing Automation processes profiling and engagement data that can trigger DPIA obligations: segmentation, behavioral profiling, tracking, attribution, deliverability, and consent history. The stamped IP did not name privacy risks or evidence. The actual gap is a per-service DPIA packet that links data classes, purposes, lawful basis, pack overlays, mitigations, and audit events.

## B. Approach

Use `dpia.md`, `compliance.md`, `policy/data-residency.md`, and local Cedar policies as the DPIA evidence spine. The packet maps every high-risk marketing flow to data minimization, purpose limitation, retention, DSR, and operator-review controls. It references real audit event classes and counterpart gaps.

## C. Deliverables

| Artifact | Change |
|---|---|
| `dpia.md` | Add or verify DPIA rows for profiling, tracking, attribution, consent, suppression, and marketplace audience licensing. |
| `compliance.md` | Map GDPR/ePrivacy/CASL/TCPA/HIPAA-adjacent obligations to Marketing Automation flows. |
| `policy/data-residency.md` | Bind residency and transfer controls to DPIA mitigations. |
| `policies/local-consent-gated-automation.cedar` | Enforce purpose and data class boundaries described by the DPIA. |
| `runbooks/local-consent-propagation-lag.md` | Provide privacy incident response for delayed revocation. |

## D. Implementation

1. Inventory personal-data flows: form submission, email tracking, behavioral event ingestion, segment materialization, suppression checks, attribution rollup, survey response, and chatflow handoff.
2. For each flow, record purpose, data class, retention, lawful basis, and DSR behavior.
3. Add high-risk controls: opt-out propagation SLO, Apple MPP/GPC/DNT handling, tombstone projection for erasure, and restricted-trait policy denial.
4. Link each DPIA risk to a Cedar policy, audit event, and runbook.
5. Add evidence for demo_trial tenants: caps reduce scale but do not reduce privacy obligations.
6. Add pack-specific notes for GDPR, KR-PIPA, CASL, TCPA, and HIPAA-adjacent marketing review.
7. Ensure no counterpart comparison is used as a privacy exemption; HubSpot/Marketo/Mailchimp parity is a floor, not a waiver.

## E. Acceptance

- `buck2 build //:quality-lane-registry-authority-check # lane=dpia --microservice marketing-automation`
- `buck2 build //:quality-lane-registry-authority-check # lane=privacy --microservice marketing-automation`
- Manual evidence: each high-risk flow has a DPIA row, policy hook, audit event, and runbook reference.

## F. Evidence

- Local docs: `dpia.md`, `compliance.md`, `policy/data-residency.md`.
- Local policies: `policies/local-consent-gated-automation.cedar`, `policies/local-attribution-export-egress.cedar`.
- Local runbook: `runbooks/local-consent-propagation-lag.md`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Privacy Center-like claims are grounded in DPIA evidence and DSR behavior. |
| Adobe Marketo Engage | Lead/activity profiling has explicit lawful-basis and export controls. |
| Mailchimp | Audience consent and campaign tracking controls are pack-specific and auditable. |

## H. Local Traceability

- DPIA file: `dpia.md`.
- Compliance file: `compliance.md`.
- Residency file: `policy/data-residency.md`.
- Cedar file: `policies/local-consent-gated-automation.cedar`.
- Cedar file: `policies/local-attribution-export-egress.cedar`.
- Runbook: `runbooks/local-consent-propagation-lag.md`.
- Flow: form submission.
- Flow: email tracking.
- Flow: behavioral event ingest.
- Flow: segment materialization.
- Flow: suppression check.
- Flow: attribution rollup.
- Flow: survey response.
- Flow: chatflow handoff.
- Control: tombstone projection.
- Control: restricted-trait denial.
- Failure state: demo_trial treated as privacy-light.
- Failure state: counterpart parity cited as privacy exemption.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-023-dpia-evidence-packet.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/marketing-automation/IP-023-dpia-evidence-packet.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-023-dpia-evidence-packet.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-023-dpia-evidence-packet.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
