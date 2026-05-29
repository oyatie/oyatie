---
doc_class: Implementation-Plan
ip_id: IP-journey-j92-br-lgpd-us-parent-dsar
journey_ref: null
status: draft
date: 2026-05-20
microservice: feature-flags
flat_layout_adr: ADR-0131
related_adrs: [ADR-0159, ADR-0243, ADR-0244, ADR-0276]
---

# IP - feature-flags role in j92 BR LGPD DSAR with US parent overlap

## Scope

`feature-flags` owns DSAR-relevant flag definitions, audit-required evaluations, experiment assignment records, and tenant-scoped export boundaries. It does not own the parent-company DSAR workflow or identity verification; those are counterpart responsibilities. This file binds only the feature-flags export, audit, and residency surfaces that exist in this service.

## Service anchors

- Contract: `microservices/feature-flags/contracts/openapi-v1.yaml` operations `listFlags`, `getFlag`, and `evaluateFlag`.
- Event contract: `microservices/feature-flags/contracts/asyncapi-v1.yaml` messages `FlagStateChanged`, `ExperimentActivated`, and `ExperimentConcluded`.
- Capability: `microservices/feature-flags/capabilities/flag-evaluate.yaml`.
- Policies: `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, and `microservices/feature-flags/policy/tenant-targeting.cedar`.
- Data and replay docs: `microservices/feature-flags/dpia.md`, `microservices/feature-flags/backfill-replay.md`, `microservices/feature-flags/compliance.md`, and `microservices/feature-flags/runbooks/audit-replay.md`.
- Implementation counterparts inside this service: `microservices/feature-flags/IP-005-flag-adapter-postgres.md`, `microservices/feature-flags/IP-012-metric-attribution.md`, and `microservices/feature-flags/IP-018-cedar-fragments.md`.

## Counterpart refs

No repo-local j92 journey counterpart exists at scrub time, so this IP does not cite a journey schema path for j92. The service contract boundary remains the real feature-flags files listed above.

## Acceptance criteria

- DSAR export includes tenant-owned flag definitions and audit-required flag evaluation history described in `backfill-replay.md`; it excludes other tenants, platform internals, and Cedar fragment internals.
- `policy/auditor-scope.cedar` governs compliance-officer, QSA, regulator, and platform-safety-officer read access to feature-flag audit records.
- `policy/data-residency.md` and `dpia.md` remain the source for residency and retention language; this IP must not invent storage regions or erasure behavior.
- `FlagStateChanged` and experiment events in `asyncapi-v1.yaml` are the only event evidence this service claims for j92.
- Cross-tenant DSAR reads fail closed through `policy/tenant-targeting.cedar`.

## Counterparts

| Counterpart | Gap closed |
|---|---|
| LaunchDarkly | Adds Oyatie tenant-pack and Cedar-deny controls around runtime flag rollout. |
| OpenFeature | Keeps provider-compatible evaluation while binding audit-chain events and pack overlays. |
| Unleash | Adds compliance-pack override evidence beyond generic gradual rollout. |
| Statsig | Keeps experiment and rollout controls tied to service-local policy and SLO evidence. |

## Failure and rollback

- Export gap: use `backfill-replay.md` DSAR export flow and then verify audit-chain continuity with `runbooks/audit-replay.md`.
- Residency conflict: preserve home-cell boundary and do not transfer feature-flag records across a residency boundary.
- Unauthorized auditor read: Cedar deny remains terminal; no manual override is described in this service.
