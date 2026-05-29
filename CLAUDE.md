# Oyatie Claude guidance

Redirect-class root hub. Read `/specs/root-hub-pointers.json` first; `docs/AGENTS.md` is the operating contract until PHASE-5 promotes `/specs/agent-operating-contract.json`.

Pointers: `/specs/master-plan-sequencing.json`; `/specs/markdown-retirement-policy.json`; `docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md`.

Agent-executable instructions are fenced for the agent-coordination lane. Human terminal shortcuts belong outside this fenced agent surface.

Manual Wave-B bootstrap note (prose only): until the webhook receiver is deployed and registered, agents enter the Foundry pipeline by creating an isolated worktree branch and opening a pull request against `dev`; ADR-0116 explains this temporary seam.

## Skill discovery doctrine (inherited)

Lifecycle skills + agent personas + intent→skill mapping are inherited from `addyosmani/agent-skills` (MIT), vendored at `tools/agent-skills/`. Universal skill catalog (`tools/agent-skills/skills/<name>/SKILL.md`), personas (`tools/agent-skills/agents/<role>.md`), and orchestration doctrine (`tools/agent-skills/AGENTS.md`) are the inherited base. Oyatie governance (`docs/AGENTS.md` operating contract + multispectrum review v2.4.0 + authority chain + Foundry pipeline + ADRs 0145+) OVERLAYS and WINS on conflict per `feedback_bominal_inheritance_precedence`. See `tools/agent-skills/INHERITANCE.md` for the full pattern, `tools/hooks/_canonical-primitives.md` for the Lifecycle Skill Map injected at SessionStart, and `tools/hook-bootstrap/install.sh` for the single-command bootstrap.

`tools/agent-skills/CLAUDE.md` is INFORMATIONAL only — it describes the vendored upstream subtree, not this oyatie repository. This file (root `CLAUDE.md`) remains the authoritative project-rules source.

<!-- agent-instructions:start -->
coordination_surface: governance_pipeline
retirement_adr: docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
retired_external_agent_coordination_tooling: true
observability_substrate: microservices/observability/ (per ADR-0130 agentic SLO-gated promotion + ADR-0131 per-microservice flat layout; SLO authoring at microservices/<ms>/slos/*.openslo.yaml mandatory before any µservice promotes past dev)
microservice_layout_authority: ADR-0131 per-microservice flat layout; new µservices ship under microservices/<ms>/ with src/ as the canonical code root
no_grouping_policy: ADR-0132 — no new bundle/grouping µservices; every new µservice is single-concern + flat
new_governance_lane_prefix: oya-governance-* (per ADR-0132); existing oya-governance-* lanes retained until each is renamed in its own migration IP

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
