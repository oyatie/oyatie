---
doc_class: Constitution
shape: ~
length_cap: 300
authority_tier: 1
excludes:
  - path: docs/AGENTS.md
    reason: Agent operating contract — Constitution sets the frame within which it operates.
  - path: docs/DOC-CATALOG.md
    reason: Per-doc lifecycle protocol — Constitution names that the protocol exists; DOC-CATALOG is the protocol.
  - path: docs/decisions/
    reason: Architectural decisions — ADRs operate within the constitutional frame.
  - path: docs/RACI-OWNERSHIP.md
    reason: Per-team RACI — Constitution names decision rights at the class level.
  - path: docs/INCIDENT-MANAGEMENT.md
    reason: Per-incident playbook — Constitution names the mistakes doctrine, not the lifecycle.
  - path: docs/standards/prevention-doctrine.md
    reason: Mechanical-prevention authoring guide — Constitution names the doctrine; the standard authors the prevention.
authority_chain_declaration: |
  docs/CONSTITUTION.md
    > rest of docs/
    > catalog records (registry/catalog/, contracts/, machine-readable/)
    > repo-root Redirect-class files (non-authoritative; lane-thin)
    > working drafts (never authoritative)
---

# Oyatie Constitution

This is the project's constitutional frame — the level above architecture, the level above operating contracts, the level above standards. Architecture, contracts, and standards operate within this frame; this frame does not operate within them.

The Constitution is read once a quarter by every contributor and every agent. It is short on purpose. It contains no specification verbs — those live in operating contracts and standards. It contains direction and decisions.

---

## Mission

Oyatie is one cohesive **ecosystem-as-a-service (EaaS)**, expressed across seven axes that share a single tenancy model, a single identity surface, a single capability registry, a single audit chain, and a single agent runtime + engineering platform.

The seven axes are: **SaaS · Workspace · Vertical · Foundry · Cloud · Search · Ads + Analytics.**

The strategic premise: a single tenant boundary, a single identity, a single audit chain, and a single agent runtime spanning every layer is more valuable than the sum of best-of-breed substitutes — because it removes the integration tax every multi-vendor stack pays.

The brand surface is **Oyatie**. The repo path / GitHub slug `oyatie` is retained for filesystem-coupling reasons. Implementation-level rebrand (`oyatie-*` Cargo crates, `@oyatie/*` npm scope, container repo paths) proceeds as a coordinated multi-batch migration.

---

## Decision principles — Do

These are the things every contributor and every agent does.

1. **Cohesion over portfolio.** Every architectural decision is judged by whether it preserves the single tenancy / identity / audit-chain / agent-runtime promise. Split-brain decisions that gain local convenience are rejected.
2. **Catalog-first authoring.** Every change updates the right doc tier in the same PR. Code without doc-update is not "done."
3. **Mechanical prevention over process.** Recurring failures produce a CI lane, hook, validator, or schema check — not a checklist line.
4. **Dual audience for instructions.** Every directive read by an agent is also readable to a human, and vice versa. No agent-only checklists; no human-only prose.
5. **Single canonical tree.** All authority lives under [`docs/`](.). Files outside the tree are non-authoritative discovery surfaces or working drafts.
6. **Bounded scope per doc.** Every canonical doc has a single class, a single owner, a hard length cap, and an explicit "Does NOT cover" clause. Drift-by-overlap is a structural failure, not a hope.
7. **Forward-reference, never invent.** A cross-link to a doc that does not yet exist uses the forward-reference sentinel; it does not silently link to nothing.
8. **Audit-chain emission on every cross-axis flow.** Cross-pillar data movement is audited; auditless cross-flow is a defect, not an optimization.
9. **Bench-and-stress before claiming performance.** Benchmark + at least two stress scenarios precede any "performance is acceptable" claim.
10. **Cancel cleanly.** Long-running loops re-walk the Done-Definition checklist before ending; loops do not exit silently.

