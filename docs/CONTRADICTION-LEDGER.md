---
purpose: Oyatie — Contradiction Ledger
doc_status: published
---

# Oyatie — Contradiction Ledger

> **Status:** Draft v0.1 — 2026-05-09. Authored per Codex critic verdict §12 BLOCKER (cohesion check). Routes every known cross-axis contradiction through a single tracking surface so cohesion guarantees are auditable.
> **Owner:** `council-architecture`. Updates per [DOC-CATALOG.md](DOC-CATALOG.md) `EVT-CROSS-AXIS-CONTRADICTION-FOUND`.
> **Companion:** [`machine-readable/contradictions.json`](machine-readable/contradictions.json) (planned).
> **Sources merged:** `docs/raw/rename-and-contradiction.md` (77 contradictions); `docs/raw/gap-docs-project.md` (8 project-doc J-001..J-008 contradictions); 15 user-input items from rename agent; Codex critic verdict §12.

---

## 1. Status taxonomy

| Status | Meaning |
|---|---|
| `OPEN` | Identified; no resolution path documented yet |
| `RESOLUTION_DRAFTED` | Resolution path proposed in a doc/ADR; awaiting approval |
| `RESOLUTION_IN_FLIGHT` | Approved; implementation in progress |
| `RESOLVED` | Implemented + verified |
| `DEFERRED` | Council-approved deferral with explicit re-review date |
| `WONT_FIX` | Council-approved that the contradiction is intentional / out-of-scope |

## 2. Severity

| Severity | Trigger |
|---|---|
| **BLOCKER** | Blocks a wave gate; cannot ship through it |
| **HIGH** | Creates contract drift; must close before next axis preview |
| **MED** | Causes confusion; should close in current quarter |
| **LOW** | Cosmetic / forensic |

---

## 3. The ledger

### 3.1 BLOCKER contradictions (gate-blocking)

