---
id: ADR-0367
status: Superseded
deciders: council-architecture, founder
date: 2026-05-26
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
related: [ADR-0366, ADR-0363, ADR-0111, ADR-0349]
planning_impact: true
milestone: M-AGENTIC-PIPELINE
depends_on: [ADR-0366]
door: one-way
affected_surfaces:
  crates: [oya-dev-cli]
  microservices: []
  specs: [/registry/quality/lanes.yaml]
deliverables:
  - id: ADR-0367-D1
    description: "Trusted-runner-signed evidence: the farm/Jenkins re-executes every gate hermetically and signs results (SLSA provenance + cosign); the author-agent's self-reported evidence is never trusted."
    exit_criteria: "merge gate verifies a trusted-runner signature over the gate results; an unsigned/self-reported evidence file is rejected."
    verified_by: "oya gate validate untrusted-evidence"
  - id: ADR-0367-D2
    description: "Separation of duties: the adversarial reviewer-agent is a CI stage powered by the Intelligence service (distinct identity from the author); its APPROVE/REJECT is posted as the trusted-runner-signed oya-pr-review status — the verdict itself is trustless, not a self-reported approval."
    exit_criteria: "reviewer identity != author; the oya-pr-review APPROVE is a trusted-runner signature (not an agent self-report); a self-reviewed changeset is rejected."
    verified_by: "oya gate validate reviewer-independence"
  - id: ADR-0367-D3
    description: "Verified-changeset gateway: merge is automatic on signed-green + independent-APPROVE via the merge-queue / GitHub auto-merge — no human-PR-review ceremony."
    exit_criteria: "a changeset auto-merges only when both signals are present; neither alone suffices."
    verified_by: "oya gate validate merge-queue-health"
  - id: ADR-0367-D4
    description: "Reproducibility / re-execution: evidence is re-derived in a hermetic sandbox, not copied from the author's working tree; flaky/non-reproducible evidence is quarantined."
    exit_criteria: "the trusted runner reproduces the author's claimed result from clean checkout; divergence blocks merge."
    verified_by: "oya gate validate self-repair-coverage"
purpose: Replace the traditional human-PR-review ceremony with a trustless pre-merge verification gateway. The producing agent never certifies its own work; a trusted runner re-executes and SIGNS the evidence, and a separate adversarial reviewer-agent approves. This closes the "the evidence collection itself could be false" hole and maximizes merge efficiency without lowering the quality bar.
---

# ADR-0367: Trustless pre-merge verification gateway (PR-ceremony-less)

## Status
Accepted — 2026-05-26.

2026-06-28 amendment note: under ADR-0515 / ADR-0548, the live substrate for this
gateway is cloud-ci / oya-ci merge admission, not a local CLI assertion. The
automation-ratchet RED/GREEN corpus pins that review authority with
specs/fixtures/phase0-automation-ratchet/tc-0.16-bad-missing-pre-merge-review-authority.json
and
specs/fixtures/phase0-automation-ratchet/tc-0.16-good-pre-merge-review-authority.json:
green CI alone is insufficient; merge requires live durable review evidence or
a machine-verifiable blocking review status with reviewer identity distinct from
the author.

## Context
The pull request was always a *proxy* for "someone independent verified this." For an agent fleet that
goal is direct: **trustless verification.** The failure mode to design against is precise — *the
evidence collection itself can be false*: an agent that both produces work and reports its own
green-checks can fabricate or self-deceive. Therefore the producing agent's word can never be the
basis for merge. Trust must come from the substrate (ADR-0363 farm + ADR-0366 pipeline), not the
agent. This removes the PR *ceremony* (human review ritual, branch-PR overhead) while *raising* the
bar, because verification becomes machine-trustless rather than a human glance.

## Decision

### 1. The producer never certifies its own work
The author-agent's self-reported evidence (claimed test/gate results) is **never trusted** for the
merge decision. It is at most a hint.

### 2. Trusted-runner-signed evidence
The **trusted runner** (the farm / Jenkins, ADR-0349/0361) re-executes every gate **hermetically from
a clean checkout** and **signs** the results (SLSA provenance + cosign). The merge gate verifies the
*signature* of the trusted runner — a fabricated or self-reported evidence file has no valid signature
and is rejected (`untrusted-evidence` gate). This is the structural answer to "evidence could be false."

### 3. Separation of duties + adversarial review (as a trusted CI stage)
A **separate reviewer-agent** (distinct identity from the author; doubt-driven — it tries to *falsify*
the claim, not confirm it) must APPROVE; a self-reviewed changeset is rejected (`reviewer-independence`
gate). Producer ≠ verifier ≠ approver. Crucially, the reviewer-agent runs as a **CI stage powered by
the Intelligence service** (ADR-0363) and its APPROVE/REJECT is posted as the **trusted-runner-signed
`oya-pr-review` status**. This closes the loop: a *self-reported* APPROVE would be fakeable like any
other evidence, so the review verdict must itself be produced + signed by the trusted CI running
Intelligence — not asserted by an agent. (This is also the Intelligence service's first concrete,
self-tenant-dogfood job: the AI platform reviews the platform's own code.)

### 4. Verified-changeset gateway replaces PR ceremony
Merge is **automatic** on `signed-green + independent-APPROVE` via the merge-queue / GitHub auto-merge
(ADR-0111/0363). The GitHub PR object is retained only as the cheap automated *mechanism* + immutable
audit record — there is **no human-PR-review ritual**. A changeset that has the author's claimed green
but no trusted-runner signature, or no independent approval, does not merge.

### 5. Reproducibility
Evidence is **re-derived** in the sandbox, never copied from the author's tree; non-reproducible or
flaky evidence is quarantined (ADR-0366 self-repair), never silently accepted.

## Rejected alternatives
- **Keep traditional human-PR review** — rejected: it doesn't scale to an agent fleet and a human
  glance is weaker than independent re-execution; the ceremony is overhead.
- **Trust the author-agent's reported evidence (fast path)** — rejected outright: this is the
  "evidence could be false" hole; the entire ADR exists to close it.
- **Fully PR-less / direct-to-branch on local green** — rejected: loses the independent re-execution +
  audit record; we keep GitHub's PR as the automated mechanism, just without the human ceremony.
- **Same agent reviews its own work** — rejected: violates separation of duties.

## Consequences
- Positive: maximum merge efficiency (no human bottleneck, auto-merge) with a *higher* quality bar
  (independent signed re-execution + adversarial review); false claims are structurally impossible to
  merge. Throughput scales with the farm, not with human review capacity.
- Negative/cost: every merge pays for independent re-execution (mitigated by affected-targets + cache,
  ADR-0366); signing infra (cosign keys) must be managed; reviewer-agent must be genuinely adversarial,
  not a rubber stamp (doubt-driven discipline + the coe-to-gate flywheel when it misses).
- Neutral: rides ADR-0363 substrate, ADR-0366 pipeline, ADR-0111 merge-queue.

## Verification
`oya gate validate untrusted-evidence | reviewer-independence | merge-queue-health | self-repair-coverage`
green; demonstrated: a changeset with fabricated/self-reported evidence is rejected; a self-reviewed
changeset is rejected; a changeset with trusted-runner-signed green + independent APPROVE auto-merges.

## References
ADR-0366 (the pipeline this gateway sits in), ADR-0363 (substrate / GitHub auto-merge), ADR-0111
(speculative merge-queue), ADR-0349/0361 (the trusted runner — CI farm / Jenkins, SLSA + cosign).
Research backlog: docs/ideas/hyperscaler-practices-to-adopt.md (separation of duties, SLSA provenance,
doubt-driven/adversarial review).
