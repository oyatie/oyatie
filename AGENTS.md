# Oyatie agent guidance

Redirect-class root hub. Read `/specs/root-hub-pointers.json` first; `docs/AGENTS.md` is the operating contract until PHASE-5 promotes `/specs/agent-operating-contract.json`.

Pointers: `/specs/master-plan-sequencing.json`; `/specs/markdown-retirement-policy.json`; `docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md`.

Agent-executable instructions are fenced for the agent-coordination lane. Human terminal shortcuts belong outside this fenced agent surface.

Manual Wave-B bootstrap note (prose only): until the webhook receiver is deployed and registered, agents enter the Foundry pipeline by creating an isolated worktree branch and opening a pull request against `dev`; ADR-0116 explains this temporary seam.

<!-- agent-instructions:start -->
sanctioned_primitives:
  - oya-vcs
  - oya-vcs-admission
required_sequence:
  - oya vcs claim --agent <id> --intent "<slice>" <file::Identifier>
  - oya vcs verify --agent <id> --changeset <id>
  - oya vcs done --agent <id> --changeset <id>
  - oya vcs promote --changeset <id>
scaffold_protocol:
  mechanism: per-agent isolated worktree plus admission-gate concurrent-safe-paths
  adr: docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
retirement_note: external agent-coordination tools are retired per ADR-0116; omx/omc/oya-tooling-agent-read remain compatibility/provenance-only during the cutover window. Oya VCS ChangeBundle -> Promotion -> ReleaseTrain is the forward closure authority.
<!-- agent-instructions:end -->
