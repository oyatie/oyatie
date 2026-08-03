## Unified Model-Routing Engine — design

Synthesized strictly from the five research sources. Where a source lacks a fact, it is flagged "not found."

### 1. Current-state map

| Source | How it routes today | Scoring |
|---|---|---|
| **OMC** (`oh-my-claudecode` 4.15.3, `features/model-routing/*`) | Two-stage: an ~18-rule priority list (first-match-wins on `agentType`+regex signals) runs in parallel with a numeric additive scorer; on >1-tier divergence, confidence caps at 0.5 and the **higher/costlier** tier wins. Tier→model via `TIER_MODELS`. Provider always redirects to Claude Task (codex/gemini MCP deprecated). A separate `task-decomposer` uses a cruder `effort<0.3/0.7` float — not wired to the scorer. | **Real additive integer scorer** (`scorer.js`): `score = lexical + structural + context` from a fixed `WEIGHTS` table; thresholds HIGH≥8 / MED≥4 / else LOW; confidence 0.5–0.9 from distance-to-threshold. All signals are hand-tuned English regex/keyword. No cost, no learning. |
| **OMX** (`oh-my-codex` 0.19.1) | Static table lookup: 34 agent roles each carry `modelClass` (fast/standard/frontier) + `reasoningEffort` + optional `exactModel`; `resolveAgentDefaultModel(role)` maps class→model. `-low` suffix forces spark tier; `executor` special-cased to frontier. A deprecated, env-gated regex prompt-classifier (`explore-routing.js`) exists but defaults OFF. | **None numeric.** Deterministic precedence chain (CLI > env > project config > codex `config.toml` > constant) + a boolean two-regex-set classifier. Cost awareness is structural (tier assignment), not metered. |
| **GJC** (`@gajae-code/coding-agent`) | Deterministic per-agent cascade (`model-resolver.ts`): task override > frontmatter `model:` > session-inherited > fallback > `modelRoles.default`. Team tasks assigned by boolean role/owner/dependency eligibility + first-claim-lease (FIFO), no score. 52-model cross-vendor catalog. | String-match tiebreak cascade (exact > canonical > fuzzy `fuzzyMatch` score > substring), broken by MRU recency, provider usage, vision-capability, deprioritized-provider set. Catalog **carries `cost` per model but it is never read in the selection path.** `roi-reconciliation.ts` is advisory-only, never gates. |
| **Hermes** (`~/.hermes`) | **Profile-based, not task-based**: a kanban task's `assignee` maps to a profile whose `config.yaml` `model:` block statically picks the LLM. A fixed MoA ensemble (`gpt-5.5` + `opus-4.8`, aggregator `opus-4.8`) for consensus. `credential_pool_strategies: least_used` load-balances credentials, not tasks. | **No formula.** One integer column `tasks.priority DEFAULT 0`, human/CLI-set, ordered everywhere by the single clause `ORDER BY priority DESC, created_at ASC`. The "auto-prioritization" lives only as a **prose rubric** in two SKILL.md files (LLM subjectively ranks, then hand-writes the integer). No RICE/ICE/WSJF anywhere. |
| **Owned substrate** (`oya-intelligence-model-routing-{kernel,domain,usecase}`) | Clean 3-layer crate stack: kernel `decide_route` (pure) → domain `validate_route_request` (fail-closed) → usecase `route` (idempotency + audit + `walk_catalog_for_route`). Selection is an **eligibility filter**, not a ranking: filter profiles by capability/credential/data-class/audience/tenant/env-tier, then pick lowest-`priority` u16 (ties: provider `Ord`, then `model_id`). | **No continuous score.** Boolean gates + integer priority. Cost is only a mandatory **evidence-ref presence check** (never a number). ADR-INT-001 (status **Proposed**, not Accepted) specifies the intended numeric design — `cost_per_1k_{input,output}`, `eval_score` floors 0.92/0.85/0.75, 15%-headroom budget fallback, ≤2 provider attempts — **none implemented**. |

