---
doc_class: Implementation-Plan
ip_id: IP-journey-j98-au-privacy-apra-cps234
journey_ref: docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/
status: draft
date: 2026-05-20
microservice: feature-flags
flat_layout_adr: ADR-0131
related_adrs: [ADR-0159, ADR-0243, ADR-0244, ADR-0251, ADR-0263]
---

# IP - feature-flags role in j98 Australian Privacy Act and APRA CPS 234 tenant onboarding

## Scope

`feature-flags` contributes APRA/Privacy tenant controls through safe rollout, pack-mandated overrides, audit-required evaluation evidence, and incident/kill-switch readiness. It does not own APRA asset classification or OAIC notification packets.

## Service anchors

- Contract: `microservices/feature-flags/contracts/openapi-v1.yaml` operations `evaluateFlag`, `updateFlag`, `engageKillSwitch`, `disengageKillSwitch`, and `streamFlagUpdates`.
- Event contract: `microservices/feature-flags/contracts/asyncapi-v1.yaml` channels `flag-state-changed`, `killswitch-engaged`, `killswitch-disengaged`, and `pack-override-applied`.
- Capabilities: `microservices/feature-flags/capabilities/flag-evaluate.yaml`, `microservices/feature-flags/capabilities/killswitch-trigger.yaml`, and `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`.
- Policies: `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/pack-overlay-authorization.cedar`, and `microservices/feature-flags/policy/safety-killswitch-authorization.cedar`.
- Operations: `microservices/feature-flags/incident-response.md`, `microservices/feature-flags/runbooks/killswitch-engaged.md`, `microservices/feature-flags/runbooks/pack-override-cascade.md`, and `microservices/feature-flags/runbooks/audit-replay.md`.
- Counterpart journey refs: `docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/README.md`, `docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/schemas/openapi-overlay-action.json`, `docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/schemas/asyncapi-overlay-events.yaml`, and `docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/schemas/journey-messages.proto`.

## Acceptance criteria

- APRA-critical feature changes require the same step-up and CI boundaries defined in `flag-mutation-authorization.cedar` and `ci-scope.cedar`.
- Incident and material-control-weakness evidence cites feature-flags events, dashboards, and runbooks only; external APRA notification remains counterpart scope.
- Pack overlays are Foundry-attested and emitted as `pack-override-applied`.
- Kill-switch operations for APRA-sensitive flags use the Class C gate and the `killswitch-fire-latency` SLO.
- Privacy export and regulator access stay bounded to tenant-scoped feature-flags evidence through `auditor-scope.cedar`.

## Counterparts

| Counterpart | Gap closed |
|---|---|
| LaunchDarkly | Adds Oyatie tenant-pack and Cedar-deny controls around runtime flag rollout. |
| OpenFeature | Keeps provider-compatible evaluation while binding audit-chain events and pack overlays. |
| Unleash | Adds compliance-pack override evidence beyond generic gradual rollout. |
| Statsig | Keeps experiment and rollout controls tied to service-local policy and SLO evidence. |

## Failure and rollback

- Control weakness in flag mutation: freeze high-risk mutation paths and replay event evidence.
- Incorrect APRA pack overlay: run `runbooks/pack-override-cascade.md`; do not manually edit pack-locked fields.
- Kill-switch failure: escalate as SEV-1 or SEV-2 under `incident-response.md`.
