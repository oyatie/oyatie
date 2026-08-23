---
doc_status: published
---

# Team: Vertical — Retail (POS / Inventory / Promotions)

## Mission
This team owns the retail vertical: point-of-sale (POS), inventory management, promotions engine, and loyalty programs. It exists to give retail tenants a compliant, search-integrated, agent-assisted commercial platform with one tenancy model and one audit chain covering every transaction. This is a **skeleton team** — scope is defined but staffing and wave timing are deferred to W-Vertical-Fan-Out. It does **not** own cloud infrastructure or the SaaS workflow engine.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Retail (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-retail-kernel` — `Product`, `SKU`, `PosTransaction`, `InventoryRecord`, `Promotion`, `LoyaltyAccount`
  - `vertical-retail-domain-*` — POS transaction lifecycle, inventory replenishment, promotions engine
  - Products owned: `products/vertical-retail/PRD.md` (skeleton)
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — POS transactions, inventory adjustments)
  - `Ad slot inventory` (consumer — in-app promotional ad slots)
- **Catalog records:** `crates/vertical-retail-*`
- **Runbooks:** TBD at W-Vertical-Fan-Out
- **ADRs:** TBD at W-Vertical-Fan-Out

## In-scope work
- POS: transaction processing (KR 부가세 included), receipt generation, cashier session management
- Inventory: SKU management, stock tracking, replenishment triggers, warehouse → store transfers
- Promotions: discount rules, coupon codes, bundle pricing, time-bounded campaigns
- Loyalty: points accumulation, tier management, redemption, expiry
- KR retail compliance: 현금영수증 (cash receipt) issuance, 부가세 (VAT) reporting
- E-commerce integration: online/offline inventory sync, fulfillment routing

## Out-of-scope (anti-scope)
- Consumer loyalty apps outside tenant context
- Cloud infrastructure (→ `axis-cloud`)
- Advertising auction logic (→ `axis-ads-analytics`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-saas` | Workflow engine, OG for Product/SKU nodes | Per-release |
| `platform-audit-evidence` | POS transaction audit records | Per transaction |
| `axis-ads-analytics` | In-app promotional ad slot inventory | Wave gate |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `gtm-customer-success` | Retail tenant sales dashboards | Monthly |

## Success metrics
- **POS transaction audit completeness:** 100%
- **KR 현금영수증 issuance accuracy:** 100%
- **Inventory sync lag (online ↔ offline):** < 5 min
*(Full OKR set defined at W-Vertical-Fan-Out wave gate)*

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for kernel contract changes
- Compliance: `ops-compliance` for KR VAT / 현금영수증 regulatory changes
- Founder: as last resort

## Communication cadence
- Stand-up: async (skeleton phase — minimal cadence)
- Weekly: 30-min sync at W-Vertical-Fan-Out activation
- Cross-team review: monthly once active

## Bandwidth + hiring
- Current FTE: 0 (skeleton — TBD at W-Vertical-Fan-Out)
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch once active

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Skeleton team delays mean retail vertical not ready for W-Vertical-Fan-Out | Medium | Council decision on vertical sequencing; scope inherited from SaaS substrate |

## Sources scanned
PRD.md §3.1 (W-Vertical-Fan-Out), DESIGN.md §1 (Axis 2), DOC-CATALOG.md §2.5, products/vertical-retail/PRD.md (skeleton).
