---
doc_class: Readme
microservice: workplace-integration
status: Accepted
date: 2026-05-20
owner_team: axis-workplace-integration
primary_adr: ADR-0320
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0319, ADR-0320]
companion_docs: [app/workplace-integration/README.md, docs/products/workplace-integration/PRD.md, docs/standards/documentation-rigor.md]
planned_enforcement_ref: D-GOVERNANCE-CENTRAL
naming_justifications: BNF v4 service_action_resource grammar and 12-layer-enum conformance are declared inline in this document
---

# Workplace Integration

Workplace Integration is the workplace agreement, e-sign, roster, and regulated workforce integration substrate for clock-in geofence, e-sign session, offer letter, engagement agreement, roster binding, informed consent, closing package, and internal-audit DLP trace evidence.

## Tenant class model

Per ADR-0330, Workplace Integration uses `tenant_class` instead of customer
capability tiers. `evaluation_limited` tenants exercise capped onboarding, CSV export,
and preview workflows. `paid` tenants use the same product surface with
commercial terms represented through `billing_components`: `revenue_share`,
`per_seat`, and `per_usage`.

Payroll breadth, e-sign ceremony selection, work-permit checks, and regulated
workflow activation are not retired customer-tier features. They are represented
by tenant_class caps, billing_component contract terms, compliance_pack
activation, or jurisdiction-specific policy.

## Start here
- Product requirements: ../../docs/products/workplace-integration/PRD.md
- Architecture and compliance model: ../../docs/products/workplace-integration/PRD.md sections 3, 5, 8, 9, 12, and this service manifest/DPIA
- Contracts: contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/workplace-integration-v1.proto
- Policies: policies/*.cedar and cedar/policies.cedar
- Operations: runbooks/*.md, dashboards/*.json, slos/*.openslo.yaml
- Implementation anchors present in this doc set: IP-journey-j37-clock-in-geofence.md, IP-journey-j38-e-sign-session.md, and IPs/*.md

## Existing journey and implementation anchors

Implementation IPs present in this doc set:
- j37: Clock In Geofence -> IP-journey-j37-clock-in-geofence.md
- j38: E Sign Session -> IP-journey-j38-e-sign-session.md

Related journey anchors live in ../../docs/user-journeys/ and remain the source for persona/story/UX/test-plan context until a workplace-integration implementation IP exists for that journey. This README does not claim missing IP files as complete.

## Naming justifications: BNF v4 and 12-layer enum conformance

Every new artifact uses the BNF v4 grammar `<service>.<bounded_context>.<action>.<resource>` for actions and `oya-workplace-integration-<bounded-context>-<layer>` for crate and catalog names.
The ADR-0105/ADR-0565 canonical 12-layer enum used by this doc set is kernel, domain, usecase, app, adapter, infrastructure, rest, grpc, worker, cli, sdk, api.
The manifest uses the ADR-0105/ADR-0565 canonical 12-layer enum for bounded-context layers; iac, policy, and observability remain operational artifact groups rather than layer-enum values.
The service slug `workplace-integration` is retained because it is already the service directory name, policy prefix, catalog prefix, and endpoint namespace.
The primitive name `WorkplaceAgreement` is retained because it is the smallest stable object that lets the journey IP slices share one contract without leaking unrelated service ownership.
The secondary primitive `ESignSession` is retained because it names the audit-backed record that downstream services consume without taking direct table ownership.

## Doctrine references

- Current SSOT chain (`registry/stores/design-store.json`, `registry/stores/instructions-store.json`, `specs/masterplan.json`, and `specs/root-hub-pointers.json`): D-CLOUD-NATIVE and D-CICD-AUTHORITY make branch-protected `oya-ci-required` the live merge authority; local command output is transition evidence only and must not become destination enforcement.
- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): retained as historical/bridge context only where it does not conflict with the current SSOT cloud-native CI authority.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by central governance brand-residue, lane-vocabulary, and rename-inventory checks in `oya-ci-required`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- Current CD posture: declarative Kubernetes/GitOps delivery is the target; manual CLI deployment is not a merge authority. Self-hostable delivery references remain subordinate to the current SSOT and branch-protected `oya-ci-required` context.
