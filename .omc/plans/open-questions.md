# Open Questions

## oyatie-mega-plan-execution — 2026-05-12

- [ ] **Canonical hook-name replacements** for `WorktreeCreated` and `WorktreeRemoved` — Wave 0 #1 needs the exact new names from the mega-plan; planner inferred drift but not the target tokens. — Blocks W0-T1 implementation.
- [ ] **Canonical layer-count** (11, 12, or 13 layers) — Wave 0 #6 needs to know which doc is the source of truth before reconciliation. — Blocks W0-T6.
- [ ] **omx preflight crate location** — is it `crates/omx-preflight`, a script in `agents/`, or part of an existing crate? — Affects W0-T3 file path enumeration.
- [ ] **`session-start` hook owner crate** — Wave 0 #7 needs to know which crate owns the session-start path that will seed the `claims` row workaround. — Blocks W0-T7.
- [ ] **Skeleton state of `oya-foundry-account-*` crates** — are `Cargo.toml` deps already pointing in the clean-arch direction, or do they need re-wiring? — Affects estimate for P00-01..P00-08.
- [ ] **`/ultrawork` concurrency cap** — confirm 4 is the right concurrency for P00-03..P00-07 vs. the team's preferred default. — Tuning, not blocking.
- [ ] **ADR target location** — should the final ADR land in `/Users/jasonlee/bominal/docs/adr/` or in the plan file itself? — Affects hand-off step.

## ralplan-oyatie-sst-consolidation — 2026-05-12

- [ ] **Helper implementation language for `tools/oya-agent-read/`** — Rust (matches workspace crate idiom) vs Node/TS (matches typical CLI helper idiom). — Decided at P2 scaffold time; affects symbol-claim identifiers and test runner.
- [ ] **Next free ADR slot numbers** — Plan assumes ADR-0026 (inventory) and ADR-0027 (cutover). Must be reconciled against `ADR-INDEX.md` at write time; bump if taken. — Mechanical, blocks P1 + ADR landing.
- [ ] **Human-orchestrator carve-out scope** — P6/P7 file moves and P9 upstream issue filing require non-agent invocations of `git mv`/`git rm`/`gh issue create`. Confirm the rule reads as "agents do not invoke git/gh"; humans orchestrating the cutover do. — Flagged inline at P6/P7/P9; affects whether the cutover can proceed at all.
- [ ] **CI extension to flag archive-path tokens** — Pre-mortem #2 mitigation requires the banned-primitives fitness lane to also grep for archive-path tokens before P7 merges. Confirm the lane crate's scope. — Affects P7 gating.
- [ ] **Demo symbol selection** — P8 demo must claim two real grit-indexed `file::Identifier` symbols in non-overlapping crates. Specific symbols TBD at demo-script-author time. — Affects P8 reproducibility.
- [ ] **Archive retention policy** — ADR Follow-up 3 proposes 60 days. Confirm duration with user before policy lands. — Tuning, not blocking.
- [ ] **`oya-agent-write` future surface** — Spec §Assumptions item 10 floats `oya-agent-write pr-finalize` if `grit done` is local-only. Defer to P9 follow-up; out of scope for this cutover. — Tracking.