**Cross-cutting finding:** four of five systems have *no* quantitative capability×cost scoring; OMC has the only real numeric scorer but it is capability-only, English-regex, and Claude-locked. The owned substrate already has the correct *shape* (deterministic total-order, denial trail, idempotency) but not the *substance* (numbers). The unified engine extends the substrate and imports OMC's scorer discipline + Hermes' rubric structure.

### 2. The Hermes-Kanban scoring pattern

**Exact formula found: there is none.** Hermes has a single manual integer `tasks.priority` and one sort clause `ORDER BY priority DESC, created_at ASC`. The "scoring" is a **qualitative rubric an LLM applies subjectively, then writes back as an integer** (`oyatie-kanban-stewardship/SKILL.md` step 4; `agent-kanban-delivery-loop/SKILL.md` "Backlog ordering"). The founder's "mechanical formula/weights" framing does not match the code — flag this before treating Hermes as a formula source.

What *is* reusable is the **named factor set** the rubric enumerates. Generalize those qualitative factors into an explicit weighted **task-complexity vector** (the mechanization Hermes never did), borrowing OMC's proven additive-integer-weight discipline:

```
complexity_raw = Σ  w_f · feature_f(task)        # OMC-style fixed integer weights
```

Hermes rubric factor → routing feature mapping:

| Hermes qualitative factor | Routing feature `feature_f` | Weight sign |
|---|---|---|
| architectural risk | `impact_scope` (local/module/system-wide) | + |
| dependency / fan-out unblock | `cross_component_deps` | + |
| production / user impact | `blast_radius` | + |
| process regression / reversibility | `reversibility` (easy/mod/difficult) | + |
| prerequisite sequencing | `chain_depth` / prior-failure count | + |
| parallel-safety | `subtask_count` | + |
| (OMC negative signal) "tidy/simple" | `simple_markers` | − |

Then threshold `complexity_raw` into a **capability-demand tier** (`Trivial / Standard / Hard / Frontier`) exactly as OMC thresholds score→tier — but that tier is now the **row selector into the observed-score matrix** (§4), not a hardwired model id. The Hermes contribution is the factor taxonomy + the "one canonical ordering clause factored into one place" discipline (`VALID_SORT_ORDERS`); the OMC contribution is turning each factor into a fixed integer weight with a tested threshold ladder.

### 3. External best-practice patterns worth adopting (Rust-native reimplementation, with precedent)

Grounded in the reusable-patterns arrays across sources — all are reimplemented as owned Rust in the substrate, not vendored:

1. **Ensemble-of-two disagreement-as-confidence** (precedent: OMC `router.js`). Run a declarative rule/policy match *and* the numeric scorer; when they diverge >1 tier, lower confidence and pick the safer (higher-capability) tier. Cheap safety net without ML.
2. **Four composable pure layers** (precedent: OMC signal→score→rule→orchestrate). Keep signal-extraction, scoring, eligibility, and decision-orchestration as independently testable pure functions — maps directly onto the substrate's kernel/domain/usecase split.
3. **Availability orthogonal to preference** (precedent: GJC `getAvailable()` auth-gate vs `pickPreferredModel`). Decide "is this model reachable/credentialed right now" separately from "is it the best-scored candidate." Fail-closed skip of an unauthenticated provider.
4. **Rate-limit reason classification → per-class backoff** (precedent: GJC `rate-limit-utils.ts`: QUOTA_EXHAUSTED 30min / RATE_LIMIT 30s / CAPACITY 45–75s jitter / SERVER 20s). Same-model retry-with-backoff distinct from cross-model fallback.
5. **Spark-then-fallback two-attempt cascade with a visible boundary notice** (precedent: OMX `omx-explore`: try cheap model at low effort, on failure emit an auditable "cost/behavior boundary changed" notice, retry a costlier model). Bounded attempts, not full-catalog walk.
6. **Date-less model-family constants in one file** (precedent: OMC `CLAUDE_FAMILY_DEFAULTS`). A version bump is a one-line diff.
7. **Advisory ROI signal that never gates control flow** (precedent: GJC `roi-reconciliation.ts`). Surface budget/efficiency hints to the caller without a hidden dependency.
8. **Provider-detect-before-trust, fail toward safe default** (precedent: OMC `validateAnthropicBaseUrl` SSRF guard). Unknown endpoint → treat as untrusted / force-inherit.

