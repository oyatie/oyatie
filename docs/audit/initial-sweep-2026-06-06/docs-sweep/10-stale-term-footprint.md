# 10 — Stale-Term Canon-Conflict Footprint (Mechanical Inventory)

**Scope:** `bash grep` over `/Users/jasonlee/Developer/source/docs`, file-level counts
(`grep -rliI`), include `*.md,*.mdx,*.txt,*.json,*.yaml,*.yml,*.toml`.
**Exclusion:** `decisions/ADR-*` files (SSOT — amended separately, not in this docs-sweep).
**Nature:** MECHANICAL inventory only. No judgment on whether to amend; just term presence,
spread, and a mechanical-rename-vs-content-change classification to size the amendment phase.

> NOTE on counts: the raw corpus number quoted in the task brief (foundry 830, tenant-tier 152,
> M0-M3 103, Redis 98, Kafka 85) is the **full-corpus** match INCLUDING `decisions/ADR-*`.
> The numbers below are **post-exclusion** (ADR-* removed) file counts, which is the footprint
> that the docs-sweep amendment actually has to touch. Both are reported where they differ.

---

## Footprint table (post-ADR-exclusion file counts)

| # | Term / pattern | Files | Top dirs | Fix class | Rename target / note |
|---|---|---|---|---|---|
| 1 | `foundry` | **731** (952 incl ADR) | user-journeys 195, personas 130, standards 51, runbooks 36, products 34, advanced-cicd 34, architecture 32 | **SPLIT: mostly MECHANICAL, sense-routed** | → `intelligence` (AI/agent substrate sense, ~274 files) OR `governance` (fitness/policy lane, ~135 files; tokens `foundry-fitness`, `governance-foundry`, `council-foundry`) |
| 2 | `Jenkins` | **35** (66 incl ADR) | ideas 10, user-journeys 5, products 4, standards 3, machine-readable 3 | **CONTENT-CHANGE** | Not a rename. Reframe to oya-ci endpoint; Jenkins = operative-until-cutover bridge, NOT canonical (build-first-cutover-later) |
| 3 | `Forgejo` | **21** (47 incl ADR) | ideas 8, runbooks 4, machine-readable 3, specs 2 | **CONTENT-CHANGE** | Not a rename. Forgejo DROPPED → GitHub now / bespoke VCS later; mirror-at-most |
| 4 | `tenant-tier\|tier-system` | **138** (153 incl ADR) | personas 129, standards 2, architecture 2 | **MECHANICAL (bulk) + FP carve-out** | → `tenant-class`. 129 are identical persona boilerplate `tenant-tier-bound`. 2 files are canon-COMPLIANT retirement docs (`tier-system-retired-replaced-by-tenant-class`) = FALSE POSITIVE, do not touch |
| 5 | `\bM0\b\|\bM1\b\|\bM2\b\|\bM3\b` | **77** (103 incl ADR) → **62 real** | user-journeys 23, architecture 7, standards 4, raw 4, machine-readable 4 | **CONTENT-CHANGE + heavy FP** | M0-M3/MVP wave-vocab RETIRED → gate-defined waves. 15/77 are pure FALSE POSITIVE (`MacBook Air M3`, `m3.material.io`, `Gate M1` funnel, `M2 meta-review` pass labels). 62 carry real wave-vocab |
| 6 | `\bMVP\b` | **35** | machine-readable 6, ideas 4, architecture 4, raw 3 | **CONTENT-CHANGE** | wave-vocab RETIRED → gate-defined waves. glossary.json already documents this (`"old":"MVP / Milestone (M0..M3)"`) — some hits are the retirement record itself (FP) |
| 7 | `Kafka` | **44** (85 incl ADR) | standards 7, teams 5, products 5, architecture 5, performance-budgets 3 | **CONTENT-CHANGE (mostly) + some mechanical** | Kafka → Pulsar (transitional bridge framing). Only 3/44 already co-mention Pulsar; 41 are raw Kafka needing migration framing. Adapter-name refs (`adapter-kafka`) are mechanical |
| 8 | `Redis` | **49** (98 incl ADR) | architecture 11, user-journeys 8, standards 8, products 3 | **CONTENT-CHANGE (mostly) + mechanical naming** | Redis → Valkey (transitional bridge). 19/49 already co-mention Valkey (`Valkey/Redis cluster`, partial). 30/49 raw Redis needing framing. CSI naming `oya-{...,redis,...}` is mechanical |
| 9 | `native.?default\|secure-by-default.*native` | **0** | — | N/A | No hits. Looser `secure-by-default` = 1 file only. Canon (assume-breach microVM DEFAULT) not actively contradicted by this phrasing in docs/ |
| 10 | `Cedar.*engine` | **170** (raw) → **24 contradiction** | user-journeys 109, architecture 23, standards 7, onboarding 7 | **CONTENT-CHANGE (subset only)** | 170 raw is mostly FALSE POSITIVE (`Workflow Engine`, `cedar-policy-engine` IaC paths). The genuine canon contradiction = **24 files** asserting Cedar AS the policy ENGINE (`one Cedar policy engine`, `Cedar evaluation engine`). Canon: Cedar = CONTRACT, owned PARC = engine |
| 11 | `eliminate.*Postgres` | **0** | — | N/A | No "eliminate Postgres" framing. Postgres = 84 files total (transitional bridge). No anti-canon "own-tier-by-eliminating-Postgres" phrasing found; Postgres-as-bridge is consistent with canon |
| 12 | `M0/M1/M2/M3 milestone` | **0** (exact) → 40 (loose) | — | **CONTENT-CHANGE** | Exact phrase = 0. Loose `M[0-3]…milestone` co-occurrence = 40 files — same wave-vocab retirement as #5/#6 |

