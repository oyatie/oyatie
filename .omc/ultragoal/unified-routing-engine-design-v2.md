## Unified Model-Routing Engine — design v2 (source-verified)

*Grounds only in the two STUDY-DATA records + the live substrate at `oya/intelligence/crates/oya-intelligence-model-routing-{kernel,domain,usecase}` and `oya/intelligence/decisions/ADR-INT-001` (status: **Proposed**), inspected this session.*

---

### 1. Hermes source findings

**Routing (source-verified).** Hermes has no single "model picker." It is a **task-name-keyed routing table** funnelled through one resolver:

- **Worker/profile scope** — each named profile (`profiles.py` `ProfileInfo`, `_read_config_model()` L680-693) carries its own `model.default`/`model.provider`; the kanban dispatcher assigns a task to a profile and *that profile's* config supplies the main model.
- **Auxiliary-task scope (the real engine)** — `auxiliary_client.py::_resolve_task_provider_model(task)` (L5857) resolves each named task string (`"triage_specifier"`, `"kanban_decomposer"`, `"compression"`, `"web_extract"`, `"moa_reference"`) independently in priority order: call-site args → `config.yaml auxiliary.<task>.{provider,model,…}` → `"auto"` provider-chain detection. Everything routes through one `call_llm()` entry point with a bounded LRU `_client_cache` (64 entries).
- **MoA ensemble** — orthogonal fan-out layer (`moa_loop.py`, `_MAX_REFERENCE_WORKERS=8`); each reference slot resolves its **own** provider/model/temperature and is cost-accounted at **its own** rate (`_RefAccounting`), never folded into the aggregator.
- **Credential pool** — `credential_pool.py` (2384 lines) is same-provider failover/load-spreading, *not* task routing.

Net: routing is a **config-driven resolver chain keyed by task string**, profile-scoped for assignment, with independent ensemble and credential-failover layers underneath.

**Kanban "scoring" — v1's "no formula" claim CONFIRMED from source (not just install artifact).** `tasks.priority` is a bare `INTEGER DEFAULT 0` column (`kanban_db.py` L1102). Every ordering path is the identical literal SQL `priority DESC, created_at ASC` (L2715 sort table, L2769 default list, recurring L7105/L7347). The CLI flag is documented as a raw `"Priority tiebreaker"` (`kanban.py` L323/L387); the MCP schema tells the LLM the same (`kanban_tools.py` L1442-1448: *"Dispatcher tiebreaker. Higher = picked sooner"*). Grep for `wsjf|rice|ice|score|weight|rank` across `kanban_db.py` returns only column defs, the sort table, and an unrelated `"priority"→"reprioritized"` event rename. **CORRECTION to v1's phrasing:** even "the LLM applies a subjective rubric" *overstates* it — the specify/decompose system prompts never reference `priority` at all; it is set **only** by a manual `--priority N`. There is zero RICE/ICE/WSJF anywhere. This is not a scorer to borrow; it is a proof that a competent single-operator harness ships **no** computed priority.

