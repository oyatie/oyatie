---
doc_status: published
---

# Team: Vertical — Public Sector (Forms / 조달청 / Global Gov)

## Mission
This team owns the public-sector vertical: government forms, procurement workflows (KR 조달청, US GSA, EU procurement directives), e-government integration, and public-sector-specific regulatory compliance. This is a **skeleton team** — scope is defined but staffing and wave timing are deferred to W-Vertical-Fan-Out. Public-sector tenants require the highest auditability standards and often mandate data residency within national boundaries.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Public Sector (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-public-sector-kernel` — `GovForm`, `ProcurementRequest`, `Submission`, `PublicRecord`, `AuditLog`
  - `vertical-public-sector-domain-*` — form lifecycle, procurement workflow, submission management
  - Per-region integrations: KR (정부24, 조달청 G2B, 국민신문고), US (Login.gov, SAM.gov, GSA eBuy), EU (TED procurement, Once-Only principle)
  - Products owned: `products/vertical-public-sector/PRD.md` (skeleton)
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (emitter — every form submission, procurement decision)
  - `Region / AZ / Cell` (consumer — data residency is mandatory per jurisdiction)
- **Catalog records:** `crates/vertical-public-sector-*`
- **Runbooks:** TBD at W-Vertical-Fan-Out
- **ADRs:** TBD; data residency + FedRAMP/CSAP compliance ADR at activation

## In-scope work
- Government form authoring and submission: KR 정부24 form integration, US federal form standards, EU Once-Only principle
- Procurement: KR 나라장터 (G2B) integration, US GSA/SAM.gov, EU TED
- Identity: KR 본인확인서비스 / 공동인증서, US Login.gov, EU eIDAS (via regional pack identity seams)
- Data residency enforcement: strict in-jurisdiction residency for all public records
- FedRAMP (US), CSAP (KR), GAIA-X (EU) compliance evidence collection
- Accessibility: WCAG 2.1 AA minimum for all public-sector form surfaces

## Out-of-scope (anti-scope)
- Consumer government services (B2G tenants — government agencies — only)
- Cloud infrastructure (→ `axis-cloud`)
- Cross-jurisdiction data sharing without explicit treaty/agreement evidence

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-cloud` | Strict in-jurisdiction cell residency for public-sector tenants | Wave gate |
| `platform-audit-evidence` | Every submission and procurement decision audit record | Per event |
| `platform-tenancy-identity` | Government-grade identity provider seams | Per-release |
| `ops-compliance` | FedRAMP / CSAP / GAIA-X regulatory watch | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `ops-compliance` | Government procurement and form audit evidence | Quarterly |

## Success metrics
*(Defined at W-Vertical-Fan-Out)*
- **Form submission audit completeness:** 100%
- **Data residency violation:** 0
- **FedRAMP/CSAP evidence pack regeneration:** ≤ 4 h

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council; `ops-compliance` for FedRAMP / CSAP incidents
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
| Public-sector data crosses jurisdiction boundary | Catastrophic | Strict residency enforcement; cell routing enforced |
| FedRAMP/CSAP evidence gap at audit | High | `ops-compliance` owns evidence pack; public-sector team provides raw data |

## Sources scanned
PRD.md §3.1, DESIGN.md §12 (regional packs), DOC-CATALOG.md §2.5, products/vertical-public-sector/PRD.md (skeleton).
