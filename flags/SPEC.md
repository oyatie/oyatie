---
doc_class: Owner-SPEC
owner: flags
status: Active
date: 2026-08-26
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - flags/ADR.md
  - flags/PRD.md
---

# Flags technical specification

<landed_contract>

## Current implementation truth

`flags-evaluation-domain` is a dependency-free Rust crate. Its current public
model is `Flag`, `Variant`, `Rule`, `Condition`, `Rollout`, and
`EvaluationContext`; `evaluate(&Flag, &EvaluationContext)` returns a variant,
typed value, reason, and optional error code.

Current precedence is disabled, first matching rule, default rollout, then
default variant. Rules are ordered and conjunctive. `Eq`, `NotEq`, `In`, and
`NotIn` compare normalized boolean/string/integer attributes. Percentage
assignment hashes the NUL-separated `(flag_key, salt, targeting_key)` with fixed
64-bit FNV-1a into 10,000 basis points. No-variant input returns `NoVariants`
with an empty key and Boolean false; a selected unknown reference returns
`UnknownVariant` with the resolvable off/first fallback. Neither path panics.

The source trait is synchronous and currently defined inside the core crate; a
unit-test-only vector implementation is its sole adapter. Definitions have no
tenant, generation, schema admission, explicit kill state, verified override,
or size limits. Some invalid references remain latent until selected.

`flags/core/server` does not serve: main loads an empty config, calls a no-op
initializer, and exits. REST, gRPC, OFREP, storage, targeting, and tenancy
modules contain no implementation. No root Cedar/IaC/OpenSLO file is wired to a
runtime producer.

</landed_contract>

<target_model>

## Definition snapshot

The canonical semantic definition includes:

```text
tenant_id
flag_key
schema_version
evaluation_algorithm_version
generation
enabled
kill_state + kill_variant
variants[]
ordered_rules[]
default_rollout?
default_variant
off_variant
policy_revision + admitted_override_generation
content_digest
```

`generation` is monotonically increasing within `(tenant_id, flag_key)` and is
never reused. Wall time may be audit metadata but does not order evaluation.
Rollback publishes a higher generation containing the prior admitted semantics.

Evaluation context contains a stable targeting key and bounded normalized
attributes. Verified override context is a separate unforgeable value carrying
tenant, Policy issuer/revision, audience, expiry/request binding, installed
pack/C0 generation, admitted override, and proof digest. It cannot be constructed
from ordinary attributes.

</target_model>

<admission_v1>

## L1d bounded definition admission

The first behavioral slice adds validated types before any network or store.
V1 limits are deliberately fixed so evaluation work can be budgeted:

| Dimension | Maximum |
|---|---:|
| flag key, rule id, variant key, attribute key, rollout salt | 128 UTF-8 bytes each |
| variants | 32 |
| ordered rules | 64 |
| conditions per rule | 8 |
| rollout buckets per rollout | 32 |
| evaluation attributes | 64 |
| targeting key | 256 UTF-8 bytes |
| string attribute or set member | 256 UTF-8 bytes |
| set members in one condition | 64 |
| `FlagValue::Str` | 4,096 UTF-8 bytes |
| `FlagValue::Object` entries | 64 |
| object key | 128 UTF-8 bytes |
| object value | 4,096 UTF-8 bytes |
| total UTF-8 string/payload bytes in one definition | 1,048,576 bytes |
| total UTF-8 bytes in one normalized evaluation context | 32,768 bytes |

Lengths are UTF-8 byte lengths, not character counts. The definition aggregate
is the sum of every identifier, salt, string operand/set member, and string or
object key/value payload; the context aggregate is the targeting key plus every
attribute key/string value. Scalar booleans and integers are fixed-width.
`FlagValue::Float` admits finite `f64` values only: NaN and positive/negative
infinity return a typed validation error and never become wire-visible. The
existing zero-byte anonymous targeting key remains valid, preserving the landed
golden vector, but it is still charged to the context count and byte limits.

Admission rejects empty/duplicate flag, variant, or rule identifiers; no
variants; missing off/default/kill/rule/rollout references; duplicate rollout
variant entries; individual or cumulative weights above 10,000 basis points;
integer overflow; non-finite float payloads; operator/operand mismatch; any
individual maximum+one; and either aggregate-byte maximum+one. A rollout below
10,000 remains valid and its unallocated remainder falls through to the
definition default, preserving current behavior.

Successful admission produces a `ValidatedFlag` that alone enters the new hot-
path evaluator. Context admission produces a `ValidatedEvaluationContext`.
Validation errors are stable typed values and do not yield an evaluation. The
existing unvalidated function may remain only as an explicitly deprecated
compatibility wrapper during one versioned internal transition; no facade may
call it.

</admission_v1>

