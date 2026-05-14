# RALPLAN — Dependency Seam Discipline + Tech-Debt Ledger + Phase-Out Trajectory (Round 5, FINAL)

> **Status:** `pending review (round 5; FINAL)`
> **Authored:** emitted by `oya-dev-cli gate emit tech-debt-ledger` (timestamps via `oya-dev-cli emit current-date` — single emit-tool resolves `current_date()` literals; never `date(1)` / never `chrono::Utc::now()` in plan body)
> **Predecessor:** `ralplan-dep-seam-phaseout-round-4.md` (Architect APPROVE-WITH-NOTES R4; Critic ITERATE R4 1 CRITICAL + 3 MAJOR; codex ITERATE R4 1 partial concern)
> **Does NOT supersede:** ADR-0090
> **New ADRs:** ADR-0091 (§8); ADR-0092 (§18.B); ADR-0093 (§4 — CI carve-out, **Status: Accepted at Step 6 only**); ADR-0094 (§3.B — `oya-http-sse-kernel/runtime` split rationale)
> **Round-5 mandate:** close every Critic CRITICAL + MAJOR + MINOR + codex C3 example + Architect 3 notes; this is the last iteration.

---

## 0. Reframe (unchanged)

Ship 5 products on hyper/tokio FIRST; phase out AFTER ontology v1 stable. Plan shape/seams/ledger/replacements NOW so release work does not deepen debt. Eliminated failure mode: *"release work accidentally deepens debt because there's no enforced seam and no ledger."*

---

## 1. RALPLAN-DR Summary (round 5)

### Principles (6 — unchanged)

1. Systematic phase-out > immediate replacement.
2. Conscious debt > accidental debt. Every dep named/layered/owned with a replacement target.
3. Seams = lane crate + `fitness-lanes/*.md` + CI job — not doc paragraphs.
4. Public APIs are wrapper types only. No `hyper::Request`/`tokio::JoinHandle`/`bytes::Bytes` outside `adapter`-layer crates.
5. Phase-out is triggered, not scheduled.
6. Machine-evaluable triggers > judgment calls.

### Decision Drivers (top 3 — unchanged)

1. **Release velocity** (no backbone rewrite in critical path).
2. **Reversibility** (dep swappable without touching kernel/runtime).
3. **No-silent-regression** (lean-a10).

### Viable Options (≥2; tiered parity bars — unchanged)

A. **Triggered-incremental + tiered GREEN/AMBER/RED** — Chosen.
B. Big-bang at T0 — rejected (regression concentration).
C. Opportunistic — rejected (deepens debt invisibly).
D. Forked-vendor-in-tree — rejected (supply-chain surface doubled).

### Pre-mortem (3 scenarios — unchanged)

- **F1: Trigger never fires.** Named DRI; quarterly review; CVE shortcut; T0+12 walk-away.
- **F2: Seam lane bypassed.** §15 + composite lane + role-gated label + same-PR guard + CODEOWNERS.
- **F3: Replacement underperforms parity.** Tiered AMBER + ADR amendment; RED → grit-mediated walk-away.

### Expanded Test Plan (unchanged from R4)

Seam fixtures (incl. `oya-http-sse-kernel` consumed from adapter without import error), ledger↔reality composite walk, parity benches, review-movement, distroless+musl smoke, active-artifact contract, trigger DSL fixtures (incl. `monotonicity-disabled` warn), layer-metadata coverage (5-value enum), ops binary cloud-native conformance, CI carve-out.

### Mode

**SHORT** — closes round-4 verdicts; no new exploration. (`--deliberate` not asserted; pre-mortem + expanded test plan retained from R4.)

---

## 2. Tech-Debt Ledger Spec (round 5 — closes codex C3 example + Architect Note 1)

### Location
`.omc/registries/tech-debt-ledger.json` (active artifact per ADR-0089).

### Schema envelope — inheritance pattern (codex C3 fix)

Round 4 mandated 4 evaluator policies on every trigger but its **example trigger row** omitted all 4 — self-contradiction. Round 5 relaxes to **registry-top-level defaults inherited by every row**, with optional per-row override. This eliminates per-row verbosity while preserving the 4-policy guarantee.

```json
{
  "$schema": "./schemas/tech-debt-ledger.schema.json",
  "version": "1.2.0",
  "generated_by": "oya-dev-cli gate emit tech-debt-ledger",
  "reviewed_quarterly": true,
  "next_review_due": "<emit-time + P90D>",
  "workspace_member_count_source": "cargo metadata --no-deps (dynamic)",
  "workspace_dependencies_source": "file:Cargo.toml:/workspace/dependencies",
  "default_evaluator_policies": {
    "staleness_policy": "fail-on-stale",
    "evidence_immutability_policy": "run-id-required",
    "pointer_missing_policy": "not-yet-armed",
    "monotonic_transitions_only": true
  },
  "entries": {
    "hyper": { "...row..." },
    "tokio": { "...row..." }
  }
}
```

**Inheritance rule (schema-enforced):** every trigger inherits `default_evaluator_policies`. A row MAY override one or more policies by emitting an `evaluator_policies` block inside the trigger; missing fields fall back to defaults. The `oya-foundry-trigger-dsl-runtime` parser resolves at load time and emits a structured warning per override (audit trail).

### `data_source` URI grammar (unchanged from R4)

```
data_source := file-uri | ci-uri-run | ci-uri-latest
file-uri      := "file:" <relative-path> ":" <json-pointer>
ci-uri-run    := "ci:" <lane-id> ":run:" <run-id> ":" <row-id>    # IMMUTABLE
ci-uri-latest := "ci:" <lane-id> ":latest:" <row-id>              # advisory only
```

**Relative-path resolution rule (round 5 — Critic MINOR fix).** All `file:` URIs are resolved relative to **the git-toplevel** (`git rev-parse --show-toplevel`). NOT cargo-workspace-root, NOT lane-CWD, NOT plan-file-directory. The `oya-foundry-trigger-dsl-runtime` evaluator calls `git rev-parse --show-toplevel` once at startup and prepends to every relative `file:` URI. The composite lane's CI invocation runs from the toplevel; local invocation (`--offline`) also resolves from toplevel.

### Evaluator semantics — 4 policies (with `monotonicity-disabled` audit treatment — Architect Note 1)

| Policy | Values | Default | Behavior |
|---|---|---|---|
| `staleness_policy` | `fail-on-stale` \| `warn-on-stale` \| `best-effort` | `fail-on-stale` | mtime older than `last_reviewed_at` → policy-dependent. |
| `evidence_immutability_policy` | `run-id-required` \| `latest-allowed` | `run-id-required` | `ci-uri-latest` in status-gating predicate while `run-id-required` → parser reject. |
| `pointer_missing_policy` | `fail` \| `not-yet-armed` | `not-yet-armed` | JSON pointer absent → fail = lane error; not-yet-armed = lane warn. |
| `monotonic_transitions_only` | `true` \| `false` (per-row override) | `true` | Schema-level guard on `status` field. **When set `false`, the row emits a warn-level lane finding `monotonicity-disabled: <dep_name>` (Architect Note 1).** The `oya-check-dependency-seam-discipline` composite reports those findings under sub-check `ledger-transition-monotonicity` but does NOT fail; downstream quarterly review must justify the disable in writing or revert. |

#### Status transition graph (monotonic — unchanged)

```
active → {scheduled, replacement-armed, replacement-armed-by-cve, keep}
scheduled → {replacement-armed, replacement-armed-by-cve, keep}
replacement-armed | replacement-armed-by-cve → {replaced, replacement-attempted-abandoned}
replaced | keep | replacement-attempted-abandoned   (terminal; fresh ADR re-opens)
```

### Trigger DSL — predicate registry

