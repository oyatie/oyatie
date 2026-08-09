---
doc_class: Reference
shape: Reference
microservice: comms-email
companion_docs:
  - microservices/comms-email/ARCHITECTURE.md
  - microservices/comms-email/PRD.md
related_adrs: [ADR-0201, ADR-0273]
---

# comms-email

Transactional + marketing email substrate. Hyperscaler precedents: SendGrid, Mailgun,
Postmark, Amazon SES, Resend, Mailchimp, Klaviyo. Per ADR-0273 per-tenant
DKIM/SPF/DMARC. Postal as self-hosted relay for sovereign packs.

## Bounded contexts

`outbound-delivery` / `inbound-receiving` / `template-rendering` / `list-management` /
`unsubscribe-handling` / `deliverability-tracking` / `dkim-spf-dmarc-management` /
`bounce-handling` / `reputation-monitoring`.

## Entry points

PRD.md, ARCHITECTURE.md, threat-model.md, dpia.md, compliance.md, runbooks/.

## Tenant Class Model

comms-email follows ADR-0330: tenant eligibility is expressed as
`tenant_class` (`demo_trial` or `paid`) plus paid `billing_components`
(`revenue_share`, `per_seat`, `per_usage`). Customer-facing capability
ladders are retired; deliverability, DKIM custody, warmup, compliance-pack
routing, and send-volume behavior are governed by tenant class,
compliance packs, and cell topology.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
