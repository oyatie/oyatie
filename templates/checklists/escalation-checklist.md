---
doc_class: Checklist
checklist_id: CHK-ESC
status: pending approval
purpose: |
  When an agent halts and emits `BLOCKED_ON_HUMAN_ORCHESTRATOR`. The autonomy directive minimizes this set: every documented case is one the system genuinely cannot resolve mechanically. New escalation classes require an ADR; council reviews quarterly to shrink the set.
lift_target: oyatie/templates/checklists/escalation.md
enforcing_fitness_lane: oya-governance-banned-primitives (audits halt events)
owner_team: council-architecture
related:
  - docs/AGENTS.md
  - docs/RACI-OWNERSHIP.md  # row: human-orchestrator-cutover + escalation contacts
  - docs/INCIDENT-MANAGEMENT.md
  - .omc/scratch/adr-draft-grit-icm-sanctioned-primitives.md  # §Glossary §Human orchestrator
  - /templates/checklists/agent-kickoff-checklist.md
  - /templates/checklists/agent-completion-checklist.md
---

# Escalation Checklist (BLOCKED_ON_HUMAN_ORCHESTRATOR)

> **Target: as few cases as possible.** Each case below is one the system genuinely cannot resolve without a human in the loop. Every new escalation class **MUST** be added via ADR and reviewed for collapse-into-mechanical-prevention at the next quarterly council review.

## When an agent halts and emits `BLOCKED_ON_HUMAN_ORCHESTRATOR`

The full list of sanctioned escalation classes (steady-state):

### E1. Cutover one-time human-orchestrator carve-out
**Trigger:** Cutover phase requires `git mv` (P6 archive) / `git rm` (P7 delete) / `gh issue create` (P9 upstream-bug filing) per `.omc/scratch/adr-draft-grit-icm-sanctioned-primitives.md §Consequences §Neutral`. **Resolution:** Named human orchestrator (per `docs/RACI-OWNERSHIP.md` row `human-orchestrator-cutover`) executes the action; `icm store -t cutover-orchestrator-actions -c "<action>" -i critical` emitted BEFORE execution. **Mechanical prevention candidate:** Add `grit mv`, `grit rm`, `oya-tooling-agent-read issue-create` to sanctioned set in future ADR. *Target collapse: yes.*

### E2. Authority-class doc edit
**Trigger:** A change to `docs/CONSTITUTION.md`, `docs/PRD.md`, `docs/MASTERPLAN.md` (council-only docs per `docs/DOC-CATALOG.md §agent_authoring_allowed: NO`). **Resolution:** Halt; emit issue request to council-architecture; council member edits manually. **Mechanical prevention candidate:** None (intentional — authority-class docs are human-only by design).

### E3. Sev-1 / Sev-2 incident declared
**Trigger:** SLO burn rate crosses 14x; or audit-chain emission failure; or tenant-boundary breach attempt; or regulator notification window opens. **Resolution:** Halt all in-flight agent work; page incident commander per `docs/INCIDENT-MANAGEMENT.md §Escalation`. *Target collapse: no — Sev-1/2 always human-led.*

### E4. Risky-action confirmation
**Trigger:** Action that would (a) force-push to `main`, (b) hard-reset another worker's branch, (c) downgrade a package, (d) migrate to shared infra without explicit user scope, (e) send external messages. *(Per `docs/AGENTS.md §Boundaries`.)* **Resolution:** Halt; request explicit user authorization scoped to the action. **Mechanical prevention candidate:** Already enforced by `oya-governance-bypass`; the halt is the **prevention** working as designed.

### E5. Stale forward-reference resolution
**Trigger:** Agent encounters `<!-- forward-reference: wave-N -->` pointing at an artifact that should exist by current wave but doesn't. **Resolution:** Halt; emit issue to wave-owner team; do not silently invent the missing artifact. **Mechanical prevention candidate:** Tighter `oya-governance-forward-reference` wave-window enforcement. *Target collapse: yes.*

### E6. IP frontmatter `final_shape_compliance: false`
**Trigger:** Agent picks an IP whose frontmatter declares MVP-shaped scope. **Resolution:** Refuse to claim; route to council-architecture for IP rewrite. **Mechanical prevention candidate:** None (intentional — `final_shape_compliance: false` would itself violate Master Plan principle 3; the IP should never exist in this state).

### E7. Capability tier uplift beyond Cedar policy
**Trigger:** Capability requires T2 → T3 uplift, but the Cedar policy + runtime gate is absent or insufficient. **Resolution:** Halt; route to ops-security + axis-foundry to author the Cedar policy + runtime check. **Mechanical prevention:** `oya-governance-autonomy-ceiling` already refuses the uplift; the halt is the prevention.

### E8. Cross-axis contract decline
**Trigger:** Consumer-axis team declines a cross-axis contract change without an alternative path. **Resolution:** Halt; council-architecture mediates per `cross-axis-contract-change-checklist.md`. *Target collapse: no — design conflict is human-mediated.*

### E9. Upstream tool bug blocking sanctioned primitive
**Trigger:** `grit session start` bug (or analogous) blocks the sanctioned pipeline. **Resolution:** Halt sanctioned flow; switch to documented workaround per `.omc/scratch/pre-cutover-drafts-2026-05-12.md §Draft 1`; emit `icm store -t upstream-tool-bugs -c "<one line>" -i high`; human orchestrator files upstream issue. *Target collapse: yes (when upstream ships fix).*

### E10. ICM / audit-chain unreachable
**Trigger:** `icm store` fails repeatedly, OR audit-chain emission endpoint returns errors. **Resolution:** Halt all in-flight work; do not silently proceed without audit trail. Route to ops-sre-reliability. *Target collapse: no — by design.*

## How to emit `BLOCKED_ON_HUMAN_ORCHESTRATOR`

<!-- agent-instructions:start -->
```
icm store \
  -t blocked-on-human \
  -c "<one-line description of block; cite escalation class E1..E10>" \
  -i critical \
  -k "BLOCKED,<escalation-class>,<axis>"
```

Then halt the loop:
- If inside Ralph / autopilot / ultrawork / team: do **NOT** `/oh-my-claudecode:cancel` silently; re-walk `done-definition-checklist.md` for the current change, then cancel with a comment naming the block.
- If at IP boundary: **DO NOT** `grit done`; the symbols remain claimed-and-stalled until a human resolves the block or revokes the claim via `grit force-release` (human orchestrator only).
- Emit issue request via `oya-tooling-agent-read issue-create --title "BLOCKED: <one-line>" --body "<context>"` if a tracking issue is needed.
<!-- agent-instructions:end -->

## Anti-patterns

- Inventing an 11th escalation class on the fly — author an ADR instead.
- Halting silently (no `icm store`) — the absence of an audit row is itself a `MISTAKES-LEDGER` row trigger.
- Halting because of a transient (single-retry-recoverable) failure — retry once with `icm store -t errors-resolved`; only halt if the retry fails.
- Re-claiming a symbol that's flagged `BLOCKED` — another agent / human orchestrator owns resolution.

## Quarterly council review

Per `docs/standards/prevention-doctrine.md §6`, the council reviews this list quarterly. Each class is evaluated for: (a) frequency of trigger, (b) mechanical-prevention candidacy, (c) collapse into a sanctioned primitive. The goal is to **shrink** this list over time.
