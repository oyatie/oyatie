---
id: ADR-0209
status: Superseded
deciders: council-architecture, axis-security, axis-compliance, axis-product, axis-platform
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0145, ADR-0153, ADR-0394, ADR-0181, ADR-0183]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0209 — Compliance evidence automation: in-house pipeline replacing Drata / Vanta

## Status

Accepted (2026-05-18). Mandates an **in-house compliance evidence pipeline** covering SOC 2 Type II, GDPR (including DSAR automation), HIPAA, and PCI-DSS — built from day one, not vendor-wrapped.

## Context

Commercial compliance-evidence vendors (Drata, Vanta, Tugboat Logic, AuditBoard, ServiceNow GRC) charge $50k-$500k/yr to wire continuous evidence collection + auditor portal access. The implementation underneath is straightforward when the underlying system has the right primitives:

- Audit-chain seal (ADR-0145 trace propagation + Bominal ADR-0028 audit chain) — every operation produces a tamper-evident seal.
- Deploy receipts (ADR-0181 container image promotion) — every prod deploy emits an attested receipt.
- Cedar policy snapshots (ADR-0183) — access-review snapshots are a Cedar policy + Zitadel role-binding diff.
- SeaweedFS storage (ADR-0145) — durable evidence storage; auditor can fetch by hash.
- First-party developer portal (ADR-0394) — read-only, capability-backed auditor module.

oyatie has all of these primitives already. Compliance-evidence automation is therefore not a 12-month integration project — it's a 4-6 week kernel + collector + portal-view stitching project that we own end-to-end.

Differentiation: every commercial vendor sits between us and the auditor; in-house puts us in direct control of evidence retention, redaction, tenant isolation, and audit-chain seal verification.

## Decision

### SOC 2 Type II continuous evidence pipeline

Continuous evidence collectors emit to SeaweedFS + audit-chain seal:

| Collector | Source | Cadence |
|---|---|---|
| CI artifact hash | every CI build (per ADR-0181) | per build |
| Deploy receipt | every prod deploy | per deploy |
| Access-review snapshot | Cedar policy + Zitadel role-bindings | weekly |
| Backup restore drill receipt | per ADR-0180 DR drills | quarterly |
| Vulnerability scan report | Trivy on every image (per ADR-0181) | per image |
| Pen-test report | annual external engagement | yearly |

Auditor portal: read-only first-party portal module (per ADR-0394) at
`/auditor/<framework>/`. It reads compliance and audit APIs through the same Cedar-gated contract
used by automation and never reads the evidence store directly. Each artifact carries an
audit-chain seal hex; the auditor verifies it through the Sigstore / Cosign chain.

### GDPR DSAR automation

DSAR API per tenant; subject can request:

- **Export** — full record set assembled from Ontology projection (per ADR-0145) across every µservice; output as encrypted zip.
- **Deletion** — Ontology cascade per ADR-0145 + `oya-dsr-domain` use-cases.
- **Rectification** — Ontology field-level update.

SLA: **30 days max** (GDPR Art. 12 statutory limit). **Target: 5 days** for automated requests.

### HIPAA

- **Minimum-necessary access logs** emitted per ADR-0174-content-now-in-standards. Each PHI access logs subject pseudonym + accessor + purpose + Cedar policy decision.
- **BAA inventory** — every sub-processor with PHI exposure tracked in `microservices/compliance/baa-inventory.json`.

### PCI-DSS

- Out-of-scope unless payments processed. If/when oyatie handles cardholder data, `microservices/payments/` lands as a Tier-1 isolated CDE (cardholder-data-environment) with separate cell + separate VPC + separate audit chain.

### Tenant isolation invariant (critical)

DSAR responses MUST NOT leak cross-tenant data. The Ontology projection traversal carries `tenant_id` at every step; the kernel rejects assembly when subject's `tenant_id` doesn't match the request's `tenant_id`. The `oya-shared-compliance-evidence-kernel` enforces this in `coverage_gaps()` (cross-tenant artifacts excluded). Per-tenant tamper-evidence verified via the audit-chain seal hex column.

### Coverage gate

`oya-check-compliance-evidence-coverage` (advisory) scans per-µservice evidence emission against required-artifact matrix per framework. Gaps drive `evidence/parent-wiring-todo-frontend-batch.json` rollout.

## Alternatives considered

### (a) Drata / Vanta SaaS — REJECTED

