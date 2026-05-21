---
doc_class: Implementation-Plan
ip_id: IP-journey-j99-multi-pack-conflict-resolution
journey_ref: docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/
status: draft
date: 2026-05-20
microservice: feature-flags
flat_layout_adr: ADR-0131
related_adrs: [ADR-0159, ADR-0243, ADR-0251, ADR-0263]
---

# IP - feature-flags role in j99 cross-jurisdiction multi-pack conflict resolution

## Scope

`feature-flags` owns the runtime consequence of a multi-pack decision: pack-mandated flag overrides, deny-wins evaluation, audit evidence, and rollback behavior when pack projections conflict. It does not own the global conflict graph or legal interpretation.

## Service anchors

- Contract: `microservices/feature-flags/contracts/openapi-v1.yaml` operations `evaluateFlag`, `updateFlag`, and `streamFlagUpdates`.
- Event contract: `microservices/feature-flags/contracts/asyncapi-v1.yaml` channels `pack-override-applied`, `flag-state-changed`, and `abuse-detection-signal`.
- Capabilities: `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`, `microservices/feature-flags/capabilities/flag-evaluate.yaml`, and `microservices/feature-flags/capabilities/killswitch-trigger.yaml`.
- Policies: `microservices/feature-flags/policy/pack-overlay-authorization.cedar`, `microservices/feature-flags/policy/pack-flag-override.cedar`, `microservices/feature-flags/policy/tenant-targeting.cedar`, `microservices/feature-flags/policy/data-residency.md`, and `microservices/feature-flags/policy/schema.cedarschema`.
- Operations: `microservices/feature-flags/runbooks/pack-override-cascade.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`, `microservices/feature-flags/runbooks/audit-replay.md`, and `microservices/feature-flags/compliance.md`.
- Counterpart journey refs: `docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/README.md`, `docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/schemas/openapi-overlay-action.json`, `docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/schemas/asyncapi-overlay-events.yaml`, and `docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/schemas/journey-messages.proto`.

## Acceptance criteria

- Conflict-resolution output enters feature-flags only as pack overlay state; feature-flags applies overrides and fails closed when attestation is missing.
- Tenant admins cannot mutate pack-locked fields under `policy/pack-flag-override.cedar`.
- Flag evaluation returns the stricter pack outcome when overlays collide; this service records the applied outcome, not the legal reasoning.
- Residency conflicts cite `policy/data-residency.md` and `multi-region.md`; no unstated cross-border replication is added.
- `PackOverrideApplied` and `FlagStateChanged` events are emitted for every service-local applied conflict result.

## Counterparts

| Counterpart | Gap closed |
|---|---|
| LaunchDarkly | Adds Oyatie tenant-pack and Cedar-deny controls around runtime flag rollout. |
| OpenFeature | Keeps provider-compatible evaluation while binding audit-chain events and pack overlays. |
| Unleash | Adds compliance-pack override evidence beyond generic gradual rollout. |
| Statsig | Keeps experiment and rollout controls tied to service-local policy and SLO evidence. |

## Failure and rollback

- Cross-pack conflict not resolved: keep the existing stricter override and stop mutation.
- Pack-overlay agent misconfiguration: follow `runbooks/pack-override-cascade.md` and preserve audit evidence.
- Tenant bypass attempt: deny through Cedar and emit service-local abuse/audit evidence.