---

## Big-five sampling (mechanical-rename vs content-change classification)

### foundry — 731 files — **SPLIT (sense-routed mechanical rename)**
Brand RETIRED → `intelligence` (AI/agent substrate) OR `governance` (fitness/policy lane), per context.
This is NOT a uniform swap: the corpus uses `foundry` in two distinct senses.

| Sample file | Sense observed | Route |
|---|---|---|
| `products/foundry/PRD.md` | "Foundry agent integration… autonomy ceiling T2", provider adapters, capability registry | → **intelligence** (AI substrate) |
| `standards/fintech-compliance.md` | "Foundry capability that touches financial accounts", "Foundry agent integration" | → **intelligence** |
| `advanced-cicd/.../feature-flag-architecture.md` | "Owner: `axis-foundry`", "Foundry adapter pattern" | → **intelligence** (owning-axis name) |
| `architecture/foundry-fitness-to-governance-transition-2026-05-21.md` | `foundry-fitness`, governance-transition (the policy/fitness lane) | → **governance** |
| `personas/apprentice-jakob-bauer.md` | `foundry` listed as an ambient integration surface (product) | → **intelligence** |

Sense split (heuristic co-occurrence, files may double-count): ~274 near intelligence/agent/provider/capability/model/adapter; ~135 near fitness/governance/amendment/council; 29 carry explicit governance tokens (`foundry-fitness`, `governance-foundry`, `council-foundry`, `amendment-foundry`).
**Verdict:** MECHANICAL per-occurrence, but requires per-token routing (a single global swap would mis-route the governance sense). Token families `oya-foundry`/`axis-foundry`/`ai foundry`/`palantir foundry` → intelligence; `*-fitness`/`council-`/`governance-`/`amendment-foundry` → governance.

### tenant-tier — 138 files — **MECHANICAL (with FP carve-out)**
| Sample file | Observed | Class |
|---|---|---|
| `personas/communications-specialist-charlotte-dubois.md` | `tenant-tier-bound` boilerplate (identical across 129 persona files) | MECHANICAL → `tenant-class` |
| `personas/intern-manager-felicia-adamou.md` | same boilerplate line 161 | MECHANICAL |
| `standards/brief-template.md` | "regional, regulatory, or tenant-tier overlay pack" | MECHANICAL → `tenant-class` |
| `architecture/corpus-rigor-audit-...snapshot.md` | references to `ADR-MAIL-0002-backend-tenant-tier-policy.md` (microservice ADR path) | FALSE POSITIVE (path ref to non-docs ADR) |
| (token `tier-system-retired-replaced-by-tenant-class`) | retirement record | FALSE POSITIVE (canon-compliant) |
**Verdict:** Overwhelmingly MECHANICAL term-swap to `tenant-class`. CAUTION: do NOT swap inside namespaced `*_tier` identifiers (`autonomy_tier`, `eu_ai_act_risk_tier`, `dr_tier`, `storage_tier`) — those are canonical and out of scope. Carve out the 2 retirement-doc FPs.

