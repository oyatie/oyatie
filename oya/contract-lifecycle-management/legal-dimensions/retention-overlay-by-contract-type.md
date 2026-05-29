---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-006
authoritative_source: Contract-type-specific retention statutes + Statute of Limitations
related_packs: [gdpr, sox-404, hipaa-baa, sec-17a-4, kr-pipa]
date: 2026-05-21
---

# Retention Overlay by Contract Type

The CLM µservice applies the maximum applicable retention period across all active packs and contract-type-specific statutes. The default retention is 7 years (SOX-404 floor); contract-type overlays extend or shorten this.

## Retention matrix

| Contract type | Default retention | Source |
|---|---|---|
| MSA (Master Service Agreement) | 7 years from termination | SOX-404 + commercial-law statutes of limitation |
| SOW (Statement of Work) | 7 years from termination (or until MSA terminates, whichever later) | Commercial-law SOL |
| NDA (mutual) | Term of agreement + N years post-termination (typically 3-5y; perpetual for trade-secret) | Restatement (Third) of Unfair Competition + Defend Trade Secrets Act 2016 |
| NDA (unilateral) | Term of agreement + N years post-termination (typically 3-5y) | Same as mutual |
| DPA (Data Processing Addendum) | Duration of underlying contract + 3 years | GDPR Article 28(3) + UK GDPR |
| BAA (Business Associate Agreement) | 6 years from termination | HIPAA § 164.530(j)(2) |
| SaaS Subscription Agreement | 7 years from termination | SOX-404 + revenue-recognition |
| Reseller / Channel Agreement | 7 years from termination | SOX-404 |
| License Agreement (software) | Duration of licensed rights + 5 years | Copyright Act 17 USC § 507 + state breach SOL |
| Settlement Agreement | Permanent (do not destroy) | Litigation-evidence floor |
| Employment Agreement | Duration + 7 years (US) or 5 years (EU under GDPR) | EEOC + FLSA + GDPR Art. 17(3)(e) |
| IP Assignment | Permanent (do not destroy) | Patent + Copyright term outliving normal retention |
| M&A SPA (Stock Purchase Agreement) | Permanent (do not destroy) | Permanent business-record + tax basis evidence |
| Real Estate Lease | Term + 10 years (US default; longer per state) | State real-property SOL + property tax |
| Real Estate Purchase | Permanent (do not destroy) | Title chain evidence |
| Government Contract | Permanent (do not destroy) | FAR 4.703 + agency-specific rules |
| Procurement Purchase Order | 7 years from delivery | SOX-404 + tax basis |
| Vendor Agreement | 7 years from termination | SOX-404 |
| Consumer Credit | 5 years from termination | Truth in Lending Act 15 USC § 1601 + Reg Z |
| Consumer Lease | 5 years from termination | Consumer Leasing Act 15 USC § 1667 + Reg M |
| Residential Lease | Term + 5 years (state-dependent; some states require permanent) | State landlord-tenant law |
| Insurance Policy | Term + 10 years (claims-tail evidence) | NAIC Model + state insurance code |
| Telecom Service Agreement | Term + 7 years | FCC + tax + SOX |
| Utility Service Agreement | Term + 7 years | State PUC + tax |
| Marriage / Civil Union (where electronic permitted) | Permanent (do not destroy) | Family-law permanent record |
| Healthcare Patient Consent | Per HIPAA (6y) + state min (typically 7-10y) | HIPAA + state healthcare records statutes |
| Anti-Bribery (FCPA) Certification | 7 years from certification | FCPA books-and-records + SEC + DOJ |
| EU AI Act Annex III Conformity Declaration | 10 years from conformity (Regulation Article 18) | EU AI Act |
| Data Processing Agreement (SCC 2021/914) | Term + 3 years post-termination | GDPR Article 28(3) + EU SCC |

## NDA-specific retention overlay (P0 LEGAL — L-006)

NDAs are unusual in that their retention period is partially set by the contract terms themselves ("the obligations of confidentiality survive for N years post-termination"). The µservice extracts the survival clause from the NDA body and applies the longer of:

- The retention period from this table (typically 5y).
- The contract-stated survival period.
- Perpetual if the NDA references "trade secrets" (Defend Trade Secrets Act 18 USC § 1836; misappropriation actions accrue when discovered).

### Survival clause extraction

The obligation-extraction pipeline (per IP-027) parses the NDA for survival language:

- "Confidentiality obligations shall survive for [N] years following termination" → retention = max(default, N years).
- "Confidentiality obligations shall survive in perpetuity" → retention = permanent.
- "Trade secrets shall be maintained as confidential for so long as they remain trade secrets" → retention = permanent.

The extracted survival horizon is stored in `contract.metadata.confidentiality_survival_period_years` and the retention enforcer respects it.

## Composition with packs

When multiple packs apply, retention = max across all packs. Examples:

- MSA + `sec-17a-4` (broker-dealer) → 6y easily accessible per SEC rule, total = max(7, 6) = 7y.
- BAA + `sox-404` (healthcare public company) → max(6, 7) = 7y.
- NDA + `kr-pipa` (Korean trade secret) → max(perpetual NDA terms, 5y PIPA) = perpetual (NDA contract terms control).
- DPA + `gdpr` → standard 3y post-termination retention.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"ContractDelete",
  resource is Contract
) when {
  resource.retention_remaining_days > 0 ||
  resource.confidentiality_survival_active == true
};
```

## Legal hold override

When a contract is on legal hold (per `state-machines/legal-hold-state-machine.md`), retention is suspended. The contract cannot be deleted regardless of retention period until the legal hold is released.

## Audit event

`oya.contract.lifecycle.management.retention.policy_applied` with dimensions:

- tenant_id, contract_id, contract_type
- default_retention_days, applied_retention_days, retention_source
- active_packs, audit_event_id

## Implementation

The retention policy is evaluated at contract execution (signature seal) and re-evaluated on:

- Pack activation / deactivation.
- Contract amendment.
- Counterparty change.
- Legal hold application / release.

Retention enforcement happens via a daily background worker that scans for `retention_remaining_days == 0` AND `legal_hold_active == false` contracts and queues them for cryptographic erasure (key destruction → unrecoverable encrypted blob).