| Predicate | Data source | Type | Cross-row? |
|---|---|---|---|
| `ontology_v1_production_uptime_sprints` | `file:.omc/registries/release-state.json:/ontology/v1/production_uptime_sprints` | int | no |
| `p99_within_budget_days` | `file:.omc/registries/perf-budget-history.json:/<dep>/p99_within_budget_days` | int | no |
| `parity_bench_pass` | `ci:lean-a-replacement-parity:run:<run-id>:<dep>` | enum | no |
| `tracked_cve_open_cvss_gte` | `file:.omc/registries/cve-watch.json:/<dep>` | float+days | no |
| `dependent_wave_status` | `file:.omc/registries/tech-debt-ledger.json:/entries/<dep_name>/status` | enum | **YES** |
| `never` | n/a | constant `disarmed` | no |

**Cross-row predicate enumeration (Critic MINOR fix to round-4 §2 "every cross-row reference" ambiguity).** Exactly ONE predicate is cross-row: `dependent_wave_status`. Schema-level coverage check rejects circular `dependent_wave_status` references via topological sort at load time.

#### `never` trigger variant — does it bypass policies? (Critic MINOR fix)

**Answer: YES, explicitly.** `{"never": true}` short-circuits the policy stack: parser does NOT require `evaluator_policies` block on a `never` trigger; evaluator does NOT consult staleness/immutability/missing-pointer policies; it always returns constant `disarmed`. `monotonic_transitions_only` still applies at the row level (status field) but is irrelevant because `never` triggers never propose a transition. Sub-check `ledger-coverage` accepts `{"never": true}` only when status ∈ `{keep, replacement-attempted-abandoned}`.

### Example trigger row (round 5 — now policy-compliant via inheritance, codex C3 fix)

```json
"hyper": {
  "dep_name": "hyper",
  "version_pin": "1",
  "category": "http-server",
  "allowed_layers": ["adapter:oya-http-runtime-hyper-adapter"],
  "allowed_crates_regex": "^oya-http-runtime-hyper-adapter$",
  "replacement_target": "std-only",
  "replacement_crate_planned": "oya-http-runtime-std-adapter",
  "replacement_trigger": {
    "all_of": [
      {"predicate": "ontology_v1_production_uptime_sprints", "gte": 2,
       "data_source": "file:.omc/registries/release-state.json:/ontology/v1/production_uptime_sprints"},
      {"predicate": "p99_within_budget_days", "gte": 14,
       "data_source": "file:.omc/registries/perf-budget-history.json:/hyper/p99_within_budget_days"},
      {"predicate": "parity_bench_pass", "band": "GREEN_OR_AMBER",
       "data_source": "ci:lean-a-replacement-parity:run:<run-id>:hyper"}
    ]
  },
  "cve_acceleration_trigger": {
    "any_of": [
      {"predicate": "tracked_cve_open_cvss_gte", "gte": 7.5, "no_upstream_fix_days_gte": 14,
       "data_source": "file:.omc/registries/cve-watch.json:/hyper"}
    ]
  },
  "dri_handles": {"primary": "jason931225 (user)", "backup": "<TBD>"},
  "status": "active",
  "added_at": "<oya-dev-cli emit current-date>",
  "last_reviewed_at": "<oya-dev-cli emit current-date>",
  "adr_cite": ["ADR-0090", "ADR-0091"]
}
```

**Policies on this row resolve to the registry-top-level defaults** (`fail-on-stale`, `run-id-required`, `not-yet-armed`, `true`). No `evaluator_policies` block needed; example matches contract. The 4-policy guarantee is preserved via inheritance.

### Seed rows (11 — keys unchanged)

`hyper`, `hyper-util`, `http-body-util`, `tokio`, `bytes`, `tracing`, `tracing-subscriber`, `serde`, `serde_json`, `toml`, `toml_edit`. Per-row `status` / `replacement_target` / `allowed_crates_regex` carried verbatim from R3 §2 seed table. Bootstrap: `primary = jason931225 (user)`.

### `ledger-transition-monotonicity` sub-check comparator (Critic MINOR fix)

Comparator: **PR parent merge-base on `main`**. Specifically: `git merge-base origin/main HEAD` resolves the parent committed ledger; sub-check loads that revision's `.omc/registries/tech-debt-ledger.json` via `git show <merge-base>:.omc/registries/tech-debt-ledger.json` and diffs status fields per row. Rationale: HEAD~1 fails on multi-commit PRs; "last green CI" requires CI state coupling. PR-parent merge-base is deterministic, branch-independent, and matches the CODEOWNERS PR review model.

---

## 3. Dependency-Seam Contracts — middleware-layer reclassification + SSE kernel/runtime split (Critic CRITICAL #1)

### 3.A — Layer enum (unchanged authority)

Canonical: **`{kernel, runtime, adapter, api, app}`** per `.omc/automation/service-map-spec.md` §3. NO `domain` layer.

### 3.B — Critic CRITICAL #1: SSE rename creates adapter→runtime upstream edge — RESOLVED via PATH (b) kernel/runtime split

**The defect.** Round 4 §18.A renamed `oya-http-sse-domain` → `oya-http-sse-runtime`. But `crates/oya-http-runtime-hyper-adapter/Cargo.toml:12` declares `oya-http-sse-domain = { path = "../oya-http-sse-domain" }`. Post-rename, an `adapter`-layer crate would depend on a `runtime`-layer crate → **violates canonical DAG** `kernel → runtime → adapter → app` (per `service-map-spec.md` §5 "Downstream-only edges" BLOCKER rule). The very `layer-metadata` sub-check enforcing the canonical DAG would BLOCK day 0 of W0.

**Path chosen: (b) split into `*-kernel` (pure) + future `*-runtime` (deferred).**

**Empirical justification:**

1. `crates/oya-http-sse-domain/Cargo.toml` has empty `[dependencies]` — pure stdlib, zero I/O deps.
2. `crates/oya-http-sse-domain/src/lib.rs:1-12` docstring states: *"Server-Sent Events (SSE) framing helper — pure std-only ... Produces the on-wire text/event-stream byte format from typed events. The hyper-runtime adapter (Layer 5) wraps the byte stream in a hyper::Body."*
3. This is **pure data transform** — a serializer from typed `SseEvent` → bytes. By the `service-map-spec.md` §3 layer table:
   - `kernel` = "Pure value-object; no I/O" ✓ matches current behavior.
   - `runtime` = "Schedulers, lifecycles, side-effect orchestrators" ✗ no orchestration today.
4. **Path (a) reclassify-to-adapter** rejected: SSE has no provider-specific I/O semantics; reclassifying as `adapter` would create a parallel adapter without hyper deps, contradicting the "Provider-specific I/O" definition.
5. **Path (c) §3 carve-out + amend service-map-spec.md** rejected: amending the canonical DAG to permit adapter→runtime upstream edges destroys the very invariant this plan is enforcing. The seam discipline becomes self-undermining.
6. **Path (b) kernel/runtime split** is the honest classification: today's code is pure-types (= `kernel`); future async orchestration (if needed) lands in a sibling `*-runtime` crate that consumes the kernel. Hyper adapter depends on `*-kernel` (downward in DAG — valid).

**Decision (W0 Step 1 deliverable):**

```
oya-http-sse-domain  →  oya-http-sse-kernel
                        (layer = kernel; pure SseEvent types + serializer)

(no oya-http-sse-runtime crate created in W0 — deferred until orchestration
 logic is actually needed; documented in ADR-0094.)
```

ADR-0094 records this decision and references future-creation criteria: a `*-runtime` sibling is added only when reconnect/keepalive/backpressure orchestration ships, at which point it consumes `*-kernel` and the hyper-adapter optionally depends on `*-runtime` (the runtime in turn depends on the kernel).

**Hyper adapter post-rename Cargo.toml (lines 9-17, exact edits):**

