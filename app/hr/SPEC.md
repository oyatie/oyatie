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
Cloud capability crates never import this app in the reverse direction. In
particular, IAM supplies identity and authorization evidence at its sold
boundary; it does not host HR routes, fixtures, stores, or an HR client crate.

</target_architecture>

<connect_boundary>

## Sold People wire contract

The reserved sold IDL identity is
`app/hr/facade/proto/hr/people/v1/people_service.proto`, protobuf package
`hr.people.v1`. It declares only unary `OnboardEmployee` and `GetEmployee`
methods. The directory matches the semantic package component; literal `api`
is not used as a placeholder segment. The server-side adapter
`adapters/draft/transport-connect` / `hr-transport-connect-draft` implements the
matching owner-local `ports/draft/transport` / `hr-transport-draft` port, and
`facade/people-app` / `hr-people-app` composes it with the use cases. There is
no provider-owned People client adapter. No other owner may import either draft
crate; a future Rust consumer requires a separate D-28 provider-port promotion,
external API review, and a client adapter owned by that consumer.

This target is not dispatchable today. The reviewed workspace has no accepted
Connect generator/runtime target. L2f.0a must first record a protocol-owner,
architecture, and Build accepted implementation with exact versions/features,
Cargo/Buck targets, generated inputs and outputs, license/removal policy, and
wire/fault evidence. HR then consumes its generated service bindings. HR source
does not parse Connect HTTP, frame protobuf, serialize Connect error envelopes,
or decide trailer behavior. Message-only generation plus hand-written framing
is not an implementation of this contract.

V1 wire behavior is the Connect unary protocol:

```text
POST /hr.people.v1.PeopleService/OnboardEmployee
POST /hr.people.v1.PeopleService/GetEmployee
Content-Type: application/proto
Connect-Protocol-Version: 1
body: one bare protobuf message (no five-byte gRPC envelope)
```

A successful response is HTTP 200 with `application/proto` and one bare
protobuf message. A failure uses the stable Connect-code-to-HTTP mapping and an
`application/json` Connect error body containing bounded `code` and redacted
`message` fields (v1 emits no `details`); it never relies on HTTP trailers,
`grpc-status`, or `grpc-message`. V1 rejects GET, JSON product payloads,
`application/grpc*`, `application/connect+proto` streaming envelopes,
unsupported compression, unknown method paths, missing/wrong protocol version,
oversized headers or body, malformed protobuf, extra framed messages, and any
outcome that requires trailers.

V1 admission uses the following binary units and closed bounds. `KiB` is 1,024
octets and `MiB` is 1,048,576 octets. Operators may lower a default, but no
configuration may exceed the hard maximum.

| Resource | Default | Hard maximum |
|---|---:|---:|
| decoded HTTP header fields | 24 | 32 |
| aggregate decoded header-name plus value octets | 12 KiB | 16 KiB |
| one decoded header value | 4 KiB | 8 KiB |
| encoded protobuf request body | 128 KiB | 256 KiB |
| one decoded protobuf string/bytes value | 4 KiB | 8 KiB |
| aggregate owned string/bytes after protobuf decode | 64 KiB | 128 KiB |
| known plus unknown protobuf field occurrences | 96 | 128 |
| entries in any repeated message field | 32 | 64 |
| protobuf nesting depth | 8 | 8 |
| active requests per tenant | 32 | 64 |
| queued requests per tenant | 32 | 64 |
| active requests per cell | 2,048 | 4,096 |
| queued requests per cell | 4,096 | 8,192 |
| reserved request bytes per tenant | 8 MiB | 16 MiB |
| reserved request bytes per cell | 512 MiB | 1 GiB |
| request deadline when absent | 5,000 ms | 30,000 ms |

`Connect-Timeout-Ms`, when present, is canonical ASCII decimal
`[1-9][0-9]{0,4}` in the inclusive range 1..=30,000; sign, whitespace, zero,
leading zero, overflow, and a longer value are `invalid_argument`. Every count,
byte sum, and reservation product uses checked `u64` arithmetic; overflow
rejects the request as `invalid_argument`, while an overflowing or above-hard-
maximum configuration prevents the process from becoming ready. A missing
content length reserves the full configured body maximum before reading.
Cancellation releases queue, active, and byte reservations exactly once.

Validation order is stable: method/path; header-count and byte accounting;
content/protocol/compression/deadline grammar; channel principal and tenant
binding; tenant then cell queue/active/byte admission; bounded body read;
generated protobuf decode plus decoded-work limits; request-resource/PDP/pack
binding; then use-case dispatch. Invalid protocol is
`invalid_argument`/400, an exceeded work or queue bound is
`resource_exhausted`/429, and an expired deadline is
`deadline_exceeded`/504. All three occur before repository mutation. Exact-limit
and limit-plus-one vectors cover every row, and saturation/cancellation proves
reservation recovery without queue growth.

The v1 mapping is fixed: validation is `invalid_argument`/400;
unauthenticated is `unauthenticated`/401; forbidden is
`permission_denied`/403; duplicate/idempotency conflict is
`already_exists`/409; optimistic-version abort is `aborted`/409; bounded-load
rejection is `resource_exhausted`/429; adapter outage is `unavailable`/503; and
corrupt/impossible state is `data_loss` or `internal`/500. Protocol parse errors
cannot be relabeled as domain success, and error messages contain no employee,
person, evidence, credential, or raw policy value.

The accepted generator must emit the Connect service/handler and message
bindings under `OUT_DIR`; Cargo and Buck stage the same IDL/import inputs and
compile the same generated membership. The structural generator must tolerate
the schema being absent so package/build/lock admission can precede the
schema-only external-contract lane. The exact runtime and build dependencies
remain deliberately unnamed until L2f.0a accepts them and amends the owner plan;
placeholders are not dispatch authority. Neither `hr-transport-connect-draft`
nor `hr-people-app` may depend on `tonic`, `tonic-prost`, `tonic-build`, or
`tonic-prost-build`, and no gRPC service/client code is generated.

Golden tests compare exact request, success, and generated Connect error bytes
through Cargo and Buck. Negative vectors cover truncated/overlong protobuf, a
gRPC five-byte prefix, two concatenated messages, wrong content/protocol
headers, gRPC status metadata, attempted trailers, timeout, every exact bound
and bound-plus-one, queue saturation, and adapter cancellation. Each rejection
is bounded, classified, observable without sensitive fields, and occurs before
repository mutation.

</connect_boundary>

<people_onboarding>

## Narrow first feature contract

After L2e and the accepted L2f.0a generated-Connect gate, the first People slice
is durable employee onboarding using the already-landed `onboard_employee`
behavior. It adds no new employment-law rule.

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
The repository port supplies a versioned canonical request byte sequence; the
adapter stores its SHA-256 digest plus versioned outcome bytes. The SQLite
adapter's only runtime dependencies are `hr-employment-repository-draft`,
`rusqlite.workspace = true`, and `sha2.workspace = true`. Its only
dev-dependencies are `hr-employment-repository-memory-draft` and
`tempfile.workspace = true`; recovery targets use the real SQLite adapter and
`tempfile`, never the memory oracle or `:memory:`.
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
