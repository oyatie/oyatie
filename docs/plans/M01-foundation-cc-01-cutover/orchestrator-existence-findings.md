---
doc_status: published
---

# Orchestrator existence findings — ralplan iter-1 → iter-2 transition

<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

Captured 2026-05-12 between Architect ITERATE verdict and Critic dispatch. The Critic SHOULD read this; iter-2 Planner MUST incorporate.

---

## Verified file/directory state

| Path | Exists? | Source check |
|---|---|---|
| `oyatie/agents/settings/claude.settings.json` | **NO** | `ls -la agents/settings/` → "does NOT exist" |
| `bominal/agents/settings/claude.settings.json` | **NO** | `ls -la /Users/jasonlee/bominal/agents/settings/` → "does NOT exist" |
| `oyatie/.codex/worktree_init.sh` | **NO** | `ls -la .codex/` → empty |
| `oyatie/.claude/skills/` | **NO** | `ls .claude/skills/` → empty |
| `oyatie/Cargo.toml` contains `tooling-*` prefix | **YES** | `tooling-cli-dev-runtime` exists as sibling crate |
| `oyatie/docs/runbooks/foundry/` | **YES** | listed |
| `oyatie/docs/RACI-OWNERSHIP.md` (70 table rows) | **YES** | wc -l on `^|` lines = 70 |
| `oyatie/docs/products/foundry/PRD.md` (75.5KB) | **YES** | landing location for salvaged Phase 00 SPEC content |
| `oyatie/docs/products/foundry/SPEC.md` | **NO** | candidate new file for Phase 00 contract surface |

## Implications

### For spec A6 (hook + skill audit)


**iter-2 implication**: A6 must be rephrased in the plan. Either:
- Scope A6 to "all hook/skill prompts that an agent might invoke inside this repo" → grep `**/*.md`, `**/*.json`, `**/.claude/**` for `git`/`gh` references in agent-instruction sections.

iter-2 Planner: pick one; document the choice in the plan; do not leave the path ambiguous.

### For the deletion target list

`.codex/worktree_init.sh` is listed in the spec's §Goal/Layer-2 and the trace as a deletion target — but it does not exist. Drop it from the deletion list; the inventory ADR (ADR-0052) should reflect "not present, no action."

### For naming the new helper crate

`tooling-agent-read` is consistent with the existing `tooling-cli-dev-runtime` sibling. Confirmed safe.

### For the demo runbook (A7 / P8)

`docs/runbooks/foundry/` exists. The parallel-claim demo runbook can land at one of:

**Recommendation**: `docs/runbooks/agentic-pipeline/` — the demo is about the *pipeline*, not about any product axis. Create the subdir as part of P8.

### For Architect revision request #5 (human-orchestrator definition)


### For the foundry salvage landing

`docs/products/foundry/PRD.md` is 75.5KB — already substantial. The salvaged Phase 00 contracts should NOT be inlined into PRD (which is product-level). They should land as a new `docs/products/foundry/PHASE-00-SPEC.md` (per `docs/products/foundry/PHASE-00-SPEC.md` — ADR-0052 classifies this as a new KEEP artifact).

**Recommendation**: `docs/products/foundry/PHASE-00-SPEC.md` containing axis-internal contract definitions (ProviderAccount, AuthSession, UsageWindow, SecretReference, provider-gateway etc.). Top-level `docs/SPEC.md §6` continues to enumerate surfaces; product-level `docs/products/foundry/PHASE-00-SPEC.md` defines their kernel types.

iter-2 Planner: P3.5 should pin this landing location.

---

## Summary action for iter-2

iter-2 Planner must, in addition to Architect revisions 1-8 and the new P3.5:
- Rephrase A6 (hook path reconciliation) per above.
- Drop `.codex/worktree_init.sh` from deletion list (it doesn't exist).
- Use `tooling-agent-read` as the helper crate name.
- Pin foundry salvage landing to `docs/products/foundry/PHASE-00-SPEC.md`.
- Add `human orchestrator` row to `docs/RACI-OWNERSHIP.md` as part of P0.5 or P1 deliverables.