## Prohibitions — Avoid

These are the things every contributor and every agent does not do.

1. **No parallel canonical trees.** Authority lives in [`docs/`](.). Re-creating a retired authority surface is a regression.
2. **No `--no-verify`, no hook bypass, no signing skip.** Hook failure is a signal; the fix is the underlying issue, not the bypass.
3. **No `force` push on `main`, no destructive history rewrite, no `reset --hard` on someone else's work.** Risky actions require explicit user authorization scoped to the action.
4. **No untyped values at API boundaries.** Use the result types prescribed by the standards.
5. **No new struct fields in kernel crates without `data_class` annotation.** Pre-commit blocks this; respect it.
6. **No quarantining a flaky test without a 14-day fix SLA.** Quarantine assigns the test to the `flaky/` lane; the SLA is tracked.
7. **No editing legacy retired paths.** If a path was retired in a consolidation event, do not recreate it.
8. **No process-only fix for a recurring incident.** Mechanical prevention is the doctrine.
9. **No AGPL / GPL / SSPL / BUSL / RSAL dependencies in product code.** License posture is fixed.
10. **No autonomy-tier uplift without policy + runtime gate.** Capability bindings declare T1 / T2 / T3 / T4; uplift requires a Cedar policy and a runtime check, not a config flag.

---

## Decision rights

Decisions are assigned to a class; each class has a single owner and a documented escalation path.

| Decision class | Owner | Escalation |
|---|---|---|
| Constitutional amendment | Founder | Council-Architecture (≥2 members ratify) |
| Cross-axis contract | Council-Architecture | Founder veto |
| Per-axis architecture | Axis team lead | Council-Architecture |
| Per-product feature | Product PM | Axis team lead |
| Per-team norm | Team lead | Axis team lead |
| Capability tier (T1–T4) | Capability owner | Council-Privacy + Council-Architecture |
| ADR (architectural binding) | Drafting team | Council-Architecture (ratification) |
| Doc admission to canonical tree | Council-Architecture (curator) | Founder |
| Doc retirement | Council-Architecture (curator) + content owner | Founder |
| Regional pack | Per-region lead | Council-Architecture |
| Incident severity / postmortem owner | Incident commander | Council-Architecture |
| License-tier review | Legal + Council-Architecture | Founder |

Per-team RACI detail lives outside this Constitution; see [`RACI-OWNERSHIP.md`](RACI-OWNERSHIP.md) <!-- forward-reference: wave-1 -->.

---

## Authority precedence

When two sources conflict, the higher source wins.

```
docs/CONSTITUTION.md
  > rest of docs/
  > catalog records (registry/catalog/, contracts/, machine-readable/)
  > repo-root Redirect-class files (non-authoritative; lane-thin)
  > working drafts (never authoritative)
```

This chain appears verbatim in this Constitution, in [`AGENTS.md`](AGENTS.md), and in [`README.md`](README.md). The `oya-foundry-fitness-authority-cohesion` lane validates the three declarations are character-identical.

---

## Mistakes doctrine

Three steps. Every Sev-1 / Sev-2 incident, every audit finding, every drift discovery follows this sequence.

1. **Root-cause to a single failure mode.** Name it in one sentence.
2. **Ship a mechanical prevention.** A CI lane, a hook, a validator, a schema check, a runtime gate. Not a checklist line; not a memo. The prevention is testable on the original failure mode (replay-as-eval).
3. **Add a row to [`MISTAKES-LEDGER.md`](MISTAKES-LEDGER.md) <!-- forward-reference: wave-1 -->.** The row records the failure mode, the system gap, the prevention class (`mechanical` or `cultural`), the prevention name, and the ship date. Future authors search the ledger before adding new behavior.

A failure mode that is structurally non-replayable (e.g., a human reviewer missed a thing) is filed under the `cultural` prevention tier and gets a quarterly human review cadence rather than a CI lane.

