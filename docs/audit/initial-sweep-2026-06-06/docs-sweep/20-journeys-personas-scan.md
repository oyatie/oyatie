# 20 — Journeys + Personas Stale-Canon Scan (HIGH-VOLUME lane)

**Lane:** `journeys-personas-scan`
**Corpus:** `/Users/jasonlee/Developer/source/docs/user-journeys/` (913 `.md`, 1413 files incl. JSON/proto/schema) + `/Users/jasonlee/Developer/source/docs/personas/` (131 `.md`) = **1044 markdown docs**.
**Method:** `grep -rliIE` file-level counts across the full corpus + a sampled deep-read of ~10 representative hits (4 full-file / block reads, ~6 contextual `grep -n` reads). **This is a SCAN, not a full read** — see Coverage Honesty at the end.

> **Path note (important):** the docs do NOT live under `/Users/jasonlee/Developer/linux/...`. They live in the sibling SOURCE corpus `/Users/jasonlee/Developer/source/docs/` (the `SRC` constant in `_workflow.js`). The `linux/` tree contains only the audit artifacts + kernel/stack pilot. An initial search of `linux/` returned zero journeys/personas; this is expected, not a missing-corpus failure.

---

## HEADLINE: this lane is ALMOST ENTIRELY MECHANICAL + FALSE-POSITIVE — very few genuine canon contradictions

The high-volume lane was expected to be foundry-/Jenkins-heavy. In reality, the journeys/personas corpus is **narrative/UX fiction + handshake graphs**, so the stale-term grep counts are dominated by:
1. **Identical templated boilerplate** (one line copied across 129 personas), and
2. **False positives** — fictional character names, fictional company names, external-product references, and substring collisions (`redistribute`→`redis`, `forgery`→`forge`).

The genuine canon contradictions are a SMALL residue. Ranked below.

---

## Per-term file-level footprint (UJ = user-journeys, PE = personas)

