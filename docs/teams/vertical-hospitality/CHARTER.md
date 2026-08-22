---
doc_status: published
---

# Team: Vertical — Hospitality (PMS)

## Mission
This team owns the hospitality vertical: Property Management System (PMS), reservations, front-desk operations, housekeeping, revenue management, and guest-experience workflows. This is a **skeleton team** — scope is defined but staffing and wave timing are deferred to W-Vertical-Fan-Out.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Hospitality (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-hospitality-kernel` — `Reservation`, `GuestProfile`, `Room`, `Folio`, `HousekeepingTask`, `Rateplan`
  - `vertical-hospitality-domain-*` — reservation lifecycle, check-in/out, folio management, revenue management
  - Products owned: `products/vertical-hospitality/PRD.md` (skeleton)
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — folio transactions, guest consent events)
  - `DSR / consent withdrawal cascade` (ack required — guest PII)
- **Catalog records:** `crates/vertical-hospitality-*`
- **Runbooks:** TBD at W-Vertical-Fan-Out
- **ADRs:** TBD at activation

## In-scope work
- PMS: reservation management (OTA integration: Booking.com, Expedia channel manager API), check-in/out, room assignment, folio management
- Housekeeping: task scheduling, room status tracking, maintenance request
- Revenue management: rate plan management, occupancy forecasting, yield optimization (agent-assisted under autonomy ceiling)
- Guest profile: KR 개인정보보호법-compliant guest data management, consent management
- KR 숙박업 registration and compliance

## Out-of-scope (anti-scope)
- Consumer booking app (B2B tenants — hotel operators — only)
- OTA platform operations
- Cloud infrastructure (→ `axis-cloud`)
- Guest PII in any ad-targeting signal (always blocked)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-saas` | Workflow engine for reservation and housekeeping workflows | Per-release |
| `platform-privacy-dub` | Guest PII classification, DSR cascade | ADR lifecycle |
| `platform-audit-evidence` | Folio transaction audit records | Per transaction |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `gtm-customer-success` | Hospitality tenant occupancy and revenue dashboards | Monthly |

## Success metrics
*(Defined at W-Vertical-Fan-Out)*
- **Folio transaction audit completeness:** 100%
- **Guest DSR completion:** 100% within 72 h
- **OTA channel sync lag:** < 15 min

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council; privacy council for guest PII disputes
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
| Guest PII enters ad targeting | Catastrophic | Forced classification + fitness gate |
| OTA sync failure causes double-booking | High | Idempotent reservation writes; OTA ack tracking |

## Sources scanned
PRD.md §3.1, DESIGN.md §1, DOC-CATALOG.md §2.5, products/vertical-hospitality/PRD.md (skeleton).
