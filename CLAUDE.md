# Oyatie Claude guidance

Redirect-class root hub. Read `.omc/specs/root-hub-pointers.json` first; `docs/AGENTS.md` is the operating contract until PHASE-5 promotes `.omc/specs/agent-operating-contract.json`.

Pointers: `.omc/specs/master-plan-sequencing.json`; `.omc/specs/markdown-retirement-policy.json`; `docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md`.

Agent-executable instructions are fenced for the banned-primitives lane. Human terminal shortcuts belong outside this fenced agent surface.

<!-- agent-instructions:start -->
sanctioned_primitives:
  - grit
  - icm
  - oya-tooling-agent-read
required_sequence:
  - icm recall-context "<task>" --limit 5
  - grit claim --agent <id> --intent "<slice>" <file::Identifier>
  - oya-tooling-agent-read diff --base <base> --head <head> --paths <path>
  - grit done --agent <id>
scaffold_fallback:
  topic: scaffold-locks-oyatie
  adr: docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md
<!-- agent-instructions:end -->
