---
doc_class: Checklist
checklist_id: CHK-SWARM-RITUAL
status: Accepted
date: 2026-08-10
purpose: |
  Tier-2 per-dispatch ritual for Swarm Delivery Law. Tier-1 north-star is the
  live programme SSOT plan (trajectory). This checklist is what every
  implement / audit / review / plan / scout / recon agent runs at start and end
  of every dispatch. Babysit uses the short form (A1 + D only).
owner_team: axis-foundry
related:
  - docs/AGENTS.md
  - docs/checklists/agent-kickoff-checklist.md
  - docs/checklists/agent-completion-checklist.md
  - specs/integ-branch-envelopes.json
doc_status: published
---

# Swarm agent ritual v1 (run every dispatch)

Two instruction tiers exist. **Tier 1 (north-star):** the programme SSOT plan — where we are going, waves, topology, holds, tips. **Tier 2 (this card):** the ritual you run on every dispatch regardless of destination. Dispatch cards MUST cite this checklist + role + lane mission; they MUST NOT paste the whole north-star plan.

Trivial read-only / babysit tasks MAY use the short form (**A1 + D** only).

## Visual forms (where applicable)

| Form | Shape | When required |
|------|-------|---------------|
| **Position diagram** | Mermaid flowchart, 4–8 nodes: inputs → you → output → consumer | Always for productive roles. Cannot name consumer = orphan work → STOP. |
| **Claim digraph** | Directed graph: claim nodes; edges = depends-on / causes / contradicts; each node tagged `E`vidence / `I`nference / `U`ncertainty | When ≥2 load-bearing claims have dependency or causal structure. Single-claim scout/recon MAY collapse to a tagged list. Digraph is the reasoning map — not decoration. |

Role-scaled evidence:

| Role | Minimum evidence |
|------|------------------|
| scout / recon | diagram + digraph-or-tagged-list + findings map |
| implement / fix | diagram + digraph + doctrine cites + Claim/`docs_touched[]` packet |
| audit / review | digraph + ≥1 real `challenges[]` (adversarial) |
| planner | diagram + digraph + northstar delta (edit the live SSOT — no +N plan) |
| babysit | window receipt only (short form); MUST NOT block fan-out |

## A. Orient (before your first tool call)

1. **Role declaration** — state which ONE role you hold: `implementer | critic | auditor | scout | recon | planner | babysit | orchestrator`. Name the one thing you MUST NOT do in that role (implementer: babysit CI / invent fleet policy; critic: implement the fix; scout: write product; babysit: write code; orchestrator: be the only live agent when ≥2 hot lanes exist).
2. **Position diagram** — draw your place in the flow (see Visual forms). If you cannot draw who consumes your output, STOP and elevate.
3. **Premise check** — verify ground truth against HEAD before acting (new HEAD = new evidence; a brief's SHA may be stale). State what you verified. Cite owning bead id (`.beads/` / `oyatie-*`); no bead → create/elevate, don't freelance. Recall hindsight before acting; never repeat a known-failed pattern without a recorded challenge.

## B. Reasoning map (before implementing / verdicting / planning)

1. **Lens selection** — pick the smallest task-fit subset (≥2) from `.grok/harness/lenses.v1.json`. Root-level keeps/overturns/pattern replacements get the full 16-lens battery; leaves inherit. High-risk domains (authz/migration/contracts/CI admission/compliance) MUST include Red Team, Operability, Blast-radius, Zero-trust or record durable N/A.
2. **Claim digraph** — build the digraph (or tagged list when single-claim). A verdict or plan built on untagged claims is narrative, not reasoning.
3. **Challenges[]** — record at least one real challenge to the brief/plan/ADR/tip you were handed (Chesterton first, then still replace if indefensible). Empty or templated challenges = stamp; reject your own work.

## C. Doctrine (during execution)

- Envelope-bounded (`specs/integ-branch-envelopes.json`); one writer per worktree; never the main checkout for foreign lanes.
- Evidence keyed to THIS tip SHA; commands recorded; observation (logs/CI/reviews) ≠ merge APPROVE.
- Friction (path outside envelope, hub collision, hook block, policy gap): **STOP → CAPTURE (worktree, tip, paths, exact error) → ELEVATE**. No workaround commits, no `--no-verify`, no scope expansion, no self-fix of process/hub/envelope law.
- Forbidden always: `*-fast` model slugs; prompt-theater Claim; dual-truth root/hub lists outside envelopes JSON; babysit-only fleet (if you are orchestrator and only a CI watch is live, dispatch ≥1 productive lane or declare PROGRAMME IDLE with reason).
- Docs packet on material change: `docs_touched[]` + `docs_action` (INV-DOC-1); load-bearing docs same-wave (INV-DOC-3).

## D. Close (before you exit)

1. **Receipt** — implementer: tip SHA + Claim/docs packet + Fix observation if fix; critic: verdict + proven-by-execution per finding; scout/auditor/recon: findings map with E/I/U tags; planner: updated SSOT section, not a new plan file; babysit: window receipt (hot-set, tip SHAs, CI conclusions keyed to tip).
2. **Flow delta** — one line: what changed for whoever consumes your output.
3. **Exit** — return and stop. No CI-watch loops after your receipt; watching belongs to babysit. If you learned a process defect, elevate one line for the SSOT — do not silently patch process. Retain lessons to hindsight and update bead state after friction/fix/OVERRULE.
