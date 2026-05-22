---
doc_class: Implementation-Plan
ip_id: IP-journey-j96-ksa-uae-mena-onboarding
journey_ref: docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/
status: draft
date: 2026-05-20
microservice: feature-flags
flat_layout_adr: ADR-0131
related_adrs: [ADR-0159, ADR-0243, ADR-0244, ADR-0251]
---

# IP - feature-flags role in j96 KSA and UAE MENA tenant onboarding

## Scope

`feature-flags` owns jurisdiction-scoped enablement during MENA tenant onboarding: flags remain tenant-scoped, residency-aware, and pack-overridable. It does not own corporate onboarding, regional legal review, or cloud-region procurement.

## Service anchors

- Contract: `microservices/feature-flags/contracts/openapi-v1.yaml` operations `evaluateFlag`, `listFlags`, `updateFlag`, and `streamFlagUpdates`.
- Event contract: `microservices/feature-flags/contracts/asyncapi-v1.yaml` channels `flag-state-changed` and `pack-override-applied`.
- Capabilities: `microservices/feature-flags/capabilities/flag-evaluate.yaml`, `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`, and `microservices/feature-flags/capabilities/killswitch-trigger.yaml`.
- Policies: `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/tenant-targeting.cedar`, `microservices/feature-flags/policy/pack-overlay-authorization.cedar`, and `microservices/feature-flags/policy/safety-killswitch-authorization.cedar`.
- Runtime/IaC anchors: `microservices/feature-flags/multi-region.md`, `microservices/feature-flags/iac/network-policy.yaml`, `microservices/feature-flags/iac/secret-bindings.yaml`, and `microservices/feature-flags/iac/openbao-policy.hcl`.
- Counterpart journey refs: `docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md`, `docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/schemas/openapi-overlay-action.json`, `docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/schemas/asyncapi-overlay-events.yaml`, and `docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/schemas/journey-messages.proto`.

## Acceptance criteria

- MENA tenant flags evaluate only with explicit `tenant_id` and home-cell context; cross-tenant reads remain denied.
- Pack activation flows through `pack-overlay-subscribe` and `policy/pack-overlay-authorization.cedar`; manual tenant-admin override of pack-locked fields remains forbidden.
- Residency posture references `policy/data-residency.md` and `multi-region.md`; this IP does not invent a KSA/UAE region file that is not present.
- Secret and network controls cite the existing IaC files only.
- Kill-switch readiness remains available through `killswitch-trigger` but cannot bypass life-safety forbids in `policy/safety-killswitch-authorization.cedar`.

## Counterparts

| Counterpart | Gap closed |
|---|---|
| LaunchDarkly | Adds Oyatie tenant-pack and Cedar-deny controls around runtime flag rollout. |
| OpenFeature | Keeps provider-compatible evaluation while binding audit-chain events and pack overlays. |
| Unleash | Adds compliance-pack override evidence beyond generic gradual rollout. |
| Statsig | Keeps experiment and rollout controls tied to service-local policy and SLO evidence. |

## Failure and rollback

- Wrong-region flag state: stop high-risk mutations, keep the stricter default, and follow `multi-region.md`.
- Pack overlay error: execute `runbooks/pack-override-cascade.md`.
- Secret binding issue: treat as deployment blocker; do not loosen Cedar or network policy to work around it.
