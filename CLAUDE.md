# Oyatie Claude guidance

Redirect-class root hub. Read `/specs/cross-cutting/root-hub-pointers.json` first; `docs/AGENTS.md` is the operating contract until PHASE-5 promotes `/specs/cross-cutting/agent-operating-contract.json`.

Pointers: `/specs/cross-cutting/master-plan-sequencing.json`; `/specs/cross-cutting/markdown-retirement-policy.json`; `docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md`.

Agent-executable instructions are fenced for the agent-coordination lane. Human terminal shortcuts belong outside this fenced agent surface.

<!-- agent-instructions:start -->
Use plain `git` + `gh` + the **Foundry pipeline** (admission-gate → merge-queue → auto-merge on green; M-CC-P11 substrate). External agent-coordination tooling (grit, rtk, icm, vox) is retired per ADR-0116.

required_workflow:
  - git worktree add (Layer 0 isolation; one worktree per agent)
  - gh pr create --base dev (Layer 2 PR enters the Foundry pipeline)
  - admission-gate + merge-queue (ADR-0111 conflict-kernel + projected-merge-state) order and admit
  - auto-merge once reviewer-agent APPROVE + CI green

substrate_adrs:
  - docs/decisions/ADR-0110-foundry-pipeline-webhook-receiver.md
  - docs/decisions/ADR-0111-merge-queue-conflict-kernel.md
  - docs/decisions/ADR-0112-changeset-state.md
  - docs/decisions/ADR-0113-review-mergequeue-kernel.md
  - docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
<!-- agent-instructions:end -->
