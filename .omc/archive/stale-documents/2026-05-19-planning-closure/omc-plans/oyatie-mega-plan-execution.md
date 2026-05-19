---
purpose: Auto-backfilled purpose for oyatie-mega-plan-execution.md
---

# Oyatie Implementation Plan — Wave 0 + Wave 1 §2

> **Status**: DRAFT — pending Architect review
> **Mode**: ralplan consensus (SHORT)
> **Source**: `/Users/jasonlee/bominal/agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md` + `/Users/jasonlee/bominal/docs`
> **Repo**: `/Users/jasonlee/bominal/` (Rust workspace)
> **Date**: 2026-05-12

---

## RALPLAN-DR Summary

### Principles
1. **Clean-architecture invariant is sacred**: `kernel ← domain ← app ← {api,worker,adapter} ← runtime`. No leak upwards. Migrations only flow toward dependency-correct crates.
2. **Evidence-first verification**: Every wave task lands with TDD red→green, executable validator, and verification command captured in the plan.
3. **grit-bounded mutation**: Every file touched goes through `grit claim → work → done`. No bare-hands edits to shared state.
4. **Parallelism only where independent**: Use `/ultrawork` or `/batch` for branches with no shared file overlap; serialize anything touching `goals.json`, validators, or hooks state.
5. **Small bounded steps**: Each task ≤ 1 logical change, ≤ 1 commit, ≤ 1 PR-equivalent slice. No mega-commits.

### Decision Drivers (top 3)
1. **Unblock CI** — Wave 0 stale hook-names, broken validators, and stale citation paths are currently red. Nothing else ships until green.
2. **Preserve the existing G001-G006 work** — `oya-foundry-agent-runtime` already has the account-auth domain logic. Wave 1 §2 must migrate (not rewrite) into `oya-foundry-account-*` per clean-architecture.
3. **Idempotent re-runs** — grit claim+lease semantics + atomic `goals.json` writes are required so parallel agents do not corrupt shared state on retry.

### Viable Options

**Option A — Serial Wave 0, then parallel Wave 1 §2 (RECOMMENDED)**
- Pros:
  - Wave 0 fixes touch overlapping files (validators, `.claude/`, hook payloads). Serial avoids merge conflicts.
  - Wave 1 §2 P00-01..P00-08 are crate-scoped and largely independent → safe to `/ultrawork`.
  - Easy rollback per grit-done boundary.
- Cons:
  - Wave 0 critical path is ~7 tasks long; serial costs wall-clock time.

**Option B — Aggressive parallel Wave 0 via `/batch` with sub-worktrees**
- Pros:
  - Faster wall-clock on Wave 0.
