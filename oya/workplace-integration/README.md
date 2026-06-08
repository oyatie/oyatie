---
doc_class: Readme
microservice: workplace-integration
status: Accepted
date: 2026-05-20
owner_team: axis-workplace-integration
primary_adr: ADR-0320
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0319, ADR-0320]
companion_docs: [microservices/workplace-integration/README.md, docs/standards/documentation-rigor.md]
planned_enforcement_ref: oya-governance-workplace-integration-doc-set
naming_justifications: BNF v4 service_action_resource grammar and 13-layer-enum conformance are declared inline in this document
---

# Workplace Integration

Workplace Integration is the workplace agreement, e-sign, roster, and regulated workforce integration substrate for clock-in geofence, e-sign session, offer letter, engagement agreement, roster binding, informed consent, closing package, and internal-audit DLP trace evidence.

## Tenant class model

Per ADR-0330, Workplace Integration uses `tenant_class` instead of customer
capability tiers. `demo_trial` tenants exercise capped onboarding, CSV export,
and preview workflows. `paid` tenants use the same product surface with
commercial terms represented through `billing_components`: `revenue_share`,
`per_seat`, and `per_usage`.

Payroll breadth, e-sign ceremony selection, work-permit checks, and regulated
workflow activation are not retired customer-tier features. They are represented
by tenant_class caps, billing_component contract terms, compliance_pack
activation, or jurisdiction-specific policy.

## Start here
- Product requirements: PRD.md
- Architecture: ARCHITECTURE.md
- Compliance: compliance.md
- Contracts: contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/workplace-integration-v1.proto
- Policies: policies/*.cedar
- Operations: runbooks/*.md, dashboards/*.json, slos/*.openslo.yaml
- Implementation sequence: ip/IP-001-*.md through ip/IP-025-*.md

## Existing journey IP anchors
- j109: Esign Roster Binding -> IP-journey-j109-esign-roster-binding.md
- j110: Esign Roster Binding -> IP-journey-j110-esign-roster-binding.md
- j112: Esign Roster Binding -> IP-journey-j112-esign-roster-binding.md
- j113: Esign Roster Binding -> IP-journey-j113-esign-roster-binding.md
- j114: Esign Roster Binding -> IP-journey-j114-esign-roster-binding.md
- j121: Esign Closing Package -> IP-journey-j121-esign-closing-package.md
- j132: Offer Letter Esign Per Jurisdiction -> IP-journey-j132-offer-letter-esign-per-jurisdiction.md
- j134: Engagement Agreement And Staffing Aware Offer -> IP-journey-j134-engagement-agreement-and-staffing-aware-offer.md
- j140: Internal Audit Dlp Egress Cross Tenant Trace -> IP-journey-j140-internal-audit-dlp-egress-cross-tenant-trace.md
- j37: Clock In Geofence -> IP-journey-j37-clock-in-geofence.md
- j38: E Sign Session -> IP-journey-j38-e-sign-session.md
- j51: E Sign On Po -> IP-journey-j51-e-sign-on-po.md
- j54: E Signature -> IP-journey-j54-e-signature.md
- j56: Offer E Sign -> IP-journey-j56-offer-e-sign.md
- j63: Informed Consent -> IP-journey-j63-informed-consent.md
- j70: E Sign -> IP-journey-j70-e-sign.md

## Naming justifications: BNF v4 and 12-layer enum conformance

Every new artifact uses the BNF v4 grammar `<service>.<bounded_context>.<action>.<resource>` for actions and `oya-workplace-integration-<bounded-context>-<layer>` for crate and catalog names.
The ADR-0105 canonical 13-layer enum used by this doc set is kernel, domain, usecase, app, adapter, infrastructure, rest, grpc, graphql, worker, cli, sdk, api.
The doc set keeps ADR-0105 compatibility by mapping the 12 deployable layers into the larger canonical enum without inventing a new layer name.
The service slug `workplace-integration` is retained because it is already the microservice directory name, policy prefix, catalog prefix, and endpoint namespace.
The primitive name `WorkplaceAgreement` is retained because it is the smallest stable object that lets the journey IP slices share one contract without leaking unrelated service ownership.
The secondary primitive `ESignSession` is retained because it names the audit-backed record that downstream services consume without taking direct table ownership.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
