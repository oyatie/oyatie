---
doc_class: MilestoneReadme
template_id: TPL-MILE-README
milestone_id: M03-first-tenant
parent: ../../../docs/MASTERPLAN.md
status: Proposed
entry_gate: |
  M02-substrate Complete; all 22 M02 phases green; all 14+ CI lanes BLOCKER;
  Application B2B shell deployed to OCI ARM64 Stage 0 cell.
  KR localization pack manifest signed; corpus.lock signed.
exit_gate: |
  - ADR-0210 35-checkbox M3 closure evidence bundle signed
    (PAYING + EDI + YEAR-END + LEGAL-HOLD + SLO + CORPUS + AUDIT + RESTORE + ONBOARD)
  - 1 KR group paying tenant live on oyatie HR + Payroll + Accounting +
    Connect Professional Mail + Connect Professional Messenger
  - 4대보험 EDI submissions green (NPS / NHIS / 고용 / 산재)
  - 연말정산 21-category cycle sealed (audit chain Ed25519-segmented per tenant + period)
  - Connect Pro legal hold + eDiscovery evidence verified (four-eyes release)
  - 7-day SLO held (p99 read ≤50ms, write ≤200ms, 10k+ RPS per cell)
  - Workflow Studio hero product live with ≥10 M3 launch templates
  - oya-check-doc-coverage --blocker exits 0 for every M03 µservice
  - git tag m3-tenant-live emitted
  - ICM milestone-complete row emitted
owner_team: axis-enterprise + axis-connect + gtm-customer-success-kr
bominal_adrs_inherited:
  - ADR-0019  # active-active compatibility
  - ADR-0028  # audit chain
  - ADR-0035  # workflow engine
  - ADR-0049  # cross-region replication
  - ADR-0111  # tenant DEK
  - ADR-0118  # tenant onboarding ≤5min
  - ADR-0123  # OIDC + passkey
  - ADR-0125  # domain naming canon
  - ADR-0126  # 8-class employment classification
  - ADR-0190  # versioned regulatory corpus.lock
  - ADR-0208  # connect dual-context unified channel hub
  - ADR-0210  # M3 KR group payroll + mail production launch
  - ADR-0215  # connect retention legal-hold dual-context
oyatie_adrs_cited:
  - ADR-0056  # BNF v4.1
  - ADR-0061  # Application B2B unified shell
  - ADR-0063  # documentation suite coverage
  - ADR-0064  # canonical base + KR localization pack #1
---

# M03-first-tenant — 1 KR paying tenant live (HR + Payroll + Accounting + Connect Pro Mail/Messenger + Workflow Studio)

## Intent

First-paying-tenant GA. Equivalent to Bominal M3 per ADR-0210. Ship the canonical product to a paying KR group tenant using the KR localization pack (pack #1). Demonstrates the canonical-base + pack architecture (ADR-0064) end-to-end: every µservice in scope has canonical base shipped + KR pack overlay shipped + acceptance evidence signed.

## Phase index

| Phase ID | Path | Scope summary |
|---|---|---|
| P01-hr | `phases/P01-hr/` | HR µservice canonical kernel + KR pack (8-class employment per ADR-0126) |
| P02-payroll | `phases/P02-payroll/` | Payroll gross-to-net + KR pack (4대보험 EDI v5.0 + 연말정산 21-category + 간이세액표) |
| P03-accounting | `phases/P03-accounting/` | Accounting double-entry + KR pack (K-GAAP COA + 재무상태표/손익계산서/현금흐름표/자본변동표 Typst) |
| P04-connect-pro-mail | `phases/P04-connect-pro-mail/` | Connect Pro Mail dual-context + KR retention (Bominal ADR-0215 KR-mode) + eDiscovery + legal hold |
| P05-connect-pro-messenger | `phases/P05-connect-pro-messenger/` | Connect Pro Messenger PQXDH + Signal double-ratchet + InternalAuditable threads + Workflow deep-links |
| P06-application-b2b-live | `phases/P06-application-b2b-live/` | Application B2B shell live (OIDC/SAML SSO + product-enablement console + Leptos SSR/SPA + ≤5-min onboarding) |
| P07-workflow-studio-editor | `phases/P07-workflow-studio-editor/` | Workflow Studio visual editor (Leptos WASM canvas + 10 KR-pack domain templates + agentic LLM nodes + durable execution Temporal-parity) |
| P08-kr-acceptance-evidence | `phases/P08-kr-acceptance-evidence/` | M3 KR group onboarding + ADR-0210 35-checkbox evidence bundle + 7-day SLO + restore drill |

## KR pack (pack #1) dependency

This milestone is the first to require KR pack `status: active` (per ADR-0064 §2). Promotion blockers in `docs/localization-packs/kr/pack.yaml`:
- corpus.lock signed (KR statutes pinned per Bominal ADR-0190)
- ≥1 µservice overlay shipped (P01-hr → P02-payroll likely first)
- ≥1 acceptance evidence signed (P08)
- 1 paying KR tenant live (P08 final gate)

## Parallelization

Per `.omc/plans/M01-M03-parallelization-manifest.md`: P01-P05 form serial chains (HR→Payroll→Accounting; HR→Mail→Messenger); P07 (Workflow Studio) runs parallel with P01-P05 (only depends on M02 engine); P06 (Application B2B live) gates on P01-P05; P08 gates on everything.

## References

- `docs/MASTERPLAN.md` §4 M03
- `docs/localization-packs/kr.md` + `docs/localization-packs/kr/pack.yaml`
- Bominal ADR-0210 (M3 closure criteria, 35 checkboxes)
- `.omc/plans/M01-M03-parallelization-manifest.md`
