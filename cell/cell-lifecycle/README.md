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

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `governance-verify-ci-mirror-coverage`, `governance-verify-ci-step-exit-semantics`, `governance-verify-skip-flag-allowlist`, `governance-submit-calls-verify`, and `governance-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `governance-*` CI lane prefix RENAMES to `governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `governance-no-foundry-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `governance-jenkins-github-actions-parity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-jenkins-jcasc-only`, and `governance-deploy-audit-chain-emit`.
