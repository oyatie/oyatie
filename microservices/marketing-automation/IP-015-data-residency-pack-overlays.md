---
doc_class: ImplementationPlan
ip_id: IP-015-data-residency-pack-overlays
microservice: marketing-automation
bounded_contexts: [segment, consent-audience, attribution, email-tracking, behavioral-profile, customer-analytics]
related_adrs: [ADR-0244, ADR-0248, ADR-0263, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-compliance
tenant_class_aware: true
---

# IP-015: Data Residency Pack Overlays

## A. Problem

Marketing Automation processes high-risk engagement data: campaign profiles, consent signals, suppression entries, attribution events, behavioral profiles, and deliverability reports. The stamped IP did not bind these data classes to regional packs. The real gap is pack-aware reads, writes, exports, and analytics for GDPR, KR-PIPA, CASL, TCPA, HIPAA-adjacent marketing, and ePrivacy contexts.

## B. Approach

Promote `policy/data-residency.md`, `compliance.md`, `dpia.md`, `manifest.json` compliance packs, and local Cedar policies into a pack overlay plan. Pack resolution is gateway-stamped; the service consumes `data_residency_pack` and refuses cross-pack movement unless the pack explicitly permits metadata-only replication.

## C. Deliverables

| Artifact | Change |
|---|---|
| `policy/data-residency.md` | Convert prose pack rules into Cedar-backed or validator-backed checks in a follow-on implementation. |
| `compliance.md` | Map pack obligations to marketing data classes and event types. |
| `dpia.md` | Bind DPIA controls to campaign, suppression, attribution, and tracking flows. |
| `policies/local-attribution-export-egress.cedar` | Deny attribution export when pack, purpose, or data class does not match. |
| `contracts/openapi-v1.yaml` | Document that clients cannot set pack; gateway/tenant context supplies it. |

## D. Implementation

1. Enumerate marketing data classes from PRD and policies: `campaign_profile`, `consent_signal`, `suppression_entry`, `attribution_event`, `segment_membership`, `deliverability_report`.
2. Map each data class to pack restrictions in `compliance.md` and `dpia.md`.
3. Add `data_residency_pack` to command context and audit events.
4. Deny exports for attribution and behavioral profiles unless purpose is `regulated-audit`, `support`, or another pack-permitted purpose.
5. Prevent cross-cell replication of profile-bearing data unless pack says metadata-only replication is allowed.
6. Add tests for EU GDPR right-to-erasure projection, KR-PIPA consent purpose mismatch, and HIPAA marketing-review hold.
7. Track pack conflict in audit-chain rather than silently merging pack rules.

## E. Acceptance

- `cargo run -p oya-dev-cli -- gate validate data-residency --microservice marketing-automation`
- `cargo run -p oya-dev-cli -- gate validate dpia --microservice marketing-automation`
- `cargo test -p oya-marketing-automation-campaign-journey-app residency`
- Manual evidence: every marketing data class has a pack rule and audit event dimension.

## F. Evidence

- Local docs: `policy/data-residency.md`, `compliance.md`, `dpia.md`.
- Local policies: `policies/local-attribution-export-egress.cedar`, `policies/local-consent-gated-automation.cedar`.
- Local manifest: `manifest.json` compliance and deployment context blocks.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Region and privacy behavior is explicit per tenant pack, not edition-default SaaS behavior. |
| Adobe Marketo Engage | Lead and activity exports are pack-gated before bulk extraction. |
| Mailchimp | Audience and campaign reporting honors jurisdictional consent and residency overlays. |

## H. Local Traceability

- Residency doc: `policy/data-residency.md`.
- Compliance doc: `compliance.md`.
- Privacy doc: `dpia.md`.
- Cedar file: `policies/local-attribution-export-egress.cedar`.
- Cedar file: `policies/local-consent-gated-automation.cedar`.
- Data class: `campaign_profile`.
- Data class: `consent_signal`.
- Data class: `suppression_entry`.
- Data class: `attribution_event`.
- Data class: `segment_membership`.
- Data class: `deliverability_report`.
- Pack example: GDPR.
- Pack example: KR-PIPA.
- Pack example: CASL.
- Pack example: TCPA.
- Pack example: HIPAA-adjacent marketing review.
- Failure state: client-supplied pack is not trusted.
- Failure state: cross-pack profile movement without permit is a blocker.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/marketing-automation/contracts/asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-openapi-v1.yaml`, `microservices/marketing-automation/contracts/local-operations-v1.proto`, `microservices/marketing-automation/contracts/marketing-automation-v1.proto`, `microservices/marketing-automation/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-015-data-residency-pack-overlays.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-015-data-residency-pack-overlays.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
