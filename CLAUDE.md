# Oyatie Claude guidance

Redirect-class root hub. Read `/specs/cross-cutting/root-hub-pointers.json` first; `docs/AGENTS.md` is the operating contract until PHASE-5 promotes `/specs/cross-cutting/agent-operating-contract.json`.

Pointers: `/specs/cross-cutting/master-plan-sequencing.json`; `/specs/cross-cutting/markdown-retirement-policy.json`; `docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md`.

Agent-executable instructions are fenced for the banned-primitives lane. Human terminal shortcuts belong outside this fenced agent surface.

<!-- agent-instructions:start -->
sanctioned_primitives:
  - grit
  - oya-tooling-agent-read
required_sequence:
  - grit claim --agent <id> --intent "<slice>" <file::Identifier>
  - oya-tooling-agent-read diff --base <base> --head <head> --paths <path>
  - grit done --agent <id>
scaffold_fallback:
  topic: scaffold-locks-oyatie
  adr: docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md
sunset_note: grit + oya-tooling-agent-read are transitional. Both are scheduled to sunset once Oya VCS (M-CC-P00) and Foundry go live, at which point no external agent-coordination tooling (rtk, vox, icm, grit, omc, omx) remains in repo.
<!-- agent-instructions:end -->
