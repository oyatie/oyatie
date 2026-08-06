---
id: ADR-0369
status: Superseded
deciders: council-architecture, founder
date: 2026-05-26
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
related: [ADR-0367, ADR-0366, ADR-0363, ADR-0111, ADR-0362]
planning_impact: true
milestone: M-AGENTIC-PIPELINE
depends_on: [ADR-0367]
door: two-way
affected_surfaces:
  crates: [oya-dev-cli]
  microservices: []
  specs: [/registry/quality/lanes.yaml]
deliverables:
  - id: ADR-0369-D1
    description: "Ownership-sharding conflict prevention: one owner-agent per service on disjoint flat paths (ADR-0362), enforced by concurrent-safe-paths admission + CODEOWNERS auto-routing."
    exit_criteria: "two concurrent lanes touching the same path are rejected at admission; CODEOWNERS routes review to the owner."
    verified_by: "oya gate validate concurrent-safe-paths"
  - id: ADR-0369-D2
    description: "Stacked-diff authoring on plain git: agents author small (<~200 LOC) dependent units as a ghstack-style chain of GitHub PRs (base = parent's branch). NOT jj."
    exit_criteria: "a stack of dependent PRs lands incrementally; each entry is independently gate-verified."
    verified_by: "oya gate validate stacked-diff-size"
  - id: ADR-0369-D3
    description: "The trustless-gateway as a single required status check (the binding gate, NOT GitHub's merge button — it has merge-despite-failed-checks races): green only when trusted-runner-signed evidence (ADR-0367) AND independent reviewer attestation both verify against the PR head SHA."
    exit_criteria: "auto-merge fires only when the signed check is green; a forced merge with the check red is structurally impossible (the check is the gate)."
    verified_by: "oya gate validate untrusted-evidence"
  - id: ADR-0369-D4
    description: "Speculative stack-aware merge-train (LATER; ADR-0363 §3 concurrency trigger): adopt gitea-mq/bors-class queue that tests the projected post-merge state, batches, bisects on failure, speculates in parallel. Adopt, do not reinvent."
    exit_criteria: "at concurrency, semantic conflicts do not reach trunk; per-PR latency stays near the single-CI floor."
    verified_by: "oya gate validate merge-queue-health"
purpose: Choose the change-flow MECHANISM that maximizes throughput for a concurrent agent fleet under the ADR-0367 trustless model — "gated stacked-trunk with a speculative train" on plain git + GitHub, evaluated against all options (PR, change-centric/Gerrit, stacked, jj, trunk+flags, pre-receive, merge-train).
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0369: Gated stacked-trunk change-flow with a speculative merge-train

## Status
Accepted — 2026-05-26.

## Context
ADR-0367 fixed the *trust* model; this ADR fixes the *mechanism*, evaluated against all options
(traditional PR, Gerrit change-centric, stacked diffs, Jujutsu, trunk+flags, server-side pre-receive
gate, speculative merge-train). GitHub ground-truth (2026): no native merge-queue (github#5102 open),
required-status-checks + Commit-Status-API + auto-merge + CODEOWNERS present, pre-receive hooks exist
but run as the GitHub OS user (RCE-grade), and known merge-despite-failed-checks races (#11224/#8189)
mean **the binding gate must be the required check itself, not the merge button.**

## Decision
**Gated stacked-trunk with a speculative train**, on plain git (ADR-0363) + GitHub PRs as the cheap
mechanism + audit record (ADR-0367 §4):

1. **Conflict prevention — ownership-sharding (now, D1).** One owner-agent per service on disjoint flat
   paths (ADR-0362); `concurrent-safe-paths` admission + CODEOWNERS. Disjoint paths → most concurrent
   work never conflicts.
2. **Authoring — stacked diffs on plain git (now, D2).** Small dependent units as a ghstack-style PR
   chain. Smaller units → cheaper affected-targets re-test, smaller conflict surface, parallel
   adversarial review. **Not jj** (see rejected).
3. **The trustless gateway — one required check (now, D3).** A required `trustless-gateway` context is
   green only when the trusted runner's cosign/SLSA signature over hermetically re-executed gates
   verifies **and** a distinct-identity reviewer-agent attestation verifies (ADR-0367). The signature
   verification *is* the check — robust to GitHub's merge-button races.
4. **Throughput — speculative stack-aware merge-train (later, D4; ADR-0363 §3 trigger).** Adopt
   `gitea-mq`/`bors`-class: test the projected post-merge state, batch, bisect on failure, speculate in
   parallel. The only mechanism that stops semantic conflicts reaching trunk at N-concurrency. Adopt,
   don't reinvent (revives ADR-0111).

## Rejected alternatives
- **Jujutsu (jj) as the agent VCS layer** — rejected: the leading agent-VCS project (agentjj) embedded
  jj then reverted core ops to plain git by v0.3.1; jj's no-staging-area collapses implement/test/docs
  into one squashed commit and its single-writer working-copy breaks two-subagents-one-tree parallelism;
  also violates ADR-0363 "git as-is."
- **Gerrit (the server)** — rejected: wrong substrate (contradicts ADR-0363 GitHub); borrow its
  *semantics* (per-change review, rebase-on-submit) only.
- **Pre-receive hook as the primary path** — rejected as primary: loses the PR audit object and runs
  RCE-grade code as the GitHub OS user; keep only as an optional defense-in-depth backstop.
- **Direct trunk push / trust author evidence** — rejected: violates ADR-0367.
- **Traditional PR + auto-merge alone** — insufficient above a few concurrent agents (tests each PR vs
  its own head, not projected trunk → semantic conflicts land); kept as the baseline mechanism under
  the speculative train.

## Consequences
- Positive: max throughput (small units + speculation + auto-merge, no human bottleneck) with conflicts
  prevented (sharding) and resolved (train); rides plain git + GitHub (no new VCS).
- Negative/cost: stacked-PR tooling on GitHub (ghstack-style) + the merge-train bot are build/adopt
  work; speculation wastes some CI (mitigated by affected-targets + cache, ADR-0366).
- Neutral: trust model unchanged (ADR-0367); this is the mechanism that carries it.

## Verification
`oya gate validate concurrent-safe-paths | stacked-diff-size | untrusted-evidence | merge-queue-health`
green; demonstrated: concurrent owner-agents land stacked PRs that auto-merge only on the signed check,
and (at concurrency) the speculative train keeps semantic conflicts off trunk.

## References
ADR-0367 (trust model), ADR-0366 (pipeline), ADR-0363 (substrate, §3 merge-queue trigger), ADR-0111
(speculative merge-queue, revived here), ADR-0362 (flat/no-grouping → disjoint paths). Research:
GitHub #5102/#11224/#8189, agentjj jj→git reversion, GitHub stacked-PRs-for-agents (2026), bors/TAP.