| Ledger ID | Source | Summary | Affected axes | Resolution doc / ADR | Status | Owner | Blocks |
|---|---|---|---|---|---|---|---|
| `LEDG-001` | `rename-and-contradiction.md` H1 + Codex §7 | Data Use Boundary 12 vs 13-class taxonomy contradiction (CHILDREN_UNDER_14 was a 13th class; should be orthogonal `subject_class` attribute) | Privacy + Search + Ads + Analytics | `PRIVACY-PROGRAM.md §2.2.1` (already amended 2026-05-09) | RESOLVED | `council-privacy` | (was W-Foundation) |
| `LEDG-002` | Codex §1 | DESIGN §3 said "Foundry sequenced first"; PRD §3.1 sequences W-Foundry-Preview second. Foundry is **second**, not first. | Foundry + Foundation | `DESIGN.md §3` (already amended 2026-05-09 with "no-shortcut clause") | RESOLVED | `council-architecture` | W-Foundry-Preview |
| `LEDG-003` | Codex §3 + §15 | Linear consent ladder violates purpose-limitation; must be a purpose-permission matrix | Privacy + Ads | `PRIVACY-PROGRAM.md §2.2.2` (already amended 2026-05-09) | RESOLVED | `council-privacy` | W-Foundation |
| `LEDG-004` | Codex §5 + §18 + ADR-0013 | License posture conflict — ADR-0013 accepts AGPL/GPL internal/server-side; new posture forbids AGPL/GPL in product code | Foundry surfaces (license gate) + Cloud (observability) + Search (extensions) | `decisions/ADR-0013-product-license-policy.md` (drafted 2026-05-09; supersedes ADR-0013 license language) | RESOLUTION_DRAFTED | `council-architecture` + `ops-security` + founder + legal | W-Foundation gate |
| `LEDG-005` | `rename-and-contradiction.md` H1/H2/H17/H18 + Codex §12 | Email mining + cross-pillar joins + tenant collection sharing + analytics sharing — only partially routed through DUB; four-pillar matrix was missing | SaaS + Search + Ads + Analytics | `PRIVACY-PROGRAM.md §2.2.2 four-pillar matrix` (added 2026-05-09) | RESOLVED | `council-privacy` | W-Foundation |
| `LEDG-006` | Codex §15 | Foundry RAG before Search Substrate is a shortcut compromising tenant/data invariants | Foundry + Search | `DESIGN.md §3 no-shortcut clause` (added 2026-05-09); split into `Foundry-Retrieval-Contract` (preview) and live RAG (post-Search-Substrate) | RESOLUTION_DRAFTED | `axis-foundry` + `axis-search` | W-Foundry-Preview live-pilot |
| `LEDG-007` | Codex §9 | Cross-axis contracts in DESIGN §10 incomplete (missed Cloud↔Search, Search↔Ads, Foundry↔Cloud, Foundry↔Search, Tenant↔Ads/Analytics, Revenue/Tax) | All cross-axis | `DESIGN.md §10` (6 new contract rows added 2026-05-09 + 1 contract-registry source-of-truth) | RESOLVED | `council-architecture` | W-Foundation |
| `LEDG-008` | `rename-and-contradiction.md` H7 + H9 + H10 | Master plan + ADR-0001 enumerate 5-6 arms with no cloud/search/ads/agent-runtime axes; ADR-0013 + ADR-0040 need new-axis horizon | Cross-cutting | New ADR required: "Axis admission contract" | OPEN | `council-architecture` | W-Foundation |
| `LEDG-009` | `gap-docs-project.md` J-001 | Client-supplied tenant auth in Emergency / Medical / Records services (X-Tenant-ID header) is a tenant isolation breach | SaaS + Vertical (healthcare) | Replace header-derivation with token-bound tenant resolution; gate via `oya-governance-tenant-isolation` | OPEN | `platform-tenancy-identity` + `vertical-healthcare` | W-Foundation |
| `LEDG-010` | `gap-docs-project.md` J-002 | Single-cluster OCI cloud posture contradicts AWS-class cloud claim | Cloud | Cell architecture ADR (planned P0) + region/AZ/cell taxonomy expansion | OPEN | `axis-cloud` | W-Cloud-Preview |
| `LEDG-011` | `gap-docs-project.md` J-003 | Zero accepted search ADRs while Search is now a first-class axis | Search | Author search axis ADR cluster (crawler, index, ranker, SERP, safety, public-corpus rights) | OPEN | `axis-search` | W-Search-Preview |
| `LEDG-012` | `gap-docs-project.md` J-004 | ClickHouse placeholder + no DP gateway; analytics axis can't enforce DUB | Analytics | Build in-house event router + DP/k-anonymity budget per [PRIVACY-PROGRAM §2.2.6](PRIVACY-PROGRAM.md) | OPEN | `axis-ads-analytics` | W-Analytics-Privacy-Substrate |
| `LEDG-013` | `gap-docs-project.md` J-005 | Foundry / Furnace gate mismatch | Foundry | Re-author Foundry sequencing + retire Furnace branding pending naming ADR | RESOLUTION_IN_FLIGHT | `axis-foundry` | W-Foundry-Preview |
| `LEDG-014` | `gap-docs-project.md` J-006 | Missing PIPA / DPIA binders — cannot evidence Korea launch readiness | Compliance + per-vertical | Author KR-pack regulatory binder per [COMPLIANCE-MATRIX §3.1](COMPLIANCE-MATRIX.md); per-vertical DPIA template | OPEN | `regional-packs/oya-pack-kr` + `council-privacy` | W-Vertical-Pilot |
| `LEDG-015` | `gap-docs-project.md` J-007 | ADR status drift / placeholders (71 Proposed; multiple shipped-but-Proposed) | Cross-cutting | `crew-adr-promotion` burndown plan; per-quarter promotion targets | RESOLUTION_IN_FLIGHT | `crew-adr-promotion` | continuous |
| `LEDG-016` | `gap-docs-project.md` J-008 | Lack of capacity / multi-region plan | Cloud | Capacity model ADR + per-region pack residency contract | OPEN | `axis-cloud` + `regional-packs` | W-Cloud-Stable |

### 3.2 HIGH contradictions (axis-blocking)

