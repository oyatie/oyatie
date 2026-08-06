---
id: ADR-0366
status: Superseded
deciders: council-architecture, founder
date: 2026-05-26
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
related: [ADR-0365, ADR-0364, ADR-0363, ADR-0111, ADR-0130, ADR-0349, ADR-0361]
planning_impact: true
milestone: M-AGENTIC-PIPELINE
depends_on: [ADR-0363, ADR-0365]
door: one-way
affected_surfaces:
  crates: [oya-dev-cli]
  microservices: []
  specs: [/specs/masterplan.json, /registry/quality/lanes.yaml]
deliverables:
  - id: ADR-0366-D1
    description: "Ownership-sharding: a single-threaded owner-agent per service/lane owning disjoint paths; concurrent-safe-paths admission prevents two lanes editing the same path."
    exit_criteria: "owner declared per service in catalog; admission gate rejects overlapping concurrent claims."
    verified_by: "oya gate validate concurrent-safe-paths"
  - id: ADR-0366-D2
    description: "High-throughput merge path: merge-queue speculative rebase (ADR-0111) + affected-targets + two-tier presubmit(fast/affected/blocking) vs postsubmit(full + regression attribution)."
    exit_criteria: "concurrent green PRs serialize through the queue without manual rebase; postsubmit bisects the culprit on a break."
    verified_by: "oya gate validate merge-queue-health"
  - id: ADR-0366-D3
    description: "Self-repair loop: deterministic auto-fix (fmt, rebase, regenerate drifted docs via ADR-0365 propagation, quarantine flaky tests, draft COE->gate); escalate to human ONLY for one-way-door / non-deterministic / unrepairable."
    exit_criteria: "a protocol violation is auto-detected and auto-repaired end-to-end with no human step in the deterministic cases; escalation only otherwise."
    verified_by: "oya gate validate self-repair-coverage"
  - id: ADR-0366-D4
    description: "Anti-waste / anti-thin-scaffold gates: PR-FAQ before a new service exists; Definition-of-Done completion gate (tests+docs+ADR+SLO+evidence) so 'green' means substance, not scaffold."
    exit_criteria: "a new service without a PR-FAQ is blocked; a half-done change fails DoD."
    verified_by: "oya gate validate definition-of-done"
  - id: ADR-0366-D5
    description: "Quality/resilience gates: automated canary analysis (Kayenta-style statistical judge on Argo Rollouts), chaos GameDay (SLO-gated), error-budget policy + promotion freeze."
    exit_criteria: "promotion is statistically gated; over-budget freezes non-critical promotion; a chaos experiment runs SLO-gated."
    verified_by: "oya gate validate error-budget-policy"
  - id: ADR-0366-D6
    description: "DORA four-keys instrumented from evidence/audit-chain + ArgoCD events (deploy freq, lead time, change-fail rate, MTTR) — the throughput/quality measurement."
    exit_criteria: "`oya dora` emits the four keys from real pipeline events."
    verified_by: "oya gate validate dora-metrics"
purpose: A high-throughput, parallelizable, conflict-free, self-enforcing and self-repairing development pipeline for a fleet of AI agents — hyperscaler multi-team discipline applied to agents. Without it, agentic development drifts into thin scaffolds without substance; with it, feature work rides a substrate that catches and repairs protocol violations with minimal human intervention.
---

# ADR-0366: Agentic high-throughput, self-enforcing, self-repairing pipeline

## Status
Accepted — 2026-05-26.

## Context
Agentic development is **pipeline-bound, not coding-bound.** A fleet of agents editing one monorepo
concurrently, without a hyperscaler-grade pipeline, produces drift, conflicts, wasted rework, and
thin scaffolds without substance — precisely the state the ADR distillation found (300 ADRs, 41%
never-ratified, "spec-saturated, code-starved"). **Agents do not reliably follow protocol**, so the
pipeline cannot rely on discipline — it must *catch, fix, repair, and automate* with minimal human or
agent intervention. This is why we adopt hyperscaler multi-team practices as if we had many teams:
the agents *are* the teams. This ADR is the prerequisite for feature work (T3+); features ride on it.

