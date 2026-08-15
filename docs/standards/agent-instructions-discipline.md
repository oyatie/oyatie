---
purpose: "Cross-cutting discipline for `<!-- agent-instructions:start -->` / `<!-- agent-instructions:end -->` fences in IPs, runbooks, ADRs, and standards."
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Cross-cutting discipline for `<!-- agent-instructions:start -->` /
  `<!-- agent-instructions:end -->` fences in IPs, runbooks, ADRs, and standards.
  Defines the fence shape, the banned-token grep scope (refuses raw `git`/`gh`
  inside fences unless the documented-rationale flow per Directive 12 is
  followed), and the dual-audience requirement (every fenced block has adjacent
  plain-English prose). Implements MASTERPLAN §7 dual-audience contract.
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: oya-governance-agent-instructions-fence
enforcement_status:
  oya-governance-agent-instructions-fence: F-PENDING-AGENT-INSTRUCTIONS-FENCE (crate missing; tracked in registry/stub-audit/2026-05-17/missing-fitness-crates.json)
  oya-governance-dual-audience: F-PENDING-DUAL-AUDIENCE (crate missing)
  oya-governance-banned-primitives: existing (via AGENTS.md + git-workflow.md)
meta_policy: ADR-0133 (chained-enforcement planning contract, pending)
companion_docs:
  - docs/standards/doc-style.md
  - docs/standards/git-workflow.md
  - docs/standards/multi-agent-tool-map.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Agent Instructions Discipline

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

Per [`decision-principles.json`](../../specs/decision-principles.json) DP-04 (dual-audience for
instructions) and MASTERPLAN §7, every artifact that contains agent-
actionable instructions wraps them in a **stable HTML-comment fence**.
The fence is machine-extractable and runs the banned-token grep narrowly
so canonical prose remains free.

## 1. Fence shape

The fence is HTML comments. Exact form (case-sensitive, byte-stable):

```
<!-- agent-instructions:start -->
<commands, JSON, structured arguments>
<!-- agent-instructions:end -->
```

Rules:

1. The fence sits inside the body of a doc (IP, runbook, ADR, standard).
2. Multiple fences per doc are allowed (e.g., per-step in a long IP).
3. Fences MUST NOT nest.
4. Fences MUST NOT span H1 boundaries (one fence per section).
5. The opening and closing tokens MUST be byte-identical to the form
   above; whitespace MAY appear around them but not inside the marker.

Lane: `oya-governance-agent-instructions-fence` validates the
mechanical shape.

## 2. What goes inside the fence

Inside a fence, the content is **agent-readable**:

  `git`/`gh` per [`git-workflow.md`](git-workflow.md) Directive 12).
- JSON payloads (capability records, audit-event templates).
- Per-tool harness arguments (Claude Code `Skill(skill="...")`, OMC
  `/oh-my-claudecode:autopilot`, etc.).

What does NOT go inside the fence:

- Explanatory prose ("First we do X because Y, then Z because W") —
  this lives OUTSIDE the fence per §4.
- Decorative emoji, marketing language, sales narrative.
- Editorial commentary or "history" prose.

## 3. Banned-token grep scope

The lane `oya-governance-banned-primitives` greps INSIDE fences for
tokens that violate the sanctioned-primitive triad or the user-machine
boundary.

| Token inside fence | Verdict |
|---|---|
| `--no-verify` | **FAIL** (unconditional) |
| `git push --force` / `--force-with-lease` to `main` | **FAIL** (unconditional) |
| `~/.claude/` mutation | **FAIL** (unconditional) |
| `~/.codex/` mutation | **FAIL** (unconditional) |
| `cargo ...`, `cosign ...`, `syft ...` | PASS |
| `gh pr merge` without `## Code Review` in target PR | **FAIL** |
| Bare `curl` / `wget` to external endpoints | **FAIL** (use a sanctioned MCP / `oya-tooling-agent-read`) |
| Process kill via `kill -9 $(pgrep claude)` | **FAIL** (use `/oh-my-claudecode:cancel`) |

The grep is scoped to fence interiors only — banned tokens in the prose
sections (e.g., "agents should not run `git --no-verify`") are
intentional documentation and pass.

Source: [`docs/AGENTS.md`](../AGENTS.md) + [`git-workflow.md`](git-workflow.md);
historical Directive-12 wording was under retired `.omc/plans/**` provenance
(ADR-0619).

## 4. Dual-audience requirement

Per MASTERPLAN §7 and [`doc-style.md`](doc-style.md) §3, every fenced
block MUST be paired with adjacent plain-English prose that explains the
intent. The prose stands alone — a human reading only the prose
understands what the directive accomplishes without the fence body.

