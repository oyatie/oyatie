---
id: ADR-0134
status: Proposed
deciders: council-architecture, ops-sre-reliability, workflow-studio-product-council
date: 2026-05-17
owner: ops-sre-reliability
supersedes: []
superseded_by: []
related:
  - ADR-0114
  - ADR-0123
  - ADR-0133
related_specs:
  - /specs/masterplan.json
  - /specs/products/workflow-studio.json
  - /evidence/autoresearch/hyperscaler-pattern-meta-audit-1779012603.json
version: 1.0.0
purpose: Record the portfolio hyperscaler remediation backlog as proposed acceptance criteria without claiming that the named validators or branch-protected CI lanes already exist.
---

# ADR-0134: Portfolio Hyperscaler Pattern Remediation Backlog

## Status

Proposed - 2026-05-17.

This ADR records candidate portfolio-wide acceptance criteria. It is not a
production-readiness claim, a branch-protection claim, or a hyperscaler-maturity
claim.

## Context

The hyperscaler pattern audit and current PR review queue identified recurring
portfolio gaps across Foundry, Workflow, Workflow Studio, Ontology, and Cloud:

- LLM/tool invocation loops need bounded retry budgets and circuit-breaker state.
- High-volume APIs need per-tenant admission control.
- Foundry needs explicit all-providers-degraded shed behavior.
- Workflow Studio needs the full SRE signal set for operator UX and safety.
- Product SLOs need error-budget burn-rate policy before GA claims.

The earlier PR #135 draft marked these lanes as accepted and enforced even
though the validators, workflow files, branch-protection rows, and negative tests
were not present. This ADR keeps the useful remediation shape while making the
enforcement state honest.

## Decision

Adopt these five items as a **proposed remediation backlog**. Each item may
become binding only in the PR that ships its validator, fixture coverage,
branch-protection integration, and product-specific wiring.

| Item | Candidate validator | Minimum acceptance criterion |
|---|---|---|
| LLM circuit breaker | `oya-governance-circuit-breaker-presence` | T1 invocation surfaces declare `max_retry_budget`, `circuit_breaker_threshold`, and circuit state, with max retry budgets <= 3 unless a product ADR justifies a higher value. |
| Per-tenant rate limit | `oya-governance-per-tenant-rate-limit` | Public capability/action/canvas APIs have tenant-keyed token buckets and explicit 429 + `Retry-After` behavior. |
| Provider-degraded shed | `oya-governance-provider-degraded-shed` | Foundry provider queues define all-providers-degraded behavior, defaulting to bounded 503 or bounded queue drop rather than unbounded enqueue. |
| Workflow Studio golden signals | `oya-governance-workflow-studio-golden-signals` | Workflow Studio exposes latency, traffic, errors, and saturation signals. Availability remains an SLO, not a substitute for the missing golden signals. |
| Error-budget burn rate | `oya-governance-error-budget-policy` | Product SLOs define fast-burn and slow-burn thresholds, notification targets, and rollback/escalation behavior. |

## Boundaries

- Named validators above are planned identifiers until their crates or CLI gates
  exist on the branch.
- Product PRDs may reference these items only as advisory targets until the
  relevant validator passes on that product's implementation and spec surfaces.
- Branch protection must not list a status check until the workflow exists and
  the check name is stable.
- Numeric thresholds in this ADR are acceptance criteria for future validators,
  not evidence that current products satisfy them.

## Rejected Alternatives

- **Keep the earlier accepted/enforced wording.** Rejected because it would
  reintroduce aspirational enforcement and allow green CI while real product
  surfaces are not checked.
- **Delete the backlog completely.** Rejected because the identified safety and
  SRE gaps are valid and need a durable planning surface.
- **Bundle all five validators into one PR.** Rejected because the implementation
  spans distinct product boundaries and would be too large to review safely.

## Consequences

- Hyperscaler-maturity claims remain blocked unless the concrete product
  validators and evidence pass.
- Workflow Studio remains the priority UX surface for the golden-signal work
  because its visual editor needs clear operator feedback for traffic, errors,
  saturation, and latency.
- Later PRs should land one validator family at a time with tests, evidence, and
  branch-protection updates in the same slice.

## Verification

- `oya doc adr-index --write --format json`
- `oya gate validate adr-citation`
- `oya gate validate hyperscaler-maturity-claims`
- Reviewer-agent check that every claim is advisory unless a validator exists
  and is wired into CI.
