---
doc_id: finops-portal/dpia
authored: 2026-05-18
status: ready
authority: GDPR Article 35 + ADR-0008 data-use-boundary
classification: internal
---

# Data Protection Impact Assessment — finops-portal

This DPIA covers the GDPR-relevant processing performed by the
`finops-portal` µservice. It is mandatory because the µservice
processes per-tenant financial data and (per pack) PHI-classified
invoices.

## 1. Description of processing

`finops-portal` processes the following personal-data categories:

1. **Tenant admin identifiers** — JWT subjects (UUIDs); names if
   the tenant configured display-names.
2. **Tenant cost data** — dollar amounts per cost-center +
   workload-class. Not personal data on its own; becomes personal
   when combined with tenant-admin identity.
3. **Customer-success agent identifiers** — for credit application
   audit trail.
4. **Regulator identifiers** — for evidence-emit auth.
5. **No raw end-user data** — finops-portal does not see end-user
   PII of the tenant's customers, except via inadvertent cost-
   center labelling, which is forbidden by the cost-attribution
   standard.

## 2. Lawful basis (GDPR Art. 6)

- **Contract** (Art. 6(1)(b)): the platform contract obligates
  cost transparency.
- **Legal obligation** (Art. 6(1)(c)): regulator evidence emit
  per local regulation (PIPA, GDPR Art. 30 records, HIPAA where
  applicable).
- **Legitimate interest** (Art. 6(1)(f)): anomaly investigation +
  customer-success outreach; balanced against tenant rights via
  the cedar policies + auditability of every credit application.

## 3. Necessity and proportionality

- Only the minimum data needed is processed (tenant id, period,
  amounts).
- Per-pack overlays narrow further (e.g. US-healthcare strips
  per-line amounts to aggregates).
- Data minimization: no end-user PII enters this µservice.

## 4. Risks to data subjects

| Risk                                          | Likelihood | Impact | Mitigation                                                  |
|-----------------------------------------------|------------|--------|-------------------------------------------------------------|
| Cross-tenant invoice leak                     | low        | high   | Cedar `tenant-isolation.cedar`; defense-in-depth in IP-001  |
| PHI exposure in US-healthcare invoice         | low        | high   | `features.phiRedaction=true` overlay + Cedar forbid clause  |
| Audit trail forgery                           | very low   | high   | Append-only + Ed25519 signing + HSM key                     |
| Disproportionate retention                    | medium     | medium | Retention policy in cost-model.md (24 months online)        |
| Cross-region regulator read                   | low        | high   | Cedar `regulator-evidence-emit.cedar` double-guard          |
| Long-lived signed-URL abuse                   | medium     | low    | 5min TTL + per-download audit emit                          |

## 5. Data subject rights

- **Right of access** (Art. 15): tenants download their invoice
  via the public API; FOCUS export gives full history.
- **Right to rectification** (Art. 16): disputes go through
  customer-success → credit-ledger append (corrective).
- **Right to erasure** (Art. 17): contractually constrained
  (finops data is required for audit); see contract template.
- **Right to portability** (Art. 20): FOCUS 1.3 export is the
  portability mechanism.
- **Right to object** (Art. 21): not applicable (contract basis).

## 6. Transfers + residency

- KR data stays in KR (per `multi-region-strategy.md`).
- EU data stays in EU.
- US-healthcare in US-east only.
- Cross-region transfers: NONE for personal data. Aggregated
  fleet-rollup numbers may cross; they are not personal data.

## 7. Audit + DPO consultation

- DPO consulted: pending sign-off (tracked in
  `evidence/dpo-consult-finops-portal.json`).
- Quarterly DPIA review on every regulator-evidence emit cycle.
- Privacy impact reassessed on every major release (semver minor
  bump).

## 8. Article 30 record

The EU pack overlay emits an Article 30 record per quarter as
part of the regulator-evidence envelope. Fields:

- Controller / processor names.
- Purpose: cost transparency + regulatory evidence.
- Categories: tenant admin id, dollar amounts.
- Recipients: regulator (DPA) on request.
- Retention: 24 months online + 7 years cold storage.
- Transfers: none cross-border.

## Verification

- DPIA reviewed annually + on major release.
- DPO sign-off captured in `evidence/dpo-consult-*.json`.
- Aligned with `compliance-matrix.md`.

## References

- GDPR Articles 6, 30, 35.
- ADR-0008 data-use-boundary.
- ADR-0162 per-tenant audit-log slicing.
- `threat-model.md`.
- `compliance-matrix.md`.