- **Pros:** turn-key; pre-mapped controls.
- **Cons:** vendor lock-in; SaaS sends control state off-cluster (sovereignty conflict with ADR-0164 per-regional pack); $50k-$500k/yr; opaque about how evidence is hashed.
- **Rejected**: lock-in + sovereignty + cost.

### (b) Tugboat Logic / AuditBoard / ServiceNow GRC — REJECTED

- **Pros:** enterprise GRC.
- **Cons:** same as above plus heavier integration cost.
- **Rejected**: same.

### (c) Defer to manual evidence collection (spreadsheet + Drive folder) — REJECTED

- **Pros:** zero engineering.
- **Cons:** doesn't scale; doesn't survive an auditor's continuous-evidence requirement.
- **Rejected**: doesn't meet SOC 2 Type II bar.

### (d) **CHOSEN: in-house pipeline on existing primitives**

- **Pros:**
  - Direct auditor relationship.
  - Tamper-evidence via audit-chain seal (we own the verification path).
  - No SaaS data egress.
  - Cost = engineering time (4-6 wks initial); no ongoing license.
  - Differentiation vs commercial offerings (sovereignty + transparency).
- **Cons:** we own the on-call rotation for the pipeline. Mitigation: ADR-0186 observability backplane already covers this.
- **Accepted**.

## Consequences

### Positive

1. **Direct auditor relationship.** No vendor in the middle.
2. **Tamper-evidence verifiable.** Audit-chain seal hex on every artifact.
3. **Sovereignty preserved.** Evidence never leaves the operator-owned cluster.
4. **Cost stable.** No license fee; engineering cost amortized.
5. **DSAR automation** delivers 5-day target vs 30-day statutory limit.

### Negative

1. **We own the auditor-facing portal.** Mitigation: the first-party portal module (ADR-0394)
   reuses the shared Leptos shell and composes compliance-owned read APIs; per-framework filtering
   remains owned by the compliance capability.
2. **Cross-tenant DSAR isolation is critical-path security.** Mitigation: kernel-level invariant + integration tests + audit-chain seal verification.

### Operational

- `microservices/compliance/` µservice ships the collectors + DSAR API + auditor portal view.
- `microservices/compliance/iac/helm/evidence-collector/` Helm chart deploys the collector tier.
- Standards doc at `docs/standards/compliance-evidence-automation.md`.

## In-house roadmap

**Vendor classification:** Commercial compliance-evidence pipelines (Drata, Vanta, etc.) are **vendor-replaceable conceptually**, BUT we are building **in-house from day one** because this pipeline IS our differentiation against commercial vendor-lock-in offerings.

- **100% in-house build** (not adapter wrapping a vendor).
- **Trigger:** required from day one. Compliance evidence pipeline IS our differentiation vs Drata/Vanta/Tugboat Logic/AuditBoard/ServiceNow GRC.
- **What we DO build:**
  - `oya-shared-compliance-evidence-kernel` (closed framework + artifact-kind enums + coverage matrix + DSAR-SLA tracking).
  - `oya-check-compliance-evidence-coverage` advisory gate.
  - `microservices/compliance/` µservice (collectors, DSAR API, auditor portal view).
  - SeaweedFS evidence storage binding (per ADR-0145).
  - Audit-chain seal verification (per ADR-0145 + Bominal ADR-0028).
  - First-party auditor portal module (per ADR-0394).

## Rollback

Each collector is independently feature-flagged. Rollback drops the collector; the kernel's coverage report flags the gap; no cascade failure.

## References

- SOC 2 Type II — AICPA Trust Services Criteria.
- GDPR Art. 5/12/15/16/17/30/32/33/35 — General Data Protection Regulation; EU.
- HIPAA — 45 CFR §§ 160, 162, 164.
- PCI-DSS 4.0 — PCI Security Standards Council; 2022.
- ADR-0145 — inter-microservice communication reform (Ontology projection + audit-chain seal).
- ADR-0153 — observability backplane (LGTM stack; outbox).
- ADR-0394 — first-party Rust developer portal (read-only auditor module).
- ADR-0181 — container image promotion pipeline (deploy receipts).
- ADR-0183 — policy engine separation (Cedar app-authz; Kyverno admission).
- Bominal ADR-0028 — audit chain seal substrate (inherited).
- Drata — commercial competitor; https://drata.com
- Vanta — commercial competitor; https://www.vanta.com
- AuditBoard — commercial competitor; https://www.auditboard.com
- LTS-rotation cadence: regulations current as of 2026-05-18; review per ADR-0098.
