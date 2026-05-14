# Oyatie agent guidance

This repo-root file is Redirect-class and intentionally lane-thin. **Agents: read [`.omc/specs/root-hub-pointers.json`](.omc/specs/root-hub-pointers.json) FIRST** — the canonical machine-readable entry-point registry. Markdown-retirement is active per [`.omc/specs/markdown-retirement-policy.json`](.omc/specs/markdown-retirement-policy.json); see ADR-0069.

Canonical authority (current paths; migrating per phases in retirement policy):
- Agent operating contract: [`docs/AGENTS.md`](docs/AGENTS.md) (→ `.omc/specs/agent-operating-contract.json` PHASE-5; will absorb decision principles from retired Constitution per user directive 2026-05-13)
- Doc lifecycle: [`docs/DOC-CATALOG.md`](docs/DOC-CATALOG.md) (→ `.omc/registries/doc-catalog.json` PHASE-5)
- Flat-crates: [`docs/decisions/ADR-0015-architectural-flattening-target.md`](docs/decisions/ADR-0015-architectural-flattening-target.md) (→ `.omc/registries/adrs.json` PHASE-4)
- Active-artifact contract v3.0.0: [`.omc/specs/active-machine-readable-artifact-contract.json`](.omc/specs/active-machine-readable-artifact-contract.json)
- Knowledge-graph catalog (Ontology): [`.omc/registries/knowledge-graph-catalog.json`](.omc/registries/knowledge-graph-catalog.json)
- Capability registry: [`.omc/registries/artifact-capabilities-registry.json`](.omc/registries/artifact-capabilities-registry.json)
- Reusable building blocks (DRY): [`.omc/registries/reusable-building-blocks-registry.json`](.omc/registries/reusable-building-blocks-registry.json)
- Master-plan sequencing (grit protocol + forbidden primitives): [`.omc/specs/master-plan-sequencing.json`](.omc/specs/master-plan-sequencing.json)
- Hyperscaler gates + claim matrix: [`.omc/specs/hyperscaler-gates.json`](.omc/specs/hyperscaler-gates.json)
- Evidence taxonomy: [`.omc/specs/evidence-taxonomy.json`](.omc/specs/evidence-taxonomy.json)
- Stop conditions: [`.omc/specs/stop-conditions.json`](.omc/specs/stop-conditions.json)
- Final-report schema: [`.omc/specs/final-report-schema.json`](.omc/specs/final-report-schema.json)
- Plan-schema (for ralplan instances): [`.omc/specs/plan-schema.json`](.omc/specs/plan-schema.json)

Before changing this repo, read the contract and follow its checklist.

<!-- icm:start -->
## Persistent memory (ICM) — MANDATORY

This project uses [ICM](https://github.com/rtk-ai/icm) for persistent memory across sessions.
You MUST use it actively. Not optional.

### Recall (before starting work)
```bash
icm recall "query"                        # search memories
icm recall "query" -t "topic-name"        # filter by topic
icm recall-context "query" --limit 5      # formatted for prompt injection
```

### Store — MANDATORY triggers
You MUST call `icm store` when ANY of the following happens:
1. **Error resolved** → `icm store -t errors-resolved -c "description" -i high -k "keyword1,keyword2"`
2. **Architecture/design decision** → `icm store -t decisions-{project} -c "description" -i high`
3. **User preference discovered** → `icm store -t preferences -c "description" -i critical`
4. **Significant task completed** → `icm store -t context-{project} -c "summary of work done" -i high`
5. **Conversation exceeds ~20 tool calls without a store** → store a progress summary

Do this BEFORE responding to the user. Not after. Not later. Immediately.

Do NOT store: trivial details, info already in CLAUDE.md, ephemeral state (build logs, git status).

### Other commands
```bash
icm update <id> -c "updated content"     # edit memory in-place
icm health                                # topic hygiene audit
icm topics                                # list all topics
```
<!-- icm:end -->
