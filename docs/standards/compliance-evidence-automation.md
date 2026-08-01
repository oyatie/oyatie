---
doc_class: Standard
shape: standard
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-18
purpose: |
  Canonical compliance-evidence pipeline standard. SOC 2 Type II + GDPR DSAR + HIPAA + PCI-DSS
  artifact kinds + emission cadence + auditor-portal access.
canonical_authority: docs/decisions/ADR-0209-compliance-evidence-automation.md
related_adrs:
  - ADR-0145
  - ADR-0394
  - ADR-0181
  - ADR-0183
  - ADR-0209
enforced_by: oya-check-compliance-evidence-coverage
---

# Compliance Evidence Automation Standard

## Authority

This standard implements ADR-0209. **In-house pipeline replacing Drata / Vanta / Tugboat /
AuditBoard / ServiceNow GRC.**

## Frameworks covered

- **SOC 2 Type II** — AICPA Trust Services Criteria.
- **GDPR** — including Art. 12 DSAR (Data Subject Access Request) automation.
- **HIPAA** — minimum-necessary access logs + BAA inventory.
- **PCI-DSS 4.0** — only when payments are in scope (defer until `microservices/payments/` lands).

## Required artifact kinds (closed enum)

| Artifact kind | Wire label | Cadence |
|---|---|---|
| CI artifact hash | `ci-artifact-hash` | per CI build |
| Deploy receipt | `deploy-receipt` | per prod deploy |
| Access-review snapshot | `access-review-snapshot` | weekly |
| Backup restore drill receipt | `backup-restore-drill-receipt` | quarterly |
| Vuln scan report | `vuln-scan-report` | per image |
| Pen test report | `pen-test-report` | yearly |
| DSAR completion record | `dsar-completion-record` | per DSAR |
| BAA inventory entry | `baa-inventory-entry` | quarterly |
| Minimum-necessary access log | `minimum-necessary-access-log` | continuous |

## Required-artifact matrix per framework

| Framework | Required artifact kinds |
|---|---|
| SOC 2 Type II | CI artifact hash + deploy receipt + access-review + backup drill + vuln scan + pen test |
| GDPR | DSAR completion record + access-review + vuln scan |
| HIPAA | Minimum-necessary access log + BAA inventory + access-review + backup drill |
| PCI-DSS | Vuln scan + pen test + access-review + deploy receipt |

## Tamper-evidence

Every artifact carries an **audit-chain seal hex (SHA-256 / 64 hex chars)** per ADR-0145.
Auditor verifies via Sigstore / Cosign chain. The kernel
(`oya-shared-compliance-evidence-kernel`) validates seal shape; the adapter does the cryptographic
verification.

## Cross-tenant isolation invariant

DSAR responses MUST NOT leak cross-tenant data. Ontology projection traversal carries `tenant_id`
at every step. Kernel rejects assembly when subject's `tenant_id` ≠ request's `tenant_id`.
Cross-tenant artifacts excluded from coverage; gate flags any cross-tenant artifact emission.

## GDPR DSAR

- **Statutory SLA: 30 days.** Per-tenant; per-subject.
- **Target SLA: 5 days.**
- API: `POST /api/v1/dsar/{export|delete|rectify}`.
- Auth: subject identity verified via Zitadel passwordless flow.
- Output (export): encrypted zip; symmetric key delivered out-of-band.

## Auditor portal

Read-only first-party portal module at `/auditor/<framework>/`. The module composes
compliance-owned, Cedar-gated read APIs and never reads the evidence store directly. Per-framework
filters show artifact inventory and audit-chain seal hex. Auditor identity is provisioned per
engagement and expires when the engagement closes.

## Coverage gate

`oya-check-compliance-evidence-coverage` (advisory) scans per-µservice evidence emission. Gaps
flagged per (microservice × framework × required artifact kind).

## Cross-references

- ADR-0209 — compliance evidence automation (this standard's authority).
- ADR-0145 — audit-chain seal substrate.
- ADR-0394 — first-party Rust developer portal (auditor module).
- ADR-0181 — container image promotion (deploy receipts).
- ADR-0183 — Cedar policy (auditor read-only access).
