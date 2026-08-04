# RALPLAN-DR — Agent dispatch protocol (codex-exec era) — REVISED r3 — pending approval

Consensus history: Planner r1 → fable Architect: SOUND-WITH-AMENDMENTS (6) → codex Critic r1: ITERATE (12) → Planner r2 (amendments landed) → codex Critic r2: ITERATE (collision-model contradiction + A1/A5 operability + INDEX pointer) → Planner r3 (this revision) → confirmation pass pending.

r3 changes: TWO-TIER collision model (HARD = sequenced at dispatch: in-place path overlap, ADR numbering, workflow/config registries, shared-lib edits; SOFT = Cargo.lock + generated faces: parallel dispatch, train-order merge — mechanically regenerable via #661 lock driver + #668 settle tool, so the collision is a cheap mechanical rebase, not a semantic conflict) landed in PREAMBLE §3.1 + merge-train "Concurrency bound"; ledger reseeded to schema v2 (expected_surfaces:{hard,soft}; escalated rows carry {reason,trigger}; intervention rows are first-class — zero such rows between dispatched and pr-open IS A1's evidence source); INDEX architect-verdict pointer fixed.

## Principles (5) — unchanged
P1 structural-over-convention · P2 fresh-context adversarial verification, narration ≠ evidence · P3 minimum-communication autonomy · P4 owned-destination + hyperscaler-precedent architecture · P5 dispatch AND orchestration reproducible from durable artifacts (r2: orchestration added per Architect A1/Critic 2).

## Decision Drivers — unchanged (D1 leader-attention, D2 review latency, D3 session-spanning continuity)
r2 note per Critic 7: D2 is re-framed — the worker-side pass is a mandatory pre-open FILTER that raises PR intake quality; review latency compression is a side effect, never the justification, and merge authority is exclusively the leader's fresh-context verification.

## Options (r2 — rewritten around disqualifying PROPERTIES per Critic 3)
O1 (CHOSEN, as amended): headless one-shot stateless workers + file contracts + durable dispatch ledger + pre-open filter + leader verification authority + projected-state merge admission (manual adapter until Tide).
O2 (rejected): any INTERACTIVE-PANE runtime (tmux codex or claude) — disqualifying property: live-session statefulness (nudge plumbing, lease expiry, pane lifecycle) whose failure cluster is the largest in the friction ledger (FRIC-016/-1781065651/-1781063713 + the 5-row lease-deadlock family). NOT rejected for being Claude: headless fable (`claude -p`) remains in use for planning subagents.
O3 (rejected for lane execution): in-session Workflow-engine pipelines — disqualifying property: lane work requires multi-hour spans with git/PR side effects owned by a process that outlives leader turns; retained for bounded read-only fan-outs (research, review panels) where it is already proven (ADR-0541 corpus, 104 agents).

## Risk → LANDED mitigation map (r2; every row names file:section)
R1 self-confirmation → pre-open FILTER doctrine + SHA-pinned tracked review-verdict.txt + leader authority: TEAMMATE-PREAMBLE §2 (filter bullet); RUBRIC unchanged as the fixed standard.
R2 one-shot wrong-premise burn → premise gate (premise.txt, first action, stop+escalate on failure): PREAMBLE §3 bullet 1.
R3 under-inclusive escalation → sixth trigger (security-sensitive = immediate): PREAMBLE §3 escalation bullet.
R4 collision-by-convention → dispatch-ledger expected_surfaces + named collision domains + dispatch-time disjointness/sequencing rule: PREAMBLE §3.1; merge-train.md "Concurrency bound".
R5 unverifiable skill propagation → required final-message skills-attestation field (loaded/applied + nested-prompt text): PREAMBLE §3 final-message fields. (Honest limit: self-attested, spot-checked by leader — mechanical verification deferred.)
A1 leader durability (Architect) → append-only dispatch-ledger.jsonl (single-writer=leader; Rust appender queued on zero-shell backlog per Critic 6) + PR-state-wins reconciliation: PREAMBLE §3.1; ledger seeded with in-flight lanes L-001/L-002.
A4 serial-train tail (Architect) → Tide-survivable admission contract + cutover assertion + concurrency bound: merge-train.md "Admission contract" + "Concurrency bound".
Critic 12 main-checkout contamination → ledger intake check (worktree must be non-canonical) PREAMBLE §3.1 + the mechanical guard itself = in-flight lane L-002 (BRIEF-g011-main-checkout-guard.md).

## Acceptance criteria (r2 — falsifiable, per Critic 5 rewrites)
A1. For the first two codex-exec lanes after adoption: zero `intervention` rows in dispatch-ledger.jsonl between each lane's `dispatched` and `pr-open` rows, except interventions responding to a ledger-recorded `escalated` row (the ledger IS the named evidence source; one-shot codex exec has no other leader→worker channel, so any intervention necessarily appears as a ledger-recorded action).
A2. Every PR accepted for review contains a tracked review-verdict.txt produced by a fresh reviewer process using RUBRIC-torvalds-review.md, pinned to the PR head SHA; PR body links the SHA and summarizes fixed BLOCK/HIGH findings.
A3. Given only INDEX.md + repo files at the adoption commit, a fresh reviewer reconstructs: read order, ledger fields, premise gate, pre-open filter, six escalation triggers, merge admission contract, final-message fields — zero chat history.
A4. Every normative MUST/NEVER/ONLY dispatch rule appears in an indexed durable file under .omc/ultragoal/; FAIL if any dispatch prompt carries a rule absent from those files.
A5. A fresh leader with repo + gh read access reconstructs every in-flight lane's {lane_id, brief, worktree, branch, base, expected_surfaces, status, PR number/state, escalations} from dispatch-ledger.jsonl + GitHub; PR state wins conflicts.

## Third-party verification steps (Critic e)
V1 `grep -n "premise.txt" TEAMMATE-PREAMBLE.md` → §3 with claim/verification/invalidating-fact fields. V2 `grep -n "review-verdict.txt" TEAMMATE-PREAMBLE.md` → SHA-pinned filter bullet. V3 `grep -n "security-sensitive" TEAMMATE-PREAMBLE.md` → trigger 6. V4 `grep -n "Admission contract" merge-train.md` → Tide cutover assertion. V5 `test -s dispatch-ledger.jsonl` + every row has expected_surfaces. V6 INDEX.md lists preamble, rubric, ledger, this plan.

Status: r2 pending approval (founder). Confirmation pass: codex critic re-check against this revision.