```toml
[dependencies]
oya-http-router-kernel = { path = "../oya-http-router-kernel" }
oya-http-middleware-kernel = { path = "../oya-http-middleware-kernel" }
oya-http-sse-kernel = { path = "../oya-http-sse-kernel" }   # WAS: oya-http-sse-domain
hyper.workspace = true
hyper-util.workspace = true
tokio.workspace = true
http-body-util.workspace = true
bytes.workspace = true
```

Adapter → kernel = downward in DAG = **valid**. The `layer-metadata` sub-check passes; the seam invariant holds.

### 3.C — Codex Concern 2 (round 4 carried forward) — middleware-domain → middleware-runtime refactor

The 3 `*-middleware-domain` crates DO have I/O orchestration semantics (deadline timers, telemetry side-effects, tenant resolution side-effects) and DO consume the hyper adapter today (round-4 audit confirmed). Path (i) **refactor** stands: rename to `*-middleware-runtime`, drop `bytes`, drop `oya-http-runtime-hyper-adapter` dep, consume kernel newtypes only. This is unchanged from round 4 §3.

`oya-http-router-kernel` newtype surface expanded: `HttpHeaderMap`, `HttpHeaderName`, `HttpHeaderValue` (added in addition to round-3's `HttpRequest`/`HttpResponse`/`HttpBody`/`HttpBodyStream`).

### 3.D — Import-direction diagram (round 5 corrected)

```
oya-http-router-kernel        oya-http-middleware-kernel       oya-http-sse-kernel
(layer=kernel; newtypes)      (layer=kernel; trait defs)       (layer=kernel; SseEvent+serializer)
        ↑                              ↑                                ↑
        | (newtypes only)              | (trait only)                   | (types only)
        |                              |                                |
oya-http-{deadline,telemetry,tenant}-middleware-runtime                 |
(layer=runtime; chain logic)                                            |
        ↑                                                               |
        | (chain bind; NEVER imports adapter)                           |
        |                                                               |
        +─────────────────────────┬─────────────────────────────────────+
                                  |
                       oya-http-runtime-hyper-adapter
                       (layer=adapter; ONLY crate touching hyper::*)
```

**Allowed edges (downward in DAG):**
- adapter → kernel ✓ (e.g., hyper-adapter → sse-kernel, router-kernel, middleware-kernel)
- runtime → kernel ✓ (middleware-runtime → router-kernel, middleware-kernel)
- adapter → runtime ✓ (hyper-adapter → middleware-runtime composition)

**Banned edges (upward / regression):**
- runtime → adapter ✗ (the round-3 leak; closed)
- kernel → runtime/adapter/api/app ✗ (canonical DAG)

### 3.E — Audit (post-W0; canonical layers only)

| Crate | `[package.metadata.oyatie.layer]` | Hyper-family deps |
|---|---|---|
| `oya-http-router-kernel` | `kernel` | none |
| `oya-http-middleware-kernel` | `kernel` | none |
| `oya-http-sse-kernel` (renamed from `*-domain`; pure types) | `kernel` | none |
| `oya-http-runtime-hyper-adapter` | `adapter` | full hyper stack |
| `oya-http-deadline-middleware-runtime` (renamed) | `runtime` | none post-refactor |
| `oya-http-telemetry-middleware-runtime` (renamed) | `runtime` | none post-refactor |
| `oya-http-tenant-middleware-runtime` (renamed) | `runtime` | none post-refactor |
| `oya-ops-workspace-shell-runtime` | `runtime` | `tokio` (binary main) |

`bytes` declared only by the adapter post-W0. Ledger row 5 (`bytes`) `allowed_crates_regex` = `^oya-http-runtime-hyper-adapter$`.

### 3.F — `oya-foundry-trigger-dsl-*` naming — kernel/runtime split (Critic MINOR fix)

Round 4 named the DSL crate `oya-foundry-trigger-dsl-runtime` and justified with "parses + reads files". Critic flagged the classification as borderline. Round 5 splits per service-map-spec.md §3 honestly:

- `oya-foundry-trigger-dsl-kernel` (layer=`kernel`): JSON AST parser + predicate-registry types + evaluator-policy enums + pure status-transition-graph validator. Zero I/O.
- `oya-foundry-trigger-dsl-runtime` (layer=`runtime`): file-reader (resolves `file:` URIs against git-toplevel) + ci-evidence fetcher + composite evaluator that consumes the kernel's pure logic.

The kernel is reusable (other lanes can validate trigger ASTs in tests without filesystem). The runtime is the lane-consumed crate. This is a Linus-grade split — not a borderline classification. Both crates are W0 Step 4 deliverables.

---

## 4. `oya-check-dependency-seam-discipline` Lane + CI-only carve-out (unchanged structure; ADR-0093 status pinned)

### Lane naming precedent (unchanged)

`oya-check-dependency-seam-discipline` (runner; layer=`runtime`). Companion kernel `oya-foundry-fitness-dependency-seam-kernel` (deferred). ADR-0092 codifies split.

### Composite scope — 8 sub-checks (unchanged from R4)

1. Seam check (per §3.D).
2. Ledger-coverage check.
3. Ledger freshness check.
4. Vendor-residue sub-check.
5. CVE-watch sub-check.
6. Ledger-review-contract sub-check.
7. **Layer-metadata sub-check** (per Codex Concern 1; closes Critic CRITICAL #1 day 0).
8. **Ledger-transition-monotonicity sub-check** (comparator pinned in §2 to PR-parent merge-base).

Report schema enumerates all 8 in `.omc/reports/dependency-seam-latest.json`. Composite emission + severity ramp (Day 0→30 `report-only`; Day 30+ `error`) + implementation sketch (`oya-dev-cli gate validate dependency-seam --mode=composite [--offline] [--severity=...]`): unchanged from R3.

### CI-only-read-side carve-out (ADR-0093 — Status pin; Critic MINOR fix)

Unchanged scope from R4 §4: `gh api` / `gh pr view` read-only, `GITHUB_ACTIONS=true` gate, same-PR roster-and-ledger guard, CODEOWNERS rule on `role-roster.json`.

**ADR-0093 status pin (round-5 disambiguation).** Round 4 §10 Step 4 listed ADR-0093 alongside ADR-0091 and ADR-0092 as "ADR authoring" deliverable; round 4 also said "Proposed → Accepted W0 Step 6." Round 5 picks **Step 6, accept** as the single landing point: ADR-0093 is **drafted in Step 4** (Proposed status) and **accepted in Step 6** when CODEOWNERS rule + same-PR guard fixture land together with the role-roster commit. Step 4 outputs the draft; Step 6 outputs the Accepted ADR via grit-mediated status amendment.

---

## 5. Replacement-Target Registry (unchanged from R3)

Tiered GREEN/AMBER/RED bands; per-dep registry; NIH honesty in ADR-0091.

---

## 6. Phase-Out Timeline + Walk-away protocol (unchanged from R4 — grit-mediated)

`grit claim → edit → grit done`. No `gh pr create`. No `emit rollback-pr`. Walk-away: `replacement_trigger = {"never": true}` + ADR-0091 §Decisions amendment via grit + rollback PR via grit.

---

## 7. Cloud-Native Deployment Plan + ReadinessGate flip condition (Architect Note 2)

Build target unchanged: musl x86_64 + aarch64; `FROM scratch`; no openssl; ≤30 MB stripped.

### Runtime contract — unchanged table from R4 (bind, port, probes, SIGTERM, distroless, cold-start, stateless)

### `ReadinessGate` flip condition (Architect Note 2 fix)

The `/readyz` probe consults a `ReadinessGate` value the catalog loader manipulates. Round 4 §19.3 said "consults a `ReadinessGate` value the catalog loader flips to `Ready` once init completes" — Architect flagged the flip condition as not pinned.

**Round 5 pin:**

```
ReadinessGate ::= Initializing | Ready | Draining
```

**Initializing → Ready** when ALL of:
1. **Catalog loaded.** `oya-foundry-catalog-runtime::load()` returns Ok and the parsed catalog version matches the binary's expected schema.
2. **Downstream count = 0 in W0.** The ops binary has zero outbound dependencies in W0 (no eventing, no ontology, no external HTTP). Future versions extend this with downstream-reachability probes; W0 explicitly asserts the empty set.
3. **Bind succeeded.** Listener is bound to `PORT`; first `accept()` may not yet have occurred (the flip can happen before traffic).

**Ready → Draining** when SIGTERM received (drain timer starts; `/readyz` returns 503 to drop from k8s endpoints; existing in-flight requests continue ≤30s).

**Draining → (exit)** when in-flight count reaches 0 OR drain budget expires.

W0 acceptance test (in §19.3 + §10 Step 8): post-startup `/readyz` returns 200 within ≤100ms of process exec (cold-start budget) AND returns 503 within ≤500ms of SIGTERM.

Cold-start harness (`oya-bench-cold-start-harness`, `CLOCK_MONOTONIC`), throughput baseline, musl-runtime-smoke, parity preservation: unchanged from R3.

---

## 8. ADR-0091 Outline (round-5 updates)

**Path:** `docs/decisions/ADR-0091-workspace-dependency-seam-debt-ledger-phaseout.md`
**Status:** Proposed → Accepted on Architect + Critic + codex consensus (round-5 review pending).

**Decision (round-5 deltas in *italics*):**

1-9 from R4 unchanged.
10. DSL evaluator policies (`staleness_policy`, `evidence_immutability_policy`, `pointer_missing_policy`, `monotonic_transitions_only`) are schema-defaulted at registry top-level; per-row override permitted; `never` trigger short-circuits all 4 policies. *(round-5 — inheritance pattern replaces per-row mandate)*
11. `gh api` / `gh pr view` permitted CI-only under `GITHUB_ACTIONS=true` (ADR-0093 carve-out).
12. Cloud-native code contract for `oya-ops-workspace-shell-runtime` (§7 + §19) including `ReadinessGate` flip condition.
13. *(NEW round-5)* `oya-http-sse-domain` splits into `oya-http-sse-kernel` (pure types; W0 Step 1) + future `*-runtime` deferred; ADR-0094 codifies.
14. *(NEW round-5)* `oya-foundry-trigger-dsl-*` ships as kernel/runtime pair (pure parser + I/O evaluator).
15. *(NEW round-5)* `monotonicity-disabled` warn-level lane finding documented when a row sets `monotonic_transitions_only: false`.

**Drivers:** unchanged from R3.

**Alternatives considered:** unchanged from R3 + *"SSE adapter-classification (path a) and DAG carve-out (path c) considered and rejected; path (b) kernel/runtime split chosen — see ADR-0094."*

**Why chosen:** unchanged + *"Path (b) preserves the canonical DAG invariant the entire plan defends; inheritance pattern preserves 4-policy guarantee without per-row verbosity (matches example trigger exactly); ReadinessGate flip pin makes /readyz semantics testable."*

**Consequences:** unchanged + *"`oya-http-sse-kernel` is reusable across future transports (e.g., std-adapter, custom-transport) without circular layer dependency; trigger-DSL kernel reusable in lane unit tests; `monotonicity-disabled` warnings audit-trail every disabled row to the quarterly review."*

**§Risks — empirical base-rate paragraph (unchanged from R3).**

**§Walk-away — grit-mediated steps:** §6 verbatim.

**Follow-ups:** W0 Steps 0-8 (§10); per-replacement ralplan; quarterly review; CVE watch; ADR-0092 (lane naming); ADR-0093 (CI carve-out; status Accepted at Step 6); **ADR-0094 (SSE kernel/runtime split rationale)**.

---

## 9. Risk Register (round-5 additions)

R4 risks all carried forward. R5 adds:

- **SSE split misclassification** (Very Low/Medium) → §3.B empirical evidence + ADR-0094 future-runtime-creation criteria. DRI: `_seam_lane`.
- **Inheritance defaults silently override per-row intent** (Low/Low) → §2 parser emits structured warning per override; quarterly review reads warnings. DRI: `_w0`.
- **`monotonicity-disabled` warn ignored** (Low/Medium) → §2 quarterly review must justify in writing or revert. DRI: `_ledger`.
- **DSL kernel/runtime split inflates W0 effort** (Low/Low) → both crates net-new; kernel is ~150 LoC pure logic, runtime is the filesystem/CI evidence reader; effort within Step 4 M+ band. DRI: `_w0`.

Inherited R4 risks: trigger-DSL-stale/mutable, status-ratchet-break, middleware-adapter-leak, gh-outside-CI, role-roster-self-promotion, ops-binary-not-routable. Plus R3 risks: F1/F2/F3, refactor-stalls-release, ledger-drift, distroless-TLS, musl-smoke-fail, T0+12-drift, parity-fail, lane-budget-overrun, serde_json-CVE-pre-replacement, ledger-self-heal-failure, trigger-DSL-parser-regression, dep-new-label-spoofing.

---

## 10. Steps (W0 — round 5)

### Step 0 → Step 1 dependency edge (Critic MINOR fix)

**Edge: Step 0 → Step 1 is hard sequential.** Step 1 renames crates (changing their identity); Step 0 authors `[package.metadata.oyatie.layer]` blocks under the **pre-rename names**. Step 0 must commit and land BEFORE Step 1 begins, otherwise Step 1's rename invalidates Step 0's metadata. Steps 2-7 have softer dependencies (see effort summary).

### Step 0 — Layer-metadata bootstrap across workspace (S; Architect)

- Walk `cargo metadata --no-deps`; for every member missing `[package.metadata.oyatie.layer]`, author block with layer ∈ `{kernel, runtime, adapter, api, app}` per crate's responsibility.
- **Tool:** `oya-dev-cli gate emit layer-metadata-bootstrap` (idempotent; refuses to overwrite).
- **Outputs:** modified `crates/*/Cargo.toml` (added blocks only); `.omc/reports/layer-metadata-bootstrap-{sha}.json`.
- **Verification:** `oya-check-dependency-seam-discipline --sub-check=layer-metadata` green on `main`; no member declares `layer: domain`.

### Step 1 — SSE kernel split + middleware-runtime refactor + crate renames + adapter-dep removal (L+; Architect) — REVISED per Critic CRITICAL #1 and MAJOR #4

**Cargo.toml file edits (exact enumeration — Critic MAJOR #4 fix).** The atomic PR for Step 1 modifies the following files. Verified against on-disk state at commit `4d6bf91b51671e37076b2a8c15f0f950cdb3ba56`:

1. **`Cargo.toml` (root workspace) line 383** — replace `"crates/oya-http-sse-domain"` → `"crates/oya-http-sse-kernel"`.
2. **`Cargo.toml` (root workspace) line 385** — replace `"crates/oya-http-tenant-middleware-domain"` → `"crates/oya-http-tenant-middleware-runtime"`.
3. **`Cargo.toml` (root workspace) line 386** — replace `"crates/oya-http-deadline-middleware-domain"` → `"crates/oya-http-deadline-middleware-runtime"`.
4. **`Cargo.toml` (root workspace) line 387** — replace `"crates/oya-http-telemetry-middleware-domain"` → `"crates/oya-http-telemetry-middleware-runtime"`.
5. **`crates/oya-http-runtime-hyper-adapter/Cargo.toml` line 12** — replace `oya-http-sse-domain = { path = "../oya-http-sse-domain" }` → `oya-http-sse-kernel = { path = "../oya-http-sse-kernel" }`. (Already audited in §3.B above.)
6. **`crates/oya-http-sse-kernel/Cargo.toml`** (renamed from `oya-http-sse-domain/Cargo.toml`) — update `[package].name` to `oya-http-sse-kernel`, `[lib].name` to `oya_http_sse_kernel`, add `[package.metadata.oyatie.layer] = "kernel"` block.
7. **`crates/oya-http-{deadline,telemetry,tenant}-middleware-runtime/Cargo.toml`** (three files; renamed from `*-domain`) — update `[package].name` and `[lib].name`; drop `bytes` workspace dep; drop `oya-http-runtime-hyper-adapter` path dep; add `oya-http-router-kernel`/`oya-http-middleware-kernel` path deps (kernel newtypes only); add `[package.metadata.oyatie.layer] = "runtime"` block.
8. **Any other Cargo.toml referencing `oya-http-sse-domain`** — verified via `grep -l "oya-http-sse-domain" Cargo.toml crates/*/Cargo.toml` at commit SHA above: returns exactly THREE files (root, hyper-adapter, sse itself). No other consumer. Step 1 atomically updates all three.
9. **`Cargo.lock`** — invalidated by package rename (cargo regenerates on next `cargo build`/`cargo check`). Step 1 includes `cargo update -p oya-http-sse-domain --precise` removal + `cargo check --workspace` to regenerate the lockfile; the regenerated `Cargo.lock` is committed in the same atomic PR.

**Directory renames** (executed via `git mv` to preserve history):
- `crates/oya-http-sse-domain` → `crates/oya-http-sse-kernel`
- `crates/oya-http-tenant-middleware-domain` → `crates/oya-http-tenant-middleware-runtime`
- `crates/oya-http-deadline-middleware-domain` → `crates/oya-http-deadline-middleware-runtime`
- `crates/oya-http-telemetry-middleware-domain` → `crates/oya-http-telemetry-middleware-runtime`

**Newtypes added to `oya-http-router-kernel`:** `HttpHeaderMap`, `HttpHeaderName`, `HttpHeaderValue` (plus the existing `HttpRequest`/`HttpResponse`/`HttpBody`/`HttpBodyStream` from R3).

**Tool:** `oya-dev-cli gate emit ops-workspace-shell-baseline --commit=<pre-refactor-sha>`; `oya-dev-cli gate emit middleware-adapter-import-audit --commit=<pre-refactor-sha>`; `oya-dev-cli gate emit sse-classification-audit` (NEW — verifies post-rename adapter→kernel direction).

**Outputs:** `.omc/reports/ops-workspace-shell-baseline-{pre-refactor-sha}.json`; `.omc/reports/middleware-adapter-import-audit-{pre-refactor-sha}.json`; `.omc/reports/sse-classification-audit-{post-refactor-sha}.json`; `.omc/reports/dependency-seam-baseline-{pre-refactor-sha}.json`.

**Verification:**
- Zero `bytes::Bytes` / `hyper::*` / `http_body::Body` imports outside `oya-http-runtime-hyper-adapter`.
- Zero `oya-http-runtime-hyper-adapter` dep in the 3 middleware-runtime crates' `Cargo.toml`.
- `oya-http-sse-kernel` declares `layer = "kernel"`; hyper-adapter declares `layer = "adapter"`; the adapter→kernel edge passes the canonical DAG sub-check.
- `(new-baseline)/baseline >= -0.02` for p50 + p99 (post-refactor parity).
- All four renamed crates declare the correct `[package.metadata.oyatie.layer]` value.

### Step 2 — Author `oya-check-dependency-seam-discipline` composite lane (L; Critic) — unchanged from R4

Lane crate + policy doc + CI job + 8 sub-checks. 6 fixture crates (kernel-imports-hyper, build-dep, dev-dep, vendor-residue, missing-layer, backward-transition).

### Step 3 — Ledger commit + generator + self-heal + object-map envelope + inheritance defaults (L; Architect+Critic) — REVISED per codex C3

New files unchanged from R4. **Schema additions per round 5:** `default_evaluator_policies` block required at registry top-level; rows inherit; per-row `evaluator_policies` block optional. Self-heal fixture also covers inheritance: delete a row's optional `evaluator_policies` override; result is bit-for-bit identical when defaults match the overridden values.

### Step 4 — `oya-foundry-trigger-dsl-{kernel,runtime}` + ADR-0091 + ADR-0092 + ADR-0093 (draft) + ADR-0094 (M+; Architect+Critic+codex) — REVISED per Critic MINOR

- Two crates net-new: `oya-foundry-trigger-dsl-kernel` (layer=`kernel`; pure AST parser + policy enum + status-graph validator); `oya-foundry-trigger-dsl-runtime` (layer=`runtime`; file resolution + CI evidence fetch + composite evaluator).
- **ADRs drafted in this step (status = Proposed):** `ADR-0091`, `ADR-0092`, `ADR-0093`, `ADR-0094`.
- **ADR-0093 status pin:** drafted (Proposed) here; accepted at Step 6 (when CODEOWNERS rule + same-PR guard land together).
- **Outputs:** both crates; fixtures covering: 11 seed parse; 5 malformed reject; stale-source; immutable-evidence; missing-pointer; monotonic-transition rejection; `{"never": true}` returns `disarmed`; `monotonic_transitions_only: false` emits `monotonicity-disabled` warn; inheritance fixture (row inherits all 4 defaults, parser emits no warning); inheritance-override fixture (row sets `staleness_policy: "warn-on-stale"`, parser emits structured override warning).
- **Verification:** all fixtures green; ADR-0091 §Decisions cites inheritance + `{"never": true}` short-circuit + carve-out; ADRs 0091-0094 indexed.

### Step 5 — Distroless smoke + `oya-bench-cold-start-harness` + `oya-check-distroless-deployment-bar` (L; DRI+Critic) — unchanged from R4

### Step 6 — `dri.json` + `role-roster.json` + CODEOWNERS + **ADR-0093 acceptance** (S; Critic+DRI) — REVISED per Critic MINOR (ADR-0093 status pin)

- Outputs: `.omc/registries/{dri,role-roster}.json` + schemas; updated `CODEOWNERS` entry: `.omc/registries/role-roster.json @jason931225`; **ADR-0093 status amended from Proposed → Accepted via grit-mediated edit** (`grit claim → edit ADR-0093 §Status → grit done`).
- **Verification:** raci-coverage green; CODEOWNERS rule active; same-PR self-promotion fixture → lane fail; ADR-0093 indexed with Status=Accepted.

### Step 7 — Flip lane to `error` after 30-day soak + INDEX.md row + quarterly template (M; Architect) — REVISED per Critic MAJOR #3 (SHA-anchored baseline)

**Lane-count baseline (round 5 — Critic MAJOR #3 fix).**

```
INDEX.md @ 4d6bf91b51671e37076b2a8c15f0f950cdb3ba56 = 64 lanes
    (30 BLOCKER + 28 HIGH + 6 MED + 0 LOW;
     verified via grep '^| [a-z]' .omc/fitness-lanes/INDEX.md | wc -l
     against the SHA-pinned content at commit 4d6bf91)

Net new lanes in this plan = 3:
    1. oya-check-dependency-seam-discipline (composite; folds 8 sub-checks)
    2. oya-check-replacement-parity
    3. oya-check-distroless-deployment-bar

Expected post-W0 lane count = 67 (30 + 3 BLOCKER soak; HIGH/MED unchanged).
```

If `INDEX.md` drifts between round-5 emit and W0 Step 7 merge, Step 7 re-anchors against the new merge-base SHA and recomputes `expected = <new-baseline> + 3`. The composite lane sub-check `fitness-lane-index-coverage` validates this delta against the SHA-anchored emit metadata.

- **Tool:** `oya-dev-cli gate validate fitness-lane-index --baseline-sha 4d6bf91b51671e37076b2a8c15f0f950cdb3ba56`.
- **Outputs:** updated `.omc/fitness-lanes/INDEX.md` to **67 lanes**; `.omc/registries/tech-debt-ledger-review-template.md`.
- **Verification:** composite at `error`, green on `main`; review-contract sub-check arms; first quarterly review scheduled.

### Step 8 — Ops-binary cloud-native code-changes (M; DRI) — unchanged from R4

See §19. `ReadinessGate` flip condition per §7 (round 5 pin).

**Effort summary:** 1S + 1L+ + 1L + 1L + 1M+ + 1L + 1S + 1M + 1M ≈ **5-6 engineer-months elapsed** (round-5 unchanged from R4 — kernel/runtime DSL split fits in the M+ band; SSE rename is a pure file move + lockfile regen, no LoC delta).

**Step dependency edges:**

```
Step 0 ───→ Step 1 ───→ Step 2 ───→ Step 7
              │            │
              ├──→ Step 3  │
              ├──→ Step 4 ─┤
              ├──→ Step 5  │
              ├──→ Step 6 ─┴──→ ADR-0093 Accepted
              └──→ Step 8
```

Step 0 is the only hard predecessor of Step 1. Steps 2-8 fan out from Step 1; Step 7 depends on Steps 2-6 (lane authored + ledger committed + ADRs accepted + roster committed); Step 7 may proceed once Step 6 lands ADR-0093 Accepted.

---

## 11. Acceptance Criteria (round-5)

**Lane-count baseline (SHA-anchored per Critic MAJOR #3):** `INDEX.md @ 4d6bf91b = 64 lanes; net new = 3; expected = 67`.

- [ ] `tech-debt-ledger.json` committed; 11 entries as object map keyed by `dep_name`; **`default_evaluator_policies` block present at registry top-level** with all 4 fields; schema validated.
- [ ] **Per-row `evaluator_policies` block optional**; inheritance fixture passes; override fixture emits structured warning.
- [ ] **Example trigger row in plan §2 matches schema contract (codex C3 fix)**; no policy mismatch between spec text and example.
- [ ] Status transition graph monotonic; backward transitions rejected by `ledger-transition-monotonicity` sub-check; comparator pinned to **`git merge-base origin/main HEAD`**.
- [ ] **`monotonic_transitions_only: false` per-row emits warn-level lane finding `monotonicity-disabled: <dep>`** (Architect Note 1).
- [ ] `{"never": true}` recognized as `replacement_trigger`; **short-circuits all 4 policies**; permitted only on `keep` / `replacement-attempted-abandoned`.
- [ ] `dri.json` + `role-roster.json` committed; named `primary` everywhere; bootstrap = jason931225.
- [ ] CODEOWNERS rule for `.omc/registries/role-roster.json`; same-PR self-promotion guard fixture → lane fail.
- [ ] `crates/oya-check-dependency-seam-discipline/` exists, `layer = "runtime"`; **8 sub-checks** active.
- [ ] `gh` invocation gated by `GITHUB_ACTIONS=true`; ADR-0093 indexed; **Status: Accepted at Step 6** (not Step 4 draft).
- [ ] `oya-dev-cli` subcommands authored: `gate validate dependency-seam [--offline]`, `gate emit tech-debt-ledger [--self-heal]`, `gate emit layer-metadata-bootstrap`, `gate validate ledger-coverage`, `gate emit ops-workspace-shell-baseline`, `gate emit middleware-adapter-import-audit`, **`gate emit sse-classification-audit`** (NEW round 5), **`gate emit current-date`** (NEW round 5 — single emit-tool for `current_date()` literals; Critic MINOR). REMOVED: `gate emit rollback-pr`.
- [ ] **`oya-foundry-trigger-dsl-kernel`** (layer=`kernel`) ships W0; pure AST parser + policy enums + status-graph validator.
- [ ] **`oya-foundry-trigger-dsl-runtime`** (layer=`runtime`) ships W0; file resolution + CI evidence fetcher + composite evaluator.
- [ ] Ledger generator self-heals (SHA-256 bit-for-bit roundtrip; inheritance defaults preserved).
- [ ] ADR-0091 written + indexed + accepted; includes §DSL-policies (inheritance), §CI-carve-out (status pin), §Cloud-native (`ReadinessGate`), §SSE-split.
- [ ] ADR-0092 written + indexed (lane runner vs kernel naming).
- [ ] **ADR-0093 written; Status: Accepted at Step 6** (not Step 4).
- [ ] **ADR-0094 written + indexed (SSE kernel/runtime split rationale per §3.B)**.
- [ ] Kernels (`oya-http-router-kernel`, `oya-http-middleware-kernel`, **`oya-http-sse-kernel`**) zero `hyper::`/`bytes::`/`http_body::` use-paths/pub-signatures.
- [ ] **`oya-http-sse-domain` renamed to `oya-http-sse-kernel`** (Critic CRITICAL #1 fix; path b chosen and justified in §3.B).
- [ ] **`oya-http-{deadline,telemetry,tenant}-middleware-runtime` renamed from `*-domain`**; all declare `layer = "runtime"`; zero `oya-http-runtime-hyper-adapter` dep; zero `bytes` dep.
- [ ] **`oya-http-router-kernel` exposes `HttpHeaderMap` + `HttpHeaderName` + `HttpHeaderValue`** in addition to R3 newtypes.
- [ ] **Cargo.toml file edits enumerated per Step 1 (Critic MAJOR #4 fix):** root workspace lines 383, 385, 386, 387; hyper-adapter line 12; sse-kernel Cargo.toml; 3 middleware-runtime Cargo.tomls; Cargo.lock regenerated. All atomic in same PR.
- [ ] Only `oya-http-runtime-hyper-adapter` declares hyper-family deps; `bytes` only there.
- [ ] `oya-bench-cold-start-harness` crate exists; `CLOCK_MONOTONIC`.
- [ ] **`oya-ops-workspace-shell` ships cloud-native (§7 + §19)** with `ReadinessGate` flip pinned (Architect Note 2).
- [ ] Throughput baseline at pre-refactor SHA; ≤2% regression.
- [ ] `oya-check-active-artifact-contract` green.
- [ ] Every workspace crate declares `[package.metadata.oyatie.layer]` ∈ `{kernel, runtime, adapter, api, app}`; NO `domain`; **NO `*-cgroup-*` hedged row** (Critic MAJOR #2 fix — see §18).
- [ ] No `[workspace.dependencies]` removals; ADR-0090 not superseded.
- [ ] Quarterly review template; CVE-watch active.
- [ ] **Net new top-level CI lanes = 3 (SHA-anchored baseline 64 → 67)**; BLOCKER 30 → 33.
- [ ] §15 bypass-vector mitigations all wired.
- [ ] Walk-away: grit-mediated; `replacement-attempted-abandoned` enum landed.

---

## 12. Out of Scope (unchanged from R3/R4)

Removing hyper/tokio/serde/etc. from `[workspace.dependencies]`; W1-W5 replacements; rustls; architecture changes; modifying ADR-0090; changing 5 product release plans.

**Round 5 additional out-of-scope:** authoring `oya-http-sse-runtime` (deferred per ADR-0094 until orchestration logic actually ships).

---

## 13. Open Questions (Architect Note 3)

Round 5 closes round-4's `oya-http-sse-runtime` audit question by adopting path (b) split — that crate is not authored in W0. The remaining open question and its resolution mechanism:

- [ ] **Whether to author `oya-http-sse-runtime` in W1 if reconnect/keepalive logic is requested** — *Why it matters:* current SSE kernel is pure serializer; client reconnect/Last-Event-ID resume orchestration would be runtime-layer work. **Resolution mechanism:** when W1 sprint planning queues SSE reconnect work, the council-architecture role files an ADR amendment proposing the runtime crate; the proposal MUST cite the new orchestration responsibility and pass the seam-discipline lane in CI. No spontaneous creation; ADR-amendment-gated.

Persisted to `.omc/plans/open-questions.md` (append-only).

---

## 14. Why Review-Ready (round-5 cross-walk)

### Critic CRITICAL #1 — sse-domain rename creates adapter→runtime edge → **CLOSED via path (b)**

§3.B chose path (b) with empirical justification: `oya-http-sse-domain/Cargo.toml` empty deps + `src/lib.rs:1-12` declares "pure std-only" + `service-map-spec.md` §3 `kernel` = "Pure value-object; no I/O" matches. Rename target updated `oya-http-sse-kernel`. §18.A audit updated; §3.D import-direction diagram updated; §10 Step 1 enumerates the Cargo.toml edits with line numbers and SHA. ADR-0094 codifies. Layer-metadata sub-check now passes day 0 of W0 — invariant preserved.

### Critic MAJOR #2 — dead `oya-runtime-cgroup-runtime` hedged row → **CLOSED via removal**

§18.A row 676 removed. `find crates -maxdepth 1 -name '*cgroup*'` returns zero matches at commit `4d6bf91b`. No such crate exists; no rename needed; no hedge in canonical audit. See §18.A round-5 table.

### Critic MAJOR #3 — lane-count baseline not SHA-anchored → **CLOSED via SHA pin**

§11 + §16 anchor: `INDEX.md @ 4d6bf91b51671e37076b2a8c15f0f950cdb3ba56 = 64 lanes; net new = 3; expected = 67`. §10 Step 7 `oya-dev-cli gate validate fitness-lane-index --baseline-sha 4d6bf91b...` rechecks at merge-base if INDEX.md drifts.

### Critic MAJOR #4 — cross-crate Cargo.toml impacts not enumerated → **CLOSED via §10 Step 1 explicit list**

§10 Step 1 lists every Cargo.toml file edit with line numbers, verified against `grep -l "oya-http-sse-domain" Cargo.toml crates/*/Cargo.toml` at commit `4d6bf91b`: returns exactly 3 files (root, hyper-adapter, sse-domain itself). 4 middleware/sse workspace member entries at lines 383, 385, 386, 387. 1 dep edit at hyper-adapter line 12. 4 directory renames via `git mv`. `Cargo.lock` regenerated. All atomic.

### Codex C3 — example trigger violates 4-policy rule → **CLOSED via inheritance pattern (path b)**

§2 schema introduces `default_evaluator_policies` at registry top-level; rows inherit; per-row override optional. Example trigger row in §2 now matches contract via inheritance — zero per-row policy fields, all 4 guarantees preserved. Self-heal fixture (Step 3) covers inheritance. Override-warning fixture (Step 4) audit-trails any per-row override.

### Architect Note 1 — `monotonic_transitions_only: false` audit treatment → **CLOSED**

§2 policy table: when set `false`, row emits warn-level lane finding `monotonicity-disabled: <dep_name>`. §11 acceptance criterion; §10 Step 4 fixture.

### Architect Note 2 — `ReadinessGate` flip condition → **CLOSED**

§7: `Initializing → Ready` when catalog-loaded + downstream-count=0 (W0 explicit) + bind-succeeded. `Ready → Draining` on SIGTERM. Acceptance test in §10 Step 8.

### Architect Note 3 — §13 open-question resolution mechanism → **CLOSED**

§13 R4 question (`oya-http-sse-runtime` audit) closed by R5 path (b). Remaining question (future W1 runtime authoring) has explicit resolution mechanism: ADR-amendment-gated; council-architecture files; seam-discipline lane validates.

### Critic MINORs → **CLOSED**

- `current_date()` literal emit-tool cited: **`oya-dev-cli emit current-date`** (§11 acceptance + plan header).
- `oya-foundry-trigger-dsl` naming: **split into kernel/runtime pair** (§3.F + §10 Step 4).
- Lockfile invalidation: **Step 1 enumerates `cargo update -p oya-http-sse-domain --precise` removal + `cargo check --workspace` + commit regenerated `Cargo.lock`** (§10 Step 1 item 9).
- Workspace member list root Cargo.toml: **lines 383, 385, 386, 387 enumerated** (§10 Step 1 items 1-4).
- `data_source` URI relative-paths: **resolved against `git rev-parse --show-toplevel`; NOT cargo-workspace-root; NOT lane-CWD** (§2 grammar).
- `ledger-transition-monotonicity` comparator: **PR-parent merge-base `git merge-base origin/main HEAD`** (§2 + Step 4 fixture).
- `never` trigger policy bypass: **YES, explicitly short-circuits all 4 policies** (§2).
- §2 cross-row reference enumeration: **exactly one: `dependent_wave_status`** (§2 predicate registry table).
- §10 Step 0 → Step 1 dependency edge: **hard sequential, made explicit** (§10 header).
- ADR-0093 status: **drafted Step 4 (Proposed); accepted Step 6** (single landing point; §4 + §10).

### R4 codex C1+C2+C4+C5+C6 + Critic + Architect-from-R4 → unchanged-and-carried

All resolutions from R4 §14 preserved verbatim.

---

## 15. Bypass-Vector Enumeration + Countermeasures (round 5)

1-22 from R4 unchanged. Round 5 adds:

23. **SSE upstream-edge regression (adapter→runtime via future SSE-runtime authored without ADR amendment)** → §13 ADR-amendment-gated mechanism + §3 seam-discipline lane subcheck flags any new adapter→runtime edge as BLOCKER.
24. **Inheritance defaults silently changed at registry top-level masking per-row arming** → §2 parser emits structured warning per registry-default change between PR-parent and HEAD; ledger-transition-monotonicity sub-check audits the default block as well as per-row status.
25. **`monotonicity-disabled` warn ignored over quarters** → §10 Step 7 quarterly review template includes a "review every `monotonicity-disabled` finding" line item; un-justified after 2 quarters → lane fail.

---

## 16. Lane Consolidation Map (round 5)

3 top-level CI lanes; composite seam lane folds 8 sub-checks (unchanged from R4).

**Lane budget impact (SHA-anchored):** **`INDEX.md @ 4d6bf91b = 64 lanes`** + 3 new = **67 total**. 30 BLOCKER + 3 new = **33 BLOCKER post-soak**.

---

## 17. DRI Registry Seed (unchanged from R4)

Round-3 entries preserved. R4 added `_role_roster`. R5 adds:

- `_sse_split` (CODEOWNERS owner for `crates/oya-http-sse-kernel/`; reviewer on any future `oya-http-sse-runtime` proposal) — bootstrap `primary = jason931225`.

Acceptance gate per R3 retained.

---

## 18. Layer-Enum Alignment Audit (round 5 — Critic MAJOR #2 fix; Critic CRITICAL #1 fix)

Canonical: `{kernel, runtime, adapter, api, app}`. No `domain`.

### §18.A — Per-crate layer mapping (round 5; NO HEDGED ROWS)

| Crate | Authored or referenced? | `[package.metadata.oyatie.layer]` | Rationale |
|---|---|---|---|
| `oya-http-router-kernel` | referenced | `kernel` | Pure value-objects (HttpRequest/HttpResponse/HttpBody/HttpHeaderMap newtypes). |
| `oya-http-middleware-kernel` | referenced | `kernel` | Middleware trait definitions; no I/O. |
| **`oya-http-sse-kernel`** (renamed from `oya-http-sse-domain` per §3.B path b) | renamed in Step 1 | `kernel` | Pure SSE framing types + serializer; `[dependencies]` empty; std-only per `src/lib.rs:1-12` doc. Replaces R4's `*-runtime` classification (which created adapter→runtime upstream edge — see Critic CRITICAL #1). |
| `oya-http-deadline-middleware-runtime` (renamed) | renamed in Step 1 | `runtime` | Deadline orchestration. |
| `oya-http-telemetry-middleware-runtime` (renamed) | renamed in Step 1 | `runtime` | Telemetry orchestration. |
| `oya-http-tenant-middleware-runtime` (renamed) | renamed in Step 1 | `runtime` | Tenant resolution orchestration. |
| `oya-http-runtime-hyper-adapter` | referenced | `adapter` | Provider-specific I/O (hyper). |
| `oya-ops-workspace-shell-runtime` | referenced; refactored Step 8 | `runtime` | Binary's runtime shell. |
| `oya-check-dependency-seam-discipline` | authored Step 2 | `runtime` | Lane runner. |
| **`oya-foundry-trigger-dsl-kernel`** (NEW; split per §3.F) | authored Step 4 | `kernel` | Pure AST parser + policy enums + status-graph validator. |
| **`oya-foundry-trigger-dsl-runtime`** (NEW; split per §3.F) | authored Step 4 | `runtime` | File resolution + CI evidence fetcher + composite evaluator. |
| `oya-bench-cold-start-harness` | authored Step 5 | `runtime` | Subprocess + `CLOCK_MONOTONIC`. |
| `oya-check-distroless-deployment-bar` | authored Step 5 | `runtime` | Lane runner. |
| `oya-check-replacement-parity` | authored W1+ | `runtime` | Lane runner. |
| `oya-dev-cli` | referenced | `app` | Top-level CLI binary. |
| `oya-foundry-fitness-dependency-seam-kernel` | deferred (post-W0) | `kernel` | Pure policy value-object. |

**Removed from R4 §18.A audit (Critic MAJOR #2 fix):** the hedged row `oya-runtime-cgroup-runtime (renamed from *-domain if pre-existing)`. `find crates -maxdepth 1 -name '*cgroup*'` at commit `4d6bf91b` returns zero matches. No such crate; no row. cgroup probing in `oya-check-distroless-deployment-bar` (§19.5) is in-process via `/sys/fs/cgroup` reads; no dedicated crate needed.

### §18.B — ADR-0092 (unchanged from R3)

### §18.C — Verification (round 5)

`oya-check-dependency-seam-discipline --sub-check=layer-metadata`:
- parses every workspace crate's `[package.metadata.oyatie.layer]`;
- rejects missing (BLOCKER);
- rejects any value outside the canonical 5 (BLOCKER);
- rejects `oya-check-*` with layer ≠ `runtime`;
- rejects `oya-foundry-fitness-*-kernel` with layer ≠ `kernel`;
- **rejects any runtime→adapter edge** (canonical DAG invariant per `service-map-spec.md` §5; reads `cargo metadata --no-deps`, walks each runtime-layer crate's deps, asserts no dep declares `layer = "adapter"`). Note: `adapter→runtime` is the post-Critic-CRITICAL-#1 composition shape (hyper-adapter composes middleware-runtime) and IS allowed per §3.D.

### §18.D — Bootstrap reality

Step 0 + Step 1 + Step 4 explicitly rename + declare layer metadata for every crate the plan touches. The §18.A audit is true post-W0 (gated deliverable, not aspirational).

---

## 19. Code-Change Deliverables — Ops Binary Cloud-Native Upgrade (unchanged structure from R4; §19.3 augmented per Architect Note 2)

### §19.1 — Bind address (unchanged from R4)

### §19.2 — Port env var (unchanged from R4)

### §19.3 — k8s probe routes + `ReadinessGate` flip pinned (Architect Note 2)

- Routes: `/healthz` (always 200 once process bound), `/livez` (always 200 once process bound), `/readyz` (consults `ReadinessGate`).
- **`ReadinessGate` flip condition (pinned per §7):**
  - `Initializing → Ready` when ALL of: (1) `oya-foundry-catalog-runtime::load()` returns Ok + version matches; (2) downstream count = 0 in W0 (no eventing/ontology/external HTTP); (3) listener bound to `PORT`.
  - `Ready → Draining` when SIGTERM received.
  - `Draining → exit` when in-flight = 0 OR 30s budget expires.
- Legacy `/workspace/api/v1/health` retained one release with `X-Deprecated` header.

### §19.4-§19.7 (unchanged from R4)

### Acceptance gate

`oya-check-distroless-deployment-bar --probe-conformance` runs all 7 §19 sub-property checks against the built distroless image; all must pass before Step 8 accepted.

---

## 20. Round-5 Verdict Cross-Walk Summary

| R4 finding | R4 owner | R5 resolution | Plan section |
|---|---|---|---|
| CRITICAL #1: SSE rename → adapter→runtime upstream edge | Critic | Path (b) kernel/runtime split; rename to `oya-http-sse-kernel`; ADR-0094 codifies | §3.B, §10 Step 1, §18.A, §11, ADR-0094 |
| MAJOR #2: dead `*-cgroup-*` hedged row | Critic | Row removed; cgroup probing in-process via `/sys/fs/cgroup` | §18.A, §11 |
| MAJOR #3: lane baseline not SHA-anchored | Critic | `INDEX.md @ 4d6bf91b = 64; net new 3; expected 67` | §11, §16, §10 Step 7 |
| MAJOR #4: cross-crate Cargo.toml impacts not enumerated | Critic | §10 Step 1 lists every file + line number (root 383/385/386/387; hyper-adapter 12; sse-kernel; 3 middleware-runtime; Cargo.lock) | §10 Step 1 |
| C3 example trigger violates 4-policy rule | codex | Inheritance pattern: `default_evaluator_policies` at registry top-level; rows inherit; example now matches | §2, §10 Step 3, §10 Step 4 |
| Architect Note 1: `monotonic_transitions_only: false` audit treatment | Architect | Emits warn `monotonicity-disabled: <dep>`; quarterly review must justify | §2, §10 Step 4 |
| Architect Note 2: `ReadinessGate` flip condition | Architect | `Initializing → Ready` pinned: catalog-loaded + downstreams=0 + bind-succeeded | §7, §19.3 |
| Architect Note 3: §13 open-question resolution mechanism | Architect | ADR-amendment-gated; council-architecture files | §13 |
| MINOR: `current_date()` emit-tool | Critic | `oya-dev-cli emit current-date` cited in plan header + acceptance | header, §11 |
| MINOR: `oya-foundry-trigger-dsl-runtime` naming borderline | Critic | Split into `*-kernel` (pure) + `*-runtime` (I/O) pair | §3.F, §10 Step 4, §18.A |
| MINOR: Lockfile invalidation impact | Critic | §10 Step 1 item 9: `cargo update --precise` removal + `cargo check --workspace` + commit regen | §10 Step 1 |
| MINOR: Workspace member list root Cargo.toml | Critic | Lines 383/385/386/387 enumerated | §10 Step 1 |
| MINOR: `data_source` relative-path resolution | Critic | `git rev-parse --show-toplevel` | §2 |
| MINOR: `ledger-transition-monotonicity` comparator | Critic | `git merge-base origin/main HEAD` | §2 |
| MINOR: `never` trigger policy bypass | Critic | Explicit YES; short-circuits all 4 policies | §2 |
| MINOR: §2 cross-row reference enumeration | Critic | Exactly one: `dependent_wave_status` | §2 |
| MINOR: §10 Step 0 → Step 1 dependency edge | Critic | Hard sequential, explicit | §10 header |
| MINOR: ADR-0093 status (Step 4 vs Step 6) | Critic | Drafted Step 4 (Proposed); Accepted Step 6 | §4, §10 Step 4, §10 Step 6 |

---

**Round 5 — FINAL.** Every CRITICAL + MAJOR + MINOR + codex C3 + Architect Note closed. Ready for re-run. If 3-of-3 APPROVE: ship to user as planning artifact. If still not 3-of-3: skill cap reached → present this best version to user per orchestration policy.