| Term / pattern | UJ | PE | TOTAL | Verdict |
|---|---|---|---|---|
| `foundry` | 195 | 130 | **325** | MECHANICAL rename, but **105 UJ are Palantir-Foundry FP** + persona/journey uses are templated component/surface names. Only ~21 UJ carry the genuine internal `foundry` component. |
| `tenant-tier\|tier-system` | 0 | 129 | **129** | **MECHANICAL** — ONE identical boilerplate line across 129 personas → `tenant-class`. |
| `Cedar.{0,40}engine` | 68 | 2 | **70** | Mostly **FP** (`workflow-engine` co-occurrence, `Cedar policy-engine logs` = audit subsystem). Genuine "Cedar AS engine" contradiction = a HANDFUL (e.g. j167). |
| `\bM[0-3]\b` (bare) | 31 | 1 | **32** | Mostly **FP / generic milestone labels** (ServiceNow cutover M1/M2/M3 in j179, device `M3`). Real wave-vocab residue is small. |
| `forge` | 29 | 1 | **30** | **~100% FP** — `forgery`/`EmergencyServiceForgeryDetected`, `Marlboro-Forge Industries` (fictional co.), `Salesforce`. NOT the GitHub/VCS forge. |
| `redis` | 9 | 0 | **9** | Split: ~3 genuine (`session-state Redis`, `Redis cluster`, `Redis session-store`) needing Valkey-bridge framing; rest are `redistribute`/`yellow_redistribute` substring **FP**. |
| `argo` | 5 | 5 | **10** | Needs spot-check (likely component/event-name refs; low priority). |
| `Postgres\|Citus` | 4 | 0 | **4** | **FP / canon-OK** — external-customer migration scenarios (j166 Skylark acquisition, j126 port `5432`). Postgres-as-bridge, not anti-canon. |
| `zitadel` | 3 | 0 | **3** | **CANON-COMPLIANT** — framed as SCIM **adapter / IdP bridge** (`IP-008 Zitadel adapter`), exactly canon #6. Do not touch. |
| `jenkins` | 5 | 0 | **5** | **100% FP** — fictional character "Tom Jenkins" in j130 (bribery-attempt journey). ZERO CI references. |
| `forgejo` | 0 | 0 | **0** | Absent from this lane. |
| `\bMVP\b` | 0 | 0 | **0** | Absent. |
| `kafka` | 2 | 0 | **2** | Negligible. |
| `native.?default\|secure-by-default.*native` | 0 | 0 | **0** | Absent. |
| `milvus\|clickhouse` | 0 | 0 | **0** | Absent. |
| `masterplan` (as authority, canon #1) | 0 | 0 | **0** | No journey/persona invokes masterplan as hand-authored authority. Clean. |

---

## GENUINE CANON-CONTRADICTIONS (lead — ranked)

These are the only hits in this lane that are real framing contradictions rather than mechanical rename / FP. The lane is LOW-yield for genuine contradictions.

### C-1. Cedar framed AS the policy ENGINE (canon #6 — Cedar = CONTRACT, owned PARC = engine)
- `source/docs/user-journeys/j167-cto-diego-vargas-platform-major-version-cutover/story.md:34` — "**The Cedar engine** asks for his passkey + face-id."
- `j167/story.md:74` — "The quorum hits 4-of-4 PERMIT. **The Cedar engine evaluates the policy**:"
- These assert Cedar as the evaluating engine. Canon #6: Cedar is the **policy CONTRACT**; the owned **PARC** is the engine.
- **BUT:** the vast majority of the 70 `Cedar...engine` hits are FALSE POSITIVES: `workflow-engine` co-located with Cedar permits (j86/j114 handshakes), and `Cedar policy-engine logs` (j139) which describes the **audit log subsystem**, not an authz-engine claim. Genuine contradiction band ≈ a handful of files (j167 confirmed; full enumeration deferred — see coverage honesty).
- **Fix:** narrow content-edit on the confirmed "Cedar engine evaluates" phrasings → "Cedar policy (evaluated by PARC)". Do NOT mass-rewrite the FP `workflow-engine`/`policy-engine-logs` hits.

### C-2. Raw Redis as canonical session/state store, no Valkey bridge framing (canon #5 — Redis→Valkey)
- `source/docs/user-journeys/j01-emergency-911-dispatch/handshake.md:35` — "in its **session-state Redis**."
- `j126-government-auditor-3pao-conducts-fedramp-audit/handshake.md:46,50` — "**Redis session-store** with TTL", "**Redis cluster** (cell us-east-1)".
- Canon #5: Redis is a TRANSITIONAL bridge → Valkey; the endpoint is owned. These present Redis as the substrate with no migration framing.
- **Fix:** CONTENT-CHANGE — add Redis→Valkey bridge framing (or rename to Valkey). ~3 genuine files; the rest of the `redis` count is `redistribute` FP.

### Non-contradictions explicitly cleared (so the amendment phase doesn't chase ghosts)
- **Jenkins (5)** — all fictional character "Tom Jenkins" (j130). NOT CI. **No amendment.**
- **Forge (30)** — `forgery`/security-event + `Marlboro-Forge Industries` fictional company. NOT VCS-forge. **No amendment.**
- **Zitadel (3)** — framed as adapter/IdP bridge per canon #6. **Canon-compliant, no amendment.**
- **Postgres (4)** — external-customer migration narrative; Postgres-as-bridge consistent with canon #5. **No amendment.**
- **Palantir Foundry (105 UJ)** — external-product "ontology action pattern" schema boilerplate. Renaming would CORRUPT a real third-party product name. **Hard FP — must NOT be swept.**

---

## MECHANICAL RENAMES (bulk, no framing change) — sized for the amendment phase

### M-1. `tenant-tier-bound` → `tenant-class-bound` (129 persona files) — pure mechanical
- Every hit is the SAME templated line: `- Region outage behavior is pack-bound and **tenant-tier-bound**.`
  - Confirmed identical across 129 files via `grep -hoiE ... | uniq -c` → `129  ...tenant-tier-bound.`
  - Samples: `personas/communications-specialist-charlotte-dubois.md:161`, `personas/apprentice-jakob-bauer.md:161`, `personas/intern-manager-felicia-adamou.md:161`, `personas/security-analyst-anna-petrova.md:161`.
- **Fix:** one-line sed across personas `tenant-tier-bound` → `tenant-class-bound` (canon #9). This is the persona day-summary TEMPLATE — fix the template/generator, not 129 files by hand.
- **CAUTION:** do NOT touch namespaced `*_tier` identifiers (`autonomy_tier`, `eu_ai_act_risk_tier`, `dr_tier`, `storage_tier`) — canon #9 keeps those. (None observed in this boilerplate line; flagged for the global sweep.)

### M-2. `foundry` → split rename, but ONLY the internal-component sense (canon #2)
- The `foundry` token in this lane is overwhelmingly a **component / integration-surface NAME**, not the retired brand prose:
  - **Journeys (195):** handshake-graph component — `foundry owns supply-chain-checks`, `foundry (prod-rollout-gate) calls dev...`, `foundry applies ADR-...`, `foundry through BNF`. Only ~21 files carry this genuine internal-component usage.
  - **Personas (130):** templated integration-surface table row — e.g. `apprentice-jakob-bauer.md:233 | 29 | foundry | ambient | ...`, and day-summary surface lists `developer-sdk + foundry`, `foundry + intelligence`.
- **Route (canon #2):** the AI/agent-substrate sense → `intelligence`; the fitness/policy-lane sense → `governance`. In THIS lane it is almost entirely the **intelligence** sense (agent/SDK/capability surface) — no `foundry-fitness`/`council-foundry` governance tokens observed in journeys/personas.
- **HARD CARVE-OUT:** `Palantir Foundry` (105 UJ files, external product) and `Marlboro-Forge`/`forgery` (forge FP) MUST be excluded. A naive global `foundry`→`intelligence` swap would corrupt 105 third-party-product references.
- **Fix:** sense-routed, token-anchored rename of the internal component only (`foundry` as a service/surface name), with the Palantir/external allow-list excluded. Best done by fixing the journey/persona GENERATOR templates (the component name + the persona surface table) rather than per-file.

### M-3. M0-M3 wave-vocab — mostly FP in this lane, small real residue
- `j179-migration-from-servicenow-itsm/...` `M1/M2/M3` = **generic ServiceNow→Oyatie cutover milestones** in a customer-migration runbook (`M1 export complete`, `M2 attachments replayed`, `M3 MID Server probes replaced`) — these are migration-step labels, arguably legitimate, NOT the retired M0-M3 platform wave-vocab. Likely FP / out-of-scope.
- `j138/README.md:167` `F1+F2+F3+M1+A1+A4+A5` = an opaque capability-token string (ambiguous).
- **Fix:** per-file judgement only where the M-token clearly means the retired platform-wave vocab; treat customer-migration cutover milestones as FP. Low priority in this lane.

---

## AI-SLOP / STALE / PLAIN-WRONG

- No systemic AI-slop surfaced in the sampled reads. The persona day-summary block (`apprentice-jakob-bauer.md:155-163`) is **templated boilerplate** repeated across the persona set (identical invariants list) — this is intentional scaffolding, not hedging/fabrication, but it does mean a single stale token (`tenant-tier`) propagates to 129 files mechanically. Same propagation risk for the persona integration-surface table and the journey handshake component names.
- **Refinement opportunity:** because so much of the corpus is GENERATED from templates, the correct amendment surface is the **generator / template**, not the 1044 output files. Fixing `tenant-tier-bound`, the `foundry` surface name, and the Palantir allow-list at the template level is O(1) instead of O(1044) and prevents drift.

---

## REACHABILITY CLASS

- **GENERATED-REFERENCE.** The journeys/personas corpus is generated scenario/UX fiction + handshake contracts, downstream of the ADRs/masterplan — not a source of DECISIONs or INSTRUCTIONs. No journey/persona asserts authority over canon (0 `masterplan`-as-authority hits). They are reachable as the worked-examples / acceptance-narrative layer.
- **Implication for canon:** none of these docs should be treated as SSOT; they must be RE-GENERATED (or template-patched) to track ADR/canon changes. They are NOT orphans (they are the acceptance-narrative layer), but they are NOT decision records either.

---

## COVERAGE HONESTY (no silent truncation)

- **Counted (grep, file-level):** ALL 1044 markdown docs for every stale-canon term in the brief. Counts above are exhaustive at file granularity.
- **Deep-read (full file / multi-line block):** ~4 — `personas/apprentice-jakob-bauer.md` (boilerplate + surface table), and contextual block reads of `j01`, `j126`, `j167`, `j130`, `j179`, `j139`.
- **Contextual `grep -n` reads:** ~6 term clusters (foundry sense-decomposition, tenant-tier uniqueness, Cedar-engine FP rate, M0-M3, forge/redis/jenkins FP, zitadel/postgres).
- **NOT done:** I did NOT full-read all 195 foundry-journey files, nor enumerate every one of the 70 `Cedar...engine` hits to count the exact genuine-contradiction subset. The C-1 contradiction band is **confirmed present (j167)** but its exact file count is an ESTIMATE ("a handful"), not a full census. Likewise the genuine-Redis subset (~3) is sampled, not exhaustively classified.
- **Confidence:** HIGH on the mechanical/FP verdict (the boilerplate-uniqueness and Palantir/forgery/Jenkins FPs are decisive); MEDIUM on the exact size of the C-1/C-2 genuine-contradiction residue (sampled, not censused).