**Work-item decomposition (newly covered — this is where Hermes' real intelligence lives).** `kanban_decompose.py` is **LLM-authored DAG generation**:

- A `triage` one-liner → `get_text_auxiliary_client("kanban_decomposer")` with `_SYSTEM_PROMPT` (L52-109) returns one JSON `{"fanout":bool,"rationale":str,"tasks":[{title,body,assignee,"parents":[int,…]}]}`.
- **DAG is index-based**: `parents` are 0-based indices into the same `tasks` array = real data dependencies; prompt: *"Tasks with no parents run in PARALLEL… Prefer parallelism… Use 2-6 tasks for normal work"* (L79-92).
- **Model output is never trusted as-is**: `clean_parents` (L433) drops non-int/out-of-range/self-ref indices; `_normalize_assignee_choice` (L252) rewrites hallucinated assignees to a `default_assignee` so no child is ever unrouted.
- **Fan-out/fan-in lifecycle**: `kb.decompose_triage_task()` atomically creates children, flips root `triage→todo`; root stays alive as parent-of-all so its orchestrator profile wakes to judge completion. `todo→ready` auto-promotes once every parent hits `done`. States: `triage→todo→ready→running→done` (+`blocked`/`archived`).
- `fanout:false` collapses to `specify` (title/body tighten only) — decompose is a *strict superset* of specify, one code path.
- Calls are **one-shot, temp 0.3, no retry, fail-soft** → typed `Outcome{ok,reason}` so a `--all` sweep degrades per-item.

**How decomposition informs task-complexity scoring:** Hermes gives the LLM the two levers we need to *mechanize* — a **fan-out count (2-6)** and a **dependency-edge structure** — as evidence of task complexity. A task the decomposer splits into 6 parallel subtasks with a deep parent-chain is objectively more complex than a `fanout:false` single spec. Our engine reads the same signal but as **numeric features** (subtask count, DAG depth, DAG width) feeding a scorer, instead of leaving it implicit in the model's head. Decompose stays LLM-authored; **scoring the resulting graph is deterministic.**

---

### 2. Current-state map

| Harness | Selection mechanism | Numeric score? | Cost used? | Availability/failover | Task-complexity signal | Portable nugget |
|---|---|---|---|---|---|---|
| **OMC** | additive integer weights (lexical+structural+context), thresholds HIGH≥8/MED≥4; ensemble-of-two disagreement caps confidence 0.5 → costlier/safer tier | **yes (only real scorer)** | no (capability-only) | none | crude decomposer effort<0.3/0.7 | the integer-weight + threshold-ladder discipline; disagreement→safe-tier |
| **OMX** | static `role→modelClass` table (fast/standard/frontier) + `reasoningEffort` | no | no | none | role name only | effort as a first-class output field |
| **GJC** | deterministic per-agent cascade over 52-model catalog | no | catalog carries `cost`, **never read** | rate-limit reason→per-class backoff (QUOTA 30m/RATE 30s/CAPACITY 45-75s/SERVER 20s); availability orthogonal to preference; ROI advisory never gates | none | reason-typed backoff table; the *unused* cost column is the gap to close |
| **Hermes** (source) | task-name-keyed config resolver → `call_llm()`; profile-scoped assignment; MoA fan-out; credential pool | no | per-member MoA accounting only | same-provider credential failover pool | **LLM-authored DAG (fan-out 2-6 + parent edges)**; manual `priority` int tiebreaker | task-keyed routing table; index-DAG-then-sanitize; fail-soft `Outcome`; per-member ensemble cost |
| **agy** (ex-Gemini) | n/a — retired from OAuth | — | — | — | — | now an API-key provider candidate, not an OAuth seat |
| **Owned substrate** (`model-routing-{kernel,domain,usecase}`) | `decide_route`→validate→`select_highest_ranked_candidate` = eligibility **filter** + **lowest `priority: u16`** (`kernel/src/lib.rs` L327-337) | **no continuous score** | cost = `evidence_refs` **presence** only (L114) | none in kernel (pure, no clock/net) | none | clean pure kernel + validate seam + audit-evidence discipline — *the extension target* |
| **ADR-INT-001** (Proposed, unimplemented) | `primary`/`same_quality_fallback`/`budget_fallback`; 15% headroom; ≤2 attempts (1 high-risk); `eval_score` floors 0.92/0.85/0.75 | **specifies numeric**, none built | `cost_per_1k_input/output` in `ModelCandidate` | health/latency-gated fallback classes | quality floor per task-impact | **the numbers to implement** |

**One line:** OMC is the only real scorer but is capability-only/English-regex/Claude-locked; the owned substrate is the only clean architecture but has *no* score; ADR-INT-001 already wrote the numbers nobody built. The engine below fuses OMC's scoring discipline + Hermes' decomposition signal + ADR-INT-001's numbers into the owned kernel.

---

### 3. The task-complexity score

**Feature vector** (generalized from Hermes' decompose factors + OMC's integer-weight discipline). Additive integer weights — deliberately copying OMC, because a linear scorer with a threshold ladder is auditable, cheap, and already proven in this stack. No ML classifier at v1.

```
CapabilityDemand = Σ wᵢ · featureᵢ      (u16, saturating)
```

| Feature (Hermes-derived taxonomy) | Source signal | Example weights |
|---|---|---|
| `fanout_count` | decompose `tasks[]` length (1 = specify, 2-6 = parallel) | 0→0, 2-3→+2, 4-6→+4 |
| `dag_depth` | longest parent-chain in `parents[]` | +2 per level >1 |
| `dag_width` | max parallel siblings (no-parent set) | +1 per sibling >2 |
| `modality_load` | count of {code, vision, tool-use, json, long-context} required (maps `ModelCapability` enum) | +2 each |
| `context_span` | prompt+retrieval token estimate bucketed | ≤4k→0, ≤32k→+2, >32k→+4 |
| `risk_tier` | `IntelligenceDataClass`/`risk_tier` (PHI/financial/legal) | +3 (also hard-gates §4) |
| `reversibility` | write-gate / irreversible-op flag | +2 |
| `caller_effort_hint` | OMC/OMX-style explicit override | additive, clamps |

**Threshold ladder → capability-demand tier** (OMC's HIGH≥8/MED≥4 shape, retuned for the wider vector):

| Score | Tier | Meaning |
|---|---|---|
| ≥ 10 | `Max` | frontier + max effort |
| 6–9 | `High` | frontier / high effort |
| 3–5 | `Standard` | mid model, standard effort |
| 0–2 | `Fast` | cheapest eligible, low effort |

**Ensemble-of-two disagreement rule (ported from OMC):** run the additive scorer *and* a cheap secondary estimator (e.g. token-margin bucket). If tiers disagree, **cap confidence at 0.5 and pick the costlier/safer tier** — identical to OMC's rule, now feeding a cost-aware fallback instead of a raw pick. Output is `{tier, effort, confidence}` — effort is a first-class field (OMX contribution).

`ponytail:` this is a linear integer scorer, not a learned router. Upgrade path is Section 5's RouteLLM classifier behind the same `ComplexityScorer` trait once we have labeled `(task, observed-outcome)` data — the tuning loop (§4) is what generates that data, so we don't add the classifier until the loop has run.

---

### 4. The engine architecture

**One engine. One trait surface. Every harness calls `decide_route`.** We do **not** rebuild — we extend the existing pure kernel `decide_route` (`kernel/src/lib.rs` L151) from *filter+lowest-priority* into *filter+score*.

```
task/idea
  │
  ├─▶ ComplexityScorer::score(task) ──▶ {tier, effort, confidence}   [§3]
  │
  ▼
decide_route(request, catalog):
  1. VALIDATE          (domain::validate_route_request — unchanged: tenant, evidence,
                        data-boundary, env-tier budget contract)   ← keep the seam
  2. ELIGIBILITY FILTER (kernel::profile_denials — unchanged: env-tier, tags,
                        data-class residency, BYOK credential mode, quality-floor gate)
  3. SCORE & RANK  ◀── NEW: replace select_highest_ranked_candidate's bare
                        `.priority.cmp()` with a continuous cost-aware score over the
                        surviving candidate set (below)
  4. COST-AWARE FALLBACK CASCADE (ADR-INT-001 classes)
  5. route(idempotency + audit)   (usecase — unchanged: emits evidence)
```

**Candidate score (the continuous score the substrate lacks today).** For each *eligible* `ModelCandidate` meeting the tier's `eval_score` floor:

```
utility = eval_score(model)  −  λ · normalized_cost(model)
normalized_cost = (cost_per_1k_input·in_tok + cost_per_1k_output·out_tok) / budget_ceiling
select = argmax utility  s.t. eval_score ≥ floor(tier)  ∧  est_cost ≤ max_cost_minor
```

`λ` is the single cost/quality dial (RouteLLM's `τ`, cascade literature's `λ` — Section 5). `eval_score` and `cost_per_1k_*` are the **OBSERVED** matrix (below); `λ` is what the **TUNED** loop moves. This makes GJC's dead `cost` column finally load-bearing and replaces "lowest u16 priority" with a measured utility.

**Model observed-score matrix (OBSERVED axis — seeded from DeepSWE).**

| Model | Pass@1 | $/run | Cap ceiling | Role |
|---|---|---|---|---|
| gpt-5.6-sol | 73% | $8.39 | Max | most capable |
| terra | 70% | $4.95 | High | balanced |
| luna | 67% | $3.03 | Standard | **most efficient** |
| fable | 70% | $21.63 | **High (never Max)** | capable-but-costly |
| opus-4.8 | 59% | $13.22 | High | — |
| sonnet-5 | 54% | $26.40 | Standard | dominated (low score, high cost) |

This matrix **is** `ModelReleaseEvidence{model_release_id, eval_set_id, eval_score, …}` (ADR-INT-001 §Data shapes) — content-addressed, referenced by `evidence_refs`, never inline. The kernel stays pure; the matrix is a **catalog snapshot the caller passes in** (the existing kernel contract).

**Tuning/feedback loop (TUNED axis).** Each `route()` emits a `ProviderAttempt{state, latency_ms, usage, normalized_error}` (ADR-INT-001 shape). A `model-routing-worker` (new usecase-side, off the kernel) aggregates attempts → rolling **observed** Pass@1 / p95-latency / realized-$ per (model, task-cluster) → rewrites the `ModelReleaseEvidence` snapshot and nudges `λ` toward the tenant's `quality_floor`/budget target. **DeepSWE numbers seed t=0; production replaces them.** Loop is content-addressed evidence writes only — no mutable model state in the kernel.

**Cost-aware fallback cascade (ADR-INT-001 numbers, now implemented).** Three route classes, evaluated only *after* the eligibility filter has produced an allowed set:

- `primary` — budget headroom ≥ **15%** of monthly AI budget ∧ provider health green.
- `same_quality_fallback` — primary unhealthy / rate-limited / above p95 latency budget → next candidate **at the same `eval_score` floor**.
- `budget_fallback` — headroom < **15%** ∧ an approved lower-cost model still meets the task quality floor.
- **Stop after 2 provider attempts** (1 for high-risk) unless human-approved retry. High-risk data-classes (PHI/financial/child/employment/education/legal) **never** fall back to a provider not approved for that class+region.
- Failover reason typing borrows GJC's table (QUOTA/RATE/CAPACITY/SERVER → per-class backoff) and LiteLLM's cooldown circuit-breaker (§5). Emits `EVT-INT-ROUTE-SELECTED / -FALLBACK-USED / -BUDGET-FALLBACK-SUPPRESSED / -HIGH-RISK-ROUTE-DENIED`.

**Policy-as-DATA schema** (ADR-INT-001 `RoutePolicy`, extended with the scorer knobs — this is the config, no code change to retune):

```json
{
  "tenant_id": "...",
  "allowed_providers": ["anthropic","openai","local"],
  "denied_providers": [],
  "preferred_models": {"High":"terra","Max":"gpt-5.6-sol"},
  "fallback_allowed": true,
  "quality_floor": {"high_impact":0.92,"limited":0.85,"minimal":0.75},
  "residency_set": ["eu"],
  "max_cost_minor": 500,
  "lambda_cost_weight": 0.4,
  "complexity_weights": {"fanout_count":[0,2,4],"dag_depth":2,"modality_load":2,"risk_tier":3},
  "tier_thresholds": {"Max":10,"High":6,"Standard":3}
}
```

**How each harness calls the ONE engine** (task-keyed like Hermes, but one binary):

| Harness | Call | What it passes |
|---|---|---|
| **claude** (OMC/OMX) | `decide_route(score(task), catalog)` | replaces OMC's Claude-locked regex scorer with the shared vector; keeps effort output |
| **codex** | same | codex model_release_ids in the catalog snapshot |
| **gjc** | same | its 52-model catalog becomes the `ModelCandidate` set; `cost` column now *read*; reason-typed backoff preserved in fallback layer |
| **hermes** | task-name key → `RoutePolicy.preferred_models` lookup, then `decide_route` | decompose still LLM-authored; the resulting fan-out/DAG **feeds the complexity vector** |
| **agy** | same | ex-Gemini enrolled as API-key `ModelProvider::Gemini` candidate (not an OAuth seat) |

Every harness gets `{model, effort}` + cost-aware fallback from the same pure kernel + one policy-data file per tenant.

---

### 5. External best-practices adopted (Rust-native, cited)

| Pattern | Source | Rust-native adoption | Where |
|---|---|---|---|
| **Single cost/quality dial** (`τ` win-prob threshold) | RouteLLM, LMSYS arXiv 2406.18665 | `λ` in `utility = eval − λ·cost`; one tunable constant, no inference-time service call | §4 candidate score |
| **Cost-adjusted error per cluster** `error + λ·cost` | cascade survey arXiv 2603.04445 | identical closed form over task-clusters | §4 tuning loop clusters |
| **Two-stage cluster→escalate** | arXiv 2606.27457, UCCI 2605.18796 | Stage-1 complexity tier (cheap), Stage-2 confidence-gated escalation = our fallback cascade; swappable `EscalationPolicy` trait | §4 cascade |
| **Isotonic calibration of a raw uncertainty → P(error)** then cost-minimizing threshold | UCCI arXiv 2605.18796 | PAVA (~50 lines, **zero deps**) to calibrate the ensemble-disagreement confidence into the escalation cutoff | §3 confidence → §4 escalate |
| **Circuit-breaker cooldown pool** (`allowed_fails`→exclude for `cooldown_time`; 429→immediate) | LiteLLM Router | `DashMap<DeploymentId, CooldownState>` + `Instant` expiry, std only; merges with GJC's reason-typed backoff | §4 fallback |
| **Inverse-square price weighting** among healthy providers | OpenRouter | `rand::distributions::WeightedIndex` over `1/price²` for *tie-break* among same-`eval_score` candidates | §4 same_quality_fallback |
| **Explicit per-caller cost ceiling / willingness-to-pay** | Martian | already `max_cost_minor` in `RoutePolicy` — hard-caps eligible set *before* scoring | §4 filter |
| **Learned meta-router over (prompt, model, score)** | NotDiamond / RoRF | deferred upgrade path for `ComplexityScorer` (needs 15–10k labels the tuning loop produces) | §3 `ponytail:` note |

`ponytail:` we take **RouteLLM's one-dial `λ` and the cascade `error+λ·cost` form as the whole scorer** and skip the learned meta-model until the tuning loop has produced labels — a linear scorer + one constant beats a trained router we can't yet feed. Isotonic PAVA and the cooldown pool are ~50 lines each with no new deps; adopt directly.

---

### 6. Fit to the owned substrate — crate-by-crate

Destination is the **de-branded `intelligence/` capability**. `oya-intelligence-*` names are the reorg de-brand target, not the ship name. **Extend, don't rebuild** — the three crates already exist and are clean.

| Crate | Today | Extension |
|---|---|---|
| `…model-routing-kernel` (502 L, pure, no clock/net) | `decide_route` = validate-refs + `profile_denials` filter + `select_highest_ranked_candidate` (bare `priority.cmp`, L327); cost = `evidence_refs` presence only | Replace the `.priority.cmp()` ranker with `score_candidates(surviving, catalog_snapshot, λ)` → `utility = eval_score − λ·norm_cost`. Add `eval_score`, `cost_per_1k_input/output` to `ProviderRouteProfile`/`ModelCandidate` (ADR-INT-001 shapes). Add `ComplexityScorer` trait + the additive integer scorer (§3). **Stays pure** — catalog + weights are caller-passed. This implements ADR-INT-001's *"exact numeric fallback thresholds"* (its stated Open Question §52). |
| `…model-routing-domain` (365 L) | `validate_route_request` (tenant/evidence/data-boundary/env-tier-budget) | Add `RoutePolicy` validation (quality_floor set, λ range 0–1, residency non-empty when data-class sensitive). Keep the fail-closed data-boundary check (`rejects_sensitive_data_for_external_audience` test L283) — the high-risk-no-fallback rule lives here. |
| `…model-routing-usecase` | validated → route(idempotency+audit) | Emit ADR-INT-001 events `EVT-INT-ROUTE-SELECTED/-FALLBACK-USED/-BUDGET-FALLBACK-SUPPRESSED/-HIGH-RISK-ROUTE-DENIED` + `ProviderAttempt` evidence. Owns the `primary/same_quality/budget` cascade + cooldown pool (impure clock/health lives here, not the kernel). |
| **NEW** `…model-routing-worker` | — | Aggregates `ProviderAttempt` → rolling observed Pass@1/latency/$; rewrites `ModelReleaseEvidence` snapshot; nudges `λ`. This is the TUNED loop. Reuses `oya-intelligence-eval-*` crates (already present) for the eval-score source. |
| `…model-routing-*-api` (REST/gRPC per catalog) | — | `POST /route` returns `RouteDecision{selected_candidate, fallback_class, reason_codes, estimated_cost, budget_headroom}`. **Zero GraphQL** (owned-stack doctrine — REST+gRPC+streaming). |

**Implement ADR-INT-001's unimplemented numbers, verbatim:** 15%-headroom `primary`/`budget_fallback` split; ≤2 attempts (1 high-risk); `eval_score ≥ 0.92 / 0.85 / 0.75` floors; the six data-shapes (`DispatchEnvelope`, `RoutePolicy`, `ModelCandidate`, `RouteDecision`, `ProviderAttempt`, `ModelReleaseEvidence`). ADR flips `Proposed → Accepted` **via propagation** (land it Proposed, formal-Accepted rides the cross-artifact gate — per repo doctrine).

Provider adapters already exist (`…adapter-anthropic-*`, `…adapter-openai-*`, `…adapter-gemini-*`, `…codex-account-adapter`) — agy = the gemini API-key adapter re-enrolled as a `ModelCandidate`, not an OAuth seat. `…provider-pool-kernel` supplies Hermes-style same-provider credential failover under the cascade.

---

### 7. Product properties + phased plan + open questions

**Product properties**

- **Configurable / policy-as-data** — all knobs (`λ`, `complexity_weights`, `tier_thresholds`, `quality_floor`, `max_cost_minor`, `preferred_models`) live in the `RoutePolicy` JSON; retuning ships **no code**. Kernel takes weights + catalog as arguments.
- **API-driven** — `POST /route` (REST + gRPC), streaming for progressive fallback events; `RouteDecision` is fully explainable (fallback_class, reason_codes, estimated_cost, budget_headroom) per ADR-INT-001's tenant-explainability requirement.
- **Cloud-native** — `RoutePolicy` and `ModelReleaseEvidence` are CRD-shaped declarative state; the tuning worker is a reconciler over `ProviderAttempt` events; no imperative ops, no CLI (retirement doctrine).
- **buck2-native** — three (soon four) existing crates already carry BUCK targets; new worker + score module land as pure-Rust crates, reindeer-wired, `buck2 test` green as part of done.

**Phased build plan**

1. **P0 — kernel score.** Add `eval_score`/`cost_per_1k_*` fields + `score_candidates` (utility = eval − λ·cost) + `ComplexityScorer` additive integer scorer + ensemble-disagreement→safe-tier. Seed matrix = DeepSWE numbers. RED/GREEN kernel tests. *Deliverable: `decide_route` picks by measured utility, not u16.*
2. **P1 — cascade + policy.** Implement ADR-INT-001 three route classes, 15% headroom, ≤2 attempts, high-risk-no-fallback in usecase; `RoutePolicy` validation in domain; cooldown pool (LiteLLM) + reason-typed backoff (GJC). Emit the four events + `ProviderAttempt`.
3. **P2 — API + harness cutover.** `POST /route`; migrate claude/codex/gjc/hermes/agy call-sites to the one engine (task-keyed lookup → `decide_route`). Delete OMC's Claude-locked regex scorer.
4. **P3 — tuning loop.** `…model-routing-worker` aggregates `ProviderAttempt` → rewrites `ModelReleaseEvidence`, nudges `λ`. DeepSWE seed replaced by production observations.
5. **P4 (deferred) — learned router.** Swap the additive scorer for a RouteLLM/RoRF classifier behind `ComplexityScorer` *only* once P3 has produced labels. `ponytail:` skip until data exists.

**Open questions**

1. **Task-cluster granularity for the tuning loop** — per (model, capability-tier), or per Hermes-style task-name key? Finer = better routing, sparser data. Start coarse (tier), split when a cluster has enough attempts.
2. **`λ` scope** — global, per-tenant, or per-task-impact? ADR-INT-001 implies per-tenant budget; high-impact tasks may need their own `λ`. Default per-tenant, override per quality-floor bucket.
3. **fable's `High`-never-`Max` ceiling** — is that a hard policy tag (residency-style filter) or a soft score penalty? Recommend **hard tag** (`forbidden_tags` already exist in the kernel, L291) — capability ceilings are eligibility, not utility.
4. **DeepSWE→production transfer** — DeepSWE Pass@1 is one eval set; does it predict *this* tenant's task mix? The `eval_set_id` in `ModelReleaseEvidence` makes the seed's provenance explicit and swappable — but P0 routing quality is only as good as the seed until P3 runs.
5. **Sonnet-5 is strictly dominated** in the seed matrix (54% @ $26.40) — keep it as an eligible candidate at all, or prune? Keep (residency/BYOK fallback value), but it will never win the `argmax` unless policy pins it.

**Verified against source this session:** kernel selection is confirmed `select_highest_ranked_candidate` on `priority: u16` with cost = `evidence_refs` presence only (`oya-intelligence-model-routing-kernel/src/lib.rs` L114, L327-337); ADR-INT-001 is `status: Proposed` with all numbers (15% headroom, ≤2 attempts, 0.92/0.85/0.75 floors, six data-shapes) present but unimplemented (`oya/intelligence/decisions/ADR-INT-001-*.md`).
