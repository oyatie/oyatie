# 20 — LINUX pilot non-ADR docs review (lane: linux-nonadr)

**Scope reviewed (judgment, not mechanical):** `docs/context/*.md`
(cloud-native-stack, component-boundaries, conformance-gates, engineering-conventions,
rust-engineering-guardrails, roadmap, source-parity-context, testing-strategy,
phase2-context, phase3-context, migration-batch-prompt, migration-slice-template),
`docs/migration/*` (source-consolidation-plan, source-manifest),
`docs/research/*` (beat-or-parity-scorecard, distributed-database-architecture-research,
hyperscaler-production-roadmap, bespoke-ci-design-2026-06-06/*).
**Ruled canon = the 12 items in the lane brief.** These docs migrate into `source`, so
stale framing here propagates — flagged accordingly.

Reachability classes used: **DECISION→ADR**, **INSTRUCTION→session-context-bundle**,
**GENERATED-REFERENCE**, **ORPHAN→not-needed**.

---

## LEAD: the genuine canon-contradictions (ranked)

### C1 — `roadmap.md` + `source-parity-context.md`: whole-doc mission contradicts the crystallized mission *(canon #11; also #7/#10 framing)*
- **`roadmap.md:4`** — "The long-term goal is a **safe, idiomatic Rust expression of the Linux
  kernel**, achieved as an honest, multi-year, multi-team program."
- **`roadmap.md:60`** — '"the entire Linux kernel in Rust" via full replacement lives here.'
- **`source-parity-context.md:4`** — "Progress toward a **source-parity, idiomatic, memory-safe
  Rust migration of Linux kernel code**…"
- **Contradiction:** the crystallized mission (`cloud-native-stack.md:3-9`) is an **enterprise,
  cloud/server-specialized OS stripped to the minimum for k8s+cloud+AI at hyperscale — NOT a
  port, NOT general-purpose, NOT a like-for-like Linux re-expression** ("modernization, not a
  port", §0). roadmap.md/source-parity-context.md still frame the entire program as a *port of
  the Linux kernel to Rust* with ZERO mention of: the hyperscaler mission, the framekernel, the
  Capsule model, the ratchet/own-endpoint posture, or any ADR. This is the single largest
  stale-framing cluster — it sells the OLD North-Star (port-the-kernel) as the goal. It violates
  the maximal-vertical-scope / own-endpoint canon (#11) by anchoring identity to Linux-parity
  rather than the owned cloud-OS endpoint.
- **Disposition:** these are genuine **Phase-0 historical artifacts** (the C2Rust-oracle leaf-port
  workflow really happened in `port/`). They should be **explicitly re-scoped as "Phase-0 port
  provenance / `port/` supplier history"** and demoted under the live mission, NOT presented as
  "the goal". As written they are canon-contradicting if migrated verbatim.
- **Reachability:** GENERATED-REFERENCE (port provenance) — currently mis-presented as DECISION/goal.

### C2 — `conformance-gates.md:64`: "secure-by-default" isolation framing *(canon #7)*
- **`conformance-gates.md:64`** — "**microVM-per-pod as the secure-by-default boundary** (the
  substrate's isolation posture…)".
- **Contradiction:** canon #7 retires the **"secure-by-default-native"** framing; the committed
  posture is **assume-breach microVM DEFAULT** (and framekernel-host is the *committed endpoint*,
  not "secure-by-default-native"). "secure-by-default boundary" is the exact stale phrasing the
  rule names. Fix → "assume-breach microVM-per-pod **default** boundary".
- **`cloud-native-stack.md:72`** — "with **secure-by-default** defaults" (in the secure-boot
  ergonomics bullet). Softer (it's about boot ergonomics, not the isolation posture) but the same
  retired phrase; tighten to the assume-breach vocabulary to avoid propagating it.
- **Reachability:** DECISION→ADR (conformance-gates already cites ADR-0018/0019 for isolation; the
  phrasing just lags the ratified posture).

### C3 — `bespoke-ci-design-2026-06-06/*`: pervasively **Forgejo-native** + **foundry** brand *(canon #2, #3, #4)*
- **Forgejo-as-live-stack (canon #3 — Forgejo DROPPED → GitHub now → bespoke VCS later; mirror at
  most):** `00-current-shape.md:7` ("oya-ci: bespoke-Rust Prow (**Forgejo-native** CI/CD platform)"),
  `:29` ("**Forgejo-native**, Kubernetes-native reimplementation of Prow"), `:37,:39,:54,:61,:65-66,
  :110,:132,:172,:273` ("**Forgejo** Commit-Status (adopt until SCM cutover)"). The design treats
  Forgejo as the live SCM/forge. Canon: **GitHub is canonical now**; Forgejo is dropped (mirror at
  most). 31 Forgejo hits across the CI-design subdir.
- **foundry brand (canon #2 — RETIRED → cloud-intelligence / governance):**
  `00-current-shape.md:172` and `:359` both reference `oya-foundry-vcs-*-kernel` crates. "foundry"
  is retired brand residue.
- **Mitigating:** most of these describe the **current/legacy** shape of `source` (what exists),
  and the **endpoint** spec (`40-PRODUCT-SPEC.md`) IS canon-aligned — oya-ci owns the contract not
  the engine, structurally closes the Jenkins failure modes, build-first (canon #4 satisfied at the
  endpoint). So the contradiction is in the *current-shape/legacy* framing carrying dropped-vendor
  vocabulary, not in the target. Still must be scrubbed before migration (these are research scratch
  that feeds ADR-0513-area decisions).
- **Reachability:** GENERATED-REFERENCE (research synthesis behind the oya-ci ADRs). The
  current-shape doc is legitimately a snapshot; re-label Forgejo→GitHub and drop foundry residue.

### C4 — `M1`/`M2` milestone vocabulary *(canon #9 — M0-M3/MVP wave-vocab RETIRED → gate-defined waves)*
- `testing-strategy.md:9,13,148-154` ("**M1** parity floor", "**M2** real-init", "P0/**M1.0** …
  P6/**M2.c**"), `conformance-gates.md:36` ("**M2**-verified"), `source-consolidation-plan.md:100`
  ("**M2** 7-phase").
- **Nuance:** these are **kernel-local milestone labels** (M1=parity-floor, M2=real-init), NOT the
  company-wide M0-M3/MVP *wave* vocabulary the canon retires. But canon #9 retires the M0-M3 token
  family in favor of **gate-defined waves**, and the brief explicitly namespaces only
  `autonomy_tier/eu_ai_act_risk_tier/dr_tier/storage_tier` for "tier" — there is no analogous
  carve-out for "M1/M2". On migration these collide with the retired wave-vocab. **Flag as
  stale-vocab to rename to gate-defined wave labels** (e.g. "parity-floor gate", "real-init gate")
  rather than M1/M2.
- **Reachability:** GENERATED-REFERENCE (kernel roadmap mapping) — rename, don't delete.

---

## AI-slop / fabricated-precision

### S1 — `cloud-native-stack.md:91`: Redis "+40%" contradicts the doc-family's own "1.31×"
- `cloud-native-stack.md:91` — "real ATC'25 numbers — **Redis +40%** vs Linux, virtio UDP 1.31×".
- Every other instance in the repo cites Asterinas's **published Redis figure as 1.31× (= +31%)**:
  `testing-strategy.md:2` ("redis 1.31x"), `:169` ("redis 1.31x"), `:545`, `:553`. "+40%" is
  **fabricated/inconsistent precision** — an internal contradiction with the canonical baseline the
  same program tracks. Fix → "+31% (1.31×)".

### S2 — minor hedging/filler (low severity, refinement-only)
- `cloud-native-stack.md` §0 is dense but largely load-bearing; the one genuine filler risk is the
  repeated "(directive 2026-06-04)" stamping on nearly every bullet — acceptable as provenance, but
  several bullets restate the same "no-legacy / least-privilege / four-lenses" thesis 3-4×
  (e.g. §0 "Latest hardware" + §5a "least privilege" + §5b + §5c all re-derive "smaller surface =
  fewer CVEs"). Consolidate on a content pass; not a canon issue.
- No fabricated-precision found in component-boundaries / engineering-conventions / conformance-gates
  / guardrails — those are tight and ADR-cited.

---

## Per-doc findings

### `cloud-native-stack.md` — MIXED (mission-canon-aligned core; pre-Capsule isolation vocab)
- **Canon-aligned:** the §0 mission, four-lenses, no-legacy, demand-driven ABI, framekernel +
  tri-arch convergence (§6) are all consistent with the live mission and even predate/feed the ADRs.
- **Stale:** S1 (Redis +40%, C2 secure-by-default at :72). Isolation is described as
  "microVM-per-pod-by-default" (`:85,:191,:280,:410`) — *content* is right (assume-breach default)
  but should adopt the canon **assume-breach** vocabulary and the **Capsule** primitive
  (component-boundaries.md has already migrated to Capsules; this doc still says "pod = microVM"
  without the Capsule abstraction). Refinement: cross-link the Capsule model (ADR-0014/0018).
- **Reachability:** INSTRUCTION→session-context-bundle (it's the mission/workflow context pack).
  Largely should be GENERATED from the mission ADR once one exists.

### `component-boundaries.md` — CANON-ALIGNED (reference exemplar)
- Already updated 2026-06-06 for ADR-0018/Capsule; cites ADRs as the boundary SSOT; ratchet posture
  (#11), Cedar-as-day-0-own + PostgreSQL/Iceberg/OTel permanent-reuse + ClickHouse/Pulsar
  defer-vendored (`:70-71`) all match canon #5/#6/#10. **No contradictions.** Minor: it owns the
  Capsule-vs-Frame terminology cleanly. Reachability: DECISION→ADR (correctly derived from ADRs).
- Gap vs canon #5: names ClickHouse/Pulsar/Postgres but **not** the Redis→Valkey / Kafka→Pulsar
  pairings explicitly; fine here (out of this component's scope) — those belong in a data-tier ADR.

### `conformance-gates.md` — CANON-ALIGNED except C2
- Strong evidence-discipline doc; cites ADR-0009/0018/0019; "no silent downgrade" (#7 isolation
  integrity) is correct. Only issue: **C2** ("secure-by-default boundary", `:64`) and the **M2**
  vocab (**C4**, `:36`). Otherwise exemplary. Reachability: DECISION→ADR.

### `engineering-conventions.md` — CANON-ALIGNED
- Own-the-differentiator / reuse-foundation (#5/#6), design-to-owned-ideal + vendor-bridge ratchet
  (§12, matches #11 exactly), ADR-cited (0003/0007/0019/0020/0024/0026). **No contradictions.**
  Reachability: INSTRUCTION→session-context-bundle (universal conventions). Refinement: §1 still
  hard-codes "`~/Developer/source`" path framing — fine for pilot, retire on migration (§10 already
  says so).

### `rust-engineering-guardrails.md` — CANON-NEUTRAL (technical, no canon surface)
- Pure Rust-practice reference; four-lenses cited from cloud-native-stack §0. No vendor/brand/wave
  canon surface. No contradictions. Reachability: INSTRUCTION→session-context-bundle. (Did not deep-
  read all 45KB; grep found no masterplan-as-authority, no retired brands, no wave-vocab.)

### `roadmap.md` — STALE FRAMING (C1, highest)
- See C1. Honest about scope-discipline, but the **goal statement is the old port-North-Star**, not
  the live cloud-OS mission. Multiple ADRs link to it as Phase-0 provenance, so it's reachable but
  **mis-labeled as the goal**. Re-scope to "port-provenance / `port/` supplier history."

### `source-parity-context.md` — STALE FRAMING (C1)
- Same as roadmap: "source-parity Rust migration of Linux kernel code" as the target outcome.
  Honest reality-boundary section is good, but the framing is the retired port-mission. Re-scope.
  Reachability: INSTRUCTION (Phase-0 context pack) — demote under live mission.

### `testing-strategy.md` — CANON-ALIGNED (methodology) with M-vocab (C4)
- 120KB methodology doc; beat-or-parity-under-load matches mission; Asterinas baselines consistently
  cited as 1.08/1.17/1.31/0.85× (this is the *correct* figure that S1 contradicts). Issues: **M1/M2/
  M1.0–M2.c** wave-vocab throughout (C4). Otherwise the gate methodology is sound and matches
  conformance-gates. Reachability: GENERATED-REFERENCE (from the `rust-os-testing-requirements`
  research). Refinement: it's a research-synthesis dump; could be distilled — much is verbatim
  workflow output ("I now have the precise repo layout…", `:7`) which is **AI-slop process-narration**
  that should be stripped before this is canon.

### `phase2-context.md` / `phase3-context.md` — STALE (Phase-0/rkernel framing)
- Both are the **leaf-port methodology** docs (C2Rust-oracle / loom-Miri / rkernel demo).
  `phase3-context.md:23-29` still centers the **retired `kernel/` rkernel demo** ("pkg `rkernel`",
  custom target `x86_64-rkernel.json`) — but cloud-native-stack §6 **RESOLVED 2026-06-04 to retire
  rkernel** and converge on `stack/kernel`. So phase3-context describes the *retired* vehicle as the
  harness. Stale vs the convergence decision. Reachability: GENERATED-REFERENCE (port provenance) —
  re-scope alongside roadmap (C1). Not a vendor/brand canon hit, but plainly-superseded content.

### `migration-batch-prompt.md` / `migration-slice-template.md` — STALE-SCOPE (Phase-0 INSTRUCTION)
- Re-runnable C2Rust leaf-slice workflow prompts. Correct for what they are (Phase-0 `port/`
  mechanism), but they are the **old port-program** instructions; on migration they're pilot-scaffold
  (`engineering-conventions §10` says retire pilot-only artifacts at integration). No vendor/brand
  canon hits. Reachability: INSTRUCTION→session-context-bundle, **retire-at-integration**.

### `migration/source-consolidation-plan.md` — CANON-ALIGNED (good); 1 stale token
- Explicitly canon-correct on the contested items: **"Forge: GitHub is canonical now"** (`:14-15`,
  matches #3), brand-residue clean lane forbids `foundry-*`/`oyatie-*`/codenames (`:36,:73`, matches
  #2). Live-state-grounded. Only stale token: **M2** (`:100`, kernel-gate label — C4). The doc target
  `~/Developer/source (oyatie)` (`:1,:90`) — "oyatie" is the live GitHub repo name
  (`github.com/jason931225/oyatie`), so it's a **real remote name**, not retired brand residue;
  acceptable. Reachability: pilot-scaffold INSTRUCTION (self-labeled "retired at integration").

### `migration/source-manifest.md` — CANON-ALIGNED
- Pure inventory + rename rules (codename→descriptive). Enforces dropping `foundry`/codenames. No
  contradictions. Reachability: pilot-scaffold INSTRUCTION (retired at integration).

### `research/beat-or-parity-scorecard.md` — CANON-ALIGNED
- Tracks the §0 governing bar vs Linux/Asterinas/Talos. Consistent with mission. (Spot-read header;
  no vendor/brand/wave canon hits in scan.) Reachability: GENERATED-REFERENCE.

### `research/distributed-database-architecture-research.md` — CANON-NEUTRAL (DB deep-research)
- CockroachDB/Spanner/Aurora/TiKV-class synthesis; these are **named reference architectures to
  learn from**, not vendor-endpoint claims, so naming them is fine (matches canon #5 "transitional
  bridges / own the tier"). 27 "Postgres" hits are pg-wire-contract context (canon #5/#10: pg-wire
  is the kept contract). No contradiction. Reachability: GENERATED-REFERENCE behind ADR-0001/0006/0020.

### `research/hyperscaler-production-roadmap.md` — CANON-ALIGNED (reconciled to operating-system DECISIONS)
- Talos/k8s control-plane research; reconciled to `operating-system/DECISIONS.md`. Names Talos as the
  reference being replaced (matches #11 vendor-bridge→owned). Reachability: GENERATED-REFERENCE.

### `research/bespoke-ci-design-2026-06-06/*` — MIXED (endpoint canon-aligned; current-shape stale: C3)
- `40-PRODUCT-SPEC.md` endpoint = canon-aligned oya-ci (#4). `00-current-shape.md` carries the
  **Forgejo-native + foundry** legacy framing (C3). `10-{argo,tekton,prow,jenkins,hyperscaler,
  buildtools}.md` are the **input surveys** (Argo 141 / Tekton 147 / Prow 158 / Jenkins 20 hits) —
  these are legitimately the *components being unified*, matching canon #4's "Prow+Tekton+Argo"
  provenance; naming them as inputs is correct, NOT a contradiction. The contradiction is narrow:
  Forgejo-as-live-SCM + foundry brand in the current-shape snapshot. Reachability: GENERATED-REFERENCE.

---

## Counts

| Category | Count | Where |
|---|---|---|
| **Genuine canon-contradictions** | **4 clusters** | C1 (roadmap+source-parity mission), C2 (secure-by-default ×2), C3 (Forgejo ×31 + foundry ×4 in CI-design), C4 (M1/M2 vocab, 5 files) |
| AI-slop / fabricated-precision | 2 | S1 (Redis +40% vs 1.31×), S2 (process-narration + restatement) |
| Stale-but-not-vendor (superseded content) | 4 docs | roadmap, source-parity-context, phase2-context, phase3-context (rkernel) |
| Canon-aligned / reference-exemplar | 8 docs | component-boundaries, conformance-gates(*ex-C2), engineering-conventions, guardrails, source-consolidation-plan, source-manifest, beat-or-parity-scorecard, hyperscaler-roadmap |
| Canon-neutral (no canon surface) | 2 docs | rust-engineering-guardrails, distributed-database-architecture-research |

**Retired-vendor/brand token footprint (this lane's docs):** Forgejo 31 · foundry 4 · oyatie 4
(real remote name, OK) · kuberos 1 (in a forbid-list, OK) · Jenkins/Argo/Tekton/Prow (input-survey
context, OK). **Wave-vocab:** M1 20 / M2 18 (kernel-local labels — rename per C4).
**No** masterplan-as-authority hits (canon #1 satisfied — docs cite ADRs, not a hand-authored
masterplan). **No** tenant-tier / tier-system hits. **No** Citus/Milvus/Zitadel/PARC hits.