### M0-M3 — 77 files (62 real wave-vocab) — **CONTENT-CHANGE + heavy false-positive**
| Sample file | Observed | Class |
|---|---|---|
| `user-journeys/j138-.../README.md` | `F1+F2+F3+M1+A1+A4+A5` (milestone wave token) | CONTENT-CHANGE (gate-defined waves) |
| `architecture/wave-3-g-synthesis-adjudication-...md` | `M1 Challenge-assumption`, `M2 Meta-review` (review-pass labels) | FALSE POSITIVE |
| `user-journeys/j169-.../ux-flow.md` | `MacBook Air M3 + iPhone 15` (device) | FALSE POSITIVE |
| `standards/ux-best-practices.md` | `m3.material.io` (Material Design URL) | FALSE POSITIVE |
| `gtm/tenant-prospect-to-active-stages.md` | `Gate M1 -- Qualified Trigger` (sales-funnel gate) | FALSE POSITIVE |
**Verdict:** NOT a mechanical swap — the wave-vocab framing itself is stale (M0-M3/MVP RETIRED → gate-defined waves), and 15/77 hits are unrelated false positives that must be excluded by hand. CONTENT-CHANGE, per-file review required.

### Redis — 49 files — **CONTENT-CHANGE (mostly) + mechanical naming**
| Sample file | Observed | Class |
|---|---|---|
| `standards/voice-video-call-architecture.md` | `LIVEKIT_REDIS_ADDR`, Redis-based discovery, shared Redis state | CONTENT-CHANGE (Redis→Valkey bridge framing) |
| `architecture/adr-cross-reference-graph-...md` | `Valkey/Redis cluster`, `per-tenant Redis cache` | already partial-framed (mechanical residue) |
| `products/_TEMPLATE.md` | "Caching tier (in-memory + Redis + CDN)" | CONTENT-CHANGE (template, propagates) |
| `user-journeys/j01-emergency-911-dispatch/handshake.md` | "session-state Redis" | CONTENT-CHANGE |
| `architecture/wave-3-g-...md` | `Heroku Redis → cloud-iac managed-redis` | mechanical (naming) |
**Verdict:** 19/49 already co-mention Valkey (migration framed) → near-mechanical residue. 30/49 raw Redis → CONTENT-CHANGE (add bridge framing Redis→Valkey, endpoint owned). Identifier/env-var refs (`LIVEKIT_REDIS_ADDR`) are mechanical.

### Kafka — 44 files — **CONTENT-CHANGE (mostly) + some mechanical**
| Sample file | Observed | Class |
|---|---|---|
| `standards/layer-enum-adr-0105.md` | "Kafka producer / Broker is adapter detail" | mechanical (adapter ref) |
| `teams/platform-eventing-og/CHARTER.md` | "Kafka topic contracts", `oya-platform-eventing-adapter-kafka` | CONTENT-CHANGE + adapter naming |
| `products/_TEMPLATE.md` | "inbound queue/Kafka consumers" | CONTENT-CHANGE (template) |
| `architecture/adr-cross-reference-graph-...md` | "Eventing backbone on Apache Kafka", "ClickHouse + Kafka Engine default" | CONTENT-CHANGE (Kafka→Pulsar bridge) |
| `performance-budgets/cedar-hot-path-1ms-p99.md` | "Kafka producer fire-and-forget" | CONTENT-CHANGE |
**Verdict:** Only 3/44 co-mention Pulsar → 41 are raw Kafka needing Kafka→Pulsar bridge framing (transitional, not endpoint). CONTENT-CHANGE for the framing; `adapter-kafka` tokens are mechanical.

---

## Reachability / canon notes surfaced during inventory
- The corpus is **mid-transition**: `machine-readable/glossary.json` already records `"old":"MVP / Milestone (M0..M3)"`, and `architecture/foundry-fitness-to-governance-transition-2026-05-21.md` exists. Some hits are the **retirement record itself** (canon-compliant) and must be carved out as false positives, not amended.
- Jenkins/Forgejo/ArgoCD still framed as **canonical** self-hostable substrate in `products/foundry/PRD.md` (ADR-0349 binding) and `ideas/hyperscaler-practices-to-adopt.md` ("Oyatie 1ES … Forgejo+Jenkins+ArgoCD as SSOT") — direct contradiction with build-first-cutover-later (oya-ci = endpoint; these = operative-until-cutover bridges).
- `Cedar policy engine` framing (24 files) contradicts Cedar=CONTRACT / PARC=engine canon.
- `native-default`/`secure-by-default-native` and `eliminate-Postgres` = **0 hits** — those specific anti-canon phrasings are absent from docs/ (the contradictions, if any, live in ADRs not swept here).
