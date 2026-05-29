---
doc_class: Implementation-Plan
ip_id: IP-journey-j91-us-msb-mtl-overlay
journey_ref: docs/user-journeys/j91-us-state-money-transmitter-licensing/
status: draft
date: 2026-05-20
microservice: feature-flags
flat_layout_adr: ADR-0131
related_adrs: [ADR-0159, ADR-0243, ADR-0244, ADR-0251, ADR-0263]
---

# IP - feature-flags role in j91 US state money transmitter licensing

## Scope

`feature-flags` does not own MSB threshold calculation, NMLS filing, or state-license adjudication. It owns the runtime controls that let the journey turn payment paths on, off, or down by tenant, state pack, and audit posture. This plan binds j91 to the feature-flag substrate instead of a generic journey checklist.

## Service anchors

- Contract: `microservices/feature-flags/contracts/openapi-v1.yaml` operations `evaluateFlag`, `createFlag`, `updateFlag`, `undoFlagMutation`, and `streamFlagUpdates`.
- Event contract: `microservices/feature-flags/contracts/asyncapi-v1.yaml` channels `flag-state-changed`, `pack-override-applied`, and `abuse-detection-signal`.
- RPC contract: `microservices/feature-flags/contracts/feature-flags-v1.proto` services `FlagEvaluationService` and `FlagManagementService`.
- Capabilities: `microservices/feature-flags/capabilities/flag-evaluate.yaml` and `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`.
- Policies: `microservices/feature-flags/policy/tenant-targeting.cedar`, `microservices/feature-flags/policy/flag-mutation-authorization.cedar`, `microservices/feature-flags/policy/pack-overlay-authorization.cedar`, `microservices/feature-flags/policy/pack-flag-override.cedar`, and `microservices/feature-flags/policy/ci-scope.cedar`.
- Operations: `microservices/feature-flags/runbooks/flag-mutation-cascade.md`, `microservices/feature-flags/runbooks/pack-override-cascade.md`, and `microservices/feature-flags/runbooks/audit-replay.md`.
- Counterpart journey refs: `docs/user-journeys/j91-us-state-money-transmitter-licensing/README.md`, `docs/user-journeys/j91-us-state-money-transmitter-licensing/schemas/openapi-overlay-action.json`, `docs/user-journeys/j91-us-state-money-transmitter-licensing/schemas/asyncapi-overlay-events.yaml`, and `docs/user-journeys/j91-us-state-money-transmitter-licensing/schemas/journey-messages.proto`.

## Acceptance criteria

- State pack overlays map to pack-controlled flags whose tenant writes are blocked by `policy/pack-flag-override.cedar`; normal tenant admins cannot unlock state-restricted payment features.
- Payment-path callers evaluate flags through `POST /api/v1/flags/{flag_key}/evaluate` with `tenant_id`, `audience_type`, `cohort_ids`, and pack context; cross-tenant reads remain denied by `policy/tenant-targeting.cedar`.
- J91 rollout gates use `pack-overlay-subscribe` and emit `PackOverrideApplied` plus `FlagStateChanged` events when MSB or state-MTL pack posture changes.
- CI may read rollout and flag state through `policy/ci-scope.cedar`, but it cannot create, update, archive, delete, or engage kill switches.
- Evidence replay for a regulator or renewal audit follows `runbooks/audit-replay.md`; replay exports only feature-flag events and does not claim ownership of licensing evidence outside this service.

## Counterparts

| Counterpart | Gap closed |
|---|---|
| LaunchDarkly | Adds Oyatie tenant-pack and Cedar-deny controls around runtime flag rollout. |
| OpenFeature | Keeps provider-compatible evaluation while binding audit-chain events and pack overlays. |
| Unleash | Adds compliance-pack override evidence beyond generic gradual rollout. |
| Statsig | Keeps experiment and rollout controls tied to service-local policy and SLO evidence. |

## Failure and rollback

- Stale pack projection: keep the most restrictive pack-mandated flag value and follow `runbooks/pack-override-cascade.md`.
- Bad flag mutation: use the 15-second undo path in `openapi-v1.yaml` / `feature-flags-v1.proto`; if outside the undo window, follow `runbooks/flag-mutation-cascade.md`.
- Audit gap: replay feature-flag event windows with `runbooks/audit-replay.md`.
