---
id: ADR-0128
status: Superseded
date: 2026-05-17
owners:
  - council-architecture
  - ops-sre-reliability
  - ops-security
supersedes: []
superseded_by: [ADR-700]
amended_by: [ADR-0252]
related:
  - ADR-0114-canary-observability-rollback.md
  - ADR-0119-specs-flat-root-topology.md
  - ADR-0123-hyperscaler-maturity-claim-gate.md
doc_class: Architecture-Decision-Record
purpose: >
  Bind the canonical hyperscaler systems + cloud architecture invariant spec
  (specs/hyperscaler-architecture-invariants.json) as the portfolio-wide source
  of truth for architectural evidence. Product PRD citation enforcement remains
  advisory until the validator, fixture tests, workflow, and branch-protection
  context exist together.
enforcement_status: advisory-until-product-prd-validator
enforced_by: oya gate validate hyperscaler-arch-invariants
---

# ADR-0128: Hyperscaler architecture invariants — canonical spec + portfolio binding

## Status

Accepted — 2026-05-17. Enforcement scope is limited to validating the invariant
catalog itself. Product-level PRD citation enforcement is advisory until the
validator, fixture tests, CI workflow, and branch-protection context land.

## Context

On 2026-05-17T11:00Z the user directive stated: *"Take hyperscaler systems
architecture and cloud architecture in mind as well."* Prior to this ADR the
portfolio had two partially-overlapping sources:

1. `docs/standards/hyperscaler-best-practices.md` — 41 KB prose research
   document capturing AWS/Google/Microsoft/Oracle practices. Useful for
   context but not machine-readable and not binding.
2. `specs/hyperscaler-gates.json` (EXE-HYPERSCALER-GATES) — claim-governance
   registry focused on the boolean "is the maturity claim allowed" gate, not
   on the per-invariant architectural rules themselves.

Neither document answered the concrete question every product team asks:
*"What does hyperscaler-grade mean in practice for my product, and which
specific rules must I comply with?"*

ADR-0123 established the claim-governance gate. This ADR establishes the
invariant content that gate references.

The five worst portfolio gaps identified across the existing evidence corpus
are:

1. **No canonical cell-isolation + shuffle-sharding rules** — every cloud
   product implicitly assumes cell isolation but no machine-readable rule
   defines what "cell" means or what cross-cell traffic is forbidden.
2. **No idempotency key mandate** — distributed retries produce duplicate
   state without it; no existing fitness lane enforces this.
3. **No supply-chain SLSA/Cosign mandate** — identified in
   hyperscaler-best-practices.md §executive-summary as top-3 gap item A.
4. **No progressive delivery rule** — identified in hyperscaler-best-practices.md
   as top-3 gap item B; ADR-0114 designs the canary gate but no invariant
   names it as binding.
5. **No data-perimeter + residency rule** — cloud PRD §10 lists cross-region
   residency leaks as a catastrophic risk but no invariant mandates the
   enforcement mechanism.

All five are now covered by INV-* entries in the new spec.

## Decision

`specs/hyperscaler-architecture-invariants.json` (spec_id:
EXE-HYPERSCALER-ARCH-INVARIANTS, version 1.0.0) is the canonical,
machine-readable, binding source of truth for what "hyperscaler-grade"
means in the Oyatie portfolio. This PR lands the catalog validator; it does not
claim that product PRDs are already blocked on the catalog.

Binding rules:

1. **Portfolio-wide applicability.** All 11 products (cloud, foundry,
   workflow, workflow-studio, ontology, connect, saas, vertical, ads, search,
   workspace) have an advisory required-invariant set in
   `per_product_required_compliance[product]`. Product readiness claims must
   provide evidence against that set before they can use the catalog as proof.

2. **PRD citation requirement.** The planned product PRD validator must require
   each product PRD `hyperscaler_bar` section to enumerate the INV-* IDs it is
   required to comply with and provide or reference fresh implementation and
   operational evidence for each. Until that validator, fixture tests, workflow,
   and branch-protection row exist, this requirement is advisory and review
   enforced.

3. **Fitness lane naming.** Each INV-* invariant names its planned
   `oya-governance-*` enforcement lane with `planned_enforced_by` and
   `planned_verification_command`. Planned lane names are not active required
   checks until their validators and CI contexts exist.

4. **Prose doc demotion.** `docs/standards/hyperscaler-best-practices.md`
   is demoted from authoritative-standard to research-context. Its header is
   updated to point to this spec as canonical. The prose remains valuable
   for rationale; it is no longer the claim surface.

5. **Relationship to ADR-0123.** ADR-0123 governs whether the exact phrase
   "we are hyperscaler mature" is allowed. This ADR governs what "hyperscaler"
   means architecturally. They are complementary: ADR-0123 is the claim gate,
   ADR-0128 + its spec are the invariant content that gate checks.

