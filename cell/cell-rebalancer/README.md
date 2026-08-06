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

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0709-general-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
