# Lifecycle fidelity, evaluation, and gap closeout

This harness is evaluated **continuously** — not as a one-shot script.  
Fine-tune: `evaluation.v1.json`. Run: `mm-evaluate static|run|full`.

## Real developer lifecycle ↔ harness

| Real life | Harness surface | Captures? | Gap / close |
|-----------|-----------------|-----------|-------------|
| Idea / ticket / bug | `mm-goals create`, workflow `args.objective` | Partial | No auto GitHub/Linear sync — paste IDs into brief |
| Clarify / spike | CONTRACT + Socratic/contrarian perspectives | Yes | Human stakeholders not in-loop unless pause |
| Design / ADR | ARCHITECT + blast-radius lenses | Partial | ADR promotion is human |
| Plan + estimate | PLAN + dual orthogonal critics | Yes | No capacity calendar |
| Branch / worktree | PREFLIGHT `origin/dev` + isolation policy | Yes | Multi-writer shards incomplete |
| Implement | EXECUTOR role (mm-role) | Partial | Parallel path-disjoint execute not fully automated |
| Local test | VERIFY + fail-closed grade | Yes | Must name real commands |
| Code review | CRITIC_DIFF lens A/B | Yes | Does not replace human review |
| PR + required CI | PR packet; never merge authority | Partial | No live `oya-ci-required` poll in v1 |
| Merge queue | Documented only | Gap | External to harness |
| Post-merge / on-call | POST_MERGE_HINT + LEARN | Partial | No pager |
| Retro / process | mm-learn tips/skills + process_edits | Yes | Skill→pack promotion human-gated |

## Continuous evaluation axes

1. **Concurrent throughput** — wall clock, parallel critics, shard efficiency, path-overlap fail closed  
2. **Outcome quality** — grade letter, dual admit, hard fails, evidence paths  
3. **Delivery pipeline fit** — origin/dev preflight, single required context, no CLI merge authority  
4. **Prompt / instruction engineering** — lens packs, tool-use commands, orthogonal critics, no forbidden CLI deps  
5. **Hyperscaler enterprise design** — blast radius, zero-trust, constant-work, telemetry, day-2, finops  
6. **Portability** — `mm-bootstrap`, `project-profile.v1.json`, no hardcoded `/Users/...` in configs  
7. **Lifecycle fidelity** — map above completeness  

Cadence: **per run** (1–4), **weekly** (5–6), **on repeat failure class** (full panel).

## Known failures (where it fails today)

| ID | Failure | Intent to close |
|----|---------|-----------------|
| F-THROUGHPUT-SERIAL | Mostly serial multi-model stages | Path-disjoint executor shards; measure `parallel_efficiency` |
| F-GROK-ONLY-WORKFLOW | Rhai uses session model | Optional mm-role bridge after plan artifacts |
| F-DIRTY-BASE | Dirty/diverged checkout | PREFLIGHT fail closed; worktree-from-dev doctrine |
| F-PROMPT-DRIFT | Rhai lens text vs JSON packs | Prefer `mm-lens-prompt`; later host inject |
| F-PORTABILITY | Oyatie-centric examples | Bootstrap + project-profile per repo (console, etc.) |
| F-NO-LIVE-CI-POLL | No GH status watch | Optional status stage; never replace required context |
| F-LEARN-NOT-AUTO-PROMOTED | Tips not auto-applied to packs | Slice 3.1: `mm-learn kpi-repeat` (threshold×2) → human-gated promote suggest; human PR to packs |

## Portability (e.g. ~/Developer/console)

```bash
# from a repo that already has the harness:
.grok/bin/mm-bootstrap ~/Developer/console
# then edit:
#   ~/Developer/console/.grok/harness/project-profile.v1.json
#   base_ref, pr_target, required_status_context, surfaces
.grok/bin/mm-preflight --cwd ~/Developer/console
.grok/bin/mm-evaluate static --cwd ~/Developer/console
```

Same workflows and lenses; **project profile** adapts VCS/CI authority language.

## Hyperscaler enterprise bar (lenses enforced in packs)

Blast-radius · zero-trust · constant-work · shared-nothing/stale · finops · telemetry-first · operability day-2  

These appear in `lenses.v1.json` and stage packs; critics and architects must cite them under high risk.

## Forbidden orchestration deps

Do not use `gjc`, `omc`, `omx`, or `hermes` CLIs as control plane. Ideas only; implementation is `.grok/*`.

## Not merge authority

Human reviewer + project `required_status_context` remain the only merge truth.