6. **Versioning.** Breaking changes to the invariant list (removal of an
   INV-*, tightening of a rule that invalidates existing evidence) require a
   new ADR. Additive changes (new INV-*) may be made via PR with council-
   architecture review; the spec version is bumped accordingly.

### Invariant count and categories

The 1.0.0 spec ships 35 invariants across 6 categories:

| Category | Count | Sample IDs |
|---|---|---|
| reliability | 13 | INV-CELL-ISOLATION, INV-SHUFFLE-SHARDING, INV-STATIC-STABILITY, INV-IDEMPOTENCY, INV-OUTBOX-PATTERN, INV-SAGA-COMPENSATION, INV-CIRCUIT-BREAKER-BULKHEAD, INV-AVOID-FALLBACK, INV-MULTI-REGION-FAILOVER, INV-CAPACITY-RESERVATION, INV-SLO-ERROR-BUDGET, INV-PROVIDER-DEGRADED-SHED, INV-TYPE-SAFETY-BOUNDARY |
| security | 9 | INV-IAM-LEAST-PRIVILEGE, INV-KMS-ENVELOPE-ENCRYPTION, INV-DATA-PERIMETER, INV-SUPPLY-CHAIN-SLSA, INV-CONFIDENTIAL-COMPUTE, INV-AUDIT-CHAIN-EMIT, INV-DATA-RESIDENCY, INV-MULTI-TENANCY-ISOLATION, INV-CARGO-VET-SUPPLY-CHAIN |
| operational_excellence | 8 | INV-SLO-ERROR-BUDGET, INV-FOUR-GOLDEN-SIGNALS, INV-USE-METHOD, INV-OBSERVABILITY-TRACING, INV-STRUCTURED-LOGS, INV-BLAMELESS-POSTMORTEM, INV-PROGRESSIVE-DELIVERY, INV-TOIL-REDUCTION |
| performance_efficiency | 3 | INV-BACKPRESSURE-LOAD-SHEDDING, INV-EDGE-CDN-OFFLOAD, INV-SPEED-AS-FEATURE |
| cost_optimization | 1 | INV-FINOPS-COST-TAGGING |
| sustainability | 1 | INV-SUSTAINABILITY-METRICS |

The table is generated from each invariant row's single `category` field.
Total unique INV-* IDs: 35.

## Rejected alternatives

- **Keep prose doc as canonical**: rejected. Machine-readable specs are the
  only format that validators and gate commands can check programmatically.
  Prose cannot be linted.

- **Fold invariants into hyperscaler-gates.json**: rejected. That file governs
  boolean claim status; it is not structured for per-invariant rule +
  rationale + planned lane tuples. Merging would conflate governance with
  content.

- **Author a new ADR per invariant**: rejected. 35 ADRs for 35 rules creates
  maintenance overhead that exceeds the value; a single versioned spec with a
  single binding ADR is the correct granularity for a portfolio-wide standard.

## Consequences

- Product PRDs have a canonical INV-* target set for their future
  `hyperscaler_bar` evidence.
- Product PRD citation enforcement remains advisory until the validator,
  fixture tests, pull-request workflow, and branch-protection row land together.
- `docs/standards/hyperscaler-best-practices.md` header is updated to reflect
  research-context status and pointer to this spec.
- New planned `oya-governance-*` lanes named in the spec that do not yet
  exist remain backlog items, not active required checks.
- The `specs/hyperscaler-gates.json` `canonical_sources` array is updated to
  include `specs/hyperscaler-architecture-invariants.json`.

## Verification

```
cargo run -p oya-dev-cli -- gate validate hyperscaler-arch-invariants
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims
python3 -m json.tool specs/hyperscaler-architecture-invariants.json > /dev/null
```

## References

- `specs/hyperscaler-architecture-invariants.json` — the spec this ADR binds
- `docs/standards/hyperscaler-best-practices.md` — demoted to research context
- `specs/hyperscaler-gates.json` — claim-governance registry (ADR-0123)
- ADR-0114 — canary observability gate (referenced by INV-PROGRESSIVE-DELIVERY)
- ADR-0119 — flat-root topology (JSON-with-pointers schema used by the spec)
- ADR-0123 — hyperscaler maturity claim gate (consumer of this spec)
- AWS Builders Library — https://aws.amazon.com/builders-library/
- Google SRE Book — https://sre.google/sre-book/
- Google SRE Workbook — https://sre.google/workbook/
- Microsoft Azure Well-Architected — https://learn.microsoft.com/en-us/azure/well-architected/
- Stripe Engineering Blog — https://stripe.com/blog/engineering
- Palantir Foundry docs — https://www.palantir.com/docs/foundry/
- Linear Engineering Blog — https://linear.app/blog
