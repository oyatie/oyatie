# cell-lifecycle

cell-lifecycle owns the logical Cell aggregate state machine introduced by ADR-0276 D-3.

It is not the retired generic `cell` service from ADR-0276. It is a narrow lifecycle authority for Registered, Activated, Promoted-T4, Promoted-T3, Promoted-T2, Promoted-T1, Promoted-T0, Draining, and Decommissioned states.

## Boundaries
- Infrastructure provisioning is delegated to cloud-iac.
- Tenant migration during drain is delegated to cell-rebalancer.
- Routing is delegated to api-gateway.
- Resident count and tenant-class coverage are read from tenancy.
- SLO, canary, and mesh evidence are read from observability.
- Transition evidence is sealed through audit-chain.
- Privileged operations are authorized through Cedar.

## Authoring Contents
- `PRD.md` defines product scope, personas, triggers, state machine, promotion gates, Cedar model, and compliance invariants.
- `ARCH.md` defines hexagonal architecture, layer placement, data model, diagrams, persistence, sharding, DR, and transport.
- `IPs/` carries eight implementation plans for downstream Wave 15-ZD-impl execution.
- `contracts/openapi.yaml` defines the OpenAPI 3.2.0 REST contract.
- `cedar/policies.cedar` defines policy fragments for Foundry automation, drain, decommission, and tier-specific promotion.
- `slos/cell-lifecycle.openslo.yaml` defines the service SLOs from ADR-0276 D-3.4.
- `runbooks/` covers promote, emergency drain, decommission, rollback, and on-call flows.

## Implementation Status
This scaffold is documentation and contract authoring only. No Rust crates, handlers, migrations, or deployment manifests are created in this wave.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
