---
doc_class: Implementation-Plan
ip_id: IP-journey-j97-sg-pdpa-mas-tenant
journey_ref: docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/
status: draft
date: 2026-05-20
microservice: feature-flags
flat_layout_adr: ADR-0131
related_adrs: [ADR-0159, ADR-0243, ADR-0244, ADR-0251, ADR-0263]
---

# IP - feature-flags role in j97 Singapore PDPA and MAS tenant onboarding

## Scope

`feature-flags` supplies MAS/PDPA onboarding controls as tenant-scoped rollout gates, audit-required flag evaluation, pack overrides, and residency-sensitive event evidence. It does not own MAS notification, MTCS certification, or tenant legal onboarding.

## Service anchors

- Contract: `microservices/feature-flags/contracts/openapi-v1.yaml` operations `evaluateFlag`, `createFlag`, `updateFlag`, `engageKillSwitch`, and `streamFlagUpdates`.
- Event contract: `microservices/feature-flags/contracts/asyncapi-v1.yaml` channels `flag-state-changed`, `killswitch-engaged`, and `pack-override-applied`.
- Capabilities: `microservices/feature-flags/capabilities/flag-evaluate.yaml`, `microservices/feature-flags/capabilities/killswitch-trigger.yaml`, and `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`.
- Policies: `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/pack-flag-override.cedar`, and `microservices/feature-flags/policy/safety-killswitch-authorization.cedar`.
- Operations: `microservices/feature-flags/multi-region.md`, `microservices/feature-flags/runbooks/killswitch-engaged.md`, `microservices/feature-flags/runbooks/audit-replay.md`, and `microservices/feature-flags/incident-response.md`.
- Counterpart journey refs: `docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/README.md`, `docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/schemas/openapi-overlay-action.json`, `docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/schemas/asyncapi-overlay-events.yaml`, and `docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/schemas/journey-messages.proto`.

## Acceptance criteria

- Singapore tenant activation uses explicit pack/home-cell context and keeps cross-border behavior bound to `policy/data-residency.md` and `multi-region.md`.
- Critical-system flags use kill-switch surfaces only through `killswitch-trigger`, `safety-killswitch-authorization.cedar`, and `runbooks/killswitch-engaged.md`.
- Audit and regulator read paths use `auditor-scope.cedar`; no cross-tenant audit reads are documented without regulator or platform-safety scope.
- Pack-mandated restrictions are applied by the pack overlay agent and cannot be overwritten by tenant admins.
- All feature-flags evidence for j97 is limited to the real REST, gRPC, and AsyncAPI surfaces in this service.

## Counterparts

| Counterpart | Gap closed |
|---|---|
| LaunchDarkly | Adds Oyatie tenant-pack and Cedar-deny controls around runtime flag rollout. |
| OpenFeature | Keeps provider-compatible evaluation while binding audit-chain events and pack overlays. |
| Unleash | Adds compliance-pack override evidence beyond generic gradual rollout. |
| Statsig | Keeps experiment and rollout controls tied to service-local policy and SLO evidence. |

## Failure and rollback

- MAS/PDPA pack mismatch: most restrictive flag value wins; follow `runbooks/pack-override-cascade.md`.
- Kill-switch propagation delay: follow `incident-response.md` and `runbooks/killswitch-engaged.md`.
- Audit evidence gap: replay service-local events through `runbooks/audit-replay.md`.
