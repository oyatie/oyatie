---
purpose: Auto-backfilled purpose for ralplan-dep-seam-phaseout-round-4.md
---

# RALPLAN — Dependency Seam Discipline + Tech-Debt Ledger + Phase-Out Trajectory (Round 4, Final)

> **Status:** `pending review (round 4 reframed; final)`
> **Authored:** emitted by `oya-dev-cli gate emit tech-debt-ledger` (timestamps via `current_date()` — no literal calendar dates in plan body)
> **Predecessor:** `ralplan-dep-seam-phaseout-round-3.md` (Architect APPROVE-WITH-NOTES R2; Critic APPROVE R3; codex gpt-5.5 ITERATE R3 — 1 CRITICAL + 5 MAJOR)
> **Does NOT supersede:** ADR-0090
> **New ADR:** ADR-0091 (§8); ADR-0092 (§18.B); ADR-0093 (§4 — CI carve-out for grit-mediated repo posture)
> **Round-4 mandate:** close every codex concern; final iteration before re-running 3 reviewers.

---

## 0. Reframe (unchanged)

Ship 5 products on hyper/tokio FIRST; phase out AFTER ontology v1 stable. Plan shape/seams/ledger/replacements NOW so release work does not deepen debt. Eliminated failure mode: *"release work accidentally deepens debt because there's no enforced seam and no ledger."*

---

## 1. RALPLAN-DR Summary (round-4)

### Principles (6 — unchanged)

1. Systematic phase-out > immediate replacement.
2. Conscious debt > accidental debt. Every dep named/layered/owned with a replacement target.
3. Seams = lane crate + `fitness-lanes/*.md` + CI job — not doc paragraphs.
4. Public APIs are wrapper types only. No `hyper::Request`/`tokio::JoinHandle`/`bytes::Bytes` outside `adapter`-layer crates.
5. Phase-out is triggered, not scheduled.
6. Machine-evaluable triggers > judgment calls. `replacement_trigger` = JSON AST; named predicates; CI-queryable; immutability- and staleness-policy explicit.

### Decision Drivers (top 3 — unchanged)

1. **Release velocity** (no backbone rewrite in critical path).
2. **Reversibility** (dep swappable without touching kernel/runtime).
3. **No-silent-regression** (lean-a10).

### Viable Options (≥2; tiered parity bars — unchanged from round 2)

A. **Triggered-incremental + tiered GREEN/AMBER/RED** — Chosen (lowest blast radius; aligns no-silent-regression; bails per-dep).
B. Big-bang at T0 — rejected (regression concentration).
C. Opportunistic — rejected (deepens debt invisibly).
D. Forked-vendor-in-tree — rejected (supply-chain surface doubled).

### Pre-mortem (3 scenarios)

- **F1: Trigger never fires.** Named DRI per row; quarterly movement-tracked review; CVE-acceleration shortcut; T0+12 walk-away.
- **F2: Seam lane bypassed** via `#[allow]`, vendoring, trait-objects, build/dev-deps, `dep:new` label spoofing, role-roster same-PR self-promotion. §15 countermeasures; composite lane; role-gated label (§4); same-PR roster guard; CODEOWNERS on `role-roster.json`.
- **F3: Replacement underperforms parity.** Tiered AMBER + ADR amendment; RED → grit-mediated walk-away (§6).

### Expanded Test Plan

1. **Seam correctness** — fixtures: `hyper::` import from non-allowed crate fails; allowed crate passes; build/dev-dep + trait-object fixtures; middleware-runtime crates importing `oya-http-runtime-hyper-adapter` fail (preferred) or carry ADR-cite exemption.
2. **Ledger ↔ reality** — composite lane walks `cargo metadata --no-deps` + each member's `[dependencies]` + `[build-dependencies]` + `[dev-dependencies]` + root `[workspace.dependencies]`; new dep without entry fails.
3. **Parity benches** — `oya-check-replacement-parity` per replacement; GREEN/AMBER/RED.
4. **Review movement** — `oya-check-debt-ledger-review-contract` sub-check; 3 quarters zero-movement fails.
5. **Distroless + musl smoke** — `oya-check-distroless-deployment-bar` (musl x86_64/aarch64; DNS+getrandom+epoll+cert-bundle; cold-start ≤100ms; SIGTERM ≤30s).
6. **Active-artifact contract** — ledger + DRI + reports + reviews machine-readable per ADR-0089; ledger generator self-heals (§10 Step 3).
7. **Trigger DSL parser + evaluator regression** — `oya-foundry-trigger-dsl-runtime` fixtures cover stale-source, immutable-evidence, monotonic-transition, missing-pointer (§10 Step 4).
8. **Layer-metadata coverage** — every crate declares `[package.metadata.oyatie.layer]` ∈ `{kernel, runtime, adapter, api, app}`; sub-check rejects missing/invalid (§18).
9. **Ops binary cloud-native conformance** — `/healthz`, `/livez`, `/readyz` per k8s; binds `0.0.0.0`/`[::]`; reads `PORT`; SIGTERM drains; distroless builds (§7 + §19).
10. **CI carve-out** — `gh api` read-side only when `GITHUB_ACTIONS=true`; local agent invocation fails fast with grit-redirect (§4).

### Mode

**SHORT** — closes codex round-3 concerns; no new exploration.

---

## 2. Tech-Debt Ledger Spec (revised for codex Concern 3)

### Location
`/registries/cross-cutting/tech-debt-ledger.json` (active artifact per ADR-0089).

### Schema change — `entries` is an **object map**, not an array (codex Concern 3 fix)

Round-3 modeled `entries` as an array, but `dependent_wave_status` referenced `/entries/<dep>/status` — invalid JSON pointer against an array. Round-4 normalizes to an object keyed by `dep_name`:

```json
{
  "$schema": "./schemas/tech-debt-ledger.schema.json",
  "version": "1.1.0",
  "generated_by": "oya-dev-cli gate emit tech-debt-ledger",
  "reviewed_quarterly": true,
  "next_review_due": "<emit-time + P90D>",
  "workspace_member_count_source": "cargo metadata --no-deps (dynamic)",
  "workspace_dependencies_source": "file:Cargo.toml:/workspace/dependencies",
  "entries": {
    "hyper": { "...row..." },
    "tokio": { "...row..." }
  }
}
```

All JSON pointers (`dependent_wave_status` + every cross-row reference) now resolve via `/entries/<dep_name>/status`. The order-stable iteration is provided by `oya-dev-cli gate emit tech-debt-ledger` (lexicographic by key).

### Schema (per row)

```json
{
  "dep_name": "hyper",
  "version_pin": "1",
  "feature_set": ["server", "http1"],
  "transitive_count": 0,
  "category": "http-server | http-client | async-runtime | serde | tracing | utility | crypto | other",
  "allowed_layers": ["adapter:oya-http-runtime-hyper-adapter"],
  "allowed_crates_regex": "^oya-http-runtime-hyper-adapter$",
  "allowed_build_deps_regex": null,
  "allowed_dev_deps_regex": "^oya-.*$",
  "replacement_target": "std-only | workspace-internal | keep",
  "replacement_crate_planned": "oya-http-runtime-std-adapter",
  "replacement_trigger": {
    "all_of": [
      {"predicate": "ontology_v1_production_uptime_sprints", "gte": 2,
       "data_source": "file:/registries/cross-cutting/release-state.json:/ontology/v1/production_uptime_sprints"},
      {"predicate": "p99_within_budget_days", "gte": 14,
       "data_source": "file:/registries/cross-cutting/perf-budget-history.json:/hyper/p99_within_budget_days"},
      {"predicate": "parity_bench_pass", "band": "GREEN_OR_AMBER",
       "data_source": "ci:lean-a-replacement-parity:run:<run-id>:hyper",
       "evidence_immutability_policy": "run-id-required"}
    ],
    "staleness_policy": "fail-on-stale",
    "pointer_missing_policy": "not-yet-armed"
  },
  "cve_acceleration_trigger": {
    "any_of": [
      {"predicate": "tracked_cve_open_cvss_gte", "gte": 7.5, "no_upstream_fix_days_gte": 14,
       "data_source": "file:/registries/cross-cutting/cve-watch.json:/hyper"}
    ],
    "staleness_policy": "warn-on-stale",
    "pointer_missing_policy": "not-yet-armed"
  },
  "dri_handles": {"primary": "jason931225 (user)", "backup": "<TBD>"},
  "status": "active | scheduled | replacement-armed | replacement-armed-by-cve | replaced | keep | replacement-attempted-abandoned",
  "monotonic_transitions_only": true,
  "added_at": "<emit-time>",
  "last_reviewed_at": "<emit-time>",
  "adr_cite": ["ADR-0090", "ADR-0091"],
  "notes": "Canonical HTTP backbone; only the hyper-adapter (layer=adapter) consumes."
}
```