| Pattern | Pass? |
|---|---|
| Fence + adjacent ≥ 2 sentences of prose | YES |
| Fence with prose split by ≥ 1 H2/H3 boundary | NO — pair the fence inside the section |
| Fence with no prose, only the fence | NO — refused at PR review |
| Prose with no fence | YES — prose-only doc is fine |

Lane: `oya-governance-dual-audience` checks the adjacency rule.

## 5. Per-doc-class fence usage

| Doc class | Fence usage |
|---|---|
| Implementation Plan (IP) | mandatory — every step that an agent executes is fenced |
| Runbook | recommended for diagnostic / mitigation commands |
| ADR | rare — fences are decision-execution artifacts, not decision records |
| Standard (this directory) | rare — standards prescribe behavior, not direct execution |
| Tier-1 strategy (PRD, DESIGN) | forbidden — these are reading material, not execution |
| README, redirect files | forbidden |

## 6. Reviewer-agent inheritance

Per [`docs/AGENTS.md`](../AGENTS.md) §Per-change-class reviewer agents,
the reviewer-agent verdict goes in the PR body's `## Code Review`
section (NOT inside an agent-instructions fence). The fence is for
**execution-side instructions**; the verdict is **post-execution
attestation** and belongs in the canonical PR shape.

## 7. Cross-harness portability

Per [`multi-agent-tool-map.md`](multi-agent-tool-map.md):

- Fence content SHOULD use canonical operation names (`read_file`,
  `run_bash`, `task_delegate`) when the instruction may be executed by
  any harness.
- When the instruction is harness-specific, the fence content uses the
  harness's actual tool names (`Bash`, `Read`, `Skill`, etc.) AND the
  prose names the target harness ("for Claude Code agents: ...").
- The harness name appears in the prose, never inside the fence (the
  fence is the directive; the prose is the audience targeting).

## 8. Anti-patterns

1. **Fence with no prose.** Refused by `dual-audience` lane.
2. **Fence content with `git push --force` to dodge Directive 12.**
   Refused unconditionally.
   harness target; mixing breaks the cross-harness portability rule.
4. **Fence in a Tier-1 strategy doc** (PRD, DESIGN).
   Tier-1 docs are reading material. Move the directive to an IP /
   runbook and cite from the strategy doc.
5. **Nested fences.** Refused by the shape lane.
6. **Fence opening/closing with a typo** (`<!-- agent-instructions:starts -->`).
   Refused; the markers are byte-stable.
7. **Editorial prose inside the fence.** Move to the adjacent prose
   block.
8. **Decorative emoji or marketing language anywhere** in canonical docs.
9. **Stale fence** referencing retired symbols or capability IDs. The
   doc-catalog freshness lane catches stale references at the doc-class
   level.

## 9. Migration: pre-existing instructions

Documents authored before this standard sometimes carry agent
directives inline without the fence. Migration rule:

1. Wrap the directive in the fence.
2. Add the adjacent prose if missing.
3. Run the banned-token grep against the new fence; address any failures
   the call with a sanctioned primitive).
4. Add a CHANGELOG row noting the migration.

The migration window is M01-P08 (per MASTERPLAN §8). After M01-P08
sign-off, the `agent-instructions-fence` lane is fully blocking.

## 10. Worked example

The following block is a complete, conformant directive:

> performs the edit, runs the evidence test, stores the decision, and
> releases the lease. Throughout, the tooling triad is sufficient — no
> direct git or gh invocation is required.
>
> ```
> <!-- agent-instructions:start -->
> # ...edits via Read/Edit/Write...
>   -c "tenant-gate-check rejects cross-tenant retrieval" \
>   -i high -k "foundry,rag,tenant-gate"
> <!-- agent-instructions:end -->
> ```

## 11. Sources scanned

- [`docs/MASTERPLAN.md`](../MASTERPLAN.md) + [`/specs/masterplan.json`](../../specs/masterplan.json) (live plan authority); dual-audience via [`doc-style.md`](doc-style.md).
- [`decision-principles.json`](../../specs/decision-principles.json) DP-04 (dual audience).
- [`docs/AGENTS.md`](../AGENTS.md) §PR shape, §Per-agent appendices.
- [`docs/standards/git-workflow.md`](git-workflow.md).
- [ADR-0619](../decisions/ADR-0619-zero-live-context-retirement-of-external-agent-harness-brand.md) (no live external harness brand authority).
- Historical fence-shape origin notes under retired harness plan paths are provenance only.
