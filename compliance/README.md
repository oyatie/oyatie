---
doc_class: Reference
shape: Reference
length_cap: 300
microservice: compliance
companion_docs:
  - microservices/compliance/ARCHITECTURE.md
  - microservices/compliance/PRD.md
related_adrs:
  - ADR-0209
  - ADR-0212
inbound_citations:
  - docs/DOC-CATALOG.md
---

# compliance

The compliance substrate. Per-pack overlay registry + DPIA orchestration + breach-notification
workflow + regulator-audit-evidence surface + cell-certification attestation +
compliance-control mapping. In-house replacement for Drata / Vanta / Tugboat Logic / OneTrust /
AuditBoard / ServiceNow GRC / AWS Audit Manager.

## Entry points

- `PRD.md` — product requirements.
- `ARCHITECTURE.md` — architecture walkthrough.
- `threat-model.md` — STRIDE-style threats.
- `dpia.md` — Article 35 DPIA.
- `compliance.md` — pack overlays + control mapping.
- `runbooks/` — operational procedures.

## Hyperscaler precedents

AWS Audit Manager, Drata, Vanta, Tugboat Logic, OneTrust, AuditBoard, ServiceNow GRC.

## Bounded contexts

`pack-registry` / `dpia-orchestration` / `breach-notification-workflow` /
`regulator-audit-evidence` / `cell-certification-attestation` / `compliance-control-mapping`.

## Tenant Class Model

compliance follows ADR-0330. The service no longer models customer capability
levels. `tenant_class` is either `demo_trial` or `paid`; paid commercial shape
is composed from `billing_components` (`revenue_share`, `per_seat`,
`per_usage`). Pack activation, residency, regulator evidence, and air-gap
custody are expressed through `compliance_pack` and `cell_topology`, not a
customer ladder.

Canonical model: `docs/decisions/ADR-0702-identity-authz-live-apex.md`.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `governance-verify-ci-mirror-coverage`, `governance-verify-ci-step-exit-semantics`, `governance-verify-skip-flag-allowlist`, `governance-submit-calls-verify`, and `governance-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `governance-*` CI lane prefix RENAMES to `governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `governance-no-foundry-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `governance-jenkins-github-actions-parity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-jenkins-jcasc-only`, and `governance-deploy-audit-chain-emit`.