<evaluation>

## Target evaluation order

For an admitted snapshot and context:

1. If explicit kill state is engaged, return the admitted kill variant with
   `KillSwitch` reason.
2. If the definition is disabled, return the off variant with `Disabled`.
3. If a verified C0 override matches this tenant, flag, definition generation,
   and Policy revision, return its admitted variant with `PolicyOverride`.
4. Evaluate ordered rules; the first complete match wins. A fixed outcome
   returns `TargetingMatch`; a rollout assignment returns `Split`.
5. Evaluate the default rollout.
6. Return the default variant.

The L1d slice implements admission and evaluates the existing disabled/rule/
rollout/default semantics only. Explicit kill and C0 override behavior land
later with their authority inputs; L1d must not simulate either with caller
attributes.

The algorithm is total for validated inputs and has no I/O, time, RNG, or
mutable global state. Every result includes definition generation and algorithm
version once those fields land. A version change ships new golden vectors and a
bounded dual-evaluation comparison before promotion; silent re-bucketing is a
failure.

</evaluation>

<control_and_distribution>

## Authority protocol

1. The Connect control facade authenticates, obtains request-bound Policy
   authorization/C0 context, normalizes input, and runs complete admission.
2. The authoritative store checks `(tenant, operation, idempotency_key)` and
   expected prior generation.
3. One transaction commits the next definition, request digest/result, and
   durable audit/outbox intent before acknowledgement.
4. A distributor signs a snapshot/delta binding tenant, prior/next generation,
   algorithm version, content digest, and policy/override generation.
5. A cell evaluator verifies provenance, ordering, schema, digest, and complete
   definition admission off the hot path, then atomically swaps the immutable
   snapshot pointer.
6. Evaluation reads exactly one pointer and reports its generation. It never
   combines definitions from two snapshots.

Same-key/same-digest mutation replay returns the stored result; changed digest
returns conflict. Duplicate deltas are idempotent. Missing or out-of-order deltas
trigger bounded resynchronization, not speculative application. An interrupted
swap leaves the prior snapshot active. Restart loads only a completely verified
snapshot and reports staleness until it catches up.

Kill-switch updates follow the same authorization, transaction, audit, and
generation rules but use a priority distribution queue and retained engaged
state. Failure to fetch newer state never infers disengagement.

</control_and_distribution>

<interfaces_and_errors>

## Semantic facade

The canonical protobuf/Connect contract separates:

- evaluation/preview: flag key, normalized context, optional named generation,
  result value/variant/reason/generation/error;
- control: validate, create/update, stage/publish, disable, engage/disengage
  kill, retire, and get/list metadata;
- distribution: authenticated snapshot/delta stream and resync.

OpenFeature/OFREP adapters map their standard reasons/default behavior to this
contract. They do not own definition storage, generations, or authorization.

Stable failure classes are `InvalidDefinition`, `InvalidContext`, `NotFound`,
`Unauthenticated`, `Forbidden`, `GenerationConflict`, `IdempotencyConflict`,
`SnapshotStale`, `SnapshotCorrupt`, `SourceUnavailable`, `OverCapacity`, and
`InternalInvariant`. Wire status is an adapter mapping, not the domain model.
Where a safe fallback is registered, errors identify that it was used; absence
of a valid fallback is a typed failure, never fabricated success.

</interfaces_and_errors>

<fault_and_operability>

## Required evidence

- Golden vectors bind algorithm version, normalized inputs, bucket, variant,
  value, reason, and error across supported architectures and N/N+1 toolchains.
- Property/fuzz tests cover arbitrary definitions/contexts, all admission
  boundaries, panic freedom, rollout totals, rule precedence, determinism, and
  safe fallback.
- Mutation tests interrupt before/after definition, idempotency, and audit/
  outbox writes and after commit-before-response; reopen and replay establish
  one committed generation.
- Distribution simulation duplicates, drops, reorders, corrupts, truncates, and
  partitions snapshot traffic; atomic readers observe the prior or verified next
  generation only.
- Security tests forge or replay IAM/PDP/C0 proofs, substitute tenant/audience/
  request, inject policy-looking targeting attributes, and race kill with an
  ordinary update; no unauthorized commit or evaluation disclosure occurs.
- Load tests use maximum admitted definitions/contexts at declared capacity,
  include update and kill propagation, and report p99/p99.9 latency, queueing,
  memory, fallback, and generation skew.

Metrics and traces use bounded labels: operation, result/reason/error class,
algorithm/schema version, generation skew bucket, adapter, and saturation state.
They exclude flag keys, targeting keys, raw attributes, proof material, and
variant payloads. Readiness remains false until the selected facade and verified
local snapshot path exist; generated SLO output cannot precede its live metric
producer.

</fault_and_operability>
