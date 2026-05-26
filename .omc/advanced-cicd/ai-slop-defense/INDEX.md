---
doc_class: AdvancedCicdIndex
parent: .omc/advanced-cicd/
status: pending approval
purpose: |
  Anchor the AI-slop-defense + impossible-to-fail environment architecture
  for oyatie. Cross-links every defense spec under this directory and
  declares the read order for a fresh agent.
owner: council-architecture + axis-foundry
date: 2026-05-12
---

# AI-Slop Defense — INDEX

> **Status:** pending approval (working draft under `.omc/advanced-cicd/ai-slop-defense/`).
> **Lift target:** none — this directory stays under `.omc/` as research-input
> for fitness-lane authoring, MASTERPLAN Directive 6/9/12 evidence, and
> ADR-XXX (TBA) drafting.

## Purpose

Capture, in mechanical form, the failure modes that AI coding agents
inject into the codebase, define the production-quality bar that oyatie
must hold against, and design a layered defense architecture that makes
the environment **asymptotically impossible to fail** under autonomous
agent operation. Per MASTERPLAN Directive 6 (hyperscaler bar) and
Directive 9 (hyperscaler toolchain), the defenses match or exceed the
AWS / Google / MS / Oracle bar.

## Read order (agent entry path)

1. [`ai-slop-failure-mode-catalogue.md`](ai-slop-failure-mode-catalogue.md)
   — the 42 catalogued failure modes that AI agents inject, with
   mechanical-prevention strategies.
2. [`production-quality-bar.md`](production-quality-bar.md)
   — 38 hyperscaler-bar dimensions of "production quality" with
   verification methods and the fitness lane that enforces each.
3. [`gap-analysis-ai-vs-production.md`](gap-analysis-ai-vs-production.md)
   — bidirectional map from failure mode to violated quality dimension(s)
   and from oyatie's current tooling to closed/open gaps. Top-3
   most-dangerous open gaps surfaced.
4. [`defense-in-depth-architecture.md`](defense-in-depth-architecture.md)
   — the 9-layer (L0 prevent → L8 postmortem) defense architecture with
   a Mermaid diagram and per-layer fitness-lane mapping.
5. [`additional-tooling-recommendations.md`](additional-tooling-recommendations.md)
   — concrete tool adoption list (BLOCKER / HIGH / MED / LOW) with
   named fitness-lane targets and ADR-XXX targets.
6. [`impossible-to-fail-environment-spec.md`](impossible-to-fail-environment-spec.md)
   — the top-level architecture binding everything: invariants,
   mechanical preventions, resilience to agent + LLM regression, the
   measurement formula, and the residual-failure budget.

## Cross-references

- [MASTERPLAN.md](../../plans/MASTERPLAN.md) — Directive 2, 3, 6, 9, 12.
- [hyperscaler-best-practices-2026-05-12.md](../../specs/hyperscaler-best-practices-2026-05-12.md)
  — baseline practices and tool roster.
- [MISTAKES-LEDGER.md](../../../docs/MISTAKES-LEDGER.md)
  — prevention doctrine and per-row mechanical-prevention obligation.
- [`.omc/governance-lanes/`](../../governance-lanes/) — current lane catalogue
  (22 lanes); new lanes proposed in this directory's spec files.
- [`.omc/advanced-cicd/progressive-delivery/`](../progressive-delivery/)
  — Layer 5 staging rails.

## Dependency graph between specs

```
catalogue ───┐
             ├──► gap-analysis ──► defense-in-depth ──► impossible-to-fail
quality-bar ─┘                                  ▲
                                                │
                          additional-tooling ───┘
```

## Authoring discipline

- Each spec ≤ 250 lines; this INDEX ≤ 100 lines (per coordinator prompt).
- All claims that exceed engineering common-sense cite a source.
- All defenses name an **existing or new fitness lane** that enforces them.
- Linus-style: no ceremony defense. Every defense reshapes data or
  blocks a class of failures; pure-process defenses are rejected.
- Per MASTERPLAN Directive 3 (final-shape): design the end-state, do
  not propose an iterative path.

## Status footer

Iteration: 1 (2026-05-12). Authored by general-purpose research agent
per founder coordinator prompt. Next reviewers: Architect (defense
soundness), Critic (gap closure vs MISTAKES-LEDGER and Top-10 RM
register), Founder (autonomous decision to lift to ADR drafts).
