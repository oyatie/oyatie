---
doc_class: Owner-PRD
owner: flags
status: Active
date: 2026-08-26
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - flags/ADR.md
---

# Flags product requirements

<product_boundary>

`flags/` is the cloud capability for tenant-scoped runtime dynamic
configuration and emergency kill switches. It owns deterministic evaluation,
ordered targeting, percentage assignment, versioned definitions, and pack-
gated overrides supplied through verified Policy C0 context.

Flags is not an experimentation/statistics product, a code-deployment gate, a
workflow engine, a clock, a cell-placement service, a Cedar PDP, or a trusted-
tenant mechanism. It provides one Connect semantic facade; OpenFeature/OFREP
and language SDKs are compatibility adapters.

</product_boundary>

<users>

- Application teams evaluate typed dynamic configuration locally and receive a
  variant, stable reason, definition generation, and correlation-safe error.
- Authorized operators create, validate, stage, publish, disable, and retire
  definitions with idempotency and immutable audit evidence.
- Incident responders engage an explicit kill switch that outranks overrides,
  targeting, and rollout assignment.
- Policy and compliance owners admit tenant/jurisdiction overrides through C0
  context without turning Flags into a second PDP or pack loader.
- Operators need snapshot age/generation, propagation, deterministic mismatch,
  safe-fallback, saturation, and audit/outbox telemetry without targeting PII.

</users>

<landed_scope>

## Current foundation

The landed `flags-evaluation-domain` crate evaluates an in-memory `Flag` value
against an `EvaluationContext`. It supports boolean/string/integer/float/object
variants, equality and set targeting, first-match precedence, fixed and
percentage outcomes, default rollout, disabled/off behavior, deterministic
FNV-1a bucketing, and typed errors for no variants or a selected unknown
variant. Nineteen unit tests cover those paths and a static source test double.

No durable flag definition, admission validator, explicit kill-switch state,
verified C0 override, mutation API, Connect facade, snapshot stream/cache,
production SDK, deployed evaluator, authorization path, audit emission,
observability producer, or SLO measurement has landed. `core/server`, root IaC,
Cedar, and OpenSLO files do not provide those capabilities.

</landed_scope>

<requirements>

## Definition and evaluation

- Represent tenant, flag key, definition generation, typed variants, explicit
  safe/off and kill variants, ordered rules, percentage buckets, and schema/
  algorithm version without provider or wire types in core.
- Validate the entire definition before admission: identifiers and references,
  unique keys/IDs, operator/operand shape, rollout sums, size/work bounds, and
  safe fallback must be known before any subject evaluates it.
- For one admitted generation and normalized context, return the same variant,
  value, reason, and error on every supported architecture and replay.
- Apply precedence as engaged kill switch, disabled definition, verified
  pack-gated override, first matching rule, default rollout, then default
  variant. Malformed or unavailable authority returns a typed failure and safe
  fallback, never an optimistic value.
- Keep the hot path free of storage, network, policy, pack, audit, clock, RNG,
  and mutable-global calls. Bound definition and context sizes so peak work is
  predictable.

## Mutation authority and distribution

- Authenticate and authorize every create/update/publish/disable/kill/retire
  request through the normal IAM and Policy path with tenant/action/resource/
  request binding.
- Commit one monotonic definition generation, idempotency outcome, and durable
  audit/outbox intent before acknowledging a mutation.
- Distribute signed immutable snapshots or ordered deltas; evaluators atomically
  publish a verified generation and never expose a partially decoded snapshot.
- Make ordinary evaluation independent of control-plane reachability. Expose
  last-known generation and staleness explicitly; never invent a newer state.
- Prioritize kill-switch propagation without allowing it to skip authorization,
  audit, generation ordering, or tenant isolation.

## Policy, tenancy, and data handling

- Keep arbitrary targeting attributes separate from verified Policy C0
  override context. Caller fields cannot grant policy authority.
- Bind every definition, mutation, snapshot, cache entry, and metric to one
  tenant. Cross-tenant reads, writes, cache keys, and replay are refused.