---

## Documentation

Every change updates the right doc tier in the same PR.

The protocol is in [`DOC-CATALOG.md`](DOC-CATALOG.md). The house style and doc-class taxonomy are in [`standards/doc-style.md`](standards/doc-style.md) <!-- forward-reference: wave-1 -->.

Doc-update protocol failure is a real defect, not a styling concern. The `oya-foundry-fitness-doc-catalog` lane refuses PRs that change a canonical surface without updating the dependent docs the catalog names.

The canonical doc tree is [`docs/`](.). Files outside this tree are either non-authoritative discovery surfaces (Redirect class, ≤25 lines, lane-thin) or working drafts (`docs/raw/`, never authoritative).

---

## Agents

Every agent (Claude Code, Codex, Gemini, OMC subagents, Foundry capabilities) operates against [`AGENTS.md`](AGENTS.md). That file is the single agent operating contract; per-agent harness deltas live in `## Per-agent appendices`.

Capabilities operate within an autonomy tier (T1–T4) declared in their capability record. Tier uplift requires a Cedar policy + a runtime gate; no config flag substitutes.

Reviewer-agent contract: every change class names the reviewer agent that runs proactively on the PR and signs the `## Code Review` section of the PR body at merge time. The merge-gate hook refuses any merge without that signature. Detail in [`AGENTS.md`](AGENTS.md) §"Per-change-class reviewer agents".

---

## Amendments

This Constitution is amended only by the Founder, ratified by ≥2 members of Council-Architecture.

The amendment procedure:

1. Open a draft amendment as a PR against this file.
2. PR body cites the failure mode or strategic shift the amendment addresses.
3. Ratifying council members sign the PR body's `## Ratification` section.
4. On merge, emit `EVT-CONSTITUTION-AMENDED` to the audit chain.
5. Add a row to [`CHANGELOG.md`](CHANGELOG.md) <!-- forward-reference: wave-1 --> describing the amendment.
6. Re-walk the new Constitution against every Tier-1 doc within one wave; resolve any contradictions by amending the lower doc, not the Constitution.

Constitutional drift is the highest-severity defect class. The `oya-foundry-fitness-constitution-cite-coverage` lane verifies every Tier-1 doc cites this Constitution at H1 or H2 level.

---

## Anti-overlap

This Constitution does not cover:

- **The agent operating contract.** Pre-flight, during-change discipline, PR shape, done-definition — see [`AGENTS.md`](AGENTS.md).
- **The per-doc lifecycle protocol.** Trigger taxonomy, update procedure, validator catalog — see [`DOC-CATALOG.md`](DOC-CATALOG.md).
- **Architectural decisions.** Per-decision rationale, alternatives, supersession — see [`decisions/`](decisions/) <!-- forward-reference: wave-1 --> indexed at [`ADR-INDEX.md`](ADR-INDEX.md) <!-- forward-reference: wave-1 -->.
- **Per-team RACI.** Detailed responsibility allocation — see [`RACI-OWNERSHIP.md`](RACI-OWNERSHIP.md) <!-- forward-reference: wave-1 -->.
- **Per-incident playbook.** Severity taxonomy, IM/CM roles, comms — see [`INCIDENT-MANAGEMENT.md`](INCIDENT-MANAGEMENT.md) <!-- forward-reference: wave-1 -->.
- **The mechanical-prevention authoring guide.** How to ship a prevention — see [`standards/prevention-doctrine.md`](standards/prevention-doctrine.md) <!-- forward-reference: wave-1 -->.

The full machine-readable list is in this file's front-matter `excludes:` block.

---

## Sources scanned

- 2026-05-10 — initial draft authored from agentic-workflow best practice (Anthropic CLAUDE.md memory + cross-tool AGENTS.md convention) + RFC-2119 + RFC-8174 + Diátaxis + Linux kernel `Documentation/process/` precedent + openai/symphony benchmark.
