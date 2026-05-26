---
doc_class: DesignNote
title: Workload-Identity Cost / FinOps
microservice: identity
bounded_context: workload-identity
status: Proposed
date: 2026-05-26
owner_team: axis-identity + council-finops
related_adrs: [ADR-0002, ADR-0344]
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# Workload-Identity — Cost / FinOps

## Dominant cost driver

Per the cited brief (§8), the dominant cost driver for an authorization substrate
is **authorize call volume**. Every inter-service call in the fleet can become an
authorize call; at fleet scale this is the single largest cost line. Token
validation is secondary but co-located on the hot path.

## Adopted mitigations (brief §8)

### 1. Batching — `POST /authorize:batch`

The AVP `BatchIsAuthorized` analog. Up to 30 PARC tuples that share a
`policyStoreId` (and optionally a shared `entities` set) are decided in one round
trip, collapsing per-call network and TLS-handshake cost. A PEP that needs to
authorize a fan-out (e.g. one request touching N resources) issues one batch
instead of N calls.

### 2. PEP-side short-TTL decision cache

Per the brief's caution (§8), decision caching trades cost for staleness — a
suspended principal could retain access until the cached decision expires.
Therefore:

- The cache lives at the **PEP**, not the PDP (the PDP stays authoritative).
- Max TTL is **coupled to the revocation SLO**: the cache TTL must be ≤ the
  acceptable revocation latency, so a suspend takes effect within the same
  bound as token TTL + denylist propagation (`design/failure-modes.md` F10/F11).
- Caching is opt-in per PEP and defaults conservative.

### 3. Embedded in-process Cedar for hot paths

The biggest single cost win (brief §8): the swap-in `WorkloadAuthorizer` trait
lets hot-path callers run Cedar **in-process**, eliminating the per-call network
cost of a centralized PDP entirely. AVP itself documents embedding the Cedar SDK
for intermittent-access cases; we generalize it to any hot path. The same policy
files (`policy/identity.cedar`) drive both the embedded and the centralized PDP,
so behavior is identical — only the transport differs.

## Cost-vs-latency-vs-staleness triangle

| Lever | Cost | Latency | Staleness risk |
|---|---|---|---|
| Centralized PDP, no cache | high (per-call network) | +network hop (brief §4) | none |
| Centralized PDP + batch | medium | amortized | none |
| PEP decision cache | low | very low | bounded by TTL ≤ revocation SLO |
| Embedded Cedar | lowest | in-process | none (always re-evaluates) |

The recommended default for hot paths is **embedded Cedar**; batch for fan-outs;
PEP cache only where the workload tolerates the bounded staleness.

## ADR-0344 emission alignment

Consistent with the human-identity PRD's ADR-0344 posture, each authorize/validate
decision record can carry `cost_usd_minor_units`, `co2_grams`, `watt_hours`,
`provider`, and `region` alongside the audit fields, so authorize volume is
visible in the FinOps portal. Carbon-aware routing is **not** applied to the
interactive authorize/validate hot path (latency + correctness dominate); it is
acceptable only for background reconciliation (e.g. golden-corpus replay,
denylist compaction).

## References

Brief §4 (PDP network hop) + §8 (caching/batching/embedded Cedar); ADR-0344.
Staleness coupling: `design/failure-modes.md`. SLO: `slos/authorize-latency-p99.openslo.yaml`.
