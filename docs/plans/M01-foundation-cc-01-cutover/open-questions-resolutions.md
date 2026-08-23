---
doc_status: published
---

# Open-questions resolutions — ralplan-oyatie-sst-consolidation

<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

Resolved during Architect run, between Planner and Critic phases. The Critic SHOULD read this alongside the plan and open-questions.md.

---

## Q2 — Next free ADR slot numbers (MECHANICAL — resolved)

**Plan assumed**: ADR-0026 (inventory), ADR-0027 (cutover).
**Actual**: Highest existing is **ADR-0051** (`mobile-and-native-client-strategy.md`). Plan's assumption is wrong by ~25 slots.

**Correct slot allocation:**

Update plan §P1 and §P2 and §"ADR Block" to use these slot numbers. Plus `ADR-INDEX.md` must be appended in the same PR per spec §Acceptance Criteria A2/A9.

Evidence: `ls docs/decisions/ | tail -15` shows ADR-0049, ADR-0050, ADR-0051 and the README/RETIRED files.

---

## Q1 — Helper implementation language for `tools/agent-read/` (RECOMMENDED — Rust)

**Choice: Rust.** Rationale:

1. Workspace idiom — every other crate in this repo is `oyatie-*` Rust (140+ crates per `Cargo.toml`).
2. The helper emits audit-chain events, which are typed in the existing `platform-audit-chain-kernel`. Reusing the kernel from Rust is direct; from Node/TS it requires an FFI or a re-implementation.
3. Distribution: Rust binaries are static and trivially shippable; no Node runtime dependency for agent boxes.
4. Fitness lanes (`governance-*`) are Rust kernels; banned-primitives lane (P7 §"Outputs") would be a sibling.

Crate naming follows the flat-crates convention: `tooling-agent-read` (kernel) + `tooling-agent-read-cli` (binary). Actually — the simpler shape per Linus discipline: a single binary crate `tooling-agent-read` with `src/main.rs` and a small `lib.rs` for the readable surface. Defer the kernel/cli split until evidence justifies it.

The plan's P2 should be updated to enumerate Rust file claims rather than language-agnostic placeholders. Since `tools/tooling-agent-read/` does not exist yet, the chicken-and-egg applies — see Q3 resolution.

---


**Verified findings:**
- `tools/` directory does NOT exist in the oyatie repo (`ls -d tools/` returns "tools/ does NOT exist").
- Therefore the scaffold-claim pattern's PRIMARY option (`Cargo.toml::workspace_members`) is **not viable**.

**Resolution:** The fallback from `docs/plans/M01-foundation-cc-01-cutover/pre-cutover-drafts.md §Draft 2` is the actual path:

```
     -c "agent=<id> path=tools/tooling-agent-read window=open started_at=<ts>" \
     -i critical \
     -k "scaffold-lock,tooling-agent-read,open"
2. <agent creates tools/tooling-agent-read/{Cargo.toml, src/main.rs, src/lib.rs}>
3. <agent appends "tools/tooling-agent-read" to Cargo.toml workspace.members>
     -c "agent=<id> path=tools/tooling-agent-read window=closed finished_at=<ts>" \
     -i critical \
     -k "scaffold-lock,tooling-agent-read,closed"
```

Plan should be updated to:
- Move the scaffold-claim pattern from "alternative" to "canonical for new-crate phases" (P2, and any future new-crate phase).


---

## Q3 (from open-questions.md) — Human-orchestrator carve-out scope (POLICY — pending user)

The plan flags P6/P7/P9 as requiring `git mv` (archive), `git rm` (delete), `gh issue create` (file upstream bug). Spec rule: "agents do not use git/gh". The Planner interprets this as "humans orchestrating the cutover may." This is consistent with the spec's §Constraints item 1 wording ("Agent-side git/gh is banned") which scopes the ban to agents, not humans.

**Defer to user confirmation** but proceed under the interpretation that human-orchestrator git invocations are permitted for the three flagged events. The `agent-read` helper does NOT need a write counterpart for these — they are explicitly human steps.

Mitigation: each carve-out emits `BLOCKED_ON_HUMAN_ORCHESTRATOR` in the autopilot worker prompt.

---

## Q4 — CI extension to flag archive-path tokens (CONFIRMED — into `governance-banned-primitives`)


---

## Q5 — Demo symbol selection (RECOMMENDED — billing-app symbols)

The pre-cutover demo script (`docs/plans/M01-foundation-cc-01-cutover/pre-cutover-drafts.md §Draft 3`) uses:


---

## Q6 — Archive retention policy (RECOMMENDED — 90 days, then DELETE)

60 days is short; 90 days gives one quarter for any rollback decision. Land in the cutover ADR (ADR-0053) Follow-up section. **Defer to user** if they prefer otherwise.

---

## Q7 — `agent-write` future surface (CONFIRMED OUT-OF-SCOPE for this cutover)


---

## Summary impact on the plan

The Critic will see these mechanical revisions are non-controversial:
- ADR slot numbers fixed (mechanical).
- Helper crate language fixed to Rust (idiom-consistent).
- Scaffold-claim pattern moved from alternative to canonical for P2 + promoted to ADR-0054 (verified-finding-driven; not optional).
- Human-orchestrator carve-out interpretation made explicit (spec-text-consistent).
- Banned-primitives lane scope extended (small, contained).
- Demo symbols pre-selected (verified-indexed).

Two items still need user input (flagged in the plan's open-questions.md):
- Q3 (carve-out scope) — proceed under stated interpretation pending explicit confirmation.
- Q6 (retention policy) — proceed with 90 days pending explicit confirmation.

Critic SHOULD verify the plan's revised text incorporates these resolutions before issuing APPROVE.
