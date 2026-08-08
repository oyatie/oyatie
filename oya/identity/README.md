# identity µservice

`identity` issues and verifies principal claims for Oyatie tenants, including
the canonical `tenant_class` claim used by ADR-0330 and ADR-0331.

## Tenant Class Model

`identity` does not model customer capability tiers. Tokens and policy context
carry `tenant_class = demo_trial | paid`; paid commercial shape is supplied by
`billing_components` (`revenue_share`, `per_seat`, `per_usage`) owned by
cloud-billing and emitted as principal context for Cedar.

The service must not expose retired customer-tier fields, examples, or contract
enums. Product availability is uniform; differences are expressed as demo_trial
caps, paid billing_components, compliance_pack activation, or cell_topology and
criticality controls.

## Key Surfaces

- `ARCHITECTURE.md` and `PRD.md` describe identity substrate behavior.
- `contracts/` contains OpenAPI, AsyncAPI, and proto contracts.
- `policy/tenant-class.cedar` is the tenant_class policy anchor.
- `capabilities/tenant-class-caps.yaml` records demo_trial cap behavior.
- `slos/` and `dashboards/` expose service health without customer tiers.

## Workload identity (task T3)

The `workload-identity` bounded context (per ADR-0002) covers machine-to-machine
principals — CI runners, microservice service accounts, agents — as distinct
from the human-identity OIDC-issuer / WebAuthn / SCIM surfaces. First slice:

- `crates/oya-identity-workload-domain` — PURE domain kernel (zero deps): the
  workload principal model (tenant + workload id + claims), a lifecycle state
  machine (Provisioned → Active ⇄ Suspended → Retired), and the PARC
  authorization decision types (`AuthorizationRequest`, `AuthorizationDecision`,
  `Effect`, `DecisionReason`).
- `crates/oya-identity-workload-authz-cedar-adapter` — Cedar authorization gate:
  evaluates a PARC request over the domain with deny-by-default + forbid-wins
  semantics behind the `WorkloadAuthorizer` trait. The upstream `cedar-policy`
  crate (Apache-2.0, listed in `manifest.json#consumes_upstream_oss`) is the
  documented drop-in swap target.
- `crates/oya-identity-workload-oidc-adapter` — OIDC token validation core:
  verifies a compact JWS (RS256/384/512, ES256) against issuer JWKs using
  `ring`, validates `iss`/`aud`/`exp`/`nbf`/`iat` against a caller-supplied
  clock, and projects verified claims into a workload principal.

Verify: `cargo nextest run -p oya-identity-workload-domain \
-p oya-identity-workload-authz-cedar-adapter -p oya-identity-workload-oidc-adapter`
and `./bin/oya gate validate architecture-boundaries`.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