| Ledger ID | Source | Summary | Affected axes | Resolution doc / ADR | Status | Owner |
|---|---|---|---|---|---|---|
| `LEDG-017` | `rename-and-contradiction.md` H8 | Lifestyle / consumer scope (Cellar / Dining IN or OUT) | SaaS + Vertical | `PRD.md §8 Q-NEW` open question; needs council decision | OPEN | `council-architecture` |
| `LEDG-018` | `rename-and-contradiction.md` H5 + GLOSSARY note | Foundry naming vs ADR-0006 "no Palantir vocabulary" clause | Foundry | Naming ADR — accept "Foundry" with differentiation rationale or rename before code-embedded | OPEN | `council-architecture` + founder |
| `LEDG-019` | `rename-and-contradiction.md` H11 | Search-engine consumer brand (Oyatie Search vs separate brand) | Search + GTM | `GTM-PLAN.md` brand-architecture decision | OPEN | `gtm-marketing` + founder |
| `LEDG-020` | `rename-and-contradiction.md` H12 | Quant repo fate (extracted vs re-merge vs federate) | Cross-cutting | New ADR superseding ADR-0017 | OPEN | `council-architecture` |
| `LEDG-021` | `rename-and-contradiction.md` H15 | Personal "no ads, ever" inviolable vs ads-axis carving it out | SaaS (Connect) + Ads | `PRIVACY-PROGRAM.md` open question (Q2); council decision | OPEN | `council-privacy` + founder |
| `LEDG-022` | `rename-and-contradiction.md` H23 | External Foundry vs internal-only Foundry (now consolidated into Foundry) | Foundry | Captured in Foundry consolidation 2026-05-09; documented in DESIGN §3 | RESOLUTION_DRAFTED | `axis-foundry` |
| `LEDG-023` | `rename-and-contradiction.md` H24 | Foundry external-product positioning (axis 3 sold separately?) | Foundry + GTM | Foundry-as-a-product per Foundry-improvements research §H.7 | RESOLUTION_DRAFTED | `axis-foundry` + `gtm-sales-se` |
| `LEDG-024` | Codex §10 | Korea posture omits 본인확인서비스 (identity verification), MyData / Open Banking, location law, telecom secrecy | Vertical-fintech + Cross-cutting + KR-pack | `DESIGN.md §12` expansion + KR-pack regulatory deepening | OPEN | `regional-packs/oya-pack-kr` + `vertical-fintech` |
| `LEDG-025` | `team-charters` review | `platform-api-sdk` vs per-axis public API ownership ambiguity | Cross-cutting | `RACI-OWNERSHIP.md` clarification; ADR-0040 gate ownership | OPEN | `council-architecture` |
| `LEDG-026` | `team-charters` review | `axis-foundry` fitness-functions vs axis-team autonomy (power asymmetry) | Foundry + all axes | `RACI-OWNERSHIP.md` 5-business-day fitness-fn dispute process | OPEN | `council-architecture` |
| `LEDG-027` | `team-charters` review | `vertical-agriculture` ↔ `vertical-food` traceability handoff | Vertical (agriculture + food) | `DESIGN.md §10` new contract `TRACEABILITY_HANDOFF`; ownership: `council-architecture` | OPEN | `council-architecture` |
| `LEDG-028` | `team-charters` review | Cloud IAM vs platform-tenancy-identity IAM lockstep | Cloud + SaaS | `oya-governance-iam-lockstep` fitness function | OPEN | `axis-cloud` + `platform-tenancy-identity` |
| `LEDG-029` | `team-charters` review | `council-privacy` secretariat conflict of interest (`platform-privacy-dub` drafts ADRs + runs governance) | Privacy | Council chair rotation policy; chair cannot be from `platform-privacy-dub` | OPEN | `council-privacy` |

### 3.3 MED + LOW (cosmetic / forensic — abbreviated)

| Ledger ID | Summary | Status |
|---|---|---|
| `LEDG-030` ... `LEDG-077` | Remaining 48 entries from `rename-and-contradiction.md` MEDIUM tier (M1..M33) and LOW tier (L1..L20) | OPEN; bulk-resolved via brand-rename and contradiction-resolution batches in [ROADMAP.md](ROADMAP.md) |
| (auto-generated rows) | Each of 6,560 brand-rename touchpoints is its own LOW entry; resolved en bloc | TRACKED |

---

## 4. Resolution batches (Foundry-batch dispatch)

