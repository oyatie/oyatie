# Oyatie agent guidance

Redirect-class root hub. Read `/specs/cross-cutting/root-hub-pointers.json` first; `docs/AGENTS.md` is the operating contract until PHASE-5 promotes `/specs/cross-cutting/agent-operating-contract.json`.

Pointers: `/specs/cross-cutting/master-plan-sequencing.json`; `/specs/cross-cutting/markdown-retirement-policy.json`; `/specs/cross-cutting/gitops-vcs-replacement.json`.

Agent-executable instructions are fenced for the banned-primitives lane.

<!-- agent-instructions:start -->
sanctioned_primitives:
  - oya-vcs
  - oya-vcs-admission
  - legacy-grit-compat
required_sequence:
  - oya vcs claim --agent <id> --intent "<slice>" <file::Identifier>
  - oya vcs verify --agent <id> --changeset <id>
  - oya vcs done --agent <id> --changeset <id>
  - oya vcs promote --changeset <id>
scaffold_fallback:
  topic: scaffold-locks-oyatie
  adr: docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md
compatibility_note: grit/icm/rtk/vox/omx/omc are legacy compatibility/provenance surfaces only while Oya VCS command adapters finish landing; they are not forward closure authority.
<!-- agent-instructions:end -->