- Cons:
  - Hook-name fixes (#1) touch `.claude/settings/` AND `agents/settings/` AND validators AND CI. High merge-conflict risk.
  - omx preflight (#3) and consensus-gate docs (#4) intersect with citation-path fix (#5) at the docs tree.
  - Higher complexity for marginal gain. **Invalidated** as primary path.

**Option C — Defer Wave 0 #6 (12-layer count) and #7 (task_id workaround) to Wave 1**
- Pros: Faster CI green if the layer-count is purely documentation.
- Cons: The mega-plan explicitly gates Wave 1 entry on Wave 0 completion. Cherry-picking risks "almost-green" state that re-pollutes once Wave 1 starts. **Invalidated** unless Architect explicitly authorizes scope reduction.

**Selected: Option A.**

---

## Plan Structure

### Wave 0 — Unblock CI (serial, ~7 tasks)

Touch-zones are too overlapping for safe parallelism. Each task is a single grit claim→work→done cycle.

#### W0-T1: Fix stale `WorktreeCreated`/`WorktreeRemoved` hook-name drift
- **Files**: `.claude/settings/*`, `agents/settings/*`, validators referencing the old names, `.github/workflows/*` CI
- **Action**: Grep for `WorktreeCreated`/`WorktreeRemoved` across repo; replace with the current canonical names per `agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md`.
- **Acceptance**:
  - `rtk grep -r "WorktreeCreated\|WorktreeRemoved" .claude/ agents/ crates/ .github/` returns 0 hits
  - Validators previously referencing old names parse the new payload (add a test asserting the renamed event flows through)
  - CI job `hook-name-drift-guard` (or equivalent) goes green
- **Verification**: `rtk cargo test -p oya-foundry-agent-runtime --test foundry_spec` + targeted hook-payload validator test

#### W0-T2: Replace declarative-only validators with executable+tested implementations
- **Validators**: `conflict-hotspot-diff-guard`, `generated-artifact-guard`, `script-value-guard`, `stale-ci-state-guard`
- **Action**: For each validator, add (a) executable implementation, (b) unit test covering pass + fail + edge case, (c) wire into hook config.
- **Acceptance**:
  - Each validator has a `#[test]` red→green pair
  - Running the validator with a deliberately-bad fixture returns non-zero
  - Running with a clean fixture returns zero
  - `rtk cargo test -p <validator-crate>` green
- **Verification**: `rtk cargo test --workspace -- --include-ignored validator_`

#### W0-T3: omx preflight — atomic `goals.json` write + lease guard
- **Files**: omx preflight tool (likely `crates/omx-*` or a script under `agents/`)
- **Action**: Replace any non-atomic `write_to_file(goals.json)` with `tempfile → fsync → rename` pattern. Add a fcntl-or-equivalent advisory lease on `goals.json` for the duration of the write.
- **Acceptance**:
  - Concurrent-write test: spawn 2 omx preflight processes; both succeed without `goals.json` corruption (one waits for lease, both observe consistent final state)
  - Crash-mid-write test: SIGKILL during write leaves `goals.json` unchanged (verified by checksum)
- **Verification**: `rtk cargo test goals_json_atomic_write` + manual race-test script

#### W0-T4: Consensus-gate doc fix
- **File**: `/Users/jasonlee/bominal/docs/roadmap/ultragoal-brief.md`
- **Action**: Replace "blocked until APPROVE" language with the actual post-2026-05-07 APPROVE'd state.
- **Acceptance**:
  - File no longer contains the stale "blocked until APPROVE" string
  - Status section reflects APPROVE date `2026-05-07`
  - Backlinks (if any) in `/Users/jasonlee/bominal/docs/roadmap/` consistent
- **Verification**: `rtk grep -r "blocked until APPROVE" /Users/jasonlee/bominal/docs/` returns 0 hits

#### W0-T5: Past-200 hotspot citation path drift
- **File**: delivery-plan document (cites `.omx/ultragoal/evidence/`)
- **Action**: Rewrite citation paths to `agents/ultragoal/evidence/` everywhere they appear.
- **Acceptance**:
  - No `.omx/ultragoal/evidence/` references remain in delivery-plan
  - All cited evidence files actually exist at the new path
- **Verification**: `rtk grep -r "\.omx/ultragoal/evidence" /Users/jasonlee/bominal/` returns 0 hits AND citation-link checker (if present) green

#### W0-T6: 12-layer count reconciliation
- **Files**: Architecture docs in `/Users/jasonlee/bominal/docs/architecture/` referring to layer count
- **Action**: Audit canonical layer enumeration vs. all doc mentions; pick the source-of-truth doc; align all others (or fix the source if drift went the other way).
- **Acceptance**:
  - Single canonical layer-count statement exists
  - All other layer-count mentions cite or match it
- **Verification**: `rtk grep -rE "(11|12|13)[- ]layer" /Users/jasonlee/bominal/docs/` — all hits consistent

#### W0-T7: `foundry-agent-pre-tool-claim` hook payload `task_id` mismatch — workaround
- **Action**: Implement the documented workaround — seed a `claims` row on session-start so the pre-tool-claim hook's `task_id` lookup succeeds.
- **Acceptance**:
  - Session-start hook (or equivalent) inserts a placeholder claims row
  - Pre-tool-claim hook test that previously failed on `task_id` mismatch now passes
  - Add a regression test
- **Verification**: `rtk cargo test pre_tool_claim_task_id_workaround`

> **Note**: Wave 0 #8 (stop-hook schema) is documented as already-fixed historical. No action.

---

### Wave 1 §2 — P00-01..P00-08: Migrate account-auth into `oya-foundry-account-*` crates

The G001-G006 logic already lives in `oya-foundry-agent-runtime` (`domain.rs`, `auth.rs`, `provider_gateway.rs`, `providers.rs`, `http.rs`). The clean-architecture target is the 8 `oya-foundry-account-*` crates that exist as skeletons.

**These 8 sub-tasks ARE parallelizable** — each crate is independently scoped. Use `/ultrawork` with 4-way concurrency (more risks llm context thrash on shared review).

| Task | Source (in `oya-foundry-agent-runtime`) | Target crate | Notes |
|---|---|---|---|
| P00-01 | `domain.rs` (account-auth types) | `oya-foundry-account-domain` | Pure types; no I/O |
| P00-02 | `auth.rs` (kernel ports/traits) | `oya-foundry-account-kernel` | Trait definitions only |
| P00-03 | use-case orchestration | `oya-foundry-account-app` | Depends on kernel + domain |
| P00-04 | `providers.rs` (Claude Code provider) | `oya-foundry-account-adapter-claude-code` | Adapter |
| P00-05 | `providers.rs` (Codex CLI provider) | `oya-foundry-account-adapter-codex-cli` | Adapter |
| P00-06 | `providers.rs` (Gemini CLI provider) | `oya-foundry-account-adapter-gemini-cli` | Adapter |
| P00-07 | OpenBao SecretReference resolution | `oya-foundry-account-adapter-openbao` | Adapter |
| P00-08 | `http.rs` + binding wire-up | `oya-foundry-account-runtime` | Composition root |

#### Per-task acceptance criteria template
For each P00-0X:
- **Dependency direction verified**: `cargo tree -p <crate>` shows imports only flow toward kernel/domain (no upward leak)
- **All ported tests green**: tests moved alongside code run green in new crate
- **Original crate cleaned**: dead code or now-duplicate definitions removed from `oya-foundry-agent-runtime` (only after target crate is green)
- **Public API parity**: a workspace-level integration test (in `oya-foundry-account-runtime` or a top-level `tests/` crate) demonstrates the runtime composes adapters end-to-end equivalently to pre-migration behavior
- **Edition 2024 / Rust 1.95**: no warnings, no `cargo clippy` violations
- **SecretReference invariant**: no raw secrets in any moved code; OpenBao adapter is the only credential resolver

#### Parallelization plan
- Run P00-01 and P00-02 first (serial-ish — domain + kernel are foundations).
- Then `/ultrawork` P00-03..P00-07 in parallel (kernel/domain are stable; adapters are independent).
- P00-08 is last (composition root depends on all others).

#### grit workflow per task
```
grit claim <task-id> --crate oya-foundry-account-<role>
# ... edits in worktree ...
rtk cargo test -p oya-foundry-account-<role>
rtk cargo clippy -p oya-foundry-account-<role>
grit done <task-id>
```

#### `/ultrawork` invocation point
After W0 complete and P00-01, P00-02 green:
```
/ultrawork P00-03 P00-04 P00-05 P00-06 P00-07 --concurrency 4
```

---

## Success Criteria (overall)

- All 7 Wave 0 tasks (W0-T1..W0-T7) green in CI
- All 8 Wave 1 §2 tasks (P00-01..P00-08) green; `oya-foundry-agent-runtime` slimmed to only its rightful runtime concerns
- Workspace `rtk cargo test --workspace` green
- Workspace `rtk cargo clippy --workspace -- -D warnings` clean
- `grit status` shows no orphaned claims
- `goals.json` accurately reflects completion of all 15 tasks
- No `WorktreeCreated`/`WorktreeRemoved`/`.omx/ultragoal/` strings remain
- Architect review APPROVE'd before any code is touched

---

## Verification Commands (canonical)

```bash
# Wave 0 sweep
rtk grep -r "WorktreeCreated\|WorktreeRemoved" .claude/ agents/ crates/ .github/
rtk grep -r "blocked until APPROVE" /Users/jasonlee/bominal/docs/
rtk grep -r "\.omx/ultragoal/evidence" /Users/jasonlee/bominal/
rtk cargo test --workspace
rtk cargo clippy --workspace -- -D warnings

# Wave 1 §2 sweep
rtk cargo test -p oya-foundry-account-domain
rtk cargo test -p oya-foundry-account-kernel
rtk cargo test -p oya-foundry-account-app
rtk cargo test -p oya-foundry-account-adapter-claude-code
rtk cargo test -p oya-foundry-account-adapter-codex-cli
rtk cargo test -p oya-foundry-account-adapter-gemini-cli
rtk cargo test -p oya-foundry-account-adapter-openbao
rtk cargo test -p oya-foundry-account-runtime

# Dependency-direction guard
cargo tree -p oya-foundry-account-runtime | grep -E "oya-foundry-account-(kernel|domain|app|adapter)"

# grit hygiene
grit status
```

---

## Out of Scope (explicit)

- Wave 2+ (Phase 01 kernel/control-plane hardening, Phase 02 Foundry self-hosting loop) — not addressed here. Triggered only after Wave 1 §2 green.
- Any change to `kernel ← domain ← app ← {api,worker,adapter} ← runtime` topology — non-negotiable.
- Any new providers beyond Claude Code, Codex CLI, Gemini CLI.
- Net-new validators beyond the four required by Wave 0 #2.
- Re-implementing G001-G006 logic — Wave 1 §2 is migration, not rewrite.

---

## Open Questions

See `/Users/jasonlee/oyatie/.omc/plans/open-questions.md`.

---

## Status

**DRAFT — pending Architect review.**

Hand-off targets:
1. Architect: review topology + migration plan
2. Critic: stress-test acceptance criteria + verification commands
3. On dual-APPROVE → emit ADR and hand off to `/oh-my-claudecode:start-work oyatie-mega-plan-execution`