Explicitly **not** adopted: any online RL/bandit learner as the *selector* (no source has one; unjustified complexity). The tuning loop (§4) recalibrates *observed scores as data*, keeping selection deterministic and replayable.

### 4. The unified engine

Single engine, one shared port, five harnesses. Selection stays deterministic and replayable (the substrate's hard invariant); "learning" is confined to updating the score matrix rows, which are **data**.

**4a. Scoring model — task feature-vector × model observed-score matrix → selection.**

- **Task feature-vector** `f(task)` — the §2 generalization; pure function, OMC-style integer weights, yields `complexity_raw` + a `capability_demand` tier and a set of hard constraints (`capability` ∈ {Chat, Embedding, JsonMode, ToolUse, Vision}, `data_class`, `audience`, `env_tier`, residency).
- **Model observed-score matrix** `M` — one row per catalog model, columns = **measured** DeepSWE-style metrics (from the research: Pass@1, $ per task, latency), plus a derived efficiency and a max-effort ceiling:

| model | Pass@1 | $ / task | efficiency (Pass@1/$) | Pareto | effort ceiling |
|---|---|---|---|---|---|
| gpt-5.6-sol | 0.73 | 8.39 | 8.7 | **frontier (most-capable)** | Max |
| terra | 0.70 | 4.95 | 14.1 | **frontier** | Max |
| luna | 0.67 | 3.03 | **22.1** | **frontier (most-efficient)** | Max |
| fable | 0.70 | 21.63 | 3.2 | dominated | **cap High, never Max** |
| opus-4.8 | 0.59 | 13.22 | 4.5 | dominated | Max |
| sonnet-5 | 0.54 | 26.40 | 2.0 | **strictly dominated** | Max |

  The Pareto frontier is **{luna, terra, gpt-5.6-sol}**. `sonnet-5` is strictly dominated (lower Pass@1 *and* higher $ than every other row); `opus-4.8` and `fable` are dominated on both axes by frontier rows. Off-frontier models are selected **only** when a hard constraint or a non-DeepSWE factor (provider diversity for MoA, subscription-pool availability, vision, refusal-avoidance, harness-locality) forces them — never on merit. `fable`'s `effort ceiling = High` is a hard cap the effort resolver must honor. (`terra`/`luna` vendor not stated in research — carried as opaque catalog rows.)

- **Selection** = for the request's `capability_demand` tier and hard constraints, take eligible rows (substrate eligibility gate, unchanged), rank by a **single explicit objective**, first pick wins, full denial trail retained:

  ```
  utility(model) = Pass@1(model)                      # capability-first (matches OMC over-provision bias)
                   − λ_cost   · normalized_$           # cost pressure, λ from policy
                   − λ_lat    · normalized_latency
  subject to:  Pass@1 ≥ quality_floor[capability_demand]   # ADR-INT-001 floors 0.92/0.85/0.75
               effort ≤ effort_ceiling(model)
               all substrate eligibility gates pass
  ```

  `λ_cost`, `λ_lat`, and the quality floors are **policy data** (§4d), so the same code yields "cheapest that clears the floor" (high `λ_cost`, e.g. Trivial→luna) or "most-capable" (`λ_cost→0`, Frontier→gpt-5.6-sol). Effort (`low/medium/high/xhigh/Max`) is resolved on the same tier, clamped by the model's ceiling. Ties break by the substrate's existing total order (priority, provider `Ord`, `model_id`) so selection stays reproducible and testable.

**4b. Tuning / feedback loop (data-only, deterministic selection preserved).**
Each real route emits a `RouteOutcomeEvent{model, capability_demand, success/pass, actual_$, actual_latency, refusal?}` (extends the substrate's existing in-memory receipt into a durable EVT-INT-* event — the ADR-INT-001 `UsageCostEvent` shape). A batch reconciler updates matrix cells as an **exponentially-weighted moving average** of observed outcomes (`Pass@1`, `$`, `latency`) per (model, demand-tier) bucket. The updated matrix is versioned, content-addressed policy data; selection reads a pinned matrix version so any decision is replayable. No online weight mutation inside the selector (rejected in §3). This is the DeepSWE-style "observed + tuned" the founder specified: bootstrap from the benchmark numbers above, converge to *this fleet's* measured reality.

**4c. Cost-aware fallback / cascade** (implements ADR-INT-001, replacing the substrate's flat full-catalog walk):
- **primary** = `argmax utility` on the demand tier.
- **same-quality fallback** — on provider-health failure / rate-limit / p95-latency breach (GJC reason-classified, §3 pattern 4), pick the next candidate **still clearing the quality floor**; emit the OMX visible boundary notice (§3 pattern 5).
- **budget fallback** — when tenant budget headroom < 15%, drop to the highest-`utility` cheaper model that still clears the floor (e.g. gpt-5.6-sol → terra → luna along the frontier).
- **Caps & guards (fail-closed, from ADR-INT-001):** ≤2 provider attempts (1 for high-risk); **no** retry to a second provider after a completed refusal/safety-block or after regulated data already went to a provider lacking residency/BYOK at the next hop — requires adding per-request **attempt-state** the current stateless walk lacks.

**4d. Policy-as-data schema (DATA vs code).**

- **DATA** (versioned, content-addressed, hot-reloadable — no redeploy): the observed-score matrix `M`; `λ_cost`/`λ_lat`; `quality_floor` per demand-tier; feature weights `w_f` and tier thresholds; per-model `effort_ceiling`; per-tenant `RoutePolicy{preferred_model, max_cost_minor, quality_floor, fallback_allowed, residency}`; provider-health/backoff constants; harness→default-demand-tier map.
- **CODE** (owned Rust, pure, tested): feature extraction `f(task)`; the `utility` objective + eligibility gate; the fallback state machine + attempt caps; idempotency/receipt/redaction guards; the EWMA reconciler. The rule of thumb: *any number a human or the tuning loop would want to change without a deploy is data; every decision procedure is code.*

**4e. How each harness calls the ONE engine (shared port).**
A single owned API — the usecase crate's `route()` — fronted by one gRPC/REST service (`intelligence.ModelRouting/Route`), returning `{model_id, provider, reasoning_effort, fallback_plan, matrix_version, decision_receipt}`. Harnesses stop deciding models locally; each maps its native call into one `RouteRequest`:

| Harness | Adapter maps → RouteRequest | Replaces its today |
|---|---|---|
| **claude** (OMC) | its scorer signals → `feature-vector`; `agentType` → `capability_demand` seed | its Claude-locked tier→`TIER_MODELS` |
| **codex** (OMX) | agent `modelClass`/`-low` suffix → demand tier; `reasoningEffort` → effort request | static `resolveAgentDefaultModel` table |
| **gjc** | per-agent role + frontmatter intent → demand tier + constraints | its string-match `modelRoles` cascade |
| **hermes** | kanban `tasks.priority` + the §2 rubric factors → feature-vector; assignee → tenant/constraints | profile-pinned static `model:` block |
| **agy** (antigravity; **Gemini RETIRED from OAuth pool**) | antigravity role → demand tier; Gemini rows excluded from OAuth-pool eligibility, reachable only as constrained/BYOK catalog entries | `"Gemini 3.1 Pro (High)"` hardcoded label |

Adapters are thin and per-harness; the scorer, matrix, tuning loop, and fallback machine exist **once** in the substrate.

### 5. Fit to the owned substrate — exact crate-by-crate extensions

`oya-intelligence-model-routing` already has: the clean kernel/domain/usecase split; deterministic total-order tie-break (tests assert it); `BTreeSet<RouteDenialReason>` explainability; per-candidate `CandidateDenial` trail; in-memory idempotency (`BTreeMap<key,(fingerprint,receipt)>`); metadata-only redaction guards; retired-authority poison-ref checks; a `ModelProvider` enum (Anthropic/AzureOpenAi/Gemini/Local/OpenAi) and opaque `model_id`/`priority` catalog. It **lacks every number** (no cost, no eval score, no latency, no route classes, no attempt cap, no health signal, no per-tenant policy object, no durable events). ADR-INT-001 is still **Proposed** — its 15%/0.92/0.85/0.75 thresholds are draft targets, land it Accepted alongside this work.

**kernel** (`-kernel/src/lib.rs`): add `ModelCandidate{ pass_at_1: f32, cost_per_task_minor: u64, p95_latency_ms: u32, effort_ceiling: Effort, model_release_id }`; add the pure `utility(candidate, weights, floors) -> Option<Ranked>` fn and `select_by_utility` **beside** (not replacing) `select_highest_ranked_candidate`, preserving the existing total order as the tie-break. Add `ModelDefaultClass`/`ModelProfileTag` → keep as hard gates layered *before* utility. Pure, zero I/O — matches the existing kernel contract.

**domain** (`-domain/src/lib.rs`): extend `validate_route_request` to fail-closed on missing `quality_floor` / malformed `RoutePolicy` / effort exceeding `effort_ceiling`; add the "already-sent-to-provider-A" **attempt-state** validation the current stateless filter cannot express (needed for §4c refusal/residency guards).

**usecase** (`-usecase/src/lib.rs`): replace `walk_catalog_for_route`'s flat whole-catalog walk with the **bounded route-class cascade** (primary → same-quality → budget, ≤2 attempts), carrying a numeric score/cost per candidate in the existing `CandidateDenial` scaffold (the research explicitly notes this trail is "exactly the scaffolding a quantitative engine would extend"). Emit durable `RouteOutcomeEvent`/`UsageCostEvent` (today: in-memory only, declared non-goal — now required for §4b). Keep idempotency + redaction unchanged.

**new crates:** `-policy` (typed loader/validator for the §4d DATA blobs, content-addressed + version-pinned, `is_safe_opaque_ref`-style guards reused); `-tuning` (EWMA reconciler consuming `RouteOutcomeEvent`, writing new matrix versions); `-adapters` (the five harness→`RouteRequest` mappers) or one thin adapter crate per harness. Eventually integrate the sibling `eval`/`guardrails`/`autonomy-ceiling` crates (ROUTING-VERDICT: intelligence is the correct home; `cloud-intelligence` is egress/credential-broker only — **do not** put scoring there).

### 6. Product properties (mechanical)

- **Configurable:** every threshold, weight, `λ`, quality floor, effort ceiling, and per-tenant `RoutePolicy` is `-policy`-crate DATA, hot-reloadable and version-pinned; zero code change / redeploy to retune (contrast: OMC/OMX/GJC/Hermes all require a source or YAML-in-repo edit).
- **API-driven:** one `intelligence.ModelRouting/Route` gRPC+REST endpoint is the sole entry; no harness embeds model choice; canonical REST/gRPC/streaming, **zero CLI** surface and **zero GraphQL** per the owned-API doctrine. Every decision returns a replayable receipt (model + `matrix_version` + denial trail).
- **Cloud-native:** policy blobs and the score matrix are content-addressed objects reconciled via GitOps/CRD; the tuning reconciler runs as a K8s operator consuming the `RouteOutcomeEvent` stream; fail-closed on policy-fault (deny, never silently route). Per-tenant isolation via the existing `allowed_tenants` + `RoutePolicy`.
- **buck2-native:** each crate ships a `BUCK` target with reindeer-managed deps; kernel/domain/usecase/policy/tuning build and test under `buck2 test`; acceptance tests (existing `acceptance.rs`) extended to assert the new total order and cascade caps — buck2 green plus the freshness/affected-set gates, not cargo-only.

### 7. Open questions, risks, phased plan

**Open questions**
1. **Hermes premise** — the founder's "Hermes-Kanban mechanical scoring formula" **does not exist** (bare integer + prose rubric). Confirm we're generalizing the *rubric factors* (§2), not porting a nonexistent equation.
2. **`terra` / `luna` provenance** — vendor, pool eligibility, and effort ceilings are **not in the research**; needed before they can sit on the routing frontier.
3. **agy/Gemini scope** — "retired from OAuth pooling" is clear; are Gemini rows fully excluded, or reachable via BYOK/console for vision-only demand? Decides whether they stay catalog entries at all.
4. **ADR-INT-001 ratification** — its numbers are draft (status Proposed); land it Accepted, or the engine's floors/headroom are unratified.
5. **Effort↔Pass@1 coupling** — the observed matrix is per-model, but effort (`low..Max`) plausibly shifts Pass@1/$; is the matrix keyed by (model × effort)? Research gives only per-model points.

**Risks**
- **Feature extraction is still English regex/keyword** (inherited OMC weakness) — paraphrase/non-English/mixed-signal prompts ("find the root cause…" mixes simple−2 with debug+2) misclassify demand tier. Mitigate: keep the OMC over-provision bias (round up on low confidence) and treat confidence as *uncalibrated* (it is).
- **Tuning-loop feedback instability** — small-sample buckets (rare capability×tenant combos) give noisy EWMA; guard with min-sample floors before a cell overrides the bootstrap benchmark.
- **Dominated-model leakage** — sonnet-5/opus-4.8/fable must be selected *only* under a hard constraint; a bug in the eligibility gate silently burns spend on strictly-worse models. Add a test: "no dominated model is ever chosen when a frontier model is eligible."
- **Attempt-state correctness** (residency/refusal no-retry) is a **security/data-exposure** path — not lazy-simplifiable; needs explicit RED fixtures.
- **Two divergent complexity paths** (OMC's scorer vs task-decomposer's `effort` float) — unify into the one feature-vector or they drift again.

**Phased build plan**
- **P0 — land the numbers as data.** Accept ADR-INT-001; add `ModelCandidate` numeric fields to kernel; seed `-policy` matrix from the six observed rows above. No behavior change yet (still priority-ordered). Test: matrix loads, total order preserved.
- **P1 — utility selection.** Add `select_by_utility` + `λ`/quality-floor policy; wire usecase to it behind a policy flag; assert dominated-model exclusion + reproducibility. Ship "capability-first with cost pressure" for one harness (claude) via its adapter.
- **P2 — bounded cost-aware cascade.** Replace flat walk with primary→same-quality→budget classes, ≤2-attempt cap, GJC-style reason-classified backoff, OMX boundary notice, and the refusal/residency attempt-state guards.
- **P3 — durable outcomes + tuning loop.** Emit `RouteOutcomeEvent`; stand up the `-tuning` EWMA reconciler as a K8s operator writing versioned matrices; pin selection to a matrix version. This closes the observed→tuned loop.
- **P4 — all five harness adapters + retire local routers.** Land claude/codex/gjc/hermes/agy adapters against the one port; deprecate each harness's local model-choice (OMC `TIER_MODELS`, OMX role table, GJC `modelRoles`, Hermes profile `model:`) by redirecting-with-warning (OMC's own deprecation pattern), not deleting call sites at once.
- **P5 — integrate siblings.** Feed `eval`/`guardrails`/`autonomy-ceiling` quality/safety signals into the matrix and floors instead of the bootstrap benchmark alone.

skipped: any online-learning selector (no source has one, unjustified); a bespoke prompt classifier beyond OMC's regime. add when the EWMA loop proves feature-extraction misroutes measurably.
