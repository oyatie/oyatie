# cell-rebalancer

cell-rebalancer owns tenant migration across cells per ADR-0276 D-2.
This scaffold is documentation, contracts, policies, SLOs, runbooks, threat model, DPIA, and capability catalog only; no Rust code is authored in this wave.

## Canonical Documents
- PRD: PRD.md
- Architecture: ARCH.md
- Manifest: manifest.json
- OpenAPI: contracts/openapi.yaml
- AsyncAPI: contracts/asyncapi.yaml
- Cedar: cedar/policies.cedar
- SLOs: slos/cell-rebalancer.openslo.yaml
- Threat model: threat-models/threat-model.md
- DPIA: dpia/dpia.md
- Capability catalog: capabilities/capability-catalog.md

## Implementation Plans
- IPs/IP-CR-001-bounded-context-and-state-machine.md: Bounded Context And State Machine
- IPs/IP-CR-002-api-surface-rest-grpc-http3.md: API Surface REST gRPC HTTP3
- IPs/IP-CR-003-tenant-migration-workflow-state-machine.md: Tenant Migration Workflow State Machine
- IPs/IP-CR-004-residency-and-compliance-pack-validation.md: Residency And Compliance Pack Validation
- IPs/IP-CR-005-cedar-authorization.md: Cedar Authorization
- IPs/IP-CR-006-audit-chain-evidence-emission.md: Audit Chain Evidence Emission
- IPs/IP-CR-007-slo-metrics-observability.md: SLO Metrics Observability
- IPs/IP-CR-008-foundry-self-modification-boundary.md: Foundry Self Modification Boundary

## Runbooks
- runbooks/auto-rebalance-trigger.md
- runbooks/emergency-drain.md
- runbooks/compliance-pack-rotation-migration.md
- runbooks/rollback-tenant-migration.md
- runbooks/on-call.md

## Boundaries
- In scope: tenant migration across cells.
- Out of scope: cell identity, first-time tenant placement, telemetry generation, and audit schema ownership.
