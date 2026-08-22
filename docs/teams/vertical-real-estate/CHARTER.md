---
doc_status: published
---

# Team: Vertical — Real Estate (Leasing)

## Mission
This team owns the real estate vertical: property listing management, leasing workflows, tenant-management (B2B property managers as Oyatie tenants), lease lifecycle, maintenance requests, and KR 부동산 regulatory compliance (공인중개사법, 임대차보호법). This is a **skeleton team** — scope is defined but staffing and wave timing are deferred to W-Vertical-Fan-Out.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Real Estate (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-real-estate-kernel` — `Property`, `Listing`, `Lease`, `TenantRecord`, `MaintenanceRequest`, `RentPayment`
  - `vertical-real-estate-domain-*` — listing lifecycle, lease workflow, maintenance dispatch, rent collection
  - Products owned: `products/vertical-real-estate/PRD.md` (skeleton)
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — lease execution, rent payment, deposit handling)
  - `DSR / consent withdrawal cascade` (ack required — tenant PII in lease records)
- **Catalog records:** `crates/vertical-real-estate-*`
- **Runbooks:** TBD at W-Vertical-Fan-Out
- **ADRs:** TBD at activation; KR 임대차보호법 compliance ADR

## In-scope work
- Property listing: listing creation, media management, availability calendar, listing portal integration (KR 직방/다방 API adapters)
- Lease lifecycle: application, screening, lease drafting (KR 표준임대차계약서), execution (e-signature), renewal, termination
- Rent collection: KR 계좌이체, auto-debit, late-fee calculation, 보증금 (deposit) escrow tracking
- Maintenance: request intake, contractor dispatch, resolution tracking, cost allocation
- KR 임대차보호법 compliance: 계약갱신청구권, 전월세상한제 enforcement in lease workflows
- KR 확정일자 (fixed-date registration) integration with 법원 / 주민센터

## Out-of-scope (anti-scope)
- Consumer real estate search (B2B property managers only as Oyatie tenants)
- Property valuation services
- Cloud infrastructure (→ `axis-cloud`)
- Tenant (renter) PII in any ad-targeting signal (always blocked)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-saas` | Workflow engine for lease and maintenance workflows | Per-release |
| `platform-privacy-dub` | Renter PII classification, DSR cascade | ADR lifecycle |
| `platform-audit-evidence` | Lease execution and rent payment audit records | Per event |
| `ops-compliance` | KR 임대차보호법 regulatory watch | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `gtm-customer-success` | Real estate tenant occupancy and collections dashboards | Monthly |

## Success metrics
*(Defined at W-Vertical-Fan-Out)*
- **Lease execution audit completeness:** 100%
- **KR 임대차보호법 계약갱신청구권 enforcement accuracy:** 100%
- **Renter DSR completion:** 100% within 72 h

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council; privacy council for renter PII disputes
- Compliance: `ops-compliance` for KR 임대차보호법 changes
- Founder: as last resort

## Communication cadence
- Stand-up: async (skeleton phase)
- Weekly: 30-min sync at W-Vertical-Fan-Out activation

## Bandwidth + hiring
- Current FTE: 0 (skeleton)
- Target FTE: TBD per axis-wave
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch once active

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| KR 임대차보호법 statutory change breaks lease enforcement logic | High | Monthly regulatory watch; lease workflow versioned per statutory revision |
| Renter PII leaked into search index | High | `internal_only` classification; fitness gate |

## Sources scanned
PRD.md §3.1, DESIGN.md §1, DOC-CATALOG.md §2.5, products/vertical-real-estate/PRD.md (skeleton).
