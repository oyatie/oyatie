---
doc_class: Reference
shape: Explanation
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0245, ADR-0273]
companion_docs:
  - microservices/mail/PRD.md
  - microservices/mail/ARCHITECTURE.md
  - comms/mail/manifest.json
inbound_citations:
  - docs/README.md
  - docs/DOC-CATALOG.md
---

# Mail µservice — README

## What this µservice does

The Mail µservice is oyatie's personal + B2B email product. JMAP RFC 8620 primary; IMAP/POP3 secondary; iCalendar + Sieve for filters; per-tenant DKIM/SPF/DMARC per ADR-0273. Hyperscaler precedent: Gmail / Outlook / Apple Mail / Fastmail / ProtonMail / Hey.com.

## Quick links

- Product requirements: `PRD.md`
- Architecture walkthrough: `ARCHITECTURE.md`
- Threat model: `threat-model.md`
- DPIA: `dpia.md`
- Compliance: `compliance.md`
- Capacity model: `capacity-model.md`
- Cost budget: `cost-budget.md`
- Failure modes: `failure-modes.md`
- Multi-region: `multi-region.md`
- Incident response: `incident-response.md`
- Backfill replay: `backfill-replay.md`
- Competitor parity: `competitor-parity-matrix.md`
- SDK plan: `sdk-plan.md`
- Contracts: `contracts/openapi/mail.yaml`, `contracts/asyncapi/mail-events.yaml`, `contracts/proto/mail.proto`
- Cedar fragments: `policy/*.cedar`
- Runbooks: `runbooks/*.md`
- IPs: `IP-*.md`
- Dashboards: `dashboards/*.{json,md}`
- SLOs: `slos/*.openslo.yaml`
- Catalog: `catalog/*.yaml`
- IaC: `iac/**`

## How to consume

- Web / mobile clients: use JMAP over HTTP/3 via `contracts/openapi/mail.yaml`.
- Legacy clients: IMAP/POP3 over TLS 1.3.
- Calendar integration: iCalendar via JMAP-calendars extension.
- Server-side filters: Sieve (RFC 5228) per-tenant.

## Tenant Class Model

Mail follows ADR-0330. Customer access is modeled with `tenant_class`
(`demo_trial`, `paid`) and paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`). Demo-trial behavior is bounded by
usage and OCI Always Free constraints; paid tenants receive the same product
quality bar with billing components composed by contract.

## Status

Product, ga. Eligible for cell topology placement. HIPAA pack overlay available for B2B PHI mailboxes.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