### `data_source` URI grammar (revised — codex Concern 3 immutability fix)

```
data_source := file-uri | ci-uri-run | ci-uri-latest
file-uri      := "file:" <relative-path> ":" <json-pointer>           # RFC 6901 JSON Pointer
ci-uri-run    := "ci:" <lane-id> ":run:" <run-id> ":" <row-id>        # IMMUTABLE — required for status transitions
ci-uri-latest := "ci:" <lane-id> ":latest:" <row-id>                  # MUTABLE — advisory dashboards only; REJECTED for status transitions
```

The `oya-foundry-trigger-dsl-runtime` evaluator rejects `ci-uri-latest` in any predicate that gates a status transition (i.e., any predicate inside `replacement_trigger.all_of` / `replacement_trigger.any_of` / `cve_acceleration_trigger.*`). `latest` is permitted only in advisory `dashboards` block (not modeled in W0).

### Trigger DSL — evaluator semantics (NEW — codex Concern 3 closes operational gap)

The `oya-foundry-trigger-dsl-runtime` crate (renamed from R3's `*-domain` per codex Concern 1 — see §18) ships a parser AND evaluator. Four policies are mandatory per trigger:

| Policy | Values | Default | Behavior |
|---|---|---|---|
| `staleness_policy` | `fail-on-stale` \| `warn-on-stale` \| `best-effort` | `fail-on-stale` | If any `file:` `data_source` has mtime older than the ledger row's `last_reviewed_at`, predicate returns `stale`. `fail-on-stale` → predicate is `false` and emits lane error; `warn-on-stale` → predicate is evaluated and lane logs warn; `best-effort` → predicate is evaluated silently. |
| `evidence_immutability_policy` | `run-id-required` \| `latest-allowed` | `run-id-required` | If `ci-uri-latest` appears in a status-gating predicate while policy is `run-id-required`, parser rejects with structured error. |
| `pointer_missing_policy` | `fail` \| `not-yet-armed` | `not-yet-armed` | If a `file:` data_source resolves the file but the JSON pointer path is absent, `fail` returns predicate=false + lane error; `not-yet-armed` returns predicate=false + lane warn (treats absence as "trigger conditions not yet observed"). |
| `monotonic_transitions_only` | `true` \| `false` (per-row) | `true` | Schema-level guard on `status` field: rejects backward transitions (e.g., `operational → scheduled`, `replaced → active`, `replacement-armed → active`). Forward graph defined below. |

#### Status transition graph (monotonic)

```
active
  ├─→ scheduled
  ├─→ replacement-armed              (T0 + GREEN/AMBER parity)
  ├─→ replacement-armed-by-cve       (CVE shortcut)
  └─→ keep                           (permanent retain via ADR)

scheduled
  ├─→ replacement-armed
  ├─→ replacement-armed-by-cve
  └─→ keep

replacement-armed | replacement-armed-by-cve
  ├─→ replaced                       (work landed)
  └─→ replacement-attempted-abandoned (RED parity or DRI decline; walk-away)

replaced                             (terminal)
keep                                 (terminal; ADR amendment can re-open via fresh ADR)
replacement-attempted-abandoned      (terminal; fresh ADR can re-open)
```

The generator rejects any transition not in this graph. CI sub-check `ledger-transition-monotonicity` (new; folded into `oya-check-dependency-seam-discipline` composite) compares current vs prior committed ledger; backward transition → lane fail.

### Trigger DSL — predicate registry (revised pointer for codex Concern 3 bug fix)

| Predicate name | Data source (URI grammar) | Type |
|---|---|---|
| `ontology_v1_production_uptime_sprints` | `file:/registries/cross-cutting/release-state.json:/ontology/v1/production_uptime_sprints` | int (sprint count) |
| `p99_within_budget_days` | `file:/registries/cross-cutting/perf-budget-history.json:/<dep>/p99_within_budget_days` | int (consecutive days) |
| `parity_bench_pass` | `ci:lean-a-replacement-parity:run:<run-id>:<dep>` | enum {GREEN, AMBER, RED} |
| `tracked_cve_open_cvss_gte` | `file:/registries/cross-cutting/cve-watch.json:/<dep>` | float + days-open |
| `dependent_wave_status` | `file:/registries/cross-cutting/tech-debt-ledger.json:/entries/<dep_name>/status` | enum (status graph above) |
| `never` (new — codex Concern 5) | n/a (constant) | constant `disarmed` |

#### `never` trigger variant (codex Concern 5 fix)

```json
{"replacement_trigger": {"never": true}}
```

DSL registry recognizes `{"never": true}` as the **frozen / walk-away post-condition** trigger. Evaluator always returns `disarmed`. Documented post-walk-away state; never spontaneously transitions. CI sub-check `ledger-coverage` accepts `{"never": true}` only when status ∈ `{keep, replacement-attempted-abandoned}`.

### Seed rows (11 — verified; object-map keys per Concern 3 envelope switch)

Keys: `hyper`, `hyper-util`, `http-body-util`, `tokio`, `bytes`, `tracing`, `tracing-subscriber`, `serde`, `serde_json`, `toml`, `toml_edit`. Per-row `replacement_target` / `replacement_crate_planned` / `status` / full `allowed_crates_regex` carried verbatim from round-3 §2 seed table; only the envelope structure changed (array → object map).

Bootstrap: every row ships `primary = jason931225 (user)`; `backup` may remain `<TBD>`; `primary` may NOT. See §17 + §18.D.

---

## 3. Dependency-Seam Contracts — middleware-layer reclassification (codex Concern 2 fix)

### Layer enum (authority: `.omc/automation/service-map-spec.md` §3)

Canonical: **`{kernel, runtime, adapter, api, app}`**. There is **NO `domain` layer**. Round-3 erroneously used `domain` in §18 and in crate naming (`*-trigger-dsl-domain`). Round-4 corrects everywhere.

### Codex Concern 2 — middleware crates directly consume `oya-http-runtime-hyper-adapter`

Round-3 reported the 3 middleware crates as `runtime`-layer importing only `bytes`. **Codex's deeper audit** found those crates also depend directly on `oya-http-runtime-hyper-adapter`, which means a `runtime`-shaped crate is reaching into adapter-boundary types. Removing only `bytes` does not restore the seam.

#### W0 Step 1 expanded scope (codex Concern 2 fix)

**(a) Per-crate import audit** (output: `.omc/reports/middleware-adapter-import-audit-{sha}.json`):

| Crate | Hyper-family deps observed | Adapter import direction observed | Decision |
|---|---|---|---|
| `oya-http-deadline-middleware-runtime` (renamed from `*-domain`) | `bytes`; depends on `oya-http-runtime-hyper-adapter` | imports `hyper::Request`, `hyper::body::Incoming` types from adapter | **(i) refactor**: consume `HttpRequest`/`HttpBody` newtypes from `oya-http-router-kernel` only; drop `bytes`; drop `oya-http-runtime-hyper-adapter` dep |
| `oya-http-telemetry-middleware-runtime` (renamed) | `bytes`; depends on `oya-http-runtime-hyper-adapter` | imports `hyper::body::Incoming` for size accounting | **(i) refactor**: consume `HttpBody` newtype; drop `bytes`; drop adapter dep |
| `oya-http-tenant-middleware-runtime` (renamed) | `bytes`; depends on `oya-http-runtime-hyper-adapter` | imports `hyper::HeaderMap` | **(i) refactor**: consume `HttpHeaderMap` newtype (added to `oya-http-router-kernel` in Step 1); drop `bytes`; drop adapter dep |

**(b) Decision per crate (chosen across all three):** **(i) refactor** — kernel newtype consumption only. Rejected: (ii) reclassify to `adapter`-layer (would multiply adapter crates with middleware logic which is policy, not I/O); (iii) accept the leak (regresses the seam).

**(c) `oya-http-router-kernel` newtype surface expanded** to include the types the middleware currently reaches for at the adapter:

```rust
// kernel-owned, no hyper/bytes deps:
pub struct HttpHeaderMap { /* opaque BTreeMap<HeaderName, Vec<HeaderValue>> */ }
pub struct HttpHeaderName(/* SmolStr */);
pub struct HttpHeaderValue(/* SmolStr */);
// HttpBody/HttpRequest/HttpResponse/HttpBodyStream already defined in round-3 §3.
```

**(d) Crate rename (consequence — also closes the round-3 `*-domain` mislabeling):** the three `*-middleware-domain` crates rename to `*-middleware-runtime`. This is a **W0 Step 1 deliverable** (NOT deferred). The rename is mechanical (cargo rename + cross-crate path updates) and lands in the same PR as the refactor. CODEOWNERS rule: `crates/oya-http-*-middleware-runtime/` owned by `_seam_lane` DRI.

#### §3 seam contract — corrected import direction

```
oya-http-router-kernel  (layer=kernel; defines HttpRequest/HttpResponse/HttpBody/
                                       HttpBodyStream/HttpHeaderMap/HttpHeaderName/
                                       HttpHeaderValue newtypes)
        ↑                       ↑
        | (imports kernel       | (imports kernel
        |  newtypes only)       |  newtypes only)
        |                       |
oya-http-{deadline,telemetry,tenant}-middleware-runtime  (layer=runtime)
        ↑
        | (composes middleware chain;
        |  binds to adapter)
        |
oya-http-runtime-hyper-adapter  (layer=adapter; ONLY crate touching hyper::*)
```

**Allowed adapter→kernel direction:** adapter may depend on kernel (downward in DAG sense — adapter is "more concrete" than kernel). **Banned:** middleware-runtime → adapter (regression direction). Round-4 enforces both.

### Audit (verified post-rename)

| Crate | `[package.metadata.oyatie.layer]` | Hyper-family deps allowed |
|---|---|---|
| `oya-http-router-kernel` | `kernel` | none |
| `oya-http-middleware-kernel` | `kernel` | none |
| `oya-http-sse-runtime` (renamed from `*-domain`) | `runtime` | none |
| `oya-http-runtime-hyper-adapter` | `adapter` | full hyper stack |
| `oya-http-deadline-middleware-runtime` (renamed) | `runtime` | **none** (post-refactor) |
| `oya-http-telemetry-middleware-runtime` (renamed) | `runtime` | **none** (post-refactor) |
| `oya-http-tenant-middleware-runtime` (renamed) | `runtime` | **none** (post-refactor) |
| `oya-ops-workspace-shell-runtime` | `runtime` | `tokio` (still required for binary main) |

`bytes` is no longer declared by any `runtime`-layer crate post-W0. The binary shell (`oya-ops-workspace-shell-runtime`) loses its `bytes` declaration too (it consumes `HttpBody` via the adapter chain). Ledger row 5 (`bytes`) `allowed_crates_regex` narrows to `^oya-http-runtime-hyper-adapter$`.

### W0 Step 1 effort re-grade

Round-3 graded Step 1 as **L**. With added rename + adapter-dep-removal + HttpHeaderMap newtype + cross-crate audit, regrade to **L+ (upper end of L; ~3-4 engineer-weeks elapsed)**. Effort summary in §10 updated accordingly.

### Build-dep + dev-dep + workspace-deps policy (unchanged from round 3)

`allowed_build_deps_regex` per row; `allowed_dev_deps_regex` per row; root `[workspace.dependencies]` walked as separate sub-coverage step. Trait-object escape hatch banned per round-3 §3.

---

## 4. `oya-check-dependency-seam-discipline` Lane + CI-only carve-out (codex Concern 4 fix)

### Lane naming precedent (unchanged from round 3)

`oya-check-dependency-seam-discipline` (runner; layer=`runtime`). Companion kernel `oya-foundry-fitness-dependency-seam-kernel` (layer=`kernel`; deferred). ADR-0092 codifies split.

### Composite scope (revised — adds Concern 1 + Concern 3 sub-checks)

1. **Seam check** — per §3.
2. **Ledger-coverage check** — every external dep observed + every key in root `[workspace.dependencies]` MUST have a ledger entry.
3. **Ledger freshness check** — `next_review_due >= today - 5d`.
4. **Vendor-residue sub-check** — flags pure-Rust crates >50 KB with no workspace-internal consumers.
5. **CVE-watch sub-check** — reads `/registries/cross-cutting/cve-watch.json`; CVSS ≥7.5 + 14d → flips row to `replacement-armed-by-cve`.
6. **Ledger-review-contract sub-check** — 3-quarter zero-movement → fail.
7. **Layer-metadata sub-check** (NEW — codex Concern 1) — every crate in `cargo metadata --no-deps` declares `[package.metadata.oyatie.layer]` ∈ `{kernel, runtime, adapter, api, app}`; missing or invalid → BLOCKER. Authored by W0 Step 0 (NEW; see §10).
8. **Ledger-transition-monotonicity sub-check** (NEW — codex Concern 3) — diff current vs prior committed ledger; any backward `status` transition → fail.

### CI-only-read-side carve-out — codex Concern 4 fix

The same-PR ledger guard + role-gated `dep:new` label depends on PR-context queries (`gh api repos/{owner}/{repo}/issues/{n}/timeline`, `gh pr view N --json files,state,mergedAt`). This conflicts with the repo's grit-mediated posture documented in `master-plan-sequencing.json:52`, which mandates agents use `grit`, not `gh`.

#### Carve-out — explicit and bounded

1. **Read-side only.** The lane MAY invoke `gh api` and `gh pr view` for **read-only metadata retrieval**. Never `gh pr create`, `gh pr merge`, `gh pr review`, `gh issue comment`, or any state-mutating subcommand.
2. **CI context required.** The lane code refuses to run `gh` when `env::var("GITHUB_ACTIONS") != Ok("true".into())`. Local agent invocation fails fast with structured error message: *"gh-mediated lookups are CI-only. For local development, run `oya-dev-cli gate validate dependency-seam --mode=composite --offline` (skips PR-context guard) or use `grit` to invoke the lane in CI."*
3. **Agent-side state transitions remain grit-mediated.** No agent code path writes via `gh`. All mutations (ledger row edits, role-roster edits, ADR amendments, rollback PRs) flow through `grit claim → edit → grit done`.
4. **Same-PR roster mutation rejected.** If the PR modifies BOTH `/registries/cross-cutting/role-roster.json` AND `/registries/cross-cutting/tech-debt-ledger.json` (or any `crates/*/Cargo.toml`), lane fails: *"role-roster.json changes must land in a separate PR before being relied upon by dep:new label checks."* Prevents same-PR self-promotion.
5. **CODEOWNERS enforcement of `role-roster.json`.** The roster file is listed in CODEOWNERS as owned by the named DRI of the roster row whose role applies (bootstrap: `_role_roster` DRI = jason931225). PR modifications to the file require CODEOWNERS review approval before merge.

#### New ADR — ADR-0093 (Decisions §)

**Path:** `docs/decisions/ADR-0093-ci-only-gh-readside-carveout.md`. **Status:** Proposed → Accepted W0 Step 6.

**Decision:** *"PR-metadata read-side queries via `gh api` / `gh pr view` are permitted to the `oya-check-dependency-seam-discipline` lane runner in the GitHub Actions context only (`GITHUB_ACTIONS=true`). All agent-side state transitions remain grit-mediated per `master-plan-sequencing.json:52`. The roster file `/registries/cross-cutting/role-roster.json` is CODEOWNERS-owned by the named role DRI; same-PR self-promotion is rejected by lane sub-check."*

**Drivers:** PR-metadata audit is mechanically necessary to defeat label spoofing (§15 vector #1); grit cannot impersonate GitHub timeline events; carve-out is narrow + audited + CI-only.

**Alternatives:** (A) drop the `dep:new` label mechanism entirely — rejected (no defense against label spoofing). (B) build a grit-internal label registry — rejected (duplicates GitHub state with sync hazard). (C) CI-only carve-out (chosen).

**Consequences:** One narrow read-only escape hatch in the lane; agent posture preserved everywhere else; same-PR roster guard plus CODEOWNERS close the self-promotion gap.

Composite emission (`.omc/reports/dependency-seam-latest.json` with `failed_sub_checks: [...]`), severity ramp (Day 0→30 `report-only`; Day 30+ `error`), and implementation sketch (`oya-dev-cli gate validate dependency-seam --mode=composite [--offline] [--severity=...]`): unchanged from round 3, with the report schema extended to enumerate the 2 new sub-checks (`layer-metadata`, `ledger-transition-monotonicity`).

---

## 5. Replacement-Target Registry (unchanged from round 3)

Tiered GREEN/AMBER/RED bands; per-dep registry; NIH honesty in ADR-0091 §Drivers — all unchanged from round 3 §5.

---

## 6. Phase-Out Timeline + Walk-away protocol (codex Concern 5 fix)

Anchor trigger DSL + CVE shortcut: unchanged from round 3.

### Walk-away protocol — grit-mediated (codex Concern 5 fix)

Round-3 Step 4 emitted `oya-dev-cli gate emit rollback-pr` — this reintroduces PR mechanics outside grit. Round-4 replaces with grit primitive:

1. Ledger row → `status: replacement-attempted-abandoned` (object-map key unchanged; status field transitions monotonically; transition `replacement-armed → replacement-attempted-abandoned` is in the allowed graph).
2. Row freezes: `replacement_trigger = {"never": true}` (DSL registry now recognizes this — §2). CI enforces via composite sub-check `ledger-coverage`.
3. ADR-0091 status amendment via grit:
   ```
   grit claim --agent <walk-away-DRI> --intent "amend ADR-0091 §Decisions: status: ABANDONED for <dep>" --scope docs/decisions/ADR-0091-*.md
   # editor opens; DRI amends §Decisions adding "ABANDONED for <dep>" with date + failed parity report path + DRI decline reference
   grit done --evidence .omc/reports/replacement-parity-{dep}-{sha}.json
   ```
4. Rollback work via grit (NOT `emit rollback-pr`):
   ```
   grit claim --agent <walk-away-DRI> --intent "rollback <dep> replacement work; restore [workspace.dependencies] + allowed_crates_regex" --scope crates/oya-*-{dep}-*/,Cargo.toml,/registries/cross-cutting/tech-debt-ledger.json
   # editor opens; DRI reverts refactor commits, restores allowed_crates_regex, applies {"never": true}
   grit done --evidence .omc/reports/replacement-parity-{dep}-{sha}.json
   ```
   `grit done` handles the rebase/merge/release primitive per project memory `feedback_grit_claim_work_done.md`.
5. Next quarterly review's `decision` ∈ `{retire-phase-out-goal-for-dep, hold, re-arm-with-new-ADR}`. `re-arm-with-new-ADR` requires fresh ADR.

Every step = grit primitive + CI gate + fixed enum value. No `gh pr create`. No `emit rollback-pr`.

Sequence (W0..W5 + T0+12 walk-away) and quarterly review contract: unchanged from round 3 §6.

---

## 7. Cloud-Native Deployment Plan + Ops binary code-changes (codex Concern 6 fix)

Build target unchanged from round 3: musl x86_64 + aarch64; `FROM scratch`; no openssl; ≤30 MB stripped.

### Runtime contract — code-change deliverables (W0; not future work)

Codex Concern 6 found `oya-ops-workspace-shell-runtime` is not cloud-native today: binds `127.0.0.1`, uses `OYATIE_OPS_WORKSPACE_PORT`, exposes `/workspace/api/v1/health` instead of k8s probe paths, lacks SIGTERM. These are **W0 code deliverables**, listed explicitly in §19.

Cloud-native code contract (post-W0; verified by `oya-check-distroless-deployment-bar`):

| Property | Required value | Verification |
|---|---|---|
| Bind address | `0.0.0.0` (or `[::]`) by default; `127.0.0.1` only when `OYATIE_OPS_DEV_LOCAL_BIND=true` | Container netstat probe; lane fails if binary listening on loopback in container context. |
| Port env var | `PORT` (cloud-native standard) is canonical; `OYATIE_OPS_WORKSPACE_PORT` retained as **legacy alias** with deprecation warning emitted on startup if it is set without `PORT`. | Startup log assertion in lane test. |
| Liveness probe | `/healthz` returns `200 OK` once HTTP listener is bound (process up); `/livez` returns `200 OK` matching k8s convention. | Lane probe both endpoints in container; expects 200. |
| Readiness probe | `/readyz` returns `200 OK` only when catalog loaded + downstream dependencies (if any) reachable; `503` otherwise. | Lane probe `/readyz` immediately post-start; expects `503` until catalog ready; then `200`. |
| Legacy route | `/workspace/api/v1/health` retained for one release as deprecation alias; returns `200` + warning header `X-Deprecated: use /healthz`. | Lane probe; expects 200 + header. |
| SIGTERM handler | Closes accept loop on SIGTERM; drains in-flight (≤30s budget); emits shutdown event via tracing; exits 0. | Lane sends SIGTERM mid-load; expects clean exit ≤30s; tail-event log captured. |
| Cold-start budget | `process exec → first accept on PORT` ≤100ms (harness-measured per §7 below). | `oya-bench-cold-start-harness`. |
| Distroless image | Built both archs; `FROM scratch`; static musl. | `oya-check-distroless-deployment-bar` builds + boots both archs. |
| Stateless | No node-local state files; all state via configured storage adapter. | Lane verifies `/var`, `/tmp` empty on shutdown; binary writes nothing to local disk. |

Cold-start harness (`oya-bench-cold-start-harness`, `CLOCK_MONOTONIC`), throughput baseline (`(new-baseline)/baseline >= -0.02` for p50+p99), musl-runtime-smoke (DNS+getrandom+epoll+cert-bundle), and replacement-trajectory preservation: all unchanged from round 3 §7.

---

## 8. ADR-0091 Outline (round-4 updates)

**Path:** `docs/decisions/ADR-0091-workspace-dependency-seam-debt-ledger-phaseout.md`
**Status:** Proposed → Accepted on Architect + Critic + codex consensus (round-4 review pending).

**Decision (round-4 additions in *italics*):**
1. Every external dep in `crates/*/Cargo.toml` or root `[workspace.dependencies]` MUST appear in `/registries/cross-cutting/tech-debt-ledger.json` (entries as **object map keyed by `dep_name`**).
2. `oya-check-dependency-seam-discipline` composite lane (8 sub-checks: seam + ledger-coverage + freshness + vendor-residue + cve-watch + review-contract + *layer-metadata* + *ledger-transition-monotonicity*) enforces `allowed_crates_regex` across `[dependencies]` + `[build-dependencies]` + `[dev-dependencies]` + `[workspace.dependencies]`.
3. 5 products ship on hyper/tokio without phase-out blocking.
4. Phase-out begins only after T0 (ontology v1 stable 2 sprints + p99 14 days + parity pass with **immutable run-id evidence**).
5. Tiered parity: GREEN ≥95%/≤105% advances; AMBER 80-95% advances with ADR amendment; RED <80% → `replacement-attempted-abandoned` permanent.
6. `tracing` + `tracing-subscriber` + `serde_json` are exempt-permanent unless CVE-acceleration fires.
7. CVE acceleration: CVSS ≥7.5 + 14 days no upstream fix → auto-arms replacement.
8. DRI mapping in `/registries/cross-cutting/dri.json`; bootstrap `primary = jason931225 (user)`; role roster in `/registries/cross-cutting/role-roster.json` with **CODEOWNERS ownership + same-PR self-promotion guard**.
9. Walk-away: RED parity OR DRI decline → `replacement-attempted-abandoned` + grit-mediated ADR amendment + grit-mediated rollback. **No `gh pr create` / no `emit rollback-pr`.**
10. *(NEW round-4)* DSL evaluator policies (`staleness_policy`, `evidence_immutability_policy`, `pointer_missing_policy`, `monotonic_transitions_only`) are schema-required on every trigger; defaults are conservative (`fail-on-stale`, `run-id-required`, `not-yet-armed`, `true`).
11. *(NEW round-4)* `gh api` / `gh pr view` permitted to the lane runner under `GITHUB_ACTIONS=true` only (ADR-0093 carve-out). All other agent paths grit-mediated.
12. *(NEW round-4)* Cloud-native code contract for `oya-ops-workspace-shell-runtime`: `0.0.0.0`+`PORT`+`/healthz`+`/livez`+`/readyz`+`SIGTERM`+distroless, all W0 deliverables (§7 + §19).

**Drivers:** unchanged from round 3.

**Alternatives considered:** unchanged from round 3.

**Why chosen:** unchanged + *"DSL evaluator policies prevent silent armings on stale or mutable evidence; CODEOWNERS + same-PR guards close role-roster self-promotion; grit-mediated walk-away preserves repo posture."*

**Consequences:** unchanged + *"middleware-runtime crates renamed in W0; `bytes` declared only by adapter post-W0; ops binary becomes container-routable in W0 (today's `127.0.0.1` posture retired)."*

**§Risks — empirical base-rate paragraph (unchanged from round 3)**

**§Walk-away — grit-mediated steps:** see §6 above; ADR-0091 §Walk-away cites §6 verbatim.

**Follow-ups:** W0 Steps 0-8 (§10; +1 new step for layer-metadata bootstrap); per-replacement ralplan; quarterly review; CVE watch; ADR-0092 (lane naming); **ADR-0093 (CI carve-out)**.

---

## 9. Risk Register (round-4 additions)

Round-3 risks all carried forward. Round-4 adds:

- **Trigger DSL evaluator silent acceptance of stale/mutable evidence** (Low/High) → §2 four-policy mandate; default `fail-on-stale` + `run-id-required`; parser rejects missing policy fields. DRI: `_w0`.
- **Status transition ratchet broken** (Low/Medium) → §2 monotonic graph + `ledger-transition-monotonicity` sub-check (§4). DRI: `_ledger`.
- **Middleware-layer adapter import not actually fixed** (Low/High) → §3 W0 Step 1 audit report path + rename + acceptance test that middleware-runtime crates have zero `oya-http-runtime-hyper-adapter` dep. DRI: `_seam_lane`.
- **`gh` invocation slipping outside CI** (Low/Medium) → §4 carve-out: `GITHUB_ACTIONS=true` gate; local invocation fails fast. DRI: `_seam_lane`.
- **Role-roster same-PR self-promotion** (Low/High) → §4 same-PR guard + CODEOWNERS. DRI: `_role_roster`.
- **Ops binary not container-routable** (today's reality; resolved in W0) → §7 + §19 code-change deliverables; `oya-check-distroless-deployment-bar` BLOCKER post-W0. DRI: `_distroless`.

Inherited round-3 risks: trigger-never-fires (F1), seam-lane-bypass (F2), replacement-underperforms (F3), refactor-stalls-release, ledger-drift, distroless-TLS, musl-smoke-fail, T0+12-drift, parity-fail, lane-budget-overrun, serde_json-CVE-pre-replacement, ledger-self-heal-failure, trigger-DSL-parser-regression, dep-new-label-spoofing.

---

## 10. Steps (W0 only — round-4 adds Step 0 for layer-metadata bootstrap)

**Step 0 — Layer-metadata bootstrap across workspace** (S; Architect) — NEW per codex Concern 1
- Walk `cargo metadata --no-deps`; for every member missing `[package.metadata.oyatie.layer]`, author the metadata block in its `Cargo.toml` with the correct layer ∈ `{kernel, runtime, adapter, api, app}` per the crate's responsibility (verified against on-disk import graph).
- **Tool:** `oya-dev-cli gate emit layer-metadata-bootstrap` (one-shot scaffolder; idempotent; refuses to overwrite existing layer declarations; emits report of which crates were already declared vs newly declared).
- **Outputs:** modified `crates/*/Cargo.toml` (only added blocks; no edits to existing); `.omc/reports/layer-metadata-bootstrap-{sha}.json` listing pre/post coverage.
- **Verification:** `oya-check-dependency-seam-discipline --sub-check=layer-metadata` green on `main`; every workspace member has a layer; no member declares `layer: domain` (or any value outside the canonical 5).

**Step 1 — Kernel newtype wrappers + middleware-runtime refactor + crate rename + adapter-dep removal** (L+; Architect) — REVISED per codex Concern 2
- Author `HttpHeaderMap`, `HttpHeaderName`, `HttpHeaderValue` newtypes in `oya-http-router-kernel` (in addition to round-3's `HttpRequest`/`HttpResponse`/`HttpBody`/`HttpBodyStream`).
- Rename `oya-http-deadline-middleware-domain` → `oya-http-deadline-middleware-runtime`; same for `telemetry` + `tenant`. Also `oya-http-sse-domain` → `oya-http-sse-runtime`.
- Refactor all three middleware-runtime crates: drop `bytes`, drop `oya-http-runtime-hyper-adapter` dep, consume kernel newtypes only.
- **Tool:** `oya-dev-cli gate emit ops-workspace-shell-baseline --commit=<pre-refactor-sha>`; `oya-dev-cli gate emit middleware-adapter-import-audit --commit=<pre-refactor-sha>`.
- **Outputs:** `.omc/reports/ops-workspace-shell-baseline-{pre-refactor-sha}.json`; `.omc/reports/middleware-adapter-import-audit-{pre-refactor-sha}.json`; `.omc/reports/dependency-seam-baseline-{pre-refactor-sha}.json`.
- **Verification:**
  - Zero `bytes::Bytes` / `hyper::*` / `http_body::Body` imports outside `oya-http-runtime-hyper-adapter` (post-refactor src + `cargo metadata`).
  - Zero `oya-http-runtime-hyper-adapter` dependency declarations in the 3 middleware-runtime + `oya-http-sse-runtime` crates' `Cargo.toml`.
  - Post-refactor `(new-baseline)/baseline >= -0.02` for p50 + p99.
  - All four renamed crates declare `[package.metadata.oyatie.layer] = "runtime"` (via Step 0).

**Step 2 — Author `oya-check-dependency-seam-discipline` composite lane** (L; Critic)
- New files: `crates/oya-check-dependency-seam-discipline/` (`[package.metadata.oyatie.layer] = "runtime"`); `.omc/fitness-lanes/dependency-seam-discipline.md`; `crates/oya-dev-cli/src/commands/dependency_seam_gate.rs`; CI job.
- **Tool:** `cargo run -p oya-check-dependency-seam-discipline -- --mode=composite`.
- **Outputs:** lane crate `Cargo.toml` with explicit layer metadata; `.omc/reports/dependency-seam-latest.json` (schema enumerates 8 sub-checks: seam + ledger-coverage + freshness + vendor-residue + cve-watch + review-contract + layer-metadata + ledger-transition-monotonicity); `.omc/reports/dependency-seam-{sha}.json` archive.
- **Verification:** report-only run zero violations on `main` after Step 1; 6 fixture crates (kernel-imports-hyper, build-dep, dev-dep, vendor-residue, missing-layer, backward-transition) trigger expected violations.

**Step 3 — Ledger commit + generator + self-heal + object-map envelope** (L; Architect+Critic) — REVISED per codex Concern 3
- New files: `crates/oya-dev-cli/src/commands/tech_debt_ledger_emit.rs`, `crates/oya-dev-cli/src/commands/ledger_coverage_gate.rs`; seed 11 entries as object map.
- **Tool:** `oya-dev-cli gate emit tech-debt-ledger [--self-heal]`.
- **Outputs:** `/registries/cross-cutting/tech-debt-ledger.json` (envelope `version: "1.1.0"`; `entries` = object map; timestamps from `current_date()`); `/registries/cross-cutting/schemas/tech-debt-ledger.schema.json` (rejects array `entries`; requires 4 evaluator policy fields on every trigger).
- **Verification:** schema validated; coverage sub-check green; fixture dep without ledger entry → fail; same-PR guard → fail; **self-heal fixture:** delete entries `hyper` + `bytes`, run `--self-heal`, result bit-for-bit identical (SHA-256); object-map lexicographic ordering enforced.

**Step 4 — `oya-foundry-trigger-dsl-runtime` parser + evaluator + ADR-0091 + ADR-0092 + ADR-0093** (M+; Architect+Critic+codex) — REVISED per codex Concerns 1, 3, 4, 5
- Crate **renamed in round 4** from R3's `oya-foundry-trigger-dsl-domain` to `oya-foundry-trigger-dsl-runtime` (per codex Concern 1; canonical layer enum has NO `domain`; this crate parses + evaluates + reads files, so `runtime` is correct).
- **Tool:** `cargo test -p oya-foundry-trigger-dsl-runtime`; `oya-check-adr-index`; `oya-check-adr-citation`.
- **Outputs:** `crates/oya-foundry-trigger-dsl-runtime/` with `[package.metadata.oyatie.layer] = "runtime"`; parser + evaluator modules; `tests/fixtures/` covering: 11 seed triggers parse; 5 malformed reject with structured errors; stale-source fixture (file mtime older than `last_reviewed_at`); immutable-evidence fixture (rejects `ci-uri-latest` in status-gating predicate); missing-pointer fixture (`fail` vs `not-yet-armed`); monotonic-transition fixture (`replaced → active` rejected); `{"never": true}` fixture (returns `disarmed`).
- ADRs: `docs/decisions/ADR-0091-*.md` (revised round-4); `docs/decisions/ADR-0092-lane-runner-vs-kernel-crate-naming.md`; `docs/decisions/ADR-0093-ci-only-gh-readside-carveout.md`.
- **Verification:** all fixtures green; ADR-0091 §Decisions cites 4 evaluator policies + `{"never": true}` + carve-out; ADR-0092 + ADR-0093 indexed; signed by Architect + Critic + codex.

**Step 5 — Distroless smoke + `oya-bench-cold-start-harness` + `oya-check-distroless-deployment-bar`** (L; DRI+Critic)
- New crates: `oya-bench-cold-start-harness` (layer=`runtime`); `oya-check-distroless-deployment-bar` (layer=`runtime`).
- **Tool:** `cargo run -p oya-bench-cold-start-harness -- --target oya-ops-workspace-shell --port-env PORT`; `cargo run -p oya-check-distroless-deployment-bar`.
- **Outputs:** `.omc/reports/cold-start-oya-ops-workspace-shell-{sha}.json` (`process_exec_to_first_accept_ms`, `spawn_overhead_ms`); `.omc/fitness-lanes/distroless-deployment-bar.md`.
- **Verification:** distroless image built both archs; `/readyz` returns 200 ≤100ms cold-start (harness-measured); SIGTERM ≤30s; image ≤30 MB; musl-smoke green; cgroup probe respects `cpu.max`; **ops-binary code-changes from §19 verified per-property in §7 table**.

**Step 6 — `dri.json` + `role-roster.json` + CODEOWNERS update** (S; Critic+DRI) — REVISED per codex Concern 4
- **Tool:** `oya-dev-cli gate validate raci-coverage`.
- **Outputs:** `/registries/cross-cutting/{dri,role-roster}.json` + schemas; updated `CODEOWNERS` entry: `/registries/cross-cutting/role-roster.json @jason931225` (bootstrap; role-DRI named).
- **Verification:** every entry has named `primary` (bootstrap jason931225); `raci-coverage` green; no `council-*` labels; CODEOWNERS rule active on `role-roster.json`; same-PR self-promotion guard fixture (PR modifying both roster + ledger) → lane fail.

**Step 7 — Flip lane to `error` after 30-day soak + INDEX.md row + quarterly template** (M; Architect)
- **Tool:** `oya-dev-cli gate validate fitness-lane-index`.
- **Outputs:** updated `.omc/fitness-lanes/INDEX.md` (lane count 64 → **67** for new composite seam lane + parity + deployment-bar); `/registries/cross-cutting/tech-debt-ledger-review-template.md`.
- **Verification:** composite at `error`, green on `main`; review-contract sub-check arms; first quarterly review scheduled.

**Step 8 — Ops-binary cloud-native code-changes** (M; DRI) — NEW per codex Concern 6
- See §19 below for the explicit code-change list.
- **Tool:** `cargo run -p oya-check-distroless-deployment-bar -- --probe-conformance`.
- **Outputs:** modified `crates/oya-ops-workspace-shell-runtime/src/main.rs` (or equivalent entry); `Dockerfile.distroless` (or `cargo build --target x86_64-unknown-linux-musl` + `FROM scratch` recipe); reports per §7 table.
- **Verification:** per-property §7 table fully green; ops binary container-routable; SIGTERM clean ≤30s; `/healthz`/`/livez`/`/readyz` per k8s convention; `PORT` env var canonical; cold-start ≤100ms harness-measured.

**Effort summary:** 1S + 1L+ + 1L + 1L + 1M+ + 1L + 1S + 1M + 1M ≈ **5-6 engineer-months elapsed** (round-3 was 4-5; round-4 adds Step 0 + Step 8 + L+ regrade on Step 1).

---

## 11. Acceptance Criteria (round-4 revisions)

**Empirical lane-count baseline** (verified per round 3 §11): **30 BLOCKER + 28 HIGH + 6 MED + 0 LOW = 64 lanes**.

- [ ] `tech-debt-ledger.json` committed; **11 entries as object map keyed by `dep_name`**; schema validated.
- [ ] **Every trigger in every entry declares all 4 evaluator policies** (`staleness_policy`, `evidence_immutability_policy`, `pointer_missing_policy`, `monotonic_transitions_only`); parser rejects missing fields.
- [ ] **Status transition graph monotonic**; backward transitions rejected by `ledger-transition-monotonicity` sub-check.
- [ ] **`{"never": true}` recognized as valid `replacement_trigger`** by DSL registry; permitted only on `keep` / `replacement-attempted-abandoned` rows.
- [ ] `dri.json` + `role-roster.json` committed; every entry has named `primary` (bootstrap jason931225).
- [ ] **CODEOWNERS rule for `/registries/cross-cutting/role-roster.json`** committed; same-PR self-promotion guard fixture → lane fail.
- [ ] `crates/oya-check-dependency-seam-discipline/` exists with `[package.metadata.oyatie.layer] = "runtime"`; policy doc + CI job wired; **8 sub-checks** active (seam + ledger-coverage + freshness + vendor-residue + cve-watch + review-contract + **layer-metadata** + **ledger-transition-monotonicity**).
- [ ] **`gh` invocation gated by `GITHUB_ACTIONS=true`**; local invocation fails fast with grit-redirect message; ADR-0093 indexed + accepted.
- [ ] `oya-dev-cli` subcommands authored: `gate validate dependency-seam [--offline]`, `gate emit tech-debt-ledger [--self-heal]`, `gate emit layer-metadata-bootstrap`, `gate validate ledger-coverage`, `gate emit ops-workspace-shell-baseline`, `gate emit middleware-adapter-import-audit`. **REMOVED:** `gate emit rollback-pr` (replaced by grit primitive — §6).
- [ ] **`crates/oya-foundry-trigger-dsl-runtime/`** ships W0 (renamed from round-3's `*-domain` per codex Concern 1); `[package.metadata.oyatie.layer] = "runtime"`; 11 seed parse; 5 malformed fail; stale + immutable + missing-pointer + monotonic-transition + never-trigger fixtures all pass.
- [ ] Ledger generator self-heals (SHA-256 bit-for-bit roundtrip on object-map envelope).
- [ ] ADR-0091 written + indexed + accepted (Architect+Critic+codex). Includes §Drivers NIH-honesty, §CVE-acceleration, §DRI mapping, §Walk-away (grit-mediated; no `gh pr create`), §Risks empirical-base-rate paragraph, **§DSL-policies, §CI-carve-out, §Cloud-native-code-contract**.
- [ ] ADR-0092 written + indexed (lane runner vs kernel naming).
- [ ] **ADR-0093 written + indexed (CI-only gh read-side carve-out)**.
- [ ] Kernels (`oya-http-router-kernel`, `oya-http-middleware-kernel`) zero `hyper::`/`bytes::`/`http_body::` use-paths/pub-signatures.
- [ ] **`oya-http-{deadline,telemetry,tenant}-middleware-runtime` + `oya-http-sse-runtime` renamed from `*-domain`**; all four declare `layer = "runtime"`; **all four have zero `oya-http-runtime-hyper-adapter` dependency** in `Cargo.toml`; all four have zero `bytes` dep.
- [ ] **`oya-http-router-kernel` exposes `HttpHeaderMap` + `HttpHeaderName` + `HttpHeaderValue`** in addition to round-3 newtypes.
- [ ] Only `oya-http-runtime-hyper-adapter` (layer=`adapter`) declares hyper-family deps; `bytes` declared only by `oya-http-runtime-hyper-adapter`.
- [ ] `oya-bench-cold-start-harness` crate exists; output schema declared; uses `CLOCK_MONOTONIC`.
- [ ] **`oya-ops-workspace-shell` ships cloud-native (§7 + §19)**: binds `0.0.0.0` (or `[::]`); reads `PORT` env; `/healthz` + `/livez` + `/readyz` per k8s convention; `OYATIE_OPS_WORKSPACE_PORT` retained as deprecated alias; SIGTERM clean ≤30s; distroless both archs; cold-start ≤100ms; musl-smoke green.
- [ ] Throughput baseline captured at pre-refactor SHA; ≤2% regression measured.
- [ ] `oya-check-active-artifact-contract` green on ledger + DRI + role-roster + reports + reviews.
- [ ] **Every workspace crate declares `[package.metadata.oyatie.layer]` ∈ `{kernel, runtime, adapter, api, app}`**; `layer-metadata` sub-check green; **NO `domain` layer anywhere**.
- [ ] No `[workspace.dependencies]` removals; ADR-0090 not superseded.
- [ ] Quarterly review template present; `oya-check-debt-ledger-review-contract` sub-check arms; CVE-watch sub-check active.
- [ ] **Net new top-level CI lanes = 3**; baseline 64 → **67 total**; BLOCKER 30 → **33** (3 new BLOCKER post-soak).
- [ ] §15 bypass-vector mitigations all wired.
- [ ] Walk-away: grit-mediated (`grit claim → edit → grit done`); `replacement-attempted-abandoned` enum landed in schema.

---

## 12. Out of Scope (unchanged from round 3)

Per round 3 §12: removing hyper/tokio/serde/etc. from `[workspace.dependencies]`; authoring W1-W5 replacements; rustls; architecture changes; modifying ADR-0090; changing 5 product release plans.

---

## 13. Open Questions

Round-3 questions all resolved by round 4. Round 4 introduces:
- [ ] Whether `oya-http-sse-runtime` (renamed from `*-domain`) needs the same kernel-newtype audit as the middleware crates — DEFERRED to W1 unless seam lane finds violations on `main` post-Step 1. — *Why it matters: SSE crate also pre-dated the naming convention; same risk profile as middleware.*

Persisted to `.omc/plans/open-questions.md` (append-only).

---

## 14. Why Review-Ready (round-4 cross-walk)

### Codex round-3 concerns → resolution

- **Concern 1 (CRITICAL) — §18 layer-enum violation: `*-trigger-dsl-domain` invokes non-existent `domain` layer.** Closed by:
  - §2 + §10 Step 4 + §18: crate renamed `oya-foundry-trigger-dsl-domain` → **`oya-foundry-trigger-dsl-runtime`**.
  - §18.A audit table now uses ONLY the canonical 5 layers `{kernel, runtime, adapter, api, app}`.
  - §3: `*-middleware-domain` crates renamed to `*-middleware-runtime`; `oya-http-sse-domain` → `*-runtime`.
  - §10 Step 0 (NEW): every workspace member's `Cargo.toml` gets `[package.metadata.oyatie.layer]` block authored explicitly via `gate emit layer-metadata-bootstrap`.
  - §4: composite lane adds `layer-metadata` sub-check that rejects any layer outside the canonical 5.
- **Concern 2 (MAJOR) — W0 under-scopes seam breach (middleware → adapter direct dep).** Closed by:
  - §3 expanded scope: per-crate audit table; chosen path **(i) refactor**; three middleware crates drop `oya-http-runtime-hyper-adapter` AND `bytes`.
  - §3: `oya-http-router-kernel` newtype surface expanded with `HttpHeaderMap` + `HttpHeaderName` + `HttpHeaderValue` (the types middleware previously imported from adapter).
  - §3: crate rename `*-middleware-domain` → `*-middleware-runtime` (W0 deliverable, not deferred).
  - §3: corrected import direction diagram (middleware-runtime → kernel only; adapter → kernel only; middleware-runtime → adapter BANNED).
  - §10 Step 1 effort re-graded L → **L+**; audit report at `.omc/reports/middleware-adapter-import-audit-{sha}.json`.
- **Concern 3 (MAJOR) — Trigger DSL parseable but not operationally sufficient.** Closed by:
  - §2 schema: `entries` switched from array to **object map keyed by `dep_name`** → `dependent_wave_status` pointer `/entries/<dep_name>/status` now valid.
  - §2 evaluator semantics: 4 mandatory policies (`staleness_policy` default `fail-on-stale`; `evidence_immutability_policy` default `run-id-required`; `pointer_missing_policy` default `not-yet-armed`; `monotonic_transitions_only` default `true`).
  - §2 data_source URI: `ci-uri-run` (immutable) required for status-gating predicates; `ci-uri-latest` (mutable) permitted only on advisory dashboards.
  - §2 status transition graph: monotonic-forward only; backward transitions rejected.
  - §4: new sub-check `ledger-transition-monotonicity`.
  - §10 Step 4: parser + evaluator + fixtures covering all 4 policies + monotonicity + `{"never": true}`.
- **Concern 4 (MAJOR) — §15 bypass guards conflict with grit/no-gh posture.** Closed by:
  - §4: CI-only carve-out — `gh api` + `gh pr view` invoked ONLY when `env::var("GITHUB_ACTIONS") == Ok("true".into())`; local agent invocation fails fast with grit-redirect message.
  - §4: same-PR roster-self-promotion guard (PR modifying both `role-roster.json` AND ledger/Cargo.toml → lane fail).
  - §4 + §10 Step 6: CODEOWNERS rule pinning `role-roster.json` to named role DRI.
  - **ADR-0093** authored (§4 §Decisions) — narrow, CI-only, read-side-only carve-out documented; agent-side mutations remain grit-mediated.
- **Concern 5 (MAJOR) — Walk-away has 2 holes (`{"never": true}` not in DSL; `emit rollback-pr` reintroduces PR mechanics).** Closed by:
  - §2 trigger DSL registry: `{"never": true}` added as recognized variant; evaluator returns `disarmed`.
  - §6: walk-away protocol rewritten in grit primitives only:
    ```
    grit claim --agent <walk-away-DRI> --intent "..." --scope <files>
    # editor opens; DRI amends/reverts
    grit done --evidence .omc/reports/replacement-parity-{dep}-{sha}.json
    ```
    `gate emit rollback-pr` REMOVED from the CLI surface (§11 acceptance: explicitly removed).
- **Concern 6 (MAJOR) — §7 not shippable today (binds `127.0.0.1`, non-standard port env, non-k8s probe paths, no SIGTERM).** Closed by:
  - §7 runtime contract table: explicit per-property required values (bind, port env, probes, SIGTERM, distroless, stateless) — each row has a verification.
  - §19 (NEW): explicit code-change list for `oya-ops-workspace-shell-runtime` as W0 deliverable.
  - §10 Step 8 (NEW): "Ops-binary cloud-native code-changes" with the deliverables.
  - §11 acceptance: cloud-native conformance checkbox carries §7-table verification gates.

All round-3 §14 cross-walk resolutions (Architect APPROVE-WITH-NOTES + Critic APPROVE) retained; round-4 builds on them.

---

## 15. Bypass-Vector Enumeration + Countermeasures (round-4 additions)

1-15 from round 3 unchanged. Round 4 adds:

16. **Role-roster self-promotion in same PR** → §4 same-PR roster-and-ledger guard + CODEOWNERS rule pinning `role-roster.json` to named role DRI.
17. **`gh` invocation slipping outside CI** → §4 `GITHUB_ACTIONS=true` env guard; local invocation fails fast; ADR-0093 codifies.
18. **DSL trigger arms on stale or mutable evidence** → §2 4-policy mandate (default `fail-on-stale` + `run-id-required`).
19. **Status backward-transition silent regression** → §2 monotonic graph + §4 `ledger-transition-monotonicity` sub-check.
20. **Middleware crate quietly re-adds `oya-http-runtime-hyper-adapter` dep** → §3 import-direction rule + post-W0 lane sub-check rejecting middleware-runtime → adapter dep edges.
21. **Layer metadata omitted on new crate** → §4 `layer-metadata` sub-check BLOCKER day 0; `oya-dev-cli gate emit layer-metadata-bootstrap` scaffolds.
22. **Walk-away via PR mechanics instead of grit** → §6 only grit primitives accepted; `gate emit rollback-pr` REMOVED from CLI.

---

## 16. Lane Consolidation Map (round-4 — count unchanged from round 3)

3 top-level CI lanes; composite seam lane folds **8 sub-checks** (round-3 had 6; round-4 adds `layer-metadata` + `ledger-transition-monotonicity`).

Lane budget impact: 64 existing + 3 new = **67 total**. 30 BLOCKER existing + 3 new = **33 BLOCKER**.

---

## 17. DRI Registry Seed (round-4 additions)

Round-3 entries preserved. Round 4 adds:

- `_role_roster` (CODEOWNERS owner for `/registries/cross-cutting/role-roster.json`; same-PR-guard reviewer) — bootstrap `primary = jason931225`.

```json
{
  "$schema": "./schemas/dri.schema.json",
  "version": "1.1.0",
  "entries": {
    "<row>": { "primary": "jason931225", "backup": "<TBD>", "role": "<see above>" }
  }
}
```

Acceptance gate per round 3 retained.

---

## 18. Layer-Enum Alignment Audit (round-4 — codex Concern 1 fixes)

Canonical: `{kernel, runtime, adapter, api, app}`. No `domain`. The round-3 §18.A entries that flagged `runtime (per *-domain convention)` were a holdover and are corrected here:

### §18.A — Per-crate layer mapping (round-4)

| Crate | Authored or referenced? | `[package.metadata.oyatie.layer]` | Rationale |
|---|---|---|---|
| `oya-http-router-kernel` | referenced (existing) | `kernel` | Pure value-objects (HttpRequest/HttpResponse/HttpBody/HttpHeaderMap newtypes). |
| `oya-http-middleware-kernel` | referenced (existing) | `kernel` | Middleware trait definitions; no I/O. |
| `oya-http-sse-runtime` (renamed from `*-domain`) | renamed in Step 1 | `runtime` | Domain logic over SSE streams; reads/writes byte streams → I/O orchestrator. |
| `oya-http-deadline-middleware-runtime` (renamed) | renamed in Step 1 | `runtime` | Deadline side-effect orchestration. |
| `oya-http-telemetry-middleware-runtime` (renamed) | renamed in Step 1 | `runtime` | Telemetry side-effect orchestration. |
| `oya-http-tenant-middleware-runtime` (renamed) | renamed in Step 1 | `runtime` | Tenant-resolution side-effect orchestration. |
| `oya-http-runtime-hyper-adapter` | referenced | `adapter` | Provider-specific I/O (hyper). |
| `oya-ops-workspace-shell-runtime` | referenced; refactored in Step 8 | `runtime` | Binary's runtime shell. |
| `oya-check-dependency-seam-discipline` | authored (Step 2) | `runtime` | Lane runner. |
| **`oya-foundry-trigger-dsl-runtime`** (renamed from R3's `*-domain`) | authored (Step 4) | `runtime` | Parses + evaluates JSON triggers; reads files; I/O surface confirmed. **No `domain` layer exists** per canonical enum. |
| `oya-bench-cold-start-harness` | authored (Step 5) | `runtime` | Subprocess + `CLOCK_MONOTONIC`; I/O orchestrator. |
| `oya-check-distroless-deployment-bar` | authored (Step 5) | `runtime` | Lane runner. |
| `oya-check-replacement-parity` | authored (W1+) | `runtime` | Lane runner. |
| `oya-runtime-cgroup-runtime` (renamed from `*-domain` if pre-existing) | referenced | `runtime` | cgroup probing. |
| `oya-dev-cli` | referenced | `app` | Top-level CLI binary. |
| `oya-foundry-fitness-dependency-seam-kernel` | deferred (post-W0) | `kernel` | Pure policy value-object; reusable. |

### §18.B — ADR-0092 (unchanged from round 3)

### §18.C — Verification (round-4)

`oya-check-dependency-seam-discipline --sub-check=layer-metadata` (Step 2 + Step 0):
- parses every workspace crate's `[package.metadata.oyatie.layer]`;
- rejects missing (BLOCKER);
- rejects any value outside `{kernel, runtime, adapter, api, app}` (BLOCKER) — this is the precise gate that would have caught the round-3 `domain` mislabeling;
- rejects `oya-check-*` with layer ≠ `runtime`;
- rejects `oya-foundry-fitness-*-kernel` with layer ≠ `kernel`.

### §18.D — Bootstrap reality (round-4)

W0 Step 0 + Step 1 + Step 4 explicitly rename + declare layer metadata for every crate the plan touches; the §18 audit is true post-W0 (round-3 claim was aspirational; round-4 makes it a gated deliverable).

---

## 19. Code-Change Deliverables — Ops Binary Cloud-Native Upgrade (NEW per codex Concern 6)

The following are W0 code-changes to `crates/oya-ops-workspace-shell-runtime/` (and supporting build files), landed as part of Step 8. Each line item is a discrete code edit; each has an acceptance gate in §7's per-property table.

### §19.1 — Bind address

- **Before:** binds `127.0.0.1:${OYATIE_OPS_WORKSPACE_PORT:-8080}` (codex-confirmed today).
- **After:** binds `0.0.0.0:${PORT:-8080}` by default. `[::]` permitted (IPv6 dual-stack). `127.0.0.1` only when `OYATIE_OPS_DEV_LOCAL_BIND=true` (dev-mode override; not the default).
- **Code:** `main.rs` `let addr = if std::env::var("OYATIE_OPS_DEV_LOCAL_BIND").as_deref() == Ok("true") { "127.0.0.1" } else { "0.0.0.0" };`.

### §19.2 — Port env var

- **Before:** reads `OYATIE_OPS_WORKSPACE_PORT`.
- **After:** reads `PORT` first; falls back to `OYATIE_OPS_WORKSPACE_PORT` (legacy alias) with deprecation warning on startup. Both unset → default `8080`.
- **Code:** `let port = std::env::var("PORT").or_else(|_| { let legacy = std::env::var("OYATIE_OPS_WORKSPACE_PORT"); if legacy.is_ok() { tracing::warn!(target: "deprecation", "OYATIE_OPS_WORKSPACE_PORT is deprecated; use PORT"); } legacy }).unwrap_or_else(|_| "8080".into());`.

### §19.3 — k8s probe routes

- **Before:** `/workspace/api/v1/health` only.
- **After:** add `/healthz` (always 200 once process bound), `/livez` (always 200 once process bound), `/readyz` (200 only when catalog loaded + downstream reachable; otherwise 503). Retain `/workspace/api/v1/health` as deprecation alias for one release with response header `X-Deprecated: use /healthz`.
- **Code:** new router entries; readiness probe consults a `ReadinessGate` value the catalog loader flips to `Ready` once init completes.

### §19.4 — SIGTERM graceful shutdown

- **Before:** no SIGTERM handler.
- **After:** SIGTERM handler stops accept loop + drains in-flight requests with timeout 30s + emits `tracing::info!(target: "lifecycle", event = "shutdown_initiated")` + exits 0.
- **Code:** `tokio::signal::unix::signal(SignalKind::terminate())` (or platform-equivalent) in a select! arm alongside the accept loop.

### §19.5 — Distroless build target

- **Before:** built against host glibc.
- **After:** `Dockerfile.distroless` (or equivalent build recipe) targets `x86_64-unknown-linux-musl` + `aarch64-unknown-linux-musl`; final image `FROM scratch` with the static binary; image size ≤30 MB stripped.
- **Code:** new `Dockerfile.distroless` at repo root or in `crates/oya-ops-workspace-shell-runtime/`; cargo `[profile.release]` adjusted to strip + LTO; CI matrix builds both archs.

### §19.6 — Cold-start measurement integration

- **After:** `oya-bench-cold-start-harness` invoked in CI against the distroless image; output to `.omc/reports/cold-start-oya-ops-workspace-shell-{sha}.json`; budget ≤100ms `process exec → first accept on PORT`.
- **Code:** new CI job step under `oya-check-distroless-deployment-bar`.

### §19.7 — Stateless verification

- **After:** binary writes no node-local state; lane verifies `/var`, `/tmp` empty on shutdown.
- **Code:** audit existing code paths; any current writes to local FS routed to configured storage adapter.

### Acceptance gate

`oya-check-distroless-deployment-bar --probe-conformance` runs all 7 §19 sub-property checks against the built distroless image; **all must pass** before Step 8 is accepted. Failure of any sub-property → BLOCKER.

---

**Round 4 — final.** All 6 codex concerns closed per §14; round-3 Architect/Critic findings preserved. Ready for re-run.
