---
doc_status: published
---

# Team: Vertical — Healthcare (Clinical / Ambulatory / HL7-FHIR)

## Mission
This team owns the healthcare vertical: clinical workflows, ambulatory care, HL7-FHIR data model, e-prescribing, claims processing, and medication/allergy/problem reconciliation across KR, US, EU, JP, and other regional packs. It exists because healthcare is the highest-sensitivity vertical — PHI is permanently and unconditionally blocked from ad targeting, the audit chain has zero tolerance for gaps, and regulatory evidence (MFDS, FDA, EMA, PMDA) must be collected on every regulated capability invocation. It does **not** own the underlying SaaS or Foundry infrastructure.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Healthcare (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-healthcare-kernel` — `Patient`, `Encounter`, `Observation`, `MedicationStatement`, `AllergyIntolerance`, `Condition`, `Claim`
  - `vertical-healthcare-domain-*` — clinical lifecycle, FHIR resource management, break-glass, e-prescribing, claims adjudication
  - `vertical-healthcare-adapter-fhir` — FHIR R4/R5 adapter; KR EDI 보건의료 adapter
  - Per-region extensions: `pack-kr` → `KrPatientId`, `KrNhisPayer`, `KrRRN`; `pack-us` → HIPAA/HITECH controls, FDA adapter; `pack-eu` → EMA adapter, GDPR Art-9 health data
  - Products owned: `products/vertical-healthcare/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — break-glass invocations, clinical events, de-id proofs)
  - `DSR / consent withdrawal cascade` (ack required — highest sensitivity)
  - All PHI fields forced to `ad_targetable_blocked` and `internal_only` (Data Use Boundary vertical override)
- **Catalog records:** `crates/vertical-healthcare-*`
- **Runbooks:** `runbooks/healthcare-break-glass.md`, `runbooks/fhir-resource-dsr.md`, `runbooks/clinical-audit-replay.md`
- **ADRs:** ADR-0016 (FHIR schema), ADR-0033 (clinical AI constraints), MFDS compliance ADR

## In-scope work
- FHIR R4/R5 resource model: Patient, Encounter, Observation, Condition, MedicationStatement, AllergyIntolerance, DiagnosticReport, Claim
- Clinical terminology and e-prescribing mappings: ICD-10-CM, SNOMED CT, RxNorm, and NCPDP SCRIPT stay owned by the healthcare vertical with ops-compliance review.
- Clinical workflow authoring: ambulatory visit, inpatient admission, discharge, e-prescribing, prior authorization
- Break-glass: emergency access override with mandatory audit-chain emission and post-event review
- Claims processing: KR NHIS claim submission (EDI 보건의료 format), US HIPAA 837/835, EU reimbursement formats
- De-identification: k-anonymous clinical data for analytics (never feeds ads)
- Regional pack seam implementations: KR (MFDS controls, 건강보험 EDI), US (HIPAA/HITECH, FDA), EU (EMA, GDPR Art-9), JP (PMDA, 健康保険)
- Regulatory evidence collection via Foundry agent (MFDS controls, FDA 21 CFR Part 11)
- Clinical AI capability authoring under strict autonomy ceiling (ADR-0033 clinical-AI constraints)

## Out-of-scope (anti-scope)
- PHI in any ad-targeting or analytics feedback loop — always blocked permanently
- Consumer health apps outside regulated tenant context
- SaaS workflow engine (→ `axis-saas`)
- Cloud infrastructure (→ `axis-cloud`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-privacy-dub` | Healthcare PHI forced override in Data Use Boundary | ADR lifecycle |
| `platform-audit-evidence` | Break-glass audit records, clinical event chain | Per event |
| `axis-saas` | Workflow engine for clinical workflows | Per-release |
| `axis-foundry` | Capability invocation for clinical AI under autonomy ceiling | Wave gate |
| `ops-compliance` | MFDS / FDA / EMA / PMDA regulatory watch | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `ops-compliance` | MFDS / HIPAA control evidence packs | Monthly + audit |
| `gtm-customer-success` | Healthcare tenant health dashboards | Monthly |

## Success metrics
- **PHI in any shared index or ad signal:** 0 (hard zero, permanent)
- **Break-glass audit chain completeness:** 100%
- **FHIR resource DSR completion:** 100% within 24 h
- **Clinical AI capability autonomy-ceiling coverage:** 100% of regulated capabilities
- **MFDS/FDA evidence pack regeneration time:** ≤ 4 h (PRD §4.2)

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for FHIR kernel contract changes; privacy council for any PHI class dispute
- Compliance: `ops-compliance` for MFDS / FDA / EMA regulatory incidents
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 45-min sync — clinical AI capability queue, FHIR schema changes, regulatory evidence status
- Cross-team review: monthly compliance review with `ops-compliance`

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; PHI-adjacent PRs require security-reviewer + privacy-reviewer
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; clinical AI constraints (ADR-0033) are P0

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| PHI enters any shared data path | Catastrophic | Forced `ad_targetable_blocked` + `internal_only`; fitness gate; zero-tolerance policy |
| Break-glass invoked without audit record | Catastrophic | `platform-audit-evidence` fitness function hard-fails on missing emission |
| Clinical AI capability exceeds autonomy ceiling | Catastrophic | ADR-0033 constraints + Cedar policy gate + mandatory break-glass review |
| MFDS regulatory change not captured | High | Monthly regulatory watch via `ops-compliance`; regional-pack versioning |

## Sources scanned
PRD.md §3.1 (W-Vertical-Fan-Out), DESIGN.md §10, ADR-0016, ADR-0033, PRIVACY-PROGRAM.md, products/vertical-healthcare/PRD.md (draft).