- Receive pack-gate outcome and provenance from Policy; do not fetch or parse
  pack content in Flags.
- Treat targeting attributes as potentially sensitive. Minimize them, avoid raw
  values in logs/metrics, and provide per-attribute allow/normalization rules at
  the facade.

## Interfaces and lifecycle

- Publish one versioned protobuf/Connect semantic contract for evaluation and
  separately authorized control operations.
- Derive OpenFeature/OFREP compatibility and supported SDKs from the same
  reasons, errors, types, and generation semantics.
- Support staged publication, deterministic preview against a named generation,
  rollback by publishing a higher generation, and explicit retirement. Never
  decrement or silently reuse a generation.
- Meter mutations, snapshot distribution, and evaluation classes without
  putting Billing or Observability in the evaluation hot path.

## Operability

- Emit bounded-cardinality metrics for evaluation latency/reason/error,
  definition validation, control commit, snapshot verify/publish/age,
  generation skew, kill propagation, safe fallback, queue saturation, audit
  outbox lag, and deterministic replay mismatch.
- Readiness is false without a supported verified local snapshot and the
  authority required by the advertised operation. Pure core tests never make a
  network endpoint ready.
- Bound queues and in-flight bytes. Reject mutations/distribution work
  retryably before memory growth; evaluation retains a pre-admitted safe path.

</requirements>

<slo_objective>

## Promotion objective (unmeasured and unqualified today)

At no more than 70% of a declared evaluator capacity envelope:

- monthly cell-local evaluation facade availability: **99.99%**;
- p99 cell-local evaluation latency: **1 ms**, with no remote call on the hot
  path;
- ordinary committed-definition propagation p99: **5 seconds**;
- authorized kill-switch commit-to-active-generation p99: **1 second** within
  the declared cell set;
- deterministic golden-vector mismatch: **zero**;
- cross-tenant or unauthorized mutation/evaluation disclosure: **zero**.

These are objectives, not current claims. They become advertised only after the
Connect facade, live metric producers, generated SLO output, load envelope,
multi-cell fault campaign, and Observability promotion gate exist.

</slo_objective>

<acceptance>

## Success

- A completely validated definition evaluates deterministically with bounded
  work and stable reason/error semantics.
- An authorized idempotent mutation publishes one higher generation with audit
  evidence; evaluators atomically move from the prior verified snapshot to the
  next and report which one they used.
- Engaged kill state always outranks pack override, targeting, and rollout; a
  stale or corrupt update cannot silently disengage it.
- Native Connect and retained OpenFeature/OFREP adapters pass the same contract
  vectors, tenant-isolation cases, and safe-fallback behavior.
- Readiness and SLO outputs name only signals and capabilities actually produced
  by the running implementation.

## Failure

- Identical admitted inputs diverge across process, architecture, toolchain, or
  replay without an explicit algorithm-version transition.
- An oversized/malformed definition reaches the hot path, panics, performs
  unbounded work, or serves an optimistic value.
- A forged C0/PDP proof, caller targeting attribute, wrong tenant, duplicate or
  reordered delta mutates authority or exposes another tenant's decision.
- A partial snapshot is visible, generation moves backward, or restart clears a
  previously engaged kill state without a higher authorized generation.
- Placeholder server, IaC, Cedar, or OpenSLO text is cited as runtime/SLO proof.

## Named fault campaigns

- Empty/duplicate variants and rules; unknown off/default/kill/rule/rollout
  variants; invalid operator shapes; zero/overflow/over-100% weights; maximum+
  one definition/context dimensions; arbitrary bytes and Unicode boundaries.
- Snapshot truncation, signature/digest corruption, duplicate/out-of-order/
  missing deltas, atomic-swap interruption, restart on old generation, and
  partition from the control plane.
- Response loss after mutation commit followed by same-key replay and changed-
  digest replay; audit/outbox outage before acknowledgement.
- Forged/expired/wrong-audience C0 proof, cross-tenant cache key, kill-switch
  race with ordinary update, queue saturation, and evaluator process loss.

</acceptance>
