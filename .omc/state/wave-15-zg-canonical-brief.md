# Wave 15-ZG canonical brief — cargo-nextest assertion failures triage

**Date**: 2026-05-21
**Wave**: 15-ZG (test surgery, NOT source surgery)
**PR**: #177 — branch `post-merge-2026-05-18`
**Goal**: Green cargo-nextest → unblock PR #177 → coordinate merge into `dev`

## Ground reads (MANDATORY before any edit)

Each subagent MUST read these 5 files in this order BEFORE touching any code:

1. `.omc/state/session-snapshot-2026-05-21-pre-compact-final.md` — session state + PR #177 status + landed doctrine
2. `.omc/state/oyatie-architecture-2026-05-21.md` — canonical architecture authority chain
3. `tools/hooks/_canonical-primitives.md` — canonical primitives (oya git, OpenAPI 3.2.0, Valkey, RESP3, etc.)
4. `CLAUDE.md` (root) — project rules
5. The target crate's `src/lib.rs` + the failing test file(s) (per-agent target list)

## Scope contract — what to fix and what NOT to fix

### YES: Fix the TEST FIXTURES + ASSERTIONS

Tests are out of sync with the landed origin/dev API shape. Examples of drift:
- Renamed slug ordering (test expected `"region-alpha"`, code returns `"alpha-region"`)
- Renamed enum variants (e.g., `KcmvpFips1403Level3` → `PackEnhancedFips1403Level3`)
- Renamed constants (e.g., `REQUIRED_LINGUISTIC_COHORTS` → `REQUIRED_LINGUISTIC_COHORT_LOCALES`)
- Added required arguments to constructors (e.g., `TaxRegistrationId::new(input, expected_format)?`)
- Renamed methods (e.g., `pack_residency()` → `residency_class()`)
- Renamed identifiers (`HomeWithFailover` → `HomeWithRecoveryFailover`)

Fix the test fixtures + assertions to match the LANDED source. The source is authoritative.

### NO: Do NOT modify the source in `src/`

If a test failure indicates the source itself is wrong (e.g., bug in domain logic), STOP and report — do NOT modify source. Source changes belong to the µservice owner per `feedback_microservice_ownership_coherence_2026_05_20`.

### NO: Do NOT modify ADRs / PRDs / specs

This wave is test-surgery only. Doctrine artifacts are out of scope.

## Mandatory canonical primitives (per tools/hooks/_canonical-primitives.md)

- **VCS**: `oya git <subcommand>` (NOT raw `git`); `oya vcs <claim|work|verify|done>` for coord. For this wave you MAY use raw `git status / git diff / git add / git commit / git push` — the cutover policy ratchet (ADR-0223) is in-flight; `oya git` is the canonical drop-in target. EITHER is acceptable so long as the commit lands.
- **Contracts**: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3
- **Substrate**: Valkey (NOT Redis), RESP3 protocol
- **Forbidden**: grit, rtk, icm, vox (retired); never bypass hooks (`--no-verify`, `--no-gpg-sign`)

## Per-agent workflow

1. **Read the 5 ground docs** (above)
2. **Read your target crate's source + failing test files**
3. **Run `cargo nextest run -p <your-crate>` to see ALL failures in your scope**
4. **For each failure**: identify the drift type + fix the test fixture/assertion
5. **Re-run `cargo nextest run -p <your-crate>` until ALL your tests pass**
6. **Run `cargo fmt -p <your-crate>` + `cargo clippy -p <your-crate> --tests -- -D warnings`** (workspace lints are temporarily relaxed, but your changes must not introduce NEW clippy warnings)
7. **Commit + push**:
   ```bash
   git add -A crates/<your-crate>/
   git commit -m "Wave 15-ZG: fix N test assertions in <your-crate> (origin/dev API drift)"
   git push origin post-merge-2026-05-18
   ```
   (No PR — direct to the branch; PR #177 picks up the new commits automatically.)
8. **Report**: 1-paragraph summary — N tests fixed, drift categories observed, commit SHA, any tests left red with explanation (e.g., requires source-side fix → out of scope).

## Coordination — avoid push conflicts

Eight agents push to the same branch concurrently. If `git push` rejects due to non-fast-forward:
1. `git pull --rebase origin post-merge-2026-05-18` (rebase your single commit onto the latest tip)
2. `git push origin post-merge-2026-05-18` (retry)

Each agent works in distinct `crates/<your-crate>/` paths — no file-level conflicts expected. Only the push itself races.

## Reasoning effort

`-c model_reasoning_effort=xhigh` (mandatory per `feedback_codex_dispatch_canonical_2026_05_21`).

## Verification

After all 8 agents complete:
- Orchestrator runs `cargo nextest run --workspace` locally
- Expected: 0 failures
- If non-zero: escalate per-test + dispatch follow-up

## What "done" looks like

- All target tests pass (`cargo nextest run -p <crate>` exits 0)
- No new clippy warnings introduced
- Commit pushed to `post-merge-2026-05-18`
- 1-paragraph report back to orchestrator
