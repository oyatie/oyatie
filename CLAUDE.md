# Oyatie Claude guidance

Redirect-class root hub. Read `/specs/cross-cutting/root-hub-pointers.json` first; `docs/AGENTS.md` is the operating contract until PHASE-5 promotes `/specs/cross-cutting/agent-operating-contract.json`.

Pointers: `/specs/cross-cutting/master-plan-sequencing.json`; `/specs/cross-cutting/markdown-retirement-policy.json`; `docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md`.

Agent-executable instructions are fenced for the agent-coordination lane. Human terminal shortcuts belong outside this fenced agent surface.

Manual Wave-B bootstrap note (prose only): until the webhook receiver is deployed and registered, agents enter the Foundry pipeline by creating an isolated worktree branch and opening a pull request against `dev`; ADR-0116 explains this temporary seam.

<!-- agent-instructions:start -->
coordination_surface: foundry_pipeline
retirement_adr: docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
retired_external_agent_coordination_tooling: true

required_workflow:
  - layer_0_isolation: one isolated worktree per agent lane
  - layer_2_entry: pull request against dev enters the Foundry pipeline
  - admission_gate: validate policy, evidence, and required status checks
  - merge_queue: order and admit via ADR-0111 projected merge state
  - completion_gate: reviewer-agent APPROVE plus CI green before auto-merge

substrate_adrs:
  - docs/decisions/ADR-0110-changeset-state-machine.md
  - docs/decisions/ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
  - docs/decisions/ADR-0112-webhook-driven-foundry-agent-invocation.md
  - docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md
  - docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
<!-- agent-instructions:end -->
