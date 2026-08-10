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
  - console/manifest.json
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# console

Capability-root shell substrate (ADR-0562 + ADR-0615): platform-owned workspace shell + docs-portal under `console/{ports,core,adapters,facade}`. Ops-dashboard composition leaves (incident-command, cluster-health, finops-portal, …) remain planning scaffolds under this tree pending `app/ops-console/<vertical>` absorb.

**Hyperscaler precedent:** AWS internal console (IAM-gated per-action), Stripe internal admin (step-up + full audit log), Backstage portal (service catalog + runbook), OpsLevel (ownership + SLO scorecard).

## Directory layout

```
console/
├── manifest.json                 — capability-root accounting (Seat A)
├── OWNERS
├── AUDIT-FINDINGS-2026-05-20.json
├── ports/                        — workspace-shell + docs-portal kernels
├── core/                         — usecase orchestration
├── adapters/                     — kernel→wire projection
├── facade/                       — REST boundaries + workspace-shell app
├── capabilities/                 — ops-dashboard planning capability YAML
├── catalog/                      — ops-dashboard planning Backstage rows
├── contracts/{openapi,asyncapi,proto}/
├── policy/cedar/
├── runbooks/
├── dashboards/
├── slos/                         — ops-dashboard OpenSLO planning scaffolds
├── iac/
├── dpia/
├── scorecards/
└── IPs/
```

## Bounded contexts (shell crates)

| BC | Purpose |
|---|---|
| `workspace-shell` | Surface catalog, visibility tiers, composition root (default-deny authn) |
| `docs-portal` | Hot/warm/cold extractors, tenant manifest filter, live-feed port |

## Ops-dashboard planning leaves (migration inventory)

| BC | Purpose |
|---|---|
| `incident-command` | Incident lifecycle, severity, communications, remediation handoff |
| `deployment-command` | Deployment approval, progressive rollout, freeze window, rollback |
| `cluster-health` | Cluster/node/cell/mesh health signals; bootstrap/recovery |
| `tenant-isolation-posture` | Tenant lifecycle, quota, isolation, policy posture views |
| `policy-audit-evidence` | Policy decisions, audit trail, SLO state, evidence-pack export |
| `on-call-handoff` | On-call handoff creation, ack, escalation |
| `finops-portal-integration` | FinOps cost-attribution panel integration |
| `observability-pivot` | Observability quick-pivot from any operator action |

## Quick links

- Capability-root manifest: `manifest.json`
- PRD / Architecture: deferred (`console/{PRD,ARCHITECTURE}.md` until in-tree homes land)
- DPIA: `dpia/dpia.md`
- Contracts: `contracts/openapi/`, `contracts/asyncapi/`, `contracts/proto/`
- Cedar policy: `policy/cedar/`
- Runbooks: `runbooks/`
- Dashboards: `dashboards/`
- SLOs: `slos/`
- IaC: `iac/`
- Catalog: `catalog/`
- Implementation plans: `IPs/`

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
cargo test -p console-workspace-shell-kernel
cargo test -p console-docs-portal-kernel
cargo test -p console-workspace-shell-app
cargo clippy -- -D warnings
```

Shell crates have no measured OpenSLO yet. Ops-dashboard planning SLO scaffolds live under `slos/` (e.g. `slos/command-availability.openslo.yaml`) and are not claimed as live shell SLIs.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
