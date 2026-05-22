---
doc_class: Implementation-Plan
ip_id: IP-journey-j93-in-dpdpa-rbi-overlay
journey_ref: docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/
status: draft
date: 2026-05-20
microservice: feature-flags
flat_layout_adr: ADR-0131
related_adrs: [ADR-0159, ADR-0243, ADR-0244, ADR-0251, ADR-0263]
---

# IP - feature-flags role in j93 India DPDPA and RBI financial overlay

## Scope

`feature-flags` provides tenant-scoped feature controls for financial-pack activation, RBI-sensitive rollout gating, audit-required evaluations, and residency-aware flag state. It does not own merchant KYC, escrow, or payment-aggregator obligations; it exposes the flags and pack overrides those counterpart flows consume.

## Service anchors

- Contract: `microservices/feature-flags/contracts/openapi-v1.yaml` operations `evaluateFlag`, `createFlag`, `updateFlag`, `archiveFlag`, and `streamFlagUpdates`.
- Event contract: `microservices/feature-flags/contracts/asyncapi-v1.yaml` channels `flag-state-changed`, `pack-override-applied`, and `abuse-detection-signal`.
- Capabilities: `microservices/feature-flags/capabilities/flag-evaluate.yaml`, `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`, and `microservices/feature-flags/capabilities/killswitch-trigger.yaml`.
- Policies: `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/pack-overlay-authorization.cedar`, `microservices/feature-flags/policy/pack-flag-override.cedar`, `microservices/feature-flags/policy/flag-mutation-authorization.cedar`, and `microservices/feature-flags/policy/auditor-scope.cedar`.
- Operations: `microservices/feature-flags/multi-region.md`, `microservices/feature-flags/runbooks/pack-override-cascade.md`, `microservices/feature-flags/runbooks/audit-replay.md`, and `microservices/feature-flags/incident-response.md`.
- Counterpart journey refs: `docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/README.md`, `docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/schemas/openapi-overlay-action.json`, `docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/schemas/asyncapi-overlay-events.yaml`, and `docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/schemas/journey-messages.proto`.

## Acceptance criteria

- RBI/DPDPA pack activation is represented as pack-mandated flag overrides, not tenant-admin mutable flags.
- Any financial-path evaluation uses `EvaluationContext.tenant_id`, `audience_type`, and consent-purpose fields from `feature-flags-v1.proto`; missing tenant context fails closed.
- Pack override writes require Foundry attestation and cosign validation through `policy/pack-overlay-authorization.cedar`.
- Audit-required flags emit event evidence through `asyncapi-v1.yaml`; export and replay follow `runbooks/audit-replay.md`.
- Regional behavior follows `multi-region.md` and `policy/data-residency.md`; this IP makes no claim that feature-flags owns RBI filing or payment movement.

## Counterparts

| Counterpart | Gap closed |
|---|---|
| LaunchDarkly | Adds Oyatie tenant-pack and Cedar-deny controls around runtime flag rollout. |
| OpenFeature | Keeps provider-compatible evaluation while binding audit-chain events and pack overlays. |
| Unleash | Adds compliance-pack override evidence beyond generic gradual rollout. |
| Statsig | Keeps experiment and rollout controls tied to service-local policy and SLO evidence. |

## Failure and rollback

- Stale pack state: keep the stricter flag value and execute `runbooks/pack-override-cascade.md`.
- Evaluation regression: fall back to safe default/off behavior and use `runbooks/flag-evaluation-regression.md`.
- Audit backpressure: stop high-risk mutations before evidence loss as described in `compliance.md` and `incident-response.md`.
