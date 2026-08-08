---
doc_class: README
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0242
  - ADR-0243
  - ADR-0263
companion_docs:
  - microservices/ops-dashboard-control-center/PRD.md
  - microservices/ops-dashboard-control-center/ARCHITECTURE.md
  - microservices/ops-dashboard-control-center/manifest.json
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# ops-dashboard-control-center

Internal ops substrate for SRE, release, tenant-support, compliance, and on-call-handoff operators. Gives the platform a single Cedar-gated, step-up-auth-protected, audit-emitting control surface for every admin action.

**Hyperscaler precedent:** AWS internal console (IAM-gated per-action), Stripe internal admin (step-up + full audit log), Backstage portal (service catalog + runbook), OpsLevel (ownership + SLO scorecard).

## Bounded contexts

| BC | Purpose |
|---|---|
| `incident-command` | Incident lifecycle, severity, communications, remediation handoff |
| `deployment-command` | Deployment approval, progressive rollout, freeze window, rollback |
| `cluster-health` | Cluster/node/cell/mesh health signals; bootstrap/recovery |
| `tenant-isolation-posture` | Tenant lifecycle, quota, isolation, policy posture views |
| `policy-audit-evidence` | Policy decisions, audit trail, SLO state, evidence-pack export |
| `tenant-admin-surface` | Tenant-admin panel (delegates to above BCs) |
| `cell-operator-surface` | Cell-operator panel |
| `pack-author-surface` | Cedar fragment + compliance-pack authoring |
| `on-call-handoff` | On-call handoff creation, ack, escalation |
| `adr-promotion-triage` | ADR promotion queue triage + recommendation |
| `cedar-admin-console` | Cedar fragment publish/retire with quorum-2 step-up |
| `finops-portal-integration` | FinOps cost-attribution panel integration |
| `observability-pivot` | Observability quick-pivot from any operator action |

## Quick links

- Architecture: `ARCHITECTURE.md`
- PRD: `PRD.md`
- Phase plan: `PHASE-01-INTERNAL-OPS-DASHBOARD.md`
- Threat model: `threat-model.md`
- DPIA: `dpia.md`
- Compliance: `compliance.md`
- Contracts: `contracts/openapi/`, `contracts/asyncapi/`, `contracts/proto/`
- Cedar policy: `policy/cedar/`
- Runbooks: `runbooks/`
- Dashboards: `dashboards/`
- SLOs: `slos/`
- IaC: `iac/`
- Catalog: `catalog/`
- Implementation plans: `IP-001` through `IP-025`

## Key invariants

1. **Every admin action is Cedar-gated.** No mutation without PERMIT + step-up auth + audit emit.
2. **Every audit event is sealed.** Per ADR-0263 + ADR-0028 Merkle chain. `audit_chain_seal_required = true`.
3. **Tenant scope is enforced.** Cross-tenant pivot blocked by default-deny + RLS. Partner-agency sees sub-tenants only.
4. **Step-up auth is mandatory for mutations.** T2 = TOTP/passkey. T3 = hardware key. Cedar fragment publish = hardware key quorum-2.
5. **No standing admin tokens.** JIT credentials via OpenBao; session TTL per step-up class.
6. **WCAG 2.2 AA floor.** Internal users under high cognitive load get glanceable, keyboard-driven, dark-mode-default UX.

## Tenant Class Model

ODCC follows ADR-0330: `tenant_class` is `demo_trial` or `paid`, and paid contracts compose `billing_components` from `revenue_share`, `per_seat`, and `per_usage`. Capability manifests use `availability` rather than customer tenant_class models; demo_trial is cap-bounded or read-only where an operator action would create production risk, while paid is always-on subject to Cedar, step-up auth, and compliance-pack gates.

## UX design principles

- **Dark mode default.** Internal operators work in low-light NOC environments. Dark theme ships as default; light mode opt-in.
- **Keyboard-driven.** Every action reachable via keyboard shortcut. Command palette (⌘K / Ctrl+K) for quick navigation. No mouse-required flows.
- **Glanceable.** Critical metrics surfaced in header row; no scrolling required for P0 incident state.
- **Cognitive load minimised.** One primary action per screen; secondary actions in expandable panel. Error states are explicit with remediation steps.
- **WCAG 2.2 AA enforced.** Axe + pa11y CI runners. Colour contrast ≥4.5:1 on all text.

## Development

```bash
cargo test -p oya-ops-dashboard-control-center-incident-command-kernel
cargo test -p oya-ops-dashboard-control-center-deployment-command-app
cargo clippy -- -D warnings
```

SLO targets: `slos/command-availability.openslo.yaml` (99.9%), `slos/incident-ack-latency.openslo.yaml` (99th ≤30s).

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
