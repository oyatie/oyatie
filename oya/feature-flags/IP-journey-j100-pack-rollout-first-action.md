---
doc_class: Implementation-Plan
ip_id: IP-journey-j100-pack-rollout-first-action
journey_ref: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/
status: draft
date: 2026-05-20
microservice: feature-flags
flat_layout_adr: ADR-0131
related_adrs: [ADR-0159, ADR-0243, ADR-0244, ADR-0251, ADR-0263]
---

# IP - feature-flags role in j100 pack rollout from tenant onboarding to first action

## Scope

`feature-flags` turns a tenant pack activation into safe first-action behavior: pack overlay subscription, flag evaluation defaults, rollout-state propagation, kill-switch readiness, and audit evidence. It does not own tenant onboarding itself.

## Service anchors

- Contract: `microservices/feature-flags/contracts/openapi-v1.yaml` operations `evaluateFlag`, `listFlags`, `updateFlag`, `engageKillSwitch`, and `streamFlagUpdates`.
- Event contract: `microservices/feature-flags/contracts/asyncapi-v1.yaml` channels `pack-override-applied`, `flag-state-changed`, `killswitch-engaged`, and `abuse-detection-signal`.
- RPC contract: `microservices/feature-flags/contracts/feature-flags-v1.proto` services `FlagEvaluationService`, `FlagManagementService`, and `KillSwitchService`.
- Capabilities: `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`, `microservices/feature-flags/capabilities/flag-evaluate.yaml`, and `microservices/feature-flags/capabilities/killswitch-trigger.yaml`.
- Policies: `microservices/feature-flags/policy/pack-overlay-authorization.cedar`, `microservices/feature-flags/policy/pack-flag-override.cedar`, `microservices/feature-flags/policy/tenant-targeting.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, and `microservices/feature-flags/policy/safety-killswitch-authorization.cedar`.
- Operations: `microservices/feature-flags/runbooks/pack-override-cascade.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/killswitch-engaged.md`, and `microservices/feature-flags/incident-response.md`.
- Counterpart journey refs: `docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md`, `docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/schemas/openapi-overlay-action.json`, `docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/schemas/asyncapi-overlay-events.yaml`, and `docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/schemas/journey-messages.proto`.

## Acceptance criteria

- First-action readiness begins only after pack overlay subscription is attested and pack-mandated overrides are applied.
- First request evaluates through `flag-evaluate` with tenant context and safe defaults; missing flags return default/off behavior rather than exposing cross-tenant state.
- Pack activation emits `PackOverrideApplied` and any resulting `FlagStateChanged` event.
- CI may verify rollout state but cannot mutate flags or engage kill switches.
- Kill-switch fallback is operational before first action for any high-risk flag path.

## Counterparts

| Counterpart | Gap closed |
|---|---|
| LaunchDarkly | Adds Oyatie tenant-pack and Cedar-deny controls around runtime flag rollout. |
| OpenFeature | Keeps provider-compatible evaluation while binding audit-chain events and pack overlays. |
| Unleash | Adds compliance-pack override evidence beyond generic gradual rollout. |
| Statsig | Keeps experiment and rollout controls tied to service-local policy and SLO evidence. |

## Failure and rollback

- Pack overlay late or missing: keep first-action flags disabled and use `runbooks/pack-override-cascade.md`.
- Evaluation regression on first action: follow `runbooks/flag-evaluation-regression.md`.
- Kill-switch need during first-action incident: use `runbooks/killswitch-engaged.md` and the SEV path in `incident-response.md`.
