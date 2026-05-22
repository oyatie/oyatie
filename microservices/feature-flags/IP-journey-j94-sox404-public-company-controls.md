---
doc_class: Implementation-Plan
ip_id: IP-journey-j94-sox404-public-company-controls
journey_ref: docs/user-journeys/j94-sox-404-public-company-controls/
status: draft
date: 2026-05-20
microservice: feature-flags
flat_layout_adr: ADR-0131
related_adrs: [ADR-0159, ADR-0243, ADR-0263]
---

# IP - feature-flags role in j94 SOX 404 public-company controls

## Scope

`feature-flags` supports SOX controls by making privileged flag changes reviewable, auditable, and replayable. It does not own the ICFR control inventory or management certification packet; it owns the flag-control evidence surface consumed by those workflows.

## Service anchors

- Contract: `microservices/feature-flags/contracts/openapi-v1.yaml` mutation operations `createFlag`, `updateFlag`, `archiveFlag`, `deleteFlag`, and `undoFlagMutation`.
- Event contract: `microservices/feature-flags/contracts/asyncapi-v1.yaml` channel `flag-state-changed`.
- Capabilities: `microservices/feature-flags/capabilities/flag-evaluate.yaml` and `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`.
- Policies: `microservices/feature-flags/policy/flag-mutation-authorization.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, and `microservices/feature-flags/policy/schema.cedarschema`.
- Operations: `microservices/feature-flags/runbooks/flag-mutation-cascade.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/dashboards/flag-state-overview.json`, and `microservices/feature-flags/dashboards/killswitch-history.json`.
- Counterpart journey refs: `docs/user-journeys/j94-sox-404-public-company-controls/README.md`, `docs/user-journeys/j94-sox-404-public-company-controls/schemas/openapi-overlay-action.json`, `docs/user-journeys/j94-sox-404-public-company-controls/schemas/asyncapi-overlay-events.yaml`, and `docs/user-journeys/j94-sox-404-public-company-controls/schemas/journey-messages.proto`.

## Acceptance criteria

- Live flag updates require Class B step-up and archived hard deletes require Class C controls from `policy/flag-mutation-authorization.cedar`.
- CI can inspect flag/SLO state but cannot mutate definitions or engage kill switches under `policy/ci-scope.cedar`.
- Auditor read access is role-scoped, warrant-scoped, or investigation-scoped through `policy/auditor-scope.cedar`.
- Every SOX-relevant feature-flag mutation is traceable to `FlagStateChanged` event evidence.
- The dashboard refs are evidence views only; they do not replace sealed audit replay from `runbooks/audit-replay.md`.

## Counterparts

| Counterpart | Gap closed |
|---|---|
| LaunchDarkly | Adds Oyatie tenant-pack and Cedar-deny controls around runtime flag rollout. |
| OpenFeature | Keeps provider-compatible evaluation while binding audit-chain events and pack overlays. |
| Unleash | Adds compliance-pack override evidence beyond generic gradual rollout. |
| Statsig | Keeps experiment and rollout controls tied to service-local policy and SLO evidence. |

## Failure and rollback

- Erroneous privileged mutation: first use `undoFlagMutation`; otherwise follow `runbooks/flag-mutation-cascade.md`.
- External auditor evidence request: serve read-only event windows through `policy/auditor-scope.cedar` and `runbooks/audit-replay.md`.
- CI bypass attempt: deny by policy and emit audit evidence through the normal feature-flags event channel.
