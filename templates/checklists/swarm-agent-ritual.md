---
doc_class: Checklist
checklist_id: CHK-SWARM-RITUAL
status: Accepted
purpose: |
  Forever home for the per-dispatch Tier-2 agent ritual (INV-DOC-9).
  Session-loaded process_meta rules under `.cursor/rules/` MUST cite this path.
  Brand-free: no third-party harness brand names as live coordination authority.
owner_team: platform-governance
related:
  - docs/AGENTS.md
  - specs/integ-branch-envelopes.json
---

# Swarm agent ritual (Tier 2)

Two instruction tiers exist.

- **Tier 1 (north-star):** the programme SSOT plan — where we are going, waves, topology.
- **Tier 2 (this checklist):** the ritual every implement / audit / review / plan / scout / recon dispatch runs at start and end. Trivial read-only / babysit tasks MAY use the short form (A1 + D only).

Session-loaded copy rides process_meta `.cursor/rules/` on `integ/ci` and **MUST cite this file** as the canonical in-repo copy (INV-DOC-9). Dispatch cards cite this checklist + role + lane mission — not the whole programme SSOT.

Operating-contract pointer: `docs/AGENTS.md` § Doctrine survival → Per-dispatch ritual.

## Visual forms (where applicable)

- **Position diagram** (mermaid flowchart, 4–8 nodes): inputs → you → output → consumer. Always for productive roles. No named consumer = orphan work → STOP / elevate.
- **Claim digraph** (directed graph): claim nodes; edges = depends-on / causes / contradicts; tag each node `E`vidence / `I`nference / `U`ncertainty. Required when ≥2 load-bearing claims have dependency/causal structure; single-claim scout/recon MAY use a tagged list. Digraph = reasoning map, not decoration.

## A. Orient (before your first tool call)

1. **Role declaration** — ONE role: `implementer | critic | auditor | scout | recon | planner | babysit | orchestrator`. Name the one thing that role MUST NOT do.
2. **Position diagram** — draw your place in the flow. Cannot name consumer → STOP / elevate.
3. **Premise check** — verify against HEAD (new HEAD = new evidence). Cite owning bead; recall hindsight; do not repeat known-failed patterns without a recorded challenge.

## B. Reasoning map (before implementing / verdicting / planning)

1. **Lens selection** — ≥2 task-fit lenses (`.grok/harness/lenses.v1.json`); full 16 only at root decisions; leaves inherit.
2. **Claim digraph** — build digraph (or tagged list if single-claim). Untagged claims = narrative, not reasoning.
3. **Challenges[]** — ≥1 real challenge (Chesterton first). Empty / templated = stamp; reject your own work.

## C. Doctrine (during execution)

- Envelope-bounded; one writer per worktree; never main checkout for foreign lanes.
- Evidence keyed to THIS tip SHA; observation ≠ merge APPROVE.
- Friction → STOP → CAPTURE → ELEVATE. No `--no-verify`, no process self-fix, no scope expansion.
- Forbidden: `*-fast` model slugs; prompt-theater Claim; dual-truth enumerations outside envelopes JSON; babysit-only fleet (orchestrator must fan out or declare PROGRAMME IDLE).
- Advisory accelerators (if present) complement babysit / `gh` / agents only — never hard dependency, never Claim APPROVE / land / firewall weaken.
- Material change: `docs_touched[]` + `docs_action` (INV-DOC-1); load-bearing docs same-wave (INV-DOC-3).

## D. Close (before you exit)

1. **Receipt** — role-scaled evidence (below).
2. **Flow delta** — one line for your consumer.
3. **Exit** — no CI-watch loops; babysit owns waiting. Elevate process defects; retain to hindsight; update bead.

### Role-scaled receipt evidence

| Role | Minimum receipt |
|---|---|
| implementer | tip SHA; commands + pass/fail; `docs_touched[]` + `docs_action`; out-of-envelope elevates |
| critic / auditor | verdict; challenged claims; concrete defects or explicit none-found with scope |
| scout / recon | findings only; paths/SHAs; no edits claimed |
| planner | decisions + overturn conditions; next consumer named |
| babysit | observed state SHA/PR/check; no merge authority claimed |
| orchestrator | fan-out / idle declaration; consumer of each seat named |

## Hindsight + beads (binding, every role)

At Design: cite owning bead work-item id — no bead → create/elevate, don't freelance; consult hindsight before acting; never repeat a known-failed pattern without a recorded challenge.

At Operate: retain the lesson and update bead state after friction / fix / OVERRULE. Recalled facts are tips, not truth — re-verify stale SHAs (new HEAD → new evidence).
