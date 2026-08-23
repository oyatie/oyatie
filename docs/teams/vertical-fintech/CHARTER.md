---
doc_status: published
---

# Team: Vertical — Fintech (PG / Open-Banking / KYC / AML)

## Mission
This team owns the fintech vertical: payment gateway integration, open-banking API, KYC/KYB workflows, AML screening, and transaction monitoring across KR (FSC/KFTC/KFB), US (OCC/NACHA/RTP), EU (PSD2/SEPA), and other regional packs. It exists because fintech tenants operate under the strictest regulatory scrutiny after healthcare, and financial account/payment data is permanently and unconditionally blocked from ad targeting. It does **not** own the cloud infrastructure, search, or ads axes.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Fintech (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-fintech-kernel` — `PaymentTransaction`, `Account`, `KycRecord`, `AmlAlert`, `OpenBankingConsent`
  - `vertical-fintech-domain-*` — PG orchestration, KYC lifecycle, AML screening, open-banking consent
  - `vertical-fintech-adapter-kftc` — KR KFTC open-banking API adapter
  - `vertical-fintech-adapter-nacha` — US NACHA/ACH adapter
  - `vertical-fintech-adapter-sepa` — EU SEPA/SEPA-Inst adapter
  - Per-region extensions: `pack-kr` → KR FSC controls, 전자금융거래법; `pack-us` → Reg-E, BSA/AML; `pack-eu` → PSD2, DORA
  - Products owned: `products/vertical-fintech/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — every payment transaction, KYC decision, AML alert)
  - `DSR / consent withdrawal cascade` (ack required — open-banking consent withdrawal)
  - All account/payment fields forced to `ad_targetable_blocked` (Data Use Boundary vertical override)
- **Catalog records:** `crates/vertical-fintech-*`
- **Runbooks:** `runbooks/fintech-payment-failure.md`, `runbooks/aml-alert-escalation.md`, `runbooks/kyc-review-queue.md`
- **ADRs:** ADR-0027 (fintech schema), KR FSC compliance ADR

## In-scope work
- Payment gateway orchestration: KR 카카오페이/네이버페이/토스/계좌이체, US ACH/Wire/RTP, EU SEPA; routing, retry, settlement, reconciliation
- Open-banking: KR KFTC API consent management, account aggregation, payment initiation (PSD2 AISP/PISP in EU)
- KYC/KYB: identity verification workflow, document OCR, liveness check, sanction screening, ongoing monitoring
- AML: transaction monitoring, rule engine, ML-scored alerts, SAR/STR filing workflow (KR FSC, US FinCEN, EU AMLD)
- Transaction monitoring: velocity rules, geography rules, anomaly detection
- KR 전자금융거래법 compliance: real-time reporting, escrow, payment-service registration evidence
- Regional pack fintech seam impls: KR (FSC, KFTC), US (OCC, NACHA, RTP), EU (PSD2, DORA)

## Out-of-scope (anti-scope)
- Account/payment data in any ad-targeting signal — always blocked permanently
- Banking license operations (Oyatie provides the software; tenant holds the license)
- Cloud infrastructure (→ `axis-cloud`)
- Consumer fintech app (B2B tenants only in this vertical)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-privacy-dub` | Fintech forced override (`ad_targetable_blocked`) in Data Use Boundary | ADR lifecycle |
| `platform-audit-evidence` | Every payment transaction and AML alert audit record | Per transaction |
| `axis-saas` | Workflow engine for KYC and AML workflows | Per-release |
| `axis-foundry` | Agent-assisted AML screening under autonomy ceiling | Wave gate |
| `ops-compliance` | FSC / KCC / OCC / FinCEN regulatory watch | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `ops-compliance` | KYC/AML evidence packs, FSC audit trail | Monthly + audit |
| `gtm-customer-success` | Fintech tenant payment and AML health dashboards | Monthly |

## Success metrics
- **Payment transaction audit chain completeness:** 100%
- **AML alert review SLA:** 100% within 24 h of alert generation
- **KYC decision audit completeness:** 100%
- **Account/payment data in any ad signal:** 0 (permanent hard zero)
- **KR FSC real-time payment reporting SLA:** met per regulatory requirement
- **Open-banking consent withdrawal processed:** 100% within 1 h

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for payment schema contract changes; privacy council for open-banking consent disputes
- Compliance: `ops-compliance` for FSC / FinCEN regulatory incidents
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 45-min sync — payment health, AML queue, KYC backlog, regulatory change watch
- Cross-team review: monthly compliance review with `ops-compliance`

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; payment and AML PRs require security-reviewer
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; KR FSC regulatory changes trigger immediate ADR

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Payment transaction missed from audit chain | Catastrophic | 100% audit completeness gate; reconciliation daily |
| Account/payment data enters ad targeting | Catastrophic | Forced `ad_targetable_blocked`; fitness gate; zero-tolerance |
| AML alert not reviewed within SLA | High | PagerDuty alert at 20 h; AML queue capacity monitoring |
| KR FSC regulatory change breaks payment reporting | High | Monthly regulatory watch; immediate ADR on change detection |

## Sources scanned
PRD.md §3.3 (anti-scope: PHI/PCI in ads), DESIGN.md §10, ADR-0027, products/vertical-fintech/PRD.md (draft).