## Decision

### 1. Parallelism with conflict PREVENTION (not just resolution)
A **single-threaded owner-agent per service/lane** (AWS STO) owns **disjoint paths** — the flat /
no-grouping doctrine (ADR-0362) makes service paths naturally disjoint. One **isolated worktree per
lane**. A **concurrent-safe-paths** admission gate rejects two in-flight lanes touching the same path.
Cross-cutting changes flow through the **merge-queue with speculative rebase** (ADR-0111). Conflicts
are *prevented* by ownership boundaries and *resolved* by the queue. (The retired `oya vcs`
claim-ratchet is NOT revived — ownership-sharding + the queue replace it.)

### 2. High throughput, minimal wasted work
**Affected-targets** (build/test only what changed) + **content-hashed gate cache** + a **two-tier**
split: *presubmit* = fast/affected/blocking per PR; *postsubmit* = full platform with **regression
attribution** (bisect the culprit). Stacked small PRs over mega-branches.

### 3. Self-enforcing + self-repairing (the keystone)
Every protocol step is a **gate** — agents cannot skip it. A **self-repair loop** runs DETERMINISTIC
auto-fixes without human intervention: `cargo fmt`, rebase via the merge-queue, **regenerate drifted
docs** (ADR-0365 propagation), **quarantine flaky tests** off the blocking path, and **draft a
COE→new-gate** on any failure. It escalates to a human ONLY for one-way-door decisions,
non-deterministic failures, or genuinely unrepairable states. The system heals; humans/agents
intervene minimally.

### 4. Anti-thin-scaffold
**PR-FAQ** gate before a new service may exist (whether-to-build). **Definition-of-Done** completion
gate (tests + docs + ADR + SLO + evidence present) so a *green* build means substance, not scaffold.

### 5. Quality + resilience as gates
**Automated canary analysis** (Kayenta-style statistical judge on Argo Rollouts), **chaos GameDay**
(SLO-gated), **error-budget policy** with promotion freeze, on top of ADR-0130 SLO-gated promotion.
**DORA four keys** measured from the audit-chain — the throughput/quality scoreboard.

## Rejected alternatives
- **Start feature work (T3) first** — rejected: features on an unenforced pipeline manufacture drift +
  thin scaffolds (the founder's argument; the distillation's evidence).
- **Claim-based file locking (revive `oya vcs` ratchet)** — rejected: retired in ADR-0363;
  ownership-sharding + merge-queue prevent/resolve conflict without a bespoke ratchet.
- **Rely on agents following protocol** — rejected: they don't; hence self-enforcing gates +
  self-repair.

## Consequences
- Positive: agents develop concurrently at high throughput without conflict or drift; protocol
  violations are caught + auto-repaired; "green" means substance; feature work (T3+) lands on solid
  ground. This is the multiplier that makes the agent fleet productive.
- Negative/cost: the pipeline (D1–D6) is substantial build work and must precede feature delivery —
  deliberately, per this ADR. Risk of over-engineering; mitigated by adopting (not building) where
  possible (Argo AnalysisTemplates, Chaos Mesh, merge-queue patterns).
- Neutral: rides the existing substrate (ADR-0363 git+Jenkins+GitHub, ADR-0349 farm).

## Verification
Per-deliverable `verified_by`: `oya gate validate concurrent-safe-paths | merge-queue-health |
self-repair-coverage | definition-of-done | error-budget-policy | dora-metrics` green; the pipeline
runs end-to-end on the farm with agents merging concurrently, conflict-free, self-repairing.

## References
ADR-0365 (automated ADR lifecycle — this pipeline executes it), ADR-0364 (generated masterplan),
ADR-0363 (substrate), ADR-0111 (speculative merge-queue), ADR-0130 (SLO-gated promotion), ADR-0349
(CI farm), ADR-0361 (Jenkins-native CI). Research backlog: docs/ideas/hyperscaler-practices-to-adopt.md
(STO, one-way/two-way-door, COE→gate, PR-FAQ, DoD, Kayenta canary, chaos, DORA, error-budget policy).
