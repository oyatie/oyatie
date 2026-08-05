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

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
