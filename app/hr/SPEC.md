---
doc_class: Owner-SPEC
owner: app/hr
status: Active
date: 2026-08-26
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - app/hr/ADR.md
  - app/hr/PRD.md
---

# HR technical specification

<landed_contract>

## Current implementation truth

The current domain is a deterministic Rust library. It validates typed
employee, tenant, legal-entity, person, evidence, workflow, policy, leave, and
rulepack references and produces classified records and intents. Its landed
operations are:

- employee construction and lifecycle-event creation;
- Korea labor-threshold obligations and workflow metadata;
- leave payroll-impact, accrual/balance, and carryover/forfeiture projections;
- onboarding readiness with stable blocker kinds and evidence checks;
- sensitive-read allow decisions only for admitted purpose/legal-basis cases;
- statutory rulepack manifests from validated official-source metadata.

`employment-app` wraps some of these results in metadata-only audit, workflow,
payroll-impact, and sensitive-read envelopes. `employment-api` provides Serde
JSON DTO conversions. `employment-infrastructure` dispatches those DTOs through
an in-process HTTP router with bearer, tenant-match, and authorizer seams. The
in-memory storage adapter retains only derived envelope metadata in a process-
local map.

There is no listener/deployment proof, durable repository, transactional use
case, installed-pack resolver, downstream delivery, or SLO implementation. The
current HTTP and storage types are compatibility evidence, not the target
semantic or durability contract.

</landed_contract>

<target_architecture>

## Dependency direction

```text
versioned HR facade
        |
portable use cases + domain
        |
app/hr-owned ports
        |
SQLite | commodity | Oyatie sold-facade adapters
```

The facade translates a versioned request into portable input, carries verified
caller/policy context, invokes a use case, and translates the typed result. Core
owns business decisions but performs no I/O. Ports own the narrow capabilities
the use cases require. Adapters implement them without leaking wire, database,
or cloud types inward.

The first port set is:

| Port | Required semantic boundary |
|---|---|
| employee repository | atomic employee/lifecycle/idempotency outcome read and write |
| installed HR overlay | resolve admitted pack-id to verified HR rule content and generation |
| authorization evidence | supply verified principal/tenant/action/resource/PDP provenance, never caller allow fields |
| audit/outbox | durably bind pre-ack evidence or intent and retry delivery |
| workflow dispatch | idempotently deliver labor/onboarding workflow intent |
| payroll-impact dispatch | idempotently deliver HR-owned payroll-impact intent, never payroll calculation |
| transport | versioned request/result/error values independent of Gateway core/runtime |
| observability/clock | correlation-safe signals and trusted time where policy requires it |

Port values use HR-owned identifiers and classification vocabulary. A Data or
Gateway adapter performs explicit translation at its outer edge. Between-app
composition is also an adapter call; HR never imports Payroll or Workflow core.

</target_architecture>

<people_onboarding>

## Narrow first feature contract

After L2e, the first People slice is durable employee onboarding using the
already-landed `onboard_employee` behavior. It adds no new employment-law rule.

Input contains tenant, legal entity, employee/person/manager references,
employment state, tier snapshot, lifecycle event id, evidence reference,
schema version, correlation id, and idempotency key. Authorization and installed
overlay context are verified before domain evaluation. A successful result
contains the employee, one created lifecycle event, and the durable outcome
identity. Reads address `(tenant_id, legal_entity_id, employee_id)` and never
cross tenant scope.

Onboarding readiness remains a separate deterministic decision. An employee is
not silently promoted to active because a record exists; a future activation
command must explicitly consume a `Ready` result and its evidence generation.

</people_onboarding>

<durable_transaction>

## SQLite v1 commit and replay protocol

The SQLite schema is adapter-private and versioned. At minimum it persists
employee state, lifecycle events, idempotency outcomes, and audit/outbox intent.
The logical idempotency key is:

```text
(tenant_id, operation_kind, idempotency_key)
```

The stored entry binds a canonical request digest, outcome bytes/version, and
commit generation. Processing is:

1. Validate syntax, tenant binding, verified authorization, and overlay
   generation without mutation.
2. Evaluate the deterministic domain command.
3. Begin one SQLite transaction and inspect the idempotency entry.
4. If the entry is committed with the same digest, return its stored outcome.
   If its digest differs, return `IdempotencyConflict` without mutation.
5. Otherwise write the idempotency entry, employee mutation, lifecycle event,
   and durable audit/outbox intent in the same transaction.
6. Commit durably according to the adapter profile, then acknowledge. A caller
   disconnect after commit does not undo the transaction.

The adapter never exposes `PREPARED` state. Interruption before commit rolls all
four effects back. Interruption after commit recovers all four. Cleanup or
delivery failure may leave a retryable outbox entry, never a missing employee or
second business effect. SQLite-to-cloud transition is cohort cutover between
single active adapters, not dual-write.

</durable_transaction>

<error_model>

## Stable failure classes

| Class | Examples | Mutation/disclosure |
|---|---|---|
| validation | malformed identifier/date/evidence, invalid checklist, changed payload on reused key | none |
| unauthenticated | missing, invalid, expired, or unbound principal proof | none |
| forbidden | PDP deny, tenant mismatch, absent legal basis, stale/conflicting overlay | none |
| conflict | employee version mismatch, duplicate identity, idempotency digest mismatch | none |
| unavailable | SQLite busy/full/unopenable, audit precondition unavailable, selected adapter unhealthy | none acknowledged |
| internal/corrupt | schema incompatibility, corrupt stored outcome, impossible state | fail closed; readiness false |

Wire adapters map these typed classes to their protocol without making status
codes the domain model. Logs carry class, operation, correlation id, policy and
overlay generation, and adapter identity; they do not carry bearer material,
person reference, evidence contents, or sensitive payload.

</error_model>

<conformance_and_faults>

## Required evidence

One parameterized repository/use-case conformance suite runs unchanged against
the in-memory reference and SQLite; promoted Postgres/Data/on-prem adapters join
the same suite. It proves:

- create/read and lifecycle visibility are tenant and legal-entity scoped;
- same-key/same-digest replay returns byte-equivalent semantic outcome and does
  not duplicate employee, lifecycle, audit/outbox, workflow, or payroll intent;
- same-key/different-digest replay returns conflict without mutation;
- domain, authorization, overlay, and adapter failures preserve no partial
  state or sensitive disclosure;
- schema N/N+1 open, migrate, reopen, and supported rollback boundaries are
  explicit.

SQLite fault injection uses a real file and a fresh process or connection after
each interruption. Failpoints cover after `BEGIN`, idempotency insertion,
employee write, lifecycle write, audit/outbox write, before `COMMIT`, after
`COMMIT` before response, and migration boundaries. Every case closes without
graceful cleanup, reopens the database, checks invariants, and performs an
idempotent replay. In-memory tests are semantic reference evidence, not crash
durability evidence.

</conformance_and_faults>

<observability>

## Signals and SLO qualification

The facade measures admitted request latency, result class, policy/overlay
generation age, selected adapter, transaction phase/latency, replay/conflict,
outbox lag/redelivery, reopen/migration result, and saturation. Cardinality is
bounded; employee, person, evidence, and idempotency values are not labels.

Readiness is false when the selected durable adapter cannot open or commit, its
schema is outside the supported window, policy authority is unusable, or a
required pre-ack audit path cannot satisfy the request class. Health does not
claim durable, network, or downstream capability merely because pure domain
tests pass. The PRD SLO remains unqualified until these signals and the declared
load envelope are exercised in promotion evidence.

</observability>
