---
doc_class: Implementation-Plan
ip_id: IP-journey-j95-iso27001-soc2-annual-audit
journey_ref: docs/user-journeys/j95-iso-27001-soc-2-annual-audit/
status: draft
date: 2026-05-20
microservice: feature-flags
flat_layout_adr: ADR-0131
related_adrs: [ADR-0159, ADR-0243, ADR-0251, ADR-0263]
---

# IP - feature-flags role in j95 ISO 27001, ISO 22301, and SOC 2 annual audit

## Scope

`feature-flags` contributes audit evidence for flag governance, kill-switch readiness, pack overrides, data residency, and experiment-control records. It does not own the enterprise audit program; it supplies service-local evidence through contracts, Cedar policies, SLO files, dashboards, and runbooks.

## Service anchors

- Contract: `microservices/feature-flags/contracts/openapi-v1.yaml` and `microservices/feature-flags/contracts/asyncapi-v1.yaml`.
- Capabilities: `microservices/feature-flags/capabilities/flag-evaluate.yaml`, `microservices/feature-flags/capabilities/experiment-design.yaml`, `microservices/feature-flags/capabilities/killswitch-trigger.yaml`, and `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`.
- Policies: `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, and `microservices/feature-flags/policy/safety-killswitch-authorization.cedar`.
- SLO/dashboard evidence: `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`, `microservices/feature-flags/dashboards/flag-state-overview.json`, and `microservices/feature-flags/dashboards/killswitch-history.json`.
- Operations: `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/killswitch-engaged.md`, and `microservices/feature-flags/incident-response.md`.
- Counterpart journey refs: `docs/user-journeys/j95-iso-27001-soc-2-annual-audit/README.md`, `docs/user-journeys/j95-iso-27001-soc-2-annual-audit/schemas/openapi-overlay-action.json`, `docs/user-journeys/j95-iso-27001-soc-2-annual-audit/schemas/asyncapi-overlay-events.yaml`, and `docs/user-journeys/j95-iso-27001-soc-2-annual-audit/schemas/journey-messages.proto`.

## Acceptance criteria

- Annual audit evidence includes current capability records, policy fragments, SLOs, dashboards, and runbooks from this service.
- Auditor access follows `policy/auditor-scope.cedar`; audit events remain append-only and cannot be modified or deleted by feature-flags operators.
- Kill-switch control evidence cites `capabilities/killswitch-trigger.yaml`, `policy/safety-killswitch-authorization.cedar`, and `runbooks/killswitch-engaged.md`.
- Control readiness does not claim certification by itself; `compliance.md` remains the service-local readiness narrative.
- Any event replay uses `runbooks/audit-replay.md` and the event schemas in `contracts/asyncapi-v1.yaml`.

## Counterparts

| Counterpart | Gap closed |
|---|---|
| LaunchDarkly | Adds Oyatie tenant-pack and Cedar-deny controls around runtime flag rollout. |
| OpenFeature | Keeps provider-compatible evaluation while binding audit-chain events and pack overlays. |
| Unleash | Adds compliance-pack override evidence beyond generic gradual rollout. |
| Statsig | Keeps experiment and rollout controls tied to service-local policy and SLO evidence. |

## Failure and rollback

- Missing evidence window: replay the feature-flags audit window and document any gap as an audit finding.
- SLO evidence drift: use SLO files as the source of target definitions and dashboards only as rendered evidence.
- Unauthorized auditor scope: deny and capture the denied attempt as service audit evidence.
