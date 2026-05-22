---
doc_class: Taxonomy
microservice: contract-lifecycle-management
dimension_id: S-004
related_packs: [gdpr, eidas, esign, sox-404, hipaa-baa, kr-pipa]
date: 2026-05-21
---

# Clause Family Taxonomy

Canonical clause families recognized by CLM. The taxonomy informs IP-027 (obligation extraction), IP-026 (deviation classification), `clause-library-inheritance.md`, and the AI redlining prompt.

## Canonical clause families

| Family | Description | Standard playbook position | Typical fallback |
|---|---|---|---|
| `TermAndTermination` | Contract duration, renewal, termination rights | Initial term + auto-renewal; either-party termination for material breach with cure | Termination for convenience with notice |
| `Indemnification` | Indemnity for third-party claims | Mutual indemnity for IP infringement; one-way for data breach by other party | Carve out for gross negligence |
| `LimitationOfLiability` | Cap on damages, exclusion of consequential | 12-month-fees cap; mutual exclusion of consequential | 2x or 3x fees cap |
| `Confidentiality` | NDA-style confidentiality | Mutual, 5-year survival post-termination | Perpetual for trade secrets |
| `DataProtection` | DPA / privacy provisions | GDPR Article 28 DPA + SCCs as applicable | KR-PIPA Article 32 if KR |
| `ServiceLevels` | SLA, uptime, response time | 99.9% monthly availability, service credits | 99.5%, no service credits |
| `PaymentTerms` | Net days, late fees, taxes | Net-30, 1.5%/mo late fee, taxes exclusive | Net-60 |
| `Assignment` | Assignment, change of control | No assignment without consent (other party); allow assignment for affiliates | Allow assignment on change of control |
| `GoverningLaw` | Choice of governing law | Delaware (US default); Frankfurt am Main (DE); London (UK); Seoul (KR) | Mutually-agreed neutral state |
| `DisputeResolution` | Arbitration, jurisdiction, venue | Binding arbitration (AAA / ICDR / KCAB / JCAA); seat per governing law | Court venue per governing law |
| `Insurance` | Required insurance coverage | $5M GL + $2M E&O + $10M Cyber for B2B | $1M GL only |
| `ForceMajeure` | Performance suspension on qualifying events | ICC 2020 catch-all + enumerated pandemic | Acts of God only |
| `IPOwnership` | IP ownership, license grants | Each party retains pre-existing IP; work-for-hire for deliverables | Joint ownership |
| `Warranty` | Performance warranty, disclaimers | Express warranty for material defects 12 months; disclaim implied warranties | Limited 90-day warranty |
| `AuditRights` | Right to audit books, financial records | Annual audit on 30 days notice, audit costs at auditor's expense | Audit at requesting party's cost |
| `Survival` | Which obligations survive termination | Confidentiality, indemnity, IP, limitation of liability survive | Add-on per request |
| `MostFavoredNation` | MFN pricing or terms | NOT in standard; only on request | If requested: 12-month MFN lookback |
| `AntiCorruption` | FCPA / UKBA / OECD certifications | Mutual certification, annual recertification | Per `legal-dimensions/fcpa-ukba-detection.md` |
| `ChangeOfControl` | Notification and consent on M&A | 30 days notice, consent not unreasonably withheld | Auto-termination on competitor acquisition |
| `Subcontracting` | Right to subcontract performance | Allowed with notice; subcontractors bound by same terms | Subcontractor approval required |
| `NoticeAndCure` | Notice + cure period before consequence | 30 days cure for non-monetary breach, 10 days for payment | Per `legal-dimensions/notice-and-cure-obligation.md` |
| `ExportControl` | EAR / ITAR / EU dual-use compliance | Mutual export-control certification | Per industry |
| `OpenSource` | Open-source software handling | Disclose, no copyleft contamination | Indemnify open-source claims |
| `RecordKeeping` | Books and records retention | 7 years from termination per SOX-404 | Per `legal-dimensions/retention-overlay-by-contract-type.md` |
| `NonSolicitation` | No-poach, customer non-solicit | 12 months post-termination, key employees only | Reduced scope |
| `Publicity` | Press release, case study consent | Each party consents in writing before publicity | Allow generic logo usage |
| `Definitions` | Definitions section | Standard definitions per playbook | Cross-reference to defined terms |
| `EntireAgreement` | Integration clause | Standard | Per playbook |
| `Severability` | Severability of unenforceable provisions | Standard | Per playbook |
| `Counterparts` | Execution in counterparts | Standard, electronic counterparts permitted | Per `packs/esign/README.md` |
| `RemoteHearings` | Remote arbitration / mediation | Permitted with adequate technology | Per playbook |

## Family detection (extraction pipeline)

The AI redlining + obligation extraction pipeline detects clause family by:

- Title heuristics ("Termination", "Limitation of Liability", "Force Majeure").
- Lexical patterns specific to each family.
- Structural patterns (clause position in standard contract structure).

Multi-family clauses (e.g. a clause covering both Confidentiality and Non-Solicitation) are tagged with all applicable families.

## Family-to-pack mapping

| Family | Triggers pack |
|---|---|
| `DataProtection` | `gdpr` (EU), `kr-pipa` (KR), DPA contract type |
| `AntiCorruption` | `fcpa-ukba` overlay; `sox-404` for public companies |
| `RecordKeeping` | `sox-404` retention floor |
| `Confidentiality` (TradeSecret) | DTSA recognition |
| `Indemnification` (HIPAA-specific) | `hipaa-baa` overlay |
| `ServiceLevels` (broker-dealer SLAs) | `sec-17a-4` overlay |

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"ClauseEdit",
  resource is Clause
) when {
  resource.family == "AntiCorruption" &&
  resource.template.prohibited_modifications.contains("TextEditForbidden")
};
```

## Audit events

- `oya.contract.lifecycle.management.clause.family_detected`
- `oya.contract.lifecycle.management.clause.family_misclassified` (after human review)
- `oya.contract.lifecycle.management.clause.pack_triggered`

## Standards references

- IACCM (now WorldCC) Contract Standards.
- ABA Model Stock Purchase Agreement.
- ICC Force Majeure Clause 2020.
- AICPA Top Risks in Contracts.
