---
doc_class: Phase
phase_id: PHASE-01
status: in-progress
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0242
  - ADR-0243
  - ADR-0248
  - ADR-0263
companion_docs:
  - microservices/ops-dashboard-control-center/PRD.md
  - microservices/ops-dashboard-control-center/ARCHITECTURE.md
planned_enforcement_ref: oya-governance-microservice-doc-suite
---

# PHASE-01 — Internal Ops Dashboard Buildout

## Objective

Ship a Cedar-gated, step-up-auth-protected, audit-emitting ops dashboard control center that gives SRE, release, tenant-support, compliance, and on-call-handoff operators a single governed control surface.

## Success criteria (phase exit gate)

- [ ] All 13 BCs have kernel + domain + usecase + app + api + rest + worker + adapter crates passing CI.
- [ ] Cedar policy fixtures pass `cedar-policy-tests` for all PERMIT + FORBID paths.
- [ ] SLO windows: 7d green on all 7 SLOs before promotion.
- [ ] Restore evidence: at least one DR failover drill documented.
- [ ] Evidence-pack export: signed pack verifiable offline.
- [ ] Step-up auth: T2 and T3 classes exercised in integration tests.
- [ ] WCAG 2.2 AA: axe + pa11y CI runners passing on all dashboard panels.

## Milestones

### M1 — Contracts + Cedar policy (IP-001 through IP-003)
- Manifest + OpenAPI + AsyncAPI + Proto3 accepted. ✅
- Cedar default-deny + admin-action-authorization + step-up-auth-required. IP-010 through IP-013.
- SLOs authored (7 OpenSLO files). ✅

### M2 — Core BCs (IP-004 through IP-009)
- incident-command kernel → usecase → app → rest.
- deployment-command kernel → usecase → app → rest.
- cluster-health kernel → usecase.
- tenant-isolation-posture kernel → usecase.
- policy-audit-evidence kernel → usecase.

### M3 — Dashboard panels (IP-014 through IP-019)
- ops-overview dashboard. Grafana JSON.
- tenant-admin-surface dashboard.
- cell-operator dashboard.
- pack-author dashboard.
- on-call-handoff panel.
- admin-action-audit-stream dashboard.

### M4 — Specialised surfaces (IP-020 through IP-022)
- Step-up auth flow implementation (IP-020).
- Audit-emission integration (IP-021).
- Cedar-admin-console + quorum gate (IP-022).

### M5 — Observability + IaC (IP-023 through IP-025)
- All 5 IaC files (K8s, Helm, Terraform, ECH, PQC, credential sidecar, SPIFFE kill-switch, edge WAF).
- Grafana dashboards finalized.
- Load tests + chaos drills.

## Dependencies

| Dependency | Status |
|---|---|
| `observability` µservice (SLO reads) | GA |
| `tenancy` µservice (tenant posture) | GA |
| `policy-engine` (Cedar eval) | GA |
| `cloud-secrets` (OpenBao) | GA |
| `foundry` (admission gate) | GA |
| `detection` µservice (UEBA) | Wave-3-D |
| ADR-0298 (emergency-services bypass) | Wave-3 backlog |

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Cedar quorum-2 step-up UX complexity | Medium | High | Prototype with YubiKey + FIDO2 passkey; fallback to backup key recovery path |
| UEBA detection µservice not available in wave-1 | High | Medium | Stub detection feed; alert on stubbed path; enable when detection µservice ships |
| Session recording storage growth | Medium | Low | Tiered retention: T3 sessions 1yr; T2 sessions 90d; auto-expire to cold storage |
| WCAG 2.2 AA failure on complex action dialogs | Low | Medium | Axe + pa11y in CI; block on violation before merge |