Each batch tag groups contradictions for parallel resolution.

| Batch | Ledger IDs | Fanout | Shared-writes | Dispatch wave |
|---|---|---|---|---|
| `contradiction-resolution-data-use-boundary-group` | LEDG-001, LEDG-003, LEDG-005, LEDG-006, LEDG-013 | 1 (sequential — same doc) | `PRIVACY-PROGRAM.md` + `decisions/ADR-0008-data-use-boundary.md` | W-Foundation |
| `contradiction-resolution-axis-admission` | LEDG-008, LEDG-018, LEDG-019, LEDG-020, LEDG-022, LEDG-023 | 6 (per ADR) | `decisions/_index.md` | W-Foundation |
| `contradiction-resolution-license-policy` | LEDG-004 | 1 | `decisions/ADR-0013-product-license-policy.md` + `vendor-partner-ledger.md` + `Cargo.toml` (deny.toml) | W-Foundation |
| `contradiction-resolution-cross-axis-contracts` | LEDG-007, LEDG-027, LEDG-028 | 3 (per contract) | `DESIGN.md §10` + `machine-readable/contracts.json` | W-Foundation |
| `contradiction-resolution-tenant-isolation` | LEDG-009 | 1 (per service) | per-service Cargo.toml + tenant resolution adapter | W-Foundation |
| `contradiction-resolution-cloud-substrate` | LEDG-010, LEDG-016 | 2 | `DESIGN.md §13.5` + cloud-axis ADRs | W-Cloud-Substrate |
| `contradiction-resolution-search-foundation` | LEDG-011 | 6+ (per ADR cluster: crawler, index, ranker, SERP, safety, rights) | search-axis ADRs | W-Search-Substrate |
| `contradiction-resolution-analytics-substrate` | LEDG-012 | 4 (per substrate component) | analytics ADRs | W-Analytics-Privacy-Substrate |
| `contradiction-resolution-kr-binders` | LEDG-014, LEDG-024 | 4 (per binder) | KR-pack docs + DPIA template | W-Vertical-Pilot |
| `contradiction-resolution-adr-burndown` | LEDG-015 | 71 (per Proposed ADR) — serialized on `decisions/_index.md` | continuous | continuous |
| `contradiction-resolution-team-scope` | LEDG-025, LEDG-026, LEDG-029 | 3 | `RACI-OWNERSHIP.md` + per-team CHARTER amendments | W-Foundation |
| `brand-rename` | LEDG-030+ (6,560 touchpoints in 17 batches) | 17 (per touchpoint type) | per-batch shared-writes per recon | continuous + W-Foundation gate |

---

## 5. Resolution SLA

- **BLOCKER**: must reach `RESOLVED` or `DEFERRED` before the wave it blocks ships its gate
- **HIGH**: must reach `RESOLVED` within 1 wave of identification
- **MED**: must reach `RESOLVED` within 2 waves
- **LOW**: best-effort; bulk-resolved in batches

---

## 6. Auto-emission

When a future contradiction is detected by:

- `oya-governance-cohesion` (cross-axis contract drift detector)
- A reviewer manually flagging via `gh issue` with `kind:contradiction`
- A new ADR explicitly identifying a prior decision conflict
- A regulator update that conflicts with prior posture

…it auto-emits an `EVT-CROSS-AXIS-CONTRADICTION-FOUND` event, appends a row to this ledger, and notifies `council-architecture`.

---

## 7. Sources scanned

- `docs/raw/rename-and-contradiction.md` (77 entries; 24 HIGH + 33 MED + 20 LOW)
- `docs/raw/gap-docs-project.md` (8 J-class contradictions)
- `docs/raw/codex-verdict.md` (8 BLOCKERs + ~20 HIGH)
- `docs/teams/README.md §7` (5 team-scope contradictions)
- `docs/PRD.md §8` open questions
- `docs/PRIVACY-PROGRAM.md §2.5` open questions

*Footer regenerated whenever this doc is edited.*


---

> **§Note (2026-05-21 transition):** References to `oya-governance-*` in this historical document are intentional — they describe past state. New work uses `oya-governance-*` per the 2026-05-21 transition directive.