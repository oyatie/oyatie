# Oyatie fresh-session handoff

This founder-authorized root file is a **thin redirect**, not a plan, backlog, baseline-SHA,
completion, or delivery-status authority. Historical handoff content remains available in Git
history.

For every fresh session, read current truth in this order:

1. [`specs/root-hub-pointers.json`](specs/root-hub-pointers.json) — canonical authority router.
2. [`docs/AGENTS.md`](docs/AGENTS.md) — live operating contract until explicit PHASE-5 promotion
   evidence elevates `specs/agent-operating-contract.json`.
3. [`specs/masterplan.json`](specs/masterplan.json), `masterplan_v2` — the only live plan authority;
   `specs/master-plan-sequencing.json` is compatibility/provenance only.
4. [`docs/decisions/README.md`](docs/decisions/README.md) and ADR frontmatter — decision lifecycle.
   A higher ADR number does not override an earlier Accepted ADR by itself. A newer ADR controls
   only when it is Accepted and carries an explicit `amends` or `supersedes` relation with the
   reciprocal lifecycle edge; Proposed or implementation-landed decisions remain nonbinding.
5. Live Git/GitHub state — derive the current baseline, PR state, review state, and
   `oya-ci-required` result; never copy them from this file.
6. [`evidence/consolidation/prewipe-session-continuity-20260804.json`](evidence/consolidation/prewipe-session-continuity-20260804.json)
   — immutable pre-wipe continuity snapshot and remote-artifact recovery index. Re-query every
   live state before acting; this snapshot is historical context, not plan or dispatch authority.

Current nonclaims entering planning are recorded inside `masterplan_v2`; no execution wave is
authorized merely because it appears in a handoff or because CI is green. Protected-PR admission,
review, rollout, and product-completion evidence remain separate gates.
