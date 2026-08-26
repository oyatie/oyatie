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
| record encryption | seal/open sensitive values, derive generation-scoped opaque idempotency locators and non-replay field indexes plus opaque commit bindings, and linearly authorize/resolve repository commits against key-generation transitions without exposing provider types |
| installed HR overlay | resolve admitted pack-id to verified HR rule content and generation |
| authorization evidence | supply verified principal/tenant/action/resource/PDP provenance, never caller allow fields |
| audit/outbox | durably bind pre-ack evidence or intent and retry delivery |
| workflow dispatch | idempotently deliver labor/onboarding workflow intent |
| payroll-impact dispatch | idempotently deliver HR-owned payroll-impact intent, never payroll calculation |
| transport | versioned request/result/error values independent of Gateway core/runtime |
| observability/clock | trusted time intervals plus bounded correlation-safe signal emission and health; no process-clock or facade-local production fallback |

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
configuration may exceed the hard maximum. A configured response-header or
error-body ceiling must still admit the fixed generated protocol headers and
the constant fallback error body; otherwise configuration validation keeps the
process unready.

| Resource | Default | Hard maximum |
|---|---:|---:|
| decoded HTTP header fields | 24 | 32 |
| aggregate decoded header-name plus value octets | 12 KiB | 16 KiB |
| one decoded header value | 4 KiB | 8 KiB |
| encoded protobuf request body | 128 KiB | 256 KiB |
| one decoded request string/bytes value | 4 KiB | 8 KiB |
| aggregate owned request string/bytes after decode | 64 KiB | 128 KiB |
| request known plus unknown field occurrences | 96 | 128 |
| entries in any repeated request field | 32 | 64 |
| protobuf nesting depth | 8 | 8 |
| active requests per tenant | 32 | 64 |
| queued requests per tenant | 32 | 64 |
| active requests per cell | 2,048 | 4,096 |
| queued requests per cell | 4,096 | 8,192 |
| reserved request bytes per tenant | 8 MiB | 16 MiB |
| reserved request bytes per cell | 512 MiB | 1 GiB |
| request deadline when absent | 5,000 ms | 30,000 ms |
| generated response header fields | 16 | 24 |
| aggregate response header-name plus value octets | 4 KiB | 8 KiB |
| encoded protobuf success body | 128 KiB | 256 KiB |
| one returned string/bytes value | 4 KiB | 8 KiB |
| aggregate returned owned string/bytes | 64 KiB | 128 KiB |
| response known plus unknown field occurrences | 96 | 128 |
| entries in any repeated response field | 32 | 64 |
| stored idempotency outcome bytes | 128 KiB | 256 KiB |
| redacted Connect error message UTF-8 octets | 256 | 512 |
| encoded Connect JSON error body | 1 KiB | 2 KiB |
| reserved response bytes per tenant | 8 MiB | 16 MiB |
| reserved response bytes per cell | 512 MiB | 1 GiB |

`Connect-Timeout-Ms`, when present, is canonical ASCII decimal
`[1-9][0-9]{0,4}` in the inclusive range 1..=30,000; sign, whitespace, zero,
leading zero, overflow, and a longer value are `invalid_argument`. Every count,
byte sum, and reservation product uses checked `u64` arithmetic; overflow
rejects the request as `invalid_argument`, while an overflowing or above-hard-
maximum configuration prevents the process from becoming ready. A missing
content length reserves the full configured body maximum before reading.
Cancellation releases queue, active, and byte reservations exactly once.

Output accounting is fail-closed and precedes HTTP response commitment. The
facade validates returned scalar, collection, and field-occurrence counts with
checked `u64` arithmetic, reserves the configured encoded-response maximum,
encodes into a capped buffer, verifies the final length, and only then emits
headers or body. It never truncates a protobuf value or collection. A result
above the configured ceiling is `resource_exhausted`/429 with no partial body;
a migrated or stored outcome above a hard maximum is `data_loss`/500 and makes
the affected serving cohort unready. New outcomes above the stored-outcome
ceiling abort the transaction before commit. Response reservations are released
exactly once after send completion, disconnect, cancellation, or encode error.

Connect error messages come only from a fixed, printable-ASCII redacted message
catalog whose every entry is proven to fit both configured error ceilings;
domain,
employee, person, evidence, credential, policy, pack, and adapter strings are
never interpolated. V1 emits no `details`. The runtime encodes the catalog value
into the bounded error buffer before committing headers. If even that mapping
cannot fit, it emits one constant bounded `internal` body and records only the
typed class plus correlation-safe metadata; it never exposes a partial or raw
fallback error.

Validation order is stable: method/path; header-count and byte accounting;
content/protocol/compression/deadline grammar; channel principal and tenant
binding; tenant then cell queue/active/byte admission; bounded body read;
generated protobuf decode plus decoded-work limits; request-resource/PDP/pack
binding; then use-case dispatch. Invalid protocol is
`invalid_argument`/400, an exceeded work or queue bound is
`resource_exhausted`/429, and an expired deadline is
`deadline_exceeded`/504. All three occur before repository mutation. Exact-limit
and limit-plus-one vectors cover every request and response row. Output vectors
include the largest allowed `GetEmployee`, every repeated-field boundary,
stored-outcome migration, error-message and JSON-escaping boundaries, encode
failure before headers, response loss, and tenant/cell response-byte
saturation. Saturation/cancellation proves request and response reservation
recovery without queue growth or partial disclosure.

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
headers, gRPC status metadata, attempted trailers, timeout, every request and
response exact bound and bound-plus-one, oversized migrated state, queue and
request/response-byte saturation, encode-before-header failure, and adapter
cancellation. Each rejection is bounded, classified, observable without
sensitive fields, and occurs before repository mutation or response commitment.

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
employee state, lifecycle events, idempotency outcomes, audit/outbox intent,
commit authorization, bounded rekey progress, and decommission-fence/scan state.
The repository port owns the canonical request and staged-write descriptor byte
formats below. The adapter asks the record-encryption port for a
generation-scoped opaque idempotency locator bound only to repository, tenant,
operation, and idempotency key; schema, format, and canonical-request bytes do
not enter that locator. It persists the canonical request only inside
authenticated ciphertext and never computes or stores an unkeyed request digest.
The SQLite adapter's only runtime dependencies
are `hr-employment-repository-draft`, `hr-record-encryption-draft`, and
`rusqlite.workspace = true`. Its only dev-dependencies are
`hr-employment-repository-memory-draft` and `tempfile.workspace = true`;
recovery targets use the real SQLite adapter and `tempfile`, never the memory
oracle or `:memory:`.
The logical idempotency key is:

```text
(tenant_id, operation_kind, idempotency_key)
```

`IdempotencySlotV1` is the repository-scoped typed lookup input
`{ repository_id: OpaqueBytes<=256, tenant_id: CanonicalAscii<=8KiB,
operation_kind: u16be, idempotency_key: CanonicalAscii<=8KiB }`. It represents
that logical key at one repository boundary; it never includes legal-entity,
schema, format, or mutable canonical-request fields. Only the provider returns
its per-generation opaque locator.

### HR canonical request v1

`CanonicalRequestV1` is the semantic retry identity for the first
`OnboardEmployee` operation. It is produced only after typed domain validation;
wire-map order, protobuf tag order, authorization/PDP evidence, overlay
generation, deadline, retry attempt, channel identity, and correlation id do
not enter it because they neither change the requested HR mutation nor replace
their own per-attempt validation. Its exact ordered semantic fields are:

| Tag | Field | V1 payload |
|---:|---|---|
| 1 | `operation_kind` | `u16be`; `1 = onboard_employee` |
| 2 | `tenant_id` | validated canonical ASCII bytes |
| 3 | `legal_entity_id` | validated canonical ASCII bytes |
| 4 | `idempotency_key` | validated canonical ASCII bytes |
| 5 | `request_schema_version` | `u32be` |
| 6 | `employee_id` | validated canonical ASCII bytes |
| 7 | `person_ref` | validated canonical UTF-8 bytes |
| 8 | `manager_id` | one presence octet (`00` or `01`); when present, validated canonical ASCII bytes follow |
| 9 | `employment_status` | `u16be`: draft=1, active=2, suspended=3, terminated=4 |
| 10 | `tenant_tier_snapshot` | `u16be`: SMB=1, single-entity=2, group=3, regulated=4 |
| 11 | `audit_evidence_ref` | validated canonical UTF-8 bytes |
| 12 | `employee_version` | `u32be` |
| 13 | `lifecycle_event_id` | validated canonical ASCII bytes |
| 14 | `lifecycle_kind` | `u16be`: created=1, updated=2, suspended=3, terminated=4 |

The byte encoding is exactly six ASCII octets `HRREQ\0`, `u16be(1)` format
version, `u16be(14)` field count, then the fourteen rows above in ascending tag
order. Each row is `u16be(tag) || u32be(payload_length) || payload`; integers
inside a payload use the fixed widths above. There are no maps, floats, packed
collections, platform-width integers, or alternate text encodings. The full
encoding is capped at 256 KiB, any individual text payload at 8 KiB, and the
aggregate text payload at 128 KiB, using checked `u64` accounting. V1 has no
free-text normalization: identifiers/references must already satisfy their
typed canonical grammar, and the encoder never trims, case-folds, or Unicode-
normalizes bytes. Empty present optionals are invalid; absent `manager_id` is
the sole `00` representation. Derived `data_class` and other server defaults
are excluded, so absent and explicitly transport-defaulted forms must first
normalize to the same typed command. An unknown, duplicate, omitted, out-of-
order, overlong, or trailing same-version field is `CanonicalFormatInvalid`,
not ignored.

The sole V1 lookup purpose is `IdempotencyLocatorV1`, encoded as fixed
`u16be(1)` `purpose_tag`. It means only “a generation-scoped opaque locator for
one logical HR idempotency slot”; it is not a field label, caller input,
provider-selected value, request-equality token, or reusable cross-generation
token. No other V1 lookup purpose is admitted. The repository port, SQLite
adapter, rekey worker, Cargo vectors, and Buck vectors use this one law; direct
`blind_index` replay lookup is forbidden.

The idempotency-locator full preimage is exactly literal ASCII domain bytes
`hr.people.idempotency-locator/v1\0` (no length prefix), followed by six
components in this order: `u16be(1)` purpose tag;
`u32be(length) || repository_id` (at most 256 opaque bytes, no normalization);
`u32be(length) || tenant_id` (at most 8 KiB, its validated canonical ASCII
bytes unchanged); `u16be(operation_kind)`; `u32be(length) || idempotency_key`
(at most 8 KiB, its validated canonical ASCII bytes unchanged); and
`u64be(key_generation)`. There is no component count because the count is
permanently six for this V1 preimage. Canonical-request bytes, request schema
version, and canonical-format version are deliberately absent. Variable
component lengths, integer widths, order, domains, bounds, and normalization
are exactly those stated here. Checked `u64` accounting rejects an overlong
component or a complete preimage over 24 KiB before any PRF call. The accepted
L2i.0d PRF authenticates these bytes without parsing or rewriting them.

The executable baseline admits and writes only format `u16be(1)`.
`ReplayGenerationSetV1.active_writer_format` is therefore exactly `1`, and each
returned generation entry has exactly `[1]` as its sorted canonical-format list.
Before any row lookup, the repository encodes the typed semantic command once as
`CanonicalRequestV1` and obtains one idempotency locator per active/draining
generation from that entry's returned opaque authority. Two generations therefore
permit at most two V1 locator derivations regardless of future reader-format
count. A missing, duplicate, unreadable, or non-V1 format is a closed
`CanonicalFormatUnsupported` refusal, never a guessed equality or a zero-match
creation.

`V1FormatEvolutionBarrier` is a future admission condition, not an executable V2
codec or an implied V2 retry. No V2 format may appear in a generation set, be
chosen by a writer, be required by replay/rekey, or be named in a V1 test until a
separately accepted owner decision and structural/content lane graph defines all
of: the V2 semantic fields and byte grammar; an independent encoder oracle or
fixed external fixtures distinct from the production encoder; provider-authoritative
active-writer-format and reader-cohort admission values; bounded format-admission
CAS and repository migration evidence; an authenticated per-repository format-
retirement receipt; and the exact Cargo/Buck paths/tests. That future decision may
admit only V1+one new format, requires an N+1 reader to retain the V1 reader
  through the admitted overlap, and may retire V1 only after every snapshot-bound
  repository proves no retained V1 canonical request/envelope, every epoch-
  eligible reader admits the successor, and the provider issues the matching
  retirement receipt. The generation-scoped idempotency locator remains format-
  independent and is never a format-retirement equality token. A third format
  remains prohibited until the oldest retired
format has that complete evidence. Golden vectors fix every V1 byte above and
prove transport-field reordering and absent/default-equivalent optionals produce
the same bytes, while every changed semantic field produces different bytes.

### HR staged-write descriptor v1

`StagedWriteDescriptorV1` is built from the actual SQLite staged row set before
`authorize_commit`; callers do not supply it. For onboarding it contains these
exact ordered top-level fields:

| Tag | Field | V1 payload |
|---:|---|---|
| 1 | `repository_id` | bounded opaque bytes |
| 2 | `repository_epoch` | `u64be` |
| 3 | `commit_authorization_id` | bounded opaque bytes |
| 4 | `tenant_id` | validated canonical ASCII bytes |
| 5 | `operation_kind` | `u16be`; `1 = onboard_employee` |
| 6 | `idempotency_key` | validated canonical ASCII bytes |
| 7 | `request_schema_version` | `u32be` |
| 8 | `canonical_request_format_version` | `u16be` |
| 9 | `repository_admission_epoch` | `u64be`; current open write-admission epoch |
| 10 | `idempotency_locator_key_generation` | `u64be` |
| 11 | `idempotency_locator` | fixed-width provider-authenticated bytes |
| 12 | `effect_count` | `u16be`; exactly four in v1 |
| 13 | `effects` | one length-delimited concatenation of exactly four effect entries |

Effect entries are ordered by `(effect_kind, opaque_row_identity)` with the
fixed kinds employee=1, lifecycle=2, idempotency-outcome=3, audit-outbox=4.
Each entry begins with `u16be(10)` field count and contains these exact fields:

| Tag | Field | V1 payload |
|---:|---|---|
| 1 | `effect_kind` | `u16be` fixed kind above |
| 2 | `opaque_row_identity` | bounded opaque bytes |
| 3 | `expected_state` | `00` for absent or `01 || u64be(revision)` |
| 4 | `result_revision` | `u64be` |
| 5 | `record_schema_version` | `u32be` |
| 6 | `encryption_key_generation` | `u64be` |
| 7 | `ciphertext_octet_length` | `u32be` |
| 8 | `envelope_commitment` | provider-authenticated bytes |
| 9 | `blind_index_count` | `u16be` |
| 10 | `blind_indexes` | length-delimited ordered entries |

Each blind-index entry begins with `u16be(3)` and contains tag 1
`purpose_tag: u16be`, tag 2 `key_generation: u64be`, and tag 3 `bytes`; entries
are ordered by `(purpose_tag, key_generation, bytes)`. In semantic shorthand,
an effect is therefore:

`IdempotencyLocatorV1` is not one of these generic per-effect entries: it is the
top-level slot locator above, and no generic field-index purpose may reuse its
`u16be(1)` purpose tag. The idempotency-outcome effect therefore carries its
generation/locator identity only through the top-level fields.

```text
effect_kind
opaque_row_identity
expected_state (Absent | Revision(u64))
result_revision u64
record_schema_version u32
encryption_key_generation u64
ciphertext_octet_length u32
provider_authenticated_envelope_commitment
blind_index_count
blind indexes ordered by (purpose_tag, key_generation, bytes)
```

The descriptor encoding is six ASCII octets `HRSTG\0`, `u16be(1)`, then the
header and effect fields as the same `u16be(tag) || u32be(length) || payload`
grammar used above; nested effect/index entries use fixed field counts and the
same framing. It is capped at 64 KiB, four effects, 8 KiB per row identity,
four blind indexes per effect, 256 octets for repository id, 128 octets for
authorization id, 8 KiB each for tenant/idempotency bytes, and 64 octets per
provider-authenticated commitment or blind index. Counts and lengths use
checked `u64` arithmetic.
Duplicate effect identities, duplicate blind-index purposes, noncanonical
order, unknown/omitted fields, trailing bytes, count disagreement, an effect
not present in the SQLite write set, or a staged row absent from the descriptor
is `StagedDescriptorInvalid` and aborts before provider authorization. Thus the
employee, lifecycle, idempotency/outcome, and audit/outbox effects cannot be
omitted while satisfying the commit contract.

`CommitBinding` authenticates one complete V1 outer preimage. It is the literal
ASCII domain bytes `hr.people.commit-binding/staged-write/v1\0` (no length
prefix), `u16be(9)` component count, then exactly nine ascending-tag
components. Each component is `u16be(tag) || u32be(payload_length) || payload`:

| Tag | Semantic component | Exact payload and bound |
|---:|---|---|
| 1 | `tenant_id` | validated canonical ASCII bytes unchanged; at most 8 KiB |
| 2 | `repository_id` | opaque bytes unchanged; at most 256 bytes |
| 3 | `repository_epoch` | exactly `u64be`, payload length 8 |
| 4 | `commit_authorization_id` | opaque bytes unchanged; at most 128 bytes |
| 5 | `request_schema_version` | exactly `u32be`, payload length 4 |
| 6 | `descriptor_format_version` | exactly `u16be`, payload length 2 |
| 7 | `key_generation` | exactly `u64be`, payload length 8 |
| 8 | `StagedWriteDescriptorV1` | exact already-valid descriptor bytes unchanged; at most 64 KiB |
| 9 | `repository_admission_epoch` | exactly `u64be`, payload length 8 |

The count, tags, component order, lengths, fixed widths, domains, bounds, and
normalization above are part of the V1 byte contract. Components cannot be
omitted, duplicated, reordered, padded, or normalized; fixed-width payloads
with any other length fail `CommitBindingFormatInvalid` before provider
authorization. Checked `u64` accounting rejects a complete preimage over 96
KiB. The provider treats the complete bytes as opaque. SQLite stores descriptor
format/version and binding with its receipt. Schema N/N+1 reader and writer
barriers retain `CanonicalRequestV1`; an unsupported descriptor keeps recovery and
readiness closed. Independent Cargo and Buck encoders share only typed semantic
inputs and fixed vector fixtures, never a preimage encoder: both assert exact
full-preimage bytes (including domain, count, tags, lengths, and payloads), not
only descriptor bytes. Their vectors cover every component, changed purpose,
schema N/N+1 dispatch, a reordered or omitted outer component, and every exact and
limit-plus-one component and total bound; deliberately divergent typed input or
encoder output must fail the independent-vector comparison.

The stored entry binds the idempotency-locator bytes and generation, encrypted
canonical request and outcome bytes/version, canonical and descriptor format
versions, repository epoch, repository admission epoch, commit binding,
authorization receipt, and commit generation. Its row is located only by the V1
generation-scoped idempotency locator; there is no stable cross-generation
equality locator and no cleartext replay key. Processing is:

1. Validate syntax, tenant binding, verified authorization, and overlay
   generation without mutation.
2. Evaluate the deterministic domain command.
3. Encode the typed semantic command once as `CanonicalRequestV1`, then acquire
   the provider-authenticated, repository-epoch/fence/membership-bound
   `ReplayGenerationSetV1`. Its exact result contains the active generation and,
   only during normal drain, its immediately prior draining generation; sorted
   generation state, lease, rotation-fence id, immutable membership snapshot,
   active writer format `1`, and opaque generation-scoped PRF authority are all
   in that one result. The provider returns neither a stable locator nor an
   arbitrary historical generation. A missing, revoked, malformed, stale,
   unregistered, or oversized set is `ReplayGenerationSetUnavailable` and
   admits no mutation.
4. Before opening SQLite, call `derive_idempotency_locator_v1` once for every
   returned generation, passing that generation's opaque authority and only the
   logical slot `(repository_id, tenant_id, operation_kind, idempotency_key)`.
   The repository supplies no raw PRF authority, caller-selected generation, or
   mutable request/schema/format input. The hard V1 bound is two derivations and
   two fixed-width locators; both are derived before any locator is selected.
   Begin one `BEGIN IMMEDIATE` SQLite writer transaction, validate repository
   epoch, open repository-admission epoch, lease, matrix digest, rotation-fence
   id, membership snapshot id/version, and active writer format, then query only
   those locators. The query reads at most five rows solely to distinguish zero,
   one, and multiple matches; it opens no row while multiple candidates match.
5. Zero matches may reserve a new entry only with the lease's active-generation
   V1 locator. Exactly one match must authenticate and open its envelope using
   recorded generation and associated data, then constant-time compare complete
   canonical plaintext bytes in memory. Ciphertext bytes, including nonce/tag/
   randomized ciphertext, are never compared for equality and no plaintext is
   persisted. Equal bytes return the original outcome; unequal bytes return
   `IdempotencyConflict` without mutation. More than one locator row is
   `IdempotencyLocatorCollision`; a matrix/locator/row generation or format
   mismatch is `IdempotencyLocatorDivergence`; an unreadable source, generation/
   locator mismatch, or lost source key is `ReplaySourceUnavailable`. Every one
   fails closed and releases the transaction without a new business effect. No
   code reads an entry to discover a generation before locating it.
6. For a zero-match reservation, seal every sensitive employee,
   lifecycle, request/outcome, and audit/outbox value through
   `hr-record-encryption-draft`, derive any non-replay field indexes required by
   their own contract, and stage the authenticated envelopes plus the active
   idempotency locator in the same transaction.
7. Enumerate that exact staged row set, encode `StagedWriteDescriptorV1`, derive
   one opaque commit binding over it, and call
   `authorize_commit` with tenant, the current repository-epoch lease, a stable
   commit-authorization id, key generation, active repository-admission epoch,
   idempotency-locator identity, and that binding. Store the returned
   authorization receipt inside the same SQLite
   transaction.
8. Commit durably according to the adapter profile, then call idempotent
   `resolve_commit(Committed { binding })`. Only the provider's resolved receipt
   authorizes acknowledgement. If SQLite rolls back, recovery calls
   `resolve_commit(Aborted)` instead. A caller disconnect after commit does not
   undo the transaction.

The adapter never exposes `PREPARED` or a pending authorization as success. On
boot, an exclusive SQLite writer first acquires a provider-serialized
`RepositoryEpochLease`; that operation fences authorization from every older
epoch. Before readiness, it drains bounded pages from `list_unresolved` for the
repository. An exact committed SQLite receipt/binding resolves `Committed`; an
absent receipt resolves `Aborted` only after the older epoch is fenced; a
mismatched receipt/binding is corruption and keeps readiness false. Thus a
crash after provider authorization but before SQLite commit is recoverable even
though the local transaction never persisted the receipt. Interruption before
commit rolls all four business effects back. Interruption after commit recovers
all four effects and resolves the stored receipt as committed before replay can
acknowledge. Provider loss leaves the cohort unavailable and the receipt
unresolved; it is never guessed from a deadline. Cleanup or delivery failure
may leave a retryable outbox entry, never a missing employee or second business
effect. SQLite-to-cloud transition is cohort cutover between single active
adapters, not dual-write.

</durable_transaction>

<data_at_rest>

## Record-encryption and key lifecycle contract

The SQLite format permits cleartext only for schema/migration version, opaque
row identity, bounded timestamps and counters, ciphertext-envelope metadata,
and keyed non-replay field indexes plus opaque idempotency locators. Employee,
person, manager, evidence, policy, pack,
lifecycle, canonical request, stored outcome, and audit/outbox payload values
are sensitive and must never be stored as cleartext. The final
`hr-record-encryption-draft` port exposes bounded `seal`, `open`, `blind_index`,
`commit_binding`, `acquire_repository_epoch`, `acquire_replay_generation_set_v1`,
`derive_idempotency_locator_v1`, keyring-membership and decommission operations,
`list_unresolved`, `authorize_commit`, and idempotent `resolve_commit` plus
`begin_normal_rotation_v1`, `list_incomplete_rotations`, `bind_rekey_checkpoint`,
`verify_rekey_checkpoint`, and `authorize_zero_reference_revocation` over
HR-owned values. Its envelope contains an algorithm identifier, provider/key
reference, monotonically ordered key generation, unique nonce, ciphertext, and
authentication tag; provider or cipher types do not cross the port.
`IdempotencyLocatorV1` is a fixed-width keyed PRF output scoped exactly to
`(repository, tenant, operation kind, idempotency key, key generation)` under
the fixed `u16be(1)` purpose tag and
`hr.people.idempotency-locator/v1\0` domain above. Its exact six-component
preimage is the SPEC grammar, not an adapter-specific field label; it excludes
canonical-request/schema/format bytes. Length-prefixing and domain separation
are canonical; it locates one logical slot inside one generation and never
permits a stable cross-generation token. After L2i.0g, a repository MUST NOT
invoke `blind_index` directly for replay lookup:
`derive_idempotency_locator_v1` is the only returned-authority-bound locator
edge. The accepted primitive, encoding, and width are decision-gated at L2i.0d.

`AcquireReplayGenerationSetV1` is the sole executable provider-truth edge for
replay; its Rust port method is `acquire_replay_generation_set_v1`. Its exact request is
`{ repository_id: OpaqueBytes<=256, repository_epoch: u64, replay_contract: u16be(1) }`;
its successful result is
`ReplayGenerationSetV1 { repository_id, repository_epoch, lease_id:
OpaqueBytes<=128, matrix_digest: FixedBytes<=64, rotation_fence_id:
OpaqueBytes<=128, keyring_id: OpaqueBytes<=128, membership_snapshot_id:
OpaqueBytes<=128, membership_version: u64, repository_member_instance_id:
OpaqueBytes<=128, active_writer_format: u16be(1), entries,
provider_authenticator: OpaqueBytes<=512 }`. `entries` is one
or two ascending, distinct `ReplayGenerationAuthorityV1` values:
`{ generation: u64, state: Active | Draining, canonical_formats: [u16be(1)],
replay_prf_authority: OpaqueBytes<=256 }`. The set has exactly one `Active`, at
most one `Draining`, no `Revoked`/`EmergencyDraining` entry, and at most two V1
derivations. `matrix_digest` commits to every returned field, including the
membership binding and active writer format; `provider_authenticator` authenticates
that digest and every result field. The repository is admitted only if
its member-instance id appears in the current immutable snapshot. The closed
error is `ReplayGenerationSetError::{RepositoryNotRegistered,
RepositoryEpochStale, RotationFenceStale, MembershipSnapshotStale, LeaseStale,
GenerationSetMalformed, GenerationSetTooLarge, GenerationStateInvalid,
FormatMatrixInvalid, NormalRotationBlocked, ProviderUnavailable,
ProviderCorrupt}`. The result is provider-authenticated and repository/epoch/
fence/membership-bound; the SQLite repository calls this port before lookup,
validates it inside its writer transaction, and has no alternate cache, database
narration, or provider-internal import. The key-service adapter implements this
call against its accepted facade but has no repository runtime edge. A set and
its authorities are single logical-replay-attempt values: they are never stored
in SQLite or reused after response loss, retry, restart, lease/fence change, or
membership change. Each retry reacquires a set; a stale authority/set/lease is
the corresponding closed result and cannot fall through to zero-match creation.

`derive_idempotency_locator_v1` is the Rust port method for the
`DeriveIdempotencyLocatorV1` domain operation. It has request
`{ generation_authority: ReplayGenerationAuthorityV1, repository_id,
tenant_id, operation_kind, idempotency_key }` and result
`IdempotencyLocatorV1 { generation: u64, bytes: FixedBytes<=64,
matrix_digest: FixedBytes<=64, provider_authenticator: OpaqueBytes<=512 }`.
`FixedBytes<=64` is an immutable exact-length value whose width is fixed by the
accepted L2i.0d port decision and cannot vary by row, generation, or request.
`generation_authority` is the opaque value returned by the same unexpired
`AcquireReplayGenerationSetV1` result; callers cannot construct, substitute,
replay across a repository/epoch/matrix digest, or inspect its PRF authority.
The adapter validates that binding, uses its opaque authority for the exact
six-component V1 preimage, and returns only the bounded provider-authenticated
locator. Its closed error is
`IdempotencyLocatorDerivationError::{AuthorityNotFromGenerationSet,
AuthoritySetStale, RepositoryEpochStale, LeaseStale, IdempotencySlotInvalid,
LocatorLimitExceeded, ProviderUnavailable, ProviderCorrupt}`. This operation,
rather than a frozen caller-selected `blind_index` input, is the real
repository-to-port-to-provider traversal.

`KeyringMembershipSnapshotV1` is
`{ keyring_id: OpaqueBytes<=128, membership_snapshot_id: OpaqueBytes<=128,
membership_version: u64, members: [KeyringRepositoryMemberV1; 1..=4096],
provider_authenticator: OpaqueBytes<=512 }`, where members are strictly
sorted/distinct by `(repository_id, member_instance_id)` and each value is
`{ repository_id: OpaqueBytes<=256, member_instance_id: OpaqueBytes<=128,
state: Active | Decommissioning, admission_epoch: u64 }`. The authenticator
covers every field and its member ordering. This is an **active-keyring** type;
it never represents zero members. `KeyringMembershipStateV1` is instead exactly
`Active { snapshot: KeyringMembershipSnapshotV1 } | Retiring {
fence: KeyringRetirementFenceV1 } | Retired { receipt:
KeyringRetirementReceiptV1 }`. A retired keyring has no membership snapshot,
active writer, live generation, unresolved authorization, or rejoin path; a
later keyring requires a new `keyring_id`.

The provider domain operations `RegisterKeyringRepositoryV1`,
`AcquireKeyringMembershipSnapshotV1`, `BeginRepositoryDecommissionV1`,
`GetRepositoryDecommissionV1`, `IssueDecommissionProofV1`,
`AbortRepositoryDecommissionV1`, `RemoveKeyringRepositoryV1`,
`BeginKeyringRetirementV1`, and `CompleteKeyringRetirementV1` are exposed as
Rust port methods `register_keyring_repository_v1`,
`acquire_keyring_membership_snapshot_v1`, `begin_repository_decommission_v1`,
`get_repository_decommission_v1`, `issue_decommission_proof_v1`,
`abort_repository_decommission_v1`, `remove_keyring_repository_v1`,
`begin_keyring_retirement_v1`, and `complete_keyring_retirement_v1`.

`ProviderOperationKindV1` is exactly `RegisterKeyringRepository |
BeginRepositoryDecommission | IssueDecommissionProof |
AbortRepositoryDecommission | RemoveKeyringRepository |
BeginKeyringRetirement | CompleteKeyringRetirement | BeginNormalRotation`.
Every provider-side
`operation_id` below is the closed value `ProviderOperationIdV1 { kind:
ProviderOperationKindV1, opaque_id: OpaqueBytes<=128 }`; the method rejects an
id whose `kind` is not its own operation kind. Its idempotency key has an exact,
kind-specific scope: Register uses `(keyring_id, repository_id,
repository_epoch, kind, opaque_id)`; Begin, Issue, Abort, Remove, and both
retirement operations use `(keyring_id, repository_id, member_instance_id,
repository_epoch, kind, opaque_id)`; and `BeginNormalRotation` uses
`(keyring_id, kind, opaque_id)`. Its canonical request digest is
`H("hr.keyring.provider-operation.v1" || kind || scope tuple || exact request
bytes)`. Thus raw opaque bytes have an explicit operation-kind namespace rather
than an implicit provider-global namespace. An adapter stores and replays only
that complete key/digest tuple: the same kind/key with changed bytes is
`MembershipOperationConflict`, while equal raw bytes in different kinds are not
the same key. Intent and removal-plan validation still requires all reserved
provider ids' `(kind, opaque_id)` pairs to be distinct, and additionally
requires distinct raw values within one plan.
`RepositoryLocalOperationKindV1` is exactly `LocalDisposition |
LocalCompletion`; its `RepositoryLocalOperationIdV1 { kind,
opaque_id: OpaqueBytes<=128 }` cannot be used as a provider id.
`RepositoryResponseOperationIdV1` is a separate local namespace exactly
`AbortResponse | Recovery | StatusQuery`; it too is never serializable as
`ProviderOperationIdV1` and cannot reach a provider side effect.

`DecommissionDigestV1` is the exact 32-byte output of `H`, carried in the
existing `FixedBytes<=64` container. `CanonicalDecommissionPlanEncodingV1` is
the only encoder for the five durable decommission plans below. Its
non-self-referential digest preimage is `ASCII(domain) || 0x00 || header ||
records`, where `domain` is the exact plan-specific ASCII literal named below,
`header` is exactly `0x4f 0x59 0x48 0x52 || 0x01 || plan_kind:u8 ||
record_count:u16be || 0:u64be`, and each record is exactly `tag:u8 ||
length:u16be || value`. Tags are strictly ascending, no tag repeats,
`record_count` equals the declared tag set exactly, and an unknown, omitted,
duplicated, out-of-order, or non-canonical-length tag is rejected before
hashing. A `u64` value has `length = 8` and big-endian bytes; an enum has
`length = 1`; a digest has `length = 32`; and a provider or local operation id
has `length = 1 + opaque_id_length`, with its kind byte first and a nonempty
`opaque_id<=128`. Every other opaque identity named in the tables is nonempty
and is bounded by its declared type: keyring/snapshot/fence ids `<=128`,
repository id `<=256`, provider authenticator `<=512`, and storage-manifest
digest `=32`. In this decommission subsection every `*_digest` field is the
`DecommissionDigestV1` 32-byte subtype, including a legacy display spelling of
`FixedBytes<=64`; no decommission digest may use the remaining width. A nested
`KeyringRetirementFenceV1` wire is exactly five top-level records
`(handoff_wire, retirement_fence_id, retirement_begin_operation_id,
begin_plan_digest, provider_authenticator)`, in that ascending tag order. The
fence authenticator covers the already-derived `begin_plan_digest` as well as
the full previously authenticated handoff, so it binds the accepted Begin plan
without a recursive plan preimage.

The complete canonical wire of `DecommissionProofReferenceV1`,
`KeyringRetirementHandoffV1`, `KeyringRetirementFenceV1`,
`ProviderDecommissionTerminalReceiptV1`, and
`LocalDecommissionStorageReceiptV1` is respectively at most `1,265`, `1,441`,
`2,273`, `1,919`, and `870` bytes, including its own 16-byte header and
records. These are closed maxima from the field bounds in this section, not
sampling targets. The maximum plan-wire lengths (not including the fixed domain
literal and terminating zero) are Removal `3,096`, Begin `1,793`, Complete
`3,832`, Disposition `2,179`, and Completion `253` bytes. Thus every
individual `u16be` length and the `4,096`-byte plan/request-wire ceiling admits
the maximum typed value; a `4,097`th byte is rejected before a plan, request
journal, or side effect is written. The exact maximum request-wire lengths are
Remove `2,267`, Begin `1,758`, Complete `2,184`, Disposition `292`, and
Completion `288` bytes. Begin remains `1,758` because its already-frozen
four-record request wire always carried the plan digest; this revision makes
that same fourth record part of the typed provider request and signed fence.
The adapter and SQLite independently rederive these sums and reject a wire
whose declared count, field length, total, domain, or byte vector differs. The
hash is always `H(ASCII(domain) || 0x00 || canonical_wire)`; no plan digest
contains itself, a request digest, or request bytes derived after it.

`KeyringMembershipMutationResultV1` is exactly `Registered {
member_instance_id: OpaqueBytes<=128, snapshot: KeyringMembershipSnapshotV1 } |
Updated { snapshot: KeyringMembershipSnapshotV1, removal_receipt:
DecommissionRemovalReceiptV1 } | RetirementHandoffReady { handoff:
KeyringRetirementHandoffV1 }`. `Registered` is the sole successful register
result; `Updated` and `RetirementHandoffReady` are the sole successful remove
results. In the last-member case the provider durably records the authenticated
handoff-ready decommission state before returning `RetirementHandoffReady`; it
does not create an empty snapshot or remove the final member. Exact replay of
Remove and Get therefore returns the same authenticated handoff rather than
asking SQLite to reconstruct or mint one after a crash.

`DecommissionProofIssuanceV1` is exactly `{ proof: DecommissionProofV1,
proof_reference: DecommissionProofReferenceV1 }`, one immutable provider-ledger
value rather than two independently produced values.
`IssueDecommissionProofResultV1` is exactly `Issued { issuance:
DecommissionProofIssuanceV1 }`; its named error is the closed
`KeyringMembershipError` sum below. The provider writes the issuance once under
the reference's canonical lookup key, and exact Issue replay or Get returns the
byte-identical proof and reference canonical bytes, including their provider
authenticators. Neither result branch creates a repository-owned identifier.

`ProviderDecommissionStatusV1` is exactly `NotStarted | Aborted { begin_operation_id:
ProviderOperationIdV1, abort_tombstone: DecommissionBeginTombstoneV1 } | Fenced
{ fence: DecommissionAdmissionFenceV1 } | ProofIssued { issuance:
DecommissionProofIssuanceV1 } | RetirementHandoffReady { handoff:
KeyringRetirementHandoffV1 } | Retiring { retirement_fence:
KeyringRetirementFenceV1 } | Removed { removal_receipt:
DecommissionRemovalReceiptV1 } | KeyringRetired { retirement_receipt:
KeyringRetirementReceiptV1 }`. `ProviderDecommissionClosedStatusV1` is exactly
`Fenced { fence: DecommissionAdmissionFenceV1 } | ProofIssued { issuance:
DecommissionProofIssuanceV1 } | RetirementHandoffReady { handoff:
KeyringRetirementHandoffV1 } | Retiring { retirement_fence:
KeyringRetirementFenceV1 } | Removed { removal_receipt:
DecommissionRemovalReceiptV1 } | KeyringRetired { retirement_receipt:
KeyringRetirementReceiptV1 }`; it deliberately excludes `NotStarted`,
`Aborted`. `ProviderDecommissionAbortResultV1` is exactly
`Aborted { tombstone: DecommissionBeginTombstoneV1, snapshot:
KeyringMembershipSnapshotV1 } | Closed { status:
ProviderDecommissionClosedStatusV1 }`. These are named Rust sums, not prose
alternatives; port, adapter, and fault tests exhaustively match every variant
without a wildcard.

`RegisterKeyringRepositoryV1` accepts `{ keyring_id, repository_id,
repository_epoch, expected_membership_version, operation_id:
ProviderOperationIdV1 }`
and returns `KeyringMembershipMutationResultV1::Registered { member_instance_id,
snapshot: KeyringMembershipSnapshotV1 }`. `AcquireKeyringMembershipSnapshotV1`
accepts `{ keyring_id }` and returns `KeyringMembershipStateV1`, rather than an
impossible empty snapshot. `BeginRepositoryDecommissionV1` accepts `{ keyring_id,
repository_id, member_instance_id, repository_epoch,
expected_membership_snapshot_id, expected_membership_version,
expected_admission_epoch, operation_id: ProviderOperationIdV1 }`. One provider
transaction serializes Begin and Abort for the same operation scope: it verifies
the exact stored Begin request bytes/digest and idempotency key, writes the
idempotency replay cell, CASes that active member to `Decommissioning`, and
writes the signed fence before that transaction commits. It returns
`DecommissionAdmissionFenceV1 { keyring_id,
repository_id, member_instance_id, repository_epoch, membership_snapshot_id,
membership_version, rotation_fence_id: OpaqueBytes<=128,
live_generation_digest: FixedBytes<=64, decommission_fence_id:
OpaqueBytes<=128, admission_epoch: u64, provider_authenticator }`. It denies
new replay-set, locator-derivation, seal, and commit authorization for that
member/fence while allowing only status, unresolved-resolution, proof, abort,
removal, and retirement operations. Consequently `IntentPending` is never a
provider status: before this transaction commits Get returns `NotStarted`;
after it commits Get and exact Begin replay return the identical `Fenced`
value. If Abort serializes first it writes the signed `Aborted` tombstone and
the delayed Begin returns `DecommissionBeginTombstoned`; if Begin serializes
first Abort returns `Closed { Fenced }` (or that exact later closed state).
Lost Begin or Abort responses are resolved only by Get/exact replay of those
stored bytes. No status observation alone can create, reopen, or change an
intent.

`DecommissionReferenceKindV1` is exactly `Ciphertext | IdempotencyLocator |
NonReplayFieldIndex`. `DecommissionReferenceScanEntryV1` is `{ logical_table_tag:
OpaqueBytes<=128, opaque_row_identity: OpaqueBytes<=128, reference_kind:
DecommissionReferenceKindV1, admitted_column_tag: OpaqueBytes<=128 }`; checkpoint
order is lexicographic over those four fields and is bounded by the page/step
limits below. `DecommissionObservationV1` is `{ keyring_id, repository_id,
member_instance_id, repository_epoch, membership_snapshot_id,
membership_version, rotation_fence_id, live_generation_digest,
decommission_fence_id, admission_epoch, terminal_write_sequence,
scan_checkpoint_digest, unresolved_receipt_digest,
durable_ciphertext_references: u64be(0), durable_locator_references: u64be(0),
durable_non_replay_index_references: u64be(0),
unresolved_authorizations: u64be(0) }`. `scan_checkpoint_digest` commits to the
complete bounded reference scan: every `(logical_table_tag, opaque_row_identity,
reference_kind, admitted_column_tag)` in stable order, including each admitted
generation-scoped non-replay field-index column and its checked per-kind count.
`IssueDecommissionProofV1` accepts `{ observation: DecommissionObservationV1,
operation_id: ProviderOperationIdV1 }` returns
`IssueDecommissionProofResultV1::Issued { issuance:
DecommissionProofIssuanceV1 }`. Before this provider call, SQLite persists
`DecommissionProofIssuePlanV1 { observation, issue_proof_operation_id:
ProviderOperationIdV1, issue_proof_request_digest: FixedBytes<=64 }`, where the id
is the one already reserved in the immutable intent and the digest is
`H("hr.decommission.issue-proof-request.v1" || exact
IssueDecommissionProofV1 request bytes)`. The provider recomputes that digest
from its exact request before signing and durably storing one issuance; exact
replay returns the same issuance and changed reuse is
`MembershipOperationConflict`. `ProviderDecommissionStatusV1::ProofIssued`
returns that same stored issuance on response loss, never a proof without its
provider-authenticated reference and never a repository-minted reference.

`GetRepositoryDecommissionV1` accepts `{ keyring_id, repository_id,
member_instance_id, repository_epoch }` and returns
`ProviderDecommissionStatusV1`. `Removed` is deliberately not a unit variant:
`DecommissionRemovalReceiptV1` includes `{ keyring_id, repository_id,
member_instance_id, repository_epoch, decommission_proof_digest:
FixedBytes<=64, prior_membership_snapshot_id, prior_membership_version,
successor_membership_snapshot_id, successor_membership_version,
removal_operation_id: ProviderOperationIdV1, removal_plan_digest: FixedBytes<=64,
provider_authenticator }`, and binds the exact proof and durable local plan used
by the provider membership CAS. It is therefore sufficient to recover a lost
remove response without inventing an id, disposition, proof, or registration.

`AbortRepositoryDecommissionV1` accepts `{ keyring_id, repository_id,
member_instance_id, repository_epoch, expected_membership_snapshot_id,
expected_membership_version, begin_operation_id: ProviderOperationIdV1,
abort_operation_id: ProviderOperationIdV1 }`. It
atomically records `DecommissionBeginTombstoneV1 { keyring_id, repository_id,
member_instance_id, repository_epoch, begin_operation_id: ProviderOperationIdV1,
begin_request_digest: FixedBytes<=64, abort_operation_id: ProviderOperationIdV1,
abort_request_digest: FixedBytes<=64,
abort_membership_snapshot_id, abort_membership_version, provider_authenticator
}` before returning `ProviderDecommissionAbortResultV1::Aborted { tombstone,
snapshot }`; exact replay returns that same named result and changed operation
reuse returns `MembershipOperationConflict`.
The provider recomputes the exact Begin and abort request-digest forms from the
stored-operation arguments before signing the tombstone; a digest or tuple
mismatch changes no membership state.
A delayed or retried `BeginRepositoryDecommissionV1` with the tombstoned
operation id returns `DecommissionBeginTombstoned` and cannot mutate membership.
If a concurrent begin won first, abort returns
`ProviderDecommissionAbortResultV1::Closed` with its exact closed status,
including `RetirementHandoffReady`, and the repository remains closed; it never
reopens on a `NotStarted` observation alone. These operations have no timeout
success.

The provider Begin/Abort response-loss query is total:

| serialized durable event | Get after a process crash | exact Begin replay | exact Abort replay |
| --- | --- | --- | --- |
| neither transaction committed | NotStarted | may atomically commit Fenced | may atomically commit Aborted tombstone |
| Begin committed, response lost | Fenced | the identical Fenced value | Closed with that Fenced value |
| Abort committed, response lost | Aborted tombstone | DecommissionBeginTombstoned | the identical Aborted tombstone |
| Begin committed before a later close transition | that signed closed status | its idempotent original Fenced value or typed operation conflict for changed bytes | Closed with that signed closed status |

The first row is the only case where a repository-local IntentPending recovery
may send its already-persisted Abort CAS. It remains local and write-closed
until one of the signed outcomes in this table is durably installed; it is not a
provider state and it is never inferred from an observation.

`DecommissionProofV1` is `{ keyring_id, repository_id, member_instance_id,
repository_epoch, membership_snapshot_id, membership_version, rotation_fence_id,
live_generation_digest, decommission_fence_id, admission_epoch,
terminal_write_sequence, scan_checkpoint_digest, unresolved_receipt_digest,
issue_proof_operation_id: ProviderOperationIdV1,
issue_proof_request_digest: FixedBytes<=64,
durable_ciphertext_references: u64be(0), durable_locator_references: u64be(0),
durable_non_replay_index_references: u64be(0),
unresolved_authorizations: u64be(0), provider_authenticator }`.
`DecommissionProofReferenceV1` is the bounded provider-authenticated immutable
reference `{ keyring_id: OpaqueBytes<=128, repository_id: OpaqueBytes<=256,
member_instance_id: OpaqueBytes<=128, repository_epoch: u64be,
decommission_proof_digest: DecommissionDigestV1,
issue_proof_operation_id: ProviderOperationIdV1,
issue_proof_request_digest: DecommissionDigestV1,
provider_authenticator: OpaqueBytes<=512 }`. Its canonical lookup key is
exactly `(keyring_id, repository_id, member_instance_id, repository_epoch,
issue_proof_operation_id)`; the adapter's durable provider decommission ledger
owns one corresponding `DecommissionProofIssuanceV1` record. Its immutable
value is the exact returned proof canonical bytes followed by the exact returned
reference canonical bytes; it verifies both authenticators, the reference
lookup identity, issue request digest, and full-proof digest before every use.
That ledger is introduced by the g.1 key-service adapter implementation behind
the g.0 port; the repository carries only this port type, so it adds no
adapter-to-repository runtime edge or g.0-to-g.2 dependency.

The ledger retains the complete issuance record at least through every accepted
exact Issue/Remove/Complete/Get/recovery replay and the associated terminal
receipt-retention/GC horizon. It may collect that record only after a matching
terminal receipt is durable and all accepted replay keys that could name it have
expired; it never reconstructs a proof or reference from observation. A missing
record is `DecommissionProofReferenceMissing`, an identity/digest/request
mismatch is `DecommissionProofReferenceMismatch`, and a bad reference or proof
authenticator is `DecommissionProofReferenceAuthenticatorInvalid`; each is fail
closed and changes no membership state. Thus the reference is sufficient for
both initial verification and exact Complete replay: the provider resolves the
same immutable issuance by its canonical key, reauthenticates it, and rechecks
the same identity, zero-reference, unresolved, fence, and plan predicates.
`RemoveKeyringRepositoryV1` accepts `{ keyring_id, repository_id,
member_instance_id, repository_epoch, expected_membership_snapshot_id,
expected_membership_version, decommission_fence_id, admission_epoch,
decommission_proof_reference: DecommissionProofReferenceV1,
removal_plan_digest: DecommissionDigestV1, operation_id: ProviderOperationIdV1
}` and atomically resolves/rechecks every referenced full-proof field and the
immutable plan digest. With two or more members it returns
`KeyringMembershipMutationResultV1::Updated { snapshot:
KeyringMembershipSnapshotV1, removal_receipt: DecommissionRemovalReceiptV1 }`.
With one member it atomically records provider decommission status
`RetirementHandoffReady` and returns the same
`KeyringMembershipMutationResultV1::RetirementHandoffReady { handoff:
KeyringRetirementHandoffV1 }`; it neither removes the last member nor constructs
an empty snapshot. The status record freezes enrollment, rejoin, normal/emergency
rotation, replay acquisition, locator derivation, seal, and commit authorization
until matching retirement or typed recovery. Exact Remove replay and
`GetRepositoryDecommissionV1` return byte-identical handoff fields and
authenticator; a changed request, plan digest, or scoped id is
`MembershipOperationConflict`, and a forged/corrupt provider authenticator is
`ProviderCorrupt`, with no transition. SQLite's comparison of a returned handoff
to its persisted handoff is the repository-side `RetirementHandoffMismatch`.
`KeyringRetirementHandoffV1` is `{ keyring_id, repository_id,
member_instance_id, repository_epoch, decommission_proof_digest: FixedBytes<=64,
membership_snapshot_id, membership_version, rotation_fence_id,
live_generation_digest, removal_plan_digest: FixedBytes<=64,
provider_authenticator }`. `KeyringRetirementFenceV1` is that handoff identity
plus `{ retirement_fence_id: OpaqueBytes<=128,
retirement_begin_operation_id: ProviderOperationIdV1,
begin_plan_digest: DecommissionDigestV1, provider_authenticator }`.
`KeyringRetirementReceiptV1` is `{ keyring_id, repository_id,
member_instance_id, repository_epoch, decommission_proof_digest,
membership_snapshot_id, membership_version, rotation_fence_id,
retirement_fence_id, removal_plan_digest: FixedBytes<=64,
retirement_begin_operation_id: ProviderOperationIdV1,
retirement_complete_operation_id: ProviderOperationIdV1,
all_generation_digest: FixedBytes<=64, scan_checkpoint_digest: FixedBytes<=64,
durable_ciphertext_references: u64be(0), durable_locator_references: u64be(0),
durable_non_replay_index_references: u64be(0),
unresolved_authorizations: u64be(0), state: Retired, provider_authenticator }`.

`BeginKeyringRetirementV1` accepts `{ handoff, retirement_fence_id:
OpaqueBytes<=128, begin_plan_digest: DecommissionDigestV1,
operation_id: ProviderOperationIdV1 }` and CASes
the active sole-member keyring to `Retiring`, returning
`KeyringRetirementFenceV1`. It denies enrollment, rejoin, replay acquisition,
locator derivation, seal, commit authorization, removal, and normal/emergency
rotation. It is legal only from the identical persisted provider
`RetirementHandoffReady` state and rejects a changed reuse of its operation id,
handoff bytes, or `begin_plan_digest`. Its SQLite sibling journal stores the
exact four-record `hr.decommission.begin-retirement-request.v1` wire and
`H(ASCII("hr.decommission.begin-retirement-request.v1") || 0x00 || wire)`;
the provider idempotency cell stores that byte-identical wire under its scoped
provider-operation digest. Neither digest substitutes for the other, and both
bind the same already-derived `begin_plan_digest`; the returned fence carries
and authenticates that exact plan digest. `CompleteKeyringRetirementV1`
accepts `{ keyring_id, repository_id, member_instance_id, repository_epoch,
retirement_fence_id, decommission_proof_reference:
DecommissionProofReferenceV1, removal_plan_digest: DecommissionDigestV1,
begin_plan_digest: DecommissionDigestV1, complete_plan_digest:
DecommissionDigestV1, operation_id: ProviderOperationIdV1 }`;
it looks up and reauthenticates the stored fence by its preplanned id, requires
its `begin_plan_digest` to equal the stored fence's authenticated digest, then
atomically resolves/rechecks the same sole-member full proof, proof reference,
and all three plan digests,
zero ciphertext/locator/non-replay-index references, the proof's complete
reference-scan checkpoint digest, zero unresolved authorizations, exact current
all-live-generation digest, and no normal or emergency drain. It then revokes
every remaining generation in that same terminal transition before returning
`KeyringRetirementReceiptV1` and the `Retired` state. The receipt binds the
handoff, retirement fence, all-generation digest, proof digest, and
zero-reference/unresolved observations, so the terminal state has no active
writer or live generation. `Retiring { retirement_fence }` is observable through
`ProviderDecommissionStatusV1`; exact Begin or Complete replay returns the same
fence or receipt, while an operation-id reuse with changed bytes is
`MembershipOperationConflict` and leaves the provider state unchanged.
Response loss is exact-operation replay; a partition, stale CAS, or changed
operation is a typed refusal that leaves `Retiring` and readiness withdrawn.
Register, begin, abort, and remove are refused whenever an ordinary or emergency
drain exists. `BeginNormalRotationV1` also refuses a `Decommissioning`,
`Retiring`, or `Retired` membership state so a member cannot disappear from the
global no-overlap obligation.

The repository-owned operations are `ProduceDecommissionProofV1`,
`AbortRepositoryDecommissionIntentV1`, `RemoveRepositoryDecommissionV1`,
`CompleteRepositoryDecommissionV1`, `GetRepositoryDecommissionStatusV1`, and
`RecoverRepositoryDecommissionV1`, exposed as `produce_decommission_proof_v1`,
`abort_repository_decommission_intent_v1`,
`remove_repository_decommission_v1`, `complete_repository_decommission_v1`,
`get_repository_decommission_status_v1`, and
`recover_repository_decommission_v1` on `hr-employment-repository-draft` and
implemented by SQLite. The proof producer request is `{ keyring_id,
repository_id, member_instance_id, repository_epoch,
expected_membership_snapshot_id, expected_membership_version,
expected_admission_epoch, begin_operation_id: ProviderOperationIdV1,
issue_proof_operation_id: ProviderOperationIdV1, abort_operation_id:
ProviderOperationIdV1 }`; its success result is
`DecommissionProofProductionV1 { admission_fence:
DecommissionAdmissionFenceV1, proof_issue_plan: DecommissionProofIssuePlanV1,
issuance: DecommissionProofIssuanceV1 }`. SQLite accepts the issuance only when
the provider's authenticated reference matches its returned full proof and the
persisted proof-issue plan; it stores the one exact issuance before any removal
plan is admissible. Its closed error is
`DecommissionProofProductionError::{RepositoryNotRegistered, MemberInstanceStale,
MembershipVersionStale, MembershipSnapshotStale, RepositoryEpochStale,
RepositoryAdmissionEpochStale, DecommissionPending, DecommissionFenceStale,
DecommissionScanIncomplete, DecommissionObservationStale,
DecommissionProofIssuePlanMismatch, DecommissionProofAlreadyIssued,
DecommissionProofReferenceMissing, DecommissionProofReferenceMismatch,
DecommissionProofReferenceAuthenticatorInvalid, MembershipMutationBlocked,
LocalStorageBusy, LocalStorageFull, LocalStorageIo, LocalCommitFailed,
MembershipOperationConflict, ProviderUnavailable, ProviderCorrupt}`.

It first commits `DecommissionIntentV1 { begin_operation_id:
ProviderOperationIdV1, issue_proof_operation_id: ProviderOperationIdV1,
abort_operation_id: ProviderOperationIdV1, expected_membership_snapshot_id,
expected_membership_version, next_admission_epoch, begin_request_digest:
FixedBytes<=64, abort_request_digest: FixedBytes<=64 }` under `BEGIN IMMEDIATE`
in the SQLite metadata row shared by every durable write. The three non-empty
ids are pairwise distinct. `begin_request_digest` is
`H("hr.decommission.begin-request.v1" ||
exact BeginRepositoryDecommissionV1 request bytes)` and
`abort_request_digest` is `H("hr.decommission.abort-request.v1" || exact
AbortRepositoryDecommissionV1 request bytes)` for those stored inputs. That
immutable intent closes new writes before the provider Begin call, so a later
abort or recovery cannot mint, select, or change either provider-operation id
or input. Exact retry returns the stored intent; reuse of any intent id with
changed bytes returns `MembershipOperationConflict` before a second provider
side effect. After the provider fence is durably recorded,
`scan_decommission_references_v1` walks every sensitive-table generation,
idempotency-locator column, and admitted generation-scoped non-replay
field-index column in stable `(logical_table_tag, opaque_row_identity,
reference_kind, admitted_column_tag)` order: at most 64 reference entries/8 MiB
per page and 8 pages/512 entries/64 MiB per step, with checked `u64` counters
for ciphertext, locator, and non-replay-index references and a persisted
checkpoint. A terminal `BEGIN IMMEDIATE` observation binds the same local
admission epoch, fence, current repository/member/epoch, membership
snapshot/version, rotation fence, live-generation digest, terminal SQLite write
sequence, zero ciphertext/locator/non-replay-index references, and zero
unresolved receipts. In that same transaction it persists the
`DecommissionProofIssuePlanV1` with the intent's reserved Issue id and exact
request digest, then and only then calls `issue_decommission_proof_v1` from that
plan. The provider rechecks its current fence, snapshot/version, generation
digest, scan checkpoint and all three reference counts, unresolved count, and
recomputes the exact Issue id/digest before returning the authenticated
issuance; SQLite accepts that issuance only if both its proof and reference
match its local proof-issue plan.

`AbortRepositoryDecommissionIntentV1` accepts `{ keyring_id, repository_id,
member_instance_id, repository_epoch, response_operation_id:
RepositoryResponseOperationIdV1 }` and returns
`RepositoryDecommissionAbortResultV1`. It loads the exact pending
`DecommissionIntentV1`, verifies
its identity/epoch, and invokes the matching provider abort only with that
intent's stored expected snapshot/version, Begin id, abort id, and canonical
abort-request digest. `response_operation_id` identifies only the repository
response record and is never sent to the provider. It accepts reopening only
from the signed `Aborted { tombstone, snapshot }` result. SQLite then uses
`BEGIN IMMEDIATE` to CAS the exact pending intent and
provider tombstone to `Active(reopened_admission_epoch)`, where
`reopened_admission_epoch` is strictly greater than the intent's next epoch;
this fences every pre-intent and pending-intent writer. Its success is
`DecommissionAbortTombstoneReceiptV1 { tombstone:
DecommissionBeginTombstoneV1, previous_admission_epoch,
reopened_admission_epoch, local_metadata_commit_digest: FixedBytes<=64 }`.
`RepositoryDecommissionAbortResultV1` is exactly `Aborted { receipt:
DecommissionAbortTombstoneReceiptV1 } | Closed { status:
RepositoryDecommissionStatusV1 }`. `Closed` is valid only for one of the named
non-reopenable status variants `Fenced | ProofIssuePlanned | ProofIssued |
RemovalPlanned | RetirementHandoffPersisted | RetirementBeginPlanned | Retiring |
RetirementCompletePlanned | ProviderTerminalPendingLocalDisposition |
LocalDispositionPlanned | LocalDispositionInProgress | LocalDispositionApplied |
LocalCompletionPlanned | Removed | KeyringRetired`; the port test rejects
`Active` and `IntentPending` in this result. If provider status is closed,
SQLite returns `Closed` and makes no local reopening write. Exact
response-operation replay returns the same named result;
changed response-id reuse, intent/membership/epoch/tombstone mismatch, local
busy/full/I/O/commit failure, or a
provider partition is `RepositoryDecommissionAbortError::{IntentNotPending,
MembershipVersionStale, MembershipSnapshotStale, RepositoryEpochStale,
RepositoryAdmissionEpochStale, DecommissionIntentMismatch, BeginTombstoneMismatch, LocalStorageBusy,
LocalStorageFull, LocalStorageIo, LocalCommitFailed, MembershipOperationConflict,
ProviderUnavailable, ProviderCorrupt}` and leaves local admission closed.

`DecommissionPlanKindV1` is exactly `Removal | BeginRetirement |
CompleteRetirement | LocalDisposition | LocalCompletion`.
`DecommissionPlanRequestJournalV1` is a sibling record, never an input to a
plan digest: `{ plan_kind: DecommissionPlanKindV1, plan_digest:
DecommissionDigestV1, exact_request_bytes: OpaqueBytes<=4096,
request_digest: DecommissionDigestV1 }`. The journal is derived only after its
parent plan digest exists, and `request_digest =
H(ASCII(request_domain) || 0x00 || canonical_request_wire)`. Its exact request
wire has the same 16-byte header and canonical TLVs as
`CanonicalDecommissionPlanEncodingV1`, uses the request-domain and tag order
listed below, includes the already-computed parent plan digest, and has the
maximum size stated above. SQLite commits the immutable plan and its matching
journal under one `BEGIN IMMEDIATE` before the relevant provider or local side
effect. A journal with a different plan kind, parent, bytes, digest, declared
length, or domain is `MembershipOperationConflict` or the corresponding
`*PlanMismatch`; it never replaces a plan.

`RemoveRepositoryDecommissionV1` first makes removal executable by writing its
following immutable **pre-Remove** record under `BEGIN IMMEDIATE`, while the
proof-issued metadata row and admission epoch remain closed:
`RepositoryDecommissionRemovalPlanV1 { keyring_id, repository_id,
member_instance_id, repository_epoch, decommission_proof_reference:
DecommissionProofReferenceV1, expected_membership_snapshot_id,
expected_membership_version, decommission_fence_id, admission_epoch,
rotation_fence_id, live_generation_digest: DecommissionDigestV1,
removal_operation_id: ProviderOperationIdV1,
retirement_begin_operation_id: ProviderOperationIdV1,
retirement_complete_operation_id: ProviderOperationIdV1,
retirement_fence_id: OpaqueBytes<=128, local_disposition_operation_id:
RepositoryLocalOperationIdV1, local_completion_operation_id:
RepositoryLocalOperationIdV1, local_storage_disposition:
LocalDecommissionStorageDispositionRequestV1, storage_manifest_digest:
DecommissionDigestV1, plan_digest: DecommissionDigestV1 }`. It binds all known
identity, proof reference, snapshot, fence, disposition, manifest, and stable
ids. The plan digest does not contain itself, any journal field, or any exact
request bytes. Only after that digest is verified may SQLite materialize and
atomically persist the matching Remove journal. The Begin, Complete,
local-disposition, and local-completion journals deliberately do not exist
here: each needs a later provider or local receipt. The reserved ids, fence,
disposition, and manifest remain immutable even when the later branch does not
use them; no cardinality result can select or change any of them.

`RepositoryDecommissionRetirementBeginPlanV1` is the distinct immutable
**post-handoff/pre-Begin** record `{ removal_plan_digest: DecommissionDigestV1,
handoff: KeyringRetirementHandoffV1, handoff_digest: DecommissionDigestV1,
retirement_begin_operation_id: ProviderOperationIdV1, retirement_fence_id:
OpaqueBytes<=128, begin_plan_digest: DecommissionDigestV1 }`. It CASes the
exact provider `RetirementHandoffReady` handoff to
`RetirementHandoffPersisted`, computes the Begin plan digest only from the
five ordered plan fields below, then commits this Begin plan, its separately
derived Begin journal, and `RetirementBeginPlanned` before it may call Begin.
The Begin request bytes contain that exact signed handoff/authenticator, the
pre-reserved fence id, pre-reserved Begin id, and the already-computed Begin
plan digest. A differing handoff/authenticator, parent plan, id, serialized
request, journal, or digest is `RetirementHandoffMismatch`,
`RetirementBeginPlanMismatch`, or `MembershipOperationConflict`, never a
replacement plan.

`RepositoryDecommissionRetirementCompletePlanV1` is the distinct immutable
**post-Begin/pre-Complete** record `{ removal_plan_digest, begin_plan_digest,
handoff_digest, retirement_fence: KeyringRetirementFenceV1,
retirement_fence_digest: DecommissionDigestV1, decommission_proof_reference:
DecommissionProofReferenceV1, retirement_complete_operation_id:
ProviderOperationIdV1, complete_plan_digest: DecommissionDigestV1 }`. Once the
exact Begin fence is stored, SQLite first persists `Retiring`, then computes
this Complete plan digest only from the seven ordered plan fields below and
commits the Complete plan plus its separately derived Complete journal in
`RetirementCompletePlanned`; only that journal may call Complete. Thus neither
a crash nor response loss from Begin to Complete can synthesize a request or
substitute a future receipt into the pre-Remove plan.

`LocalDecommissionStorageDispositionRequestV1` is exactly `Quarantine | Delete`;
its terminal `LocalDecommissionStorageDispositionV1` is exactly `Quarantined |
Deleted`. `storage_manifest_digest` commits, in canonical bounded path order, to the local
durable data affected by that disposition, its byte/row limits, and the
deterministic quarantine target when applicable. The SQLite recovery journal and
metadata row that store the plan are outside a delete manifest and remain as a
tombstone until terminal local completion is durable; deletion cannot erase the
evidence needed to resume it. `RepositoryDecommissionRemovalPlanReceiptV1` is
`{ plan, local_admission_epoch, local_metadata_commit_digest: FixedBytes<=64 }`;
it is the only evidence that permits provider removal. The subsequent Begin,
Complete, disposition, and local-completion calls each require their own later
immutable request plan below.

`RemoveRepositoryDecommissionV1` accepts `{ decommission_proof_reference:
DecommissionProofReferenceV1,
expected_membership_snapshot_id, expected_membership_version,
removal_operation_id: ProviderOperationIdV1, retirement_begin_operation_id:
ProviderOperationIdV1, retirement_complete_operation_id:
ProviderOperationIdV1, local_disposition_operation_id:
RepositoryLocalOperationIdV1, local_completion_operation_id:
RepositoryLocalOperationIdV1, local_storage_disposition: Quarantine | Delete,
storage_manifest_digest: FixedBytes<=64, retirement_fence_id: OpaqueBytes<=128
}`. It derives the pre-Remove plan above
from the authenticated proof reference and request, CASes `ProofIssued` to
`RemovalPlanned { plan_receipt }` before a provider side effect, and never
mutates a stored plan. Exact replay
of its byte-identical request returns `RepositoryDecommissionRemoveResultV1`;
reuse of any one of those ids with changed bytes is
`MembershipOperationConflict`. A crash before this CAS has no provider effect
and leaves `ProofIssued`; retrying the same request is the only way to create a
plan. A crash after it commits is recoverable without caller-supplied operation
ids or disposition.

The planned driver invokes only `remove_keyring_repository_v1` with the stored
Remove request journal, proof reference, and `plan_digest`. A non-last `Updated` result
must contain a `DecommissionRemovalReceiptV1` with that exact plan digest and is
persisted by `BEGIN IMMEDIATE` as `ProviderTerminalPendingLocalDisposition`.
For `RetirementHandoffReady`, it first persists
`RetirementHandoffPersisted { plan_receipt, handoff, handoff_digest }`; it then
creates the exact Begin plan above and only from `RetirementBeginPlanned` calls
Begin. It persists `Retiring { plan_receipt, begin_plan, retirement_fence }`,
then creates the exact Complete plan and only from `RetirementCompletePlanned`
calls Complete. Only a signed `KeyringRetirementReceiptV1` whose plan, handoff,
Begin plan, pre-reserved fence, Begin/Complete ids, proof, scan checkpoint, and
zero counts all match is persisted as `ProviderTerminalPendingLocalDisposition`.
Every provider-visible intermediate has a durable local counterpart; no terminal
receipt, handoff, request byte sequence, or request digest is invented after
response loss.

`ProviderDecommissionTerminalReceiptV1` is exactly `Removed {
removal_receipt: DecommissionRemovalReceiptV1 } | KeyringRetired {
retirement_receipt: KeyringRetirementReceiptV1 }`. Before local completion,
SQLite validates that terminal receipt against plan, proof, identity,
snapshot/version, admission/rotation/decommission fences, live-generation
digest, and all zero-reference/unresolved fields.
`LocalDecommissionDispositionPlanV1` is the immutable **post-provider-terminal/
pre-disposition** record `{ removal_plan_digest, provider_terminal_receipt:
ProviderDecommissionTerminalReceiptV1, provider_terminal_receipt_digest:
FixedBytes<=64, local_disposition_operation_id: RepositoryLocalOperationIdV1,
local_storage_disposition: LocalDecommissionStorageDispositionRequestV1,
storage_manifest_digest: DecommissionDigestV1, disposition_plan_digest:
DecommissionDigestV1 }`. SQLite computes its digest only from the six ordered
plan fields below, then commits it and its separately derived disposition
journal in `LocalDispositionPlanned` before it starts Quarantine or Delete.
The pre-Remove plan cannot predict the later provider-terminal receipt or this
new plan/journal.
`LocalDecommissionStorageReceiptV1` is `{
keyring_id, repository_id, member_instance_id, repository_epoch,
decommission_proof_digest: FixedBytes<=64, removal_plan_digest: FixedBytes<=64,
provider_terminal_receipt_digest: FixedBytes<=64, local_disposition_operation_id:
RepositoryLocalOperationIdV1, storage_manifest_digest: FixedBytes<=64,
disposition: LocalDecommissionStorageDispositionV1::{Quarantined | Deleted},
local_admission_epoch: u64, local_metadata_commit_digest: FixedBytes<=64 }`.
It is a separately persisted immutable SQLite metadata receipt, not a field in
a completion-plan wire. Its canonical lookup key is exactly `(keyring_id,
repository_id, member_instance_id, repository_epoch, removal_plan_digest,
local_disposition_operation_id)`. `CanonicalDecommissionStorageReceiptEncodingV1`
is distinct from `DecommissionPlanKindV1`: its exact 16-byte header is
`0x4f 0x59 0x48 0x52 || 0x01 || 0x06 || 12:u16be || 0:u64be`, where `0x06`
means only `LocalDecommissionStorageReceiptV1`. Its twelve listed fields use
ascending tags `1..=12` and the same nonempty/bounded TLV length rules, so its
maximum is exactly `16 + 12*3 + 128+256+128+8+32+32+32+129+32+1+8+32 = 870`.
`storage_receipt_digest =
H("hr.decommission.local-storage-receipt.v1" || canonical_receipt_wire)`.
SQLite retains the receipt bytes and lookup row through every accepted exact
completion/recovery/status replay and the matching local-terminal-receipt GC
horizon. A missing, duplicate, malformed, or digest/identity/metadata-commit
mismatch is `LocalDispositionReceiptInvalid`, leaves admission closed, and
cannot be repaired by caller input or a newly constructed receipt.

`LocalDecommissionCompletionPlanV1` is the immutable **post-storage-receipt/
pre-local-completion** record `{ removal_plan_digest,
provider_terminal_receipt_digest, storage_receipt_digest: DecommissionDigestV1,
local_completion_operation_id: RepositoryLocalOperationIdV1,
completion_plan_digest: DecommissionDigestV1 }`. SQLite computes its digest
only from the four ordered plan fields below, after resolving and rederiving the
retained storage receipt digest, then commits it and its separately derived
completion journal in `LocalCompletionPlanned` before its terminal local CAS.

The plan preimages, in their only legal tag order, and their later request
journals are:

| plan kind / digest domain | plan tag order (every listed tag is required) | request domain / request tag order |
| --- | --- | --- |
| Removal / hr.decommission.removal-plan.v1 | 1 keyring_id, 2 repository_id, 3 member_instance_id, 4 repository_epoch, 5 proof_reference_wire, 6 expected_snapshot_id, 7 expected_version, 8 decommission_fence_id, 9 admission_epoch, 10 rotation_fence_id, 11 live_generation_digest, 12 removal_operation_id, 13 retirement_begin_operation_id, 14 retirement_complete_operation_id, 15 retirement_fence_id, 16 local_disposition_operation_id, 17 local_completion_operation_id, 18 disposition_enum, 19 storage_manifest_digest | hr.decommission.remove-request.v1: 1 keyring_id, 2 repository_id, 3 member_instance_id, 4 repository_epoch, 5 expected_snapshot_id, 6 expected_version, 7 decommission_fence_id, 8 admission_epoch, 9 proof_reference_wire, 10 removal_plan_digest, 11 removal_operation_id |
| BeginRetirement / hr.decommission.begin-retirement-plan.v1 | 1 removal_plan_digest, 2 handoff_wire, 3 handoff_digest, 4 retirement_begin_operation_id, 5 retirement_fence_id | hr.decommission.begin-retirement-request.v1: 1 handoff_wire, 2 retirement_fence_id, 3 retirement_begin_operation_id, 4 begin_plan_digest |
| CompleteRetirement / hr.decommission.complete-retirement-plan.v1 | 1 removal_plan_digest, 2 begin_plan_digest, 3 handoff_digest, 4 retirement_fence_wire, 5 retirement_fence_digest, 6 proof_reference_wire, 7 retirement_complete_operation_id | hr.decommission.complete-retirement-request.v1: 1 keyring_id, 2 repository_id, 3 member_instance_id, 4 repository_epoch, 5 retirement_fence_id, 6 proof_reference_wire, 7 removal_plan_digest, 8 begin_plan_digest, 9 complete_plan_digest, 10 retirement_complete_operation_id |
| LocalDisposition / hr.decommission.local-disposition-plan.v1 | 1 removal_plan_digest, 2 provider_terminal_receipt_wire, 3 provider_terminal_receipt_digest, 4 local_disposition_operation_id, 5 disposition_enum, 6 storage_manifest_digest | hr.decommission.local-disposition-request.v1: 1 removal_plan_digest, 2 provider_terminal_receipt_digest, 3 local_disposition_operation_id, 4 disposition_enum, 5 storage_manifest_digest, 6 disposition_plan_digest |
| LocalCompletion / hr.decommission.local-completion-plan.v1 | 1 removal_plan_digest, 2 provider_terminal_receipt_digest, 3 storage_receipt_digest, 4 local_completion_operation_id | hr.decommission.local-completion-request.v1: 1 removal_plan_digest, 2 provider_terminal_receipt_digest, 3 storage_receipt_digest, 4 local_completion_operation_id, 5 completion_plan_digest |

For avoidance of doubt, a plan's final plan digest is not a plan tag in its own
preimage; it is H(domain || 0x00 || wire) after all and only the listed input
tags have been encoded. A request's parent plan digest is a request tag because
it already exists. No request digest or exact request bytes are a plan tag. The
arithmetic behind the frozen maxima is:

| value | exact maximum wire sum |
| --- | --- |
| Removal plan | 16 + 19*3 + 128+256+128+8+1265+128+8+128+8+128+32+129+129+129+128+129+129+1+32 = 3096 |
| Begin plan | 16 + 5*3 + 32+1441+32+129+128 = 1793 |
| Complete plan | 16 + 7*3 + 32+32+32+2273+32+1265+129 = 3832 |
| Disposition plan | 16 + 6*3 + 32+1919+32+129+1+32 = 2179 |
| Completion plan | 16 + 4*3 + 32+32+32+129 = 253 |
| Remove request | 16 + 11*3 + 128+256+128+8+128+8+128+8+1265+32+129 = 2267 |
| Begin request | 16 + 4*3 + 1441+128+129+32 = 1758 |
| Complete request | 16 + 10*3 + 128+256+128+8+128+1265+32+32+32+129 = 2184 |
| Disposition request | 16 + 6*3 + 32+32+129+1+32+32 = 292 |
| Completion request | 16 + 5*3 + 32+32+32+129+32 = 288 |

The g.0 contract tests freeze an independently encoded byte-for-byte golden
vector for the minimum and maximum legal value of each of those ten rows,
including the exact domain prefix and expected digest. They also test each
single-field mutation, id mutation, parent-digest/receipt mutation, record
reordering/duplicate/omission, and exact maximum versus maximum-plus-one
bytes. The g.1 adapter and g.2 SQLite suites use a separately maintained
encoder oracle and must obtain the same ten vectors/digests; neither may use
the production encoder to generate its oracle. Completion vectors resolve the
same retained storage-receipt key first and prove that a missing, changed,
duplicate, or metadata-commit-mismatched receipt is
`LocalDispositionReceiptInvalid`, never an alternate 870-byte plan input.
Those suites additionally freeze independent minimum/maximum canonical
local-storage-receipt bytes and digest vectors, their exact lookup key, and
maximum-plus-one/field-order/tag/length failures before deriving the completion
plan; neither implementation may generate those receipt vectors from its
production encoder.
`LocalDecommissionCompletionReceiptV1` is `{ keyring_id,
repository_id, member_instance_id, repository_epoch,
decommission_proof_digest: FixedBytes<=64, removal_plan_digest: FixedBytes<=64,
provider_terminal_receipt_digest: FixedBytes<=64,
storage_receipt_digest: DecommissionDigestV1, local_admission_epoch: u64,
local_completion_operation_id: RepositoryLocalOperationIdV1,
local_metadata_commit_digest: FixedBytes<=64 }`; it binds the matching storage
receipt and is the local terminal-CAS evidence.

`RepositoryDecommissionTerminalResultV1` is exactly `Removed { value:
RepositoryDecommissionRemovalV1 } | KeyringRetired { value:
RepositoryDecommissionRetirementV1 }`.
`RepositoryDecommissionRemovalProgressV1` is exactly `RemovalPlanned {
plan_receipt: RepositoryDecommissionRemovalPlanReceiptV1 } |
RetirementHandoffPersisted { plan_receipt: RepositoryDecommissionRemovalPlanReceiptV1,
handoff: KeyringRetirementHandoffV1, handoff_digest: FixedBytes<=64 } |
RetirementBeginPlanned { plan_receipt: RepositoryDecommissionRemovalPlanReceiptV1,
begin_plan: RepositoryDecommissionRetirementBeginPlanV1 } | Retiring {
plan_receipt: RepositoryDecommissionRemovalPlanReceiptV1, begin_plan:
RepositoryDecommissionRetirementBeginPlanV1, retirement_fence:
KeyringRetirementFenceV1 } | RetirementCompletePlanned { plan_receipt:
RepositoryDecommissionRemovalPlanReceiptV1, begin_plan:
RepositoryDecommissionRetirementBeginPlanV1, retirement_fence:
KeyringRetirementFenceV1, complete_plan:
RepositoryDecommissionRetirementCompletePlanV1 } |
ProviderTerminalPendingLocalDisposition { plan_receipt:
RepositoryDecommissionRemovalPlanReceiptV1, terminal_receipt:
ProviderDecommissionTerminalReceiptV1 }`.
`RepositoryDecommissionRemoveResultV1` is exactly `Progress { status:
RepositoryDecommissionRemovalProgressV1 } | Terminal { terminal:
RepositoryDecommissionTerminalResultV1 }`.
`RepositoryDecommissionCompletionProgressV1` is exactly
`ProviderTerminalPendingLocalDisposition { plan_receipt:
RepositoryDecommissionRemovalPlanReceiptV1, terminal_receipt:
ProviderDecommissionTerminalReceiptV1 } | LocalDispositionPlanned {
plan_receipt: RepositoryDecommissionRemovalPlanReceiptV1, terminal_receipt:
ProviderDecommissionTerminalReceiptV1, disposition_plan:
LocalDecommissionDispositionPlanV1 } | LocalDispositionInProgress {
plan_receipt: RepositoryDecommissionRemovalPlanReceiptV1, terminal_receipt:
ProviderDecommissionTerminalReceiptV1, disposition_plan:
LocalDecommissionDispositionPlanV1 } | LocalDispositionApplied { plan_receipt:
RepositoryDecommissionRemovalPlanReceiptV1, terminal_receipt:
ProviderDecommissionTerminalReceiptV1, storage_receipt:
LocalDecommissionStorageReceiptV1 } | LocalCompletionPlanned { plan_receipt:
RepositoryDecommissionRemovalPlanReceiptV1, terminal_receipt:
ProviderDecommissionTerminalReceiptV1, storage_receipt:
LocalDecommissionStorageReceiptV1, completion_plan:
LocalDecommissionCompletionPlanV1 }`.
`RepositoryDecommissionCompleteResultV1` is exactly `Progress { status:
RepositoryDecommissionCompletionProgressV1 } | Terminal { terminal:
RepositoryDecommissionTerminalResultV1 }`.

`CompleteRepositoryDecommissionV1` accepts `{ keyring_id, repository_id,
member_instance_id, repository_epoch, removal_plan_digest: FixedBytes<=64,
operation_id: RepositoryLocalOperationIdV1 }` and returns
`RepositoryDecommissionCompleteResultV1`. Its id must equal the removal plan's
LocalCompletion id; it supplies neither a terminal receipt nor a new
disposition. Under `BEGIN IMMEDIATE`, it first writes the disposition plan,
starts only its stored disposition, records `LocalDispositionApplied`, writes
the completion plan, and only then performs the one matching terminal CAS. The
admission row stays closed. Busy/timeout, full disk, I/O, transaction-commit,
quarantine, or deletion failure is respectively `DecommissionLocalDrainBusy`,
`DecommissionLocalDrainTimeout`, `DecommissionLocalStorageFull`,
`DecommissionLocalStorageIo`, `DecommissionLocalCommitFailed`,
`DecommissionLocalQuarantineFailed`, or `DecommissionLocalDeleteFailed`; each
leaves a recoverable non-`Active`, non-ready named progress result. Exact replay
returns the same named result; changed reuse is `MembershipOperationConflict`.

`RepositoryDecommissionStatusV1` is exactly `Active | IntentPending
{ intent } | Fenced { fence } | ProofIssuePlanned { proof_issue_plan } |
ProofIssued { issuance: DecommissionProofIssuanceV1 } | RemovalPlanned { plan_receipt } |
RetirementHandoffPersisted { plan_receipt, handoff, handoff_digest } |
RetirementBeginPlanned { plan_receipt, begin_plan } | Retiring { plan_receipt,
begin_plan, retirement_fence } | RetirementCompletePlanned { plan_receipt,
begin_plan, retirement_fence, complete_plan } |
ProviderTerminalPendingLocalDisposition { plan_receipt, terminal_receipt } |
LocalDispositionPlanned { plan_receipt, terminal_receipt, disposition_plan } |
LocalDispositionInProgress { plan_receipt, terminal_receipt, disposition_plan } |
LocalDispositionApplied { plan_receipt, terminal_receipt, storage_receipt } |
LocalCompletionPlanned { plan_receipt, terminal_receipt, storage_receipt,
completion_plan } | Removed { plan_receipt, removal_receipt, storage_receipt,
local_completion } | KeyringRetired { plan_receipt, retirement_receipt,
storage_receipt, local_completion }`. `GetRepositoryDecommissionStatusV1`
accepts `{ keyring_id, repository_id, member_instance_id, repository_epoch }`
and returns that type. It verifies the corresponding signed provider status or
receipt before returning any non-local-only state; a mismatch is a closed error,
never an inferred state. Its `IntentPending { intent }` variant is exclusively
the repository's durable SQLite pre-Begin state; it has no corresponding
`ProviderDecommissionStatusV1` variant and is never exposed as provider truth.

The persisted removal recovery table is total for every local state:

| local status | required durable evidence | only recovery action |
| --- | --- | --- |
| `Active` | current admission epoch | return `Active`; no provider call |
| `IntentPending` (repository-local only) | immutable Begin/abort tuple and both request digests | query provider: `NotStarted` invokes only the persisted Abort CAS; `Aborted` installs only the matching tombstone and may reopen; `Fenced` installs only the exact signed fence; every later signed closed status installs the matching closed local state; mismatch/changed bytes is `ProviderStatusMismatch` or `MembershipOperationConflict` and remains closed |
| `Fenced` | provider admission fence and any persisted bounded scan checkpoint | resume only the fenced bounded scan; atomically persist `ProofIssuePlanned` before any Issue call |
| `ProofIssuePlanned` | immutable observation, Issue id, and Issue request digest | query provider; replay only stored Issue while provider remains `Fenced`, otherwise persist/return the matching signed issuance |
| `ProofIssued` | byte-identical authenticated proof/reference issuance, no plan | return `ProofIssued`; only the byte-identical Remove request may create the pre-provider plan |
| `RemovalPlanned` | immutable plan receipt and matching Remove request journal | query provider; replay only stored Remove journal if provider remains `ProofIssued`; if it is `RetirementHandoffReady`, atomically persist that identical signed handoff; if it is `Removed`, persist only the matching terminal receipt; every other status is `ProviderStatusMismatch` |
| `RetirementHandoffPersisted` | plan receipt, signed handoff, handoff digest | query `ProviderDecommissionStatusV1`; require the identical `RetirementHandoffReady`, then atomically write only the derived Begin plan |
| `RetirementBeginPlanned` | plan receipt, exact signed handoff, Begin plan and matching Begin journal | replay only the stored Begin journal; persist `Retiring` only when its signed fence matches that plan |
| `Retiring` | plan receipt, Begin plan, signed retirement fence | atomically write only the derived Complete plan; do not call Complete yet |
| `RetirementCompletePlanned` | plan receipt, Begin plan, fence, Complete plan and matching Complete journal | replay only the stored Complete journal |
| `ProviderTerminalPendingLocalDisposition` | plan and matching terminal receipt | atomically write only the derived disposition plan |
| `LocalDispositionPlanned` | plan, terminal receipt, disposition plan and matching disposition journal | start or resume only that stored disposition |
| `LocalDispositionInProgress` | plan, terminal receipt, disposition plan/id | resume only that disposition |
| `LocalDispositionApplied` | plan, terminal receipt, retained storage receipt and its exact digest | resolve and rederive only that receipt, then atomically write the derived completion plan |
| `LocalCompletionPlanned` | plan, terminal receipt, retained storage receipt, completion plan and matching completion journal | re-resolve the matching receipt digest, then replay only the stored local completion |
| `Removed` / `KeyringRetired` | plan, provider, storage, and local-completion receipts | return the stored terminal value |

`RecoverRepositoryDecommissionV1` accepts `{ keyring_id, repository_id,
member_instance_id, repository_epoch, recovery_operation_id:
RepositoryResponseOperationIdV1 }` and returns
`RepositoryDecommissionRecoveryV1 { status:
RepositoryDecommissionStatusV1 }` or
`RepositoryDecommissionRecoveryError::{Removal(RepositoryDecommissionRemovalError)
| Abort(RepositoryDecommissionAbortError) |
Proof(DecommissionProofProductionError)}`. Its `recovery_operation_id`
identifies only this response and must be replayed with identical bytes; it is
never substituted for a plan operation id or any stored intent/provider
operation id. Recovery reads the immutable intent plus any proof-issue and
removal plan and applies this total transition table: `Fenced` resumes only its
bounded scan and atomically persists `ProofIssuePlanned`; `ProofIssuePlanned`
queries provider status and repeats only stored Issue while the provider remains
`Fenced`; `RemovalPlanned` queries provider status and repeats stored Remove only
if it remains `ProofIssued`, persists only the identical signed handoff from
`RetirementHandoffReady`, or records only the matching `Removed` receipt;
`RetirementHandoffPersisted` requires the exact
provider `RetirementHandoffReady` and writes the Begin plan; `RetirementBeginPlanned`
replays only its persisted Begin bytes; `Retiring` writes the Complete plan;
`RetirementCompletePlanned` replays only its persisted Complete bytes; a provider
terminal records only its matching terminal receipt; and the later disposition
and completion states write/replay only their respective stored plan. A provider
`NotStarted` is legal only for an unplanned `IntentPending` abort path; it is
`ProviderStatusMismatch` for any removal/retirement plan and cannot reopen the
local fence. The crash/response-loss boundaries after intent write, Begin
mutation or response, fenced scan checkpoint, proof-issue-plan write, Issue
mutation or response, removal-plan write, Remove mutation or response,
provider-handoff persistence, Begin-plan persistence, retirement Begin mutation
or response, `Retiring` persistence, Complete-plan persistence, Complete
mutation or response, terminal-receipt persistence, disposition-plan
persistence/mutation/receipt, completion-plan persistence, and final local CAS
all converge without minting an id, changing disposition, re-registering, or
writing under the old instance.

If a persisted local intent observes provider `NotStarted`, recovery instead
invokes `abort_repository_decommission_v1` only with its persisted Begin/abort
tuple and request digests; `recovery_operation_id` remains response-only. It
opens locally only from the resulting signed `Aborted` tombstone. Thus an abort
that wins first tombstones a delayed begin, while a begin that wins first
initially returns the signed `Fenced` value; a later Get may report only its
matching signed closed successor and the local gate remains closed. The non-last
provider membership CAS or
the sole-member terminal-retirement CAS (attested by its signed terminal receipt)
is the global removal linearization point;
SQLite's plan, intermediate, and terminal CASes are independently atomic locally
and the admission fence spans every remote/local gap.

Every durable transaction reads this same metadata row under `BEGIN IMMEDIATE`,
requires `state = Active` and the current admission epoch before provider
authorization, and rechecks both at commit. The intent write therefore
linearizes after any older committing writer and before every later writer; the
provider fence closes the matching remote authorization interval. Boot observes
a pending intent through `get_repository_decommission_status_v1` and restores
the closed state before readiness or recovery work.

An operation is idempotent only for byte-identical payload and `operation_id`;
after response loss, repeating that exact call returns its original result,
while a changed reuse returns `MembershipOperationConflict`. A
`MembershipVersionStale`, `MembershipSnapshotStale`, `RepositoryEpochStale`,
`MembershipMutationBlocked`, `DecommissionFenceStale`,
`DecommissionObservationStale`, `DecommissionBeginTombstoned`, or provider-
unavailable result changes no provider membership state. The caller must
reacquire the current signed membership or retirement state and, where
applicable, the exact recovered decommission status before a new attempt. It
cannot retry removal from a stale proof or silently omit a member. A pending
local intent remains write-closed until provider status plus the begin-tombstone
CAS resolves it; a crash resumes from that exact state before readiness. Abort
is legal only before proof issuance: `abort_repository_decommission_intent_v1`
reopens the local gate only after the provider has durably recorded the matching
begin tombstone and SQLite has installed its greater reopened epoch. The closed provider
error is `KeyringMembershipError::{MembershipVersionStale,
MembershipSnapshotStale, RepositoryAlreadyRegistered, RepositoryNotRegistered,
MemberInstanceStale, MembershipMutationBlocked, RepositoryEpochStale,
DecommissionFenceStale, DecommissionPending, DecommissionBeginTombstoned,
DecommissionProofMissing, DecommissionProofInvalid,
DecommissionProofAlreadyIssued, DecommissionScanIncomplete,
DecommissionObservationStale, DecommissionProofReferenceMissing,
DecommissionProofReferenceMismatch, DecommissionProofReferenceAuthenticatorInvalid,
KeyringRetiring, KeyringRetired, KeyringRetirementFenceStale,
KeyringRetirementPreconditionFailed, MembershipOperationConflict,
ProviderUnavailable, ProviderCorrupt}`. The repository removal error is
`RepositoryDecommissionRemovalError::{ProofMissing, ProofInvalid,
RemovalPlanMissing, RemovalPlanConflict, RemovalPlanDigestMismatch,
RemovalPlanInvalid, StorageManifestMismatch, MembershipVersionStale,
MembershipSnapshotStale, RepositoryEpochStale, MembershipMutationBlocked,
DecommissionFenceStale, DecommissionObservationStale, ProviderStatusMismatch,
DecommissionProofReferenceMissing, DecommissionProofReferenceMismatch,
DecommissionProofReferenceAuthenticatorInvalid,
RetirementHandoffMismatch, RetirementBeginPlanMissing,
RetirementBeginPlanMismatch, RetirementCompletePlanMissing,
RetirementCompletePlanMismatch, LocalDispositionPlanMissing,
LocalDispositionPlanMismatch, LocalCompletionPlanMissing,
LocalCompletionPlanMismatch,
KeyringRetirementFenceStale, KeyringRetirementPreconditionFailed,
ProviderTerminalReceiptMissing, ProviderTerminalReceiptInvalid,
LocalDispositionReceiptInvalid, LocalCompletionConflict,
DecommissionLocalDrainBusy, DecommissionLocalDrainTimeout,
DecommissionLocalStorageFull, DecommissionLocalStorageIo,
DecommissionLocalCommitFailed, DecommissionLocalQuarantineFailed,
DecommissionLocalDeleteFailed, MembershipOperationConflict,
ProviderUnavailable, ProviderCorrupt}`. This is the shared closed error type
for `RemoveRepositoryDecommissionV1`, `CompleteRepositoryDecommissionV1`,
and `GetRepositoryDecommissionStatusV1`. `RepositoryDecommissionRecoveryError`
is exactly `Removal(RepositoryDecommissionRemovalError) |
Abort(RepositoryDecommissionAbortError) |
Proof(DecommissionProofProductionError)` and preserves, rather than maps or
hides, the typed error from its one stored-step recovery action;
in particular, changed bytes under any intent, plan, provider, local, or
response operation id are never translated into a new reservation or an untyped
local conflict. A
partitioned, omitted, or stale member cannot be removed to evade a drain; after
removal or retirement the old member remains fenced and cannot rejoin the old
keyring.

The g.0 adapter mapping is closed and exhaustive. Every
`KeyringMembershipError` variant is matched by name, never `_`: identity/CAS
variants (`MembershipVersionStale`, `MembershipSnapshotStale`,
`RepositoryNotRegistered`, `MemberInstanceStale`, `RepositoryEpochStale`,
`MembershipMutationBlocked`, `DecommissionFenceStale`) map to the same named
proof/abort/removal error where that operation admits it; proof variants
(`DecommissionPending`, `DecommissionProofMissing`,
`DecommissionProofInvalid`, `DecommissionProofAlreadyIssued`,
`DecommissionScanIncomplete`, `DecommissionObservationStale`,
`DecommissionProofReferenceMissing`, `DecommissionProofReferenceMismatch`,
`DecommissionProofReferenceAuthenticatorInvalid`) map to the same-named proof
or removal error; terminal/retirement variants
(`DecommissionBeginTombstoned`, `KeyringRetiring`, `KeyringRetired`,
`KeyringRetirementFenceStale`, `KeyringRetirementPreconditionFailed`) map to
the named abort/retirement/removal result or error; and
`MembershipOperationConflict`, `ProviderUnavailable`, and `ProviderCorrupt`
preserve their exact name. A variant impossible for a method is absent from that
method's closed error type and is rejected by the provider before an adapter
translation; it is not hidden in a catch-all. In particular,
`KeyringMembershipError::DecommissionObservationStale` maps to
`DecommissionProofProductionError::DecommissionObservationStale` for Issue and
to `RepositoryDecommissionRemovalError::DecommissionObservationStale` for
Remove/retirement rechecks. Port, key-adapter, memory, and SQLite tests compile
an explicit match arm for every provider status/result/error and fault-inject
each changed scoped-id/request-digest branch.

Provider-to-local status routing is likewise an explicit exhaustive match:
`NotStarted` and `Aborted` are legal only for the stored repository-local
intent/abort-tombstone path; `Fenced` maps only to local `Fenced` or
`ProofIssuePlanned`; `ProofIssued` only to `ProofIssued` or `RemovalPlanned`;
`RetirementHandoffReady` only to `RemovalPlanned`,
`RetirementHandoffPersisted`, or `RetirementBeginPlanned`; `Retiring` only to
`RetirementBeginPlanned`, `Retiring`, or `RetirementCompletePlanned`; `Removed`
only to `RemovalPlanned`, `ProviderTerminalPendingLocalDisposition`, a named
local disposition/completion progress state, or `Removed`; and `KeyringRetired`
only to `RetirementCompletePlanned`,
`ProviderTerminalPendingLocalDisposition`, a named local disposition/completion
progress state, or `KeyringRetired`. Any other pair is
`ProviderStatusMismatch`. `RepositoryAlreadyRegistered` is valid only for
Register; if a decommission adapter ever decodes that or another method-
impossible provider error, its explicit match arm returns `ProviderCorrupt`, not
a catch-all or a fabricated local status.

`ZeroReferenceReceiptV1` is `{ keyring_id, source_generation: u64,
rotation_fence_id: OpaqueBytes<=128, membership_snapshot_id,
membership_version, repository_id, member_instance_id, repository_epoch,
repository_admission_epoch: u64, repository_admission_state: Active,
terminal_checkpoint_digest: FixedBytes<=64, unresolved_receipt_digest,
durable_ciphertext_references: u64be(0), durable_locator_references: u64be(0),
durable_non_replay_index_references: u64be(0),
unresolved_authorizations: u64be(0), provider_authenticator }`.
`RetirementReceiptV1` is `{ keyring_id,
source_generation: u64, rotation_fence_id, membership_snapshot_id,
membership_version, member_receipt_digests: [FixedBytes<=64; 1..=4096 in exact
snapshot-member order], provider_unresolved_authorizations: u64be(0), state: Revoked,
provider_authenticator }`. The provider issues the latter only after it verifies
one exact former receipt from every frozen snapshot member; neither receipt can
be reused across fence, snapshot, member instance, source generation, epoch,
admission epoch/state, or unresolved receipt digest. A decommission proof is
never interchangeable with a normal-rotation zero-reference receipt.

`NormalRotationBlocked` carries
`{ source_generation: u64, reason: NormalRotationBlockReason }`, where
`NormalRotationBlockReason::{DrainingGenerationPresent, EmergencyDrainPresent,
IncompleteRekey, DurableReferencesRemain, UnresolvedAuthorization,
ZeroReferenceReceiptMissing, MembershipTerminalReceiptMissing,
RetirementReceiptMissing, MembershipSnapshotStale, RepositoryDecommissioning,
SourceUnavailable}`.
`IdempotencyConflict` is the closed no-payload replay outcome for a located row
whose authenticated canonical plaintext differs from the submitted
`CanonicalRequestV1`; it creates no reservation, mutation, or second business
effect and exposes neither locator bytes, ciphertext, nor plaintext.
`IdempotencyLocatorCollision` carries the bounded matched-row count; a locator
query exceeding five row reads returns `IdempotencyLocatorLimitExceeded`;
`IdempotencyLocatorDivergence` carries only generation/format identifiers and
never ciphertext or plaintext. These are closed typed outcomes, not diagnostic
strings or retry-to-create hints.

`BeginNormalRotationV1` is the provider CAS operation, exposed by
`begin_normal_rotation_v1`:
`{ keyring_id: OpaqueBytes<=128, expected_active_generation: u64,
expected_rotation_fence_id: OpaqueBytes<=128, expected_membership_snapshot_id:
OpaqueBytes<=128, expected_membership_version: u64,
operation_id: ProviderOperationIdV1 }`. It returns exactly
`NormalRotationResult::{Started { active_generation, draining_generation,
rotation_fence_id, membership_snapshot_id, membership_version }, Blocked {
source_generation, reason }}` or
`NormalRotationError::{RotationFenceStale, MembershipSnapshotStale,
GenerationNotActive, RotationOperationConflict, ProviderUnavailable,
ProviderCorrupt}`. `Started` atomically
freezes the exact current membership snapshot with the new rotation fence and is
possible only for the one permitted `G -> G+1` transition. A G+2 attempt reports
the typed `Blocked` result and does not change provider or membership state. A
lost response retries only with the byte-identical request and `operation_id`,
which returns the original result; a stale fence/snapshot requires a fresh signed
snapshot and a new operation id, never an automatic successor attempt. This
operator, not repository narration, owns global keyring transition truth.

`CommitBinding` is a fixed-width provider-authenticated value over the exact
nine-component outer preimage and canonical staged-write descriptor specified
above, domain-separated by tenant, repository, repository/admission epoch,
authorization id, schema, and key generation. Neither it nor
`CommitAuthorizationId` is an unkeyed request digest: the authorization id is an
opaque repository-unique transaction identity, and neither value is a telemetry
label or cross-slot equality token.

Associated data canonically binds tenant, legal entity, logical table and
field, opaque row identity, schema version, and record generation. Every size
sum uses checked arithmetic and the existing stored-outcome/body hard ceilings;
oversize plaintext or envelope fails before transaction commit. A nonce is
unique for every `(key generation, seal operation)` and is never caller-
selected. Authentication failure, unknown algorithm/generation, malformed
envelope, provider timeout, missing key, and revoked key are typed fail-closed
errors. There is no plaintext, zero-key, cached-forever, environment, or test-
fake fallback in a production composition.

L2i.0d is a non-dispatchable decision gate until it names the exact accepted
authenticated-encryption primitive/library and commodity or sold key-service
facade, versions/features/licenses, generated client and Cargo/Buck targets,
key custody and zeroization boundary, nonce source, retry/deadline bounds,
  idempotency-locator PRF, exact `AcquireReplayGenerationSetV1` and
  `derive_idempotency_locator_v1` request/result/error and signature/lease
  validation, V1-only active-writer format, future-format evolution barrier,
  exact keyring enrollment/decommission/remove/snapshot semantics, global
  normal-rotation CAS/refusal semantics, generation and commit-authorization
  linearization semantics, bounded unresolved-receipt enumeration, repository-
  epoch and admission-fence fencing, recovery/administrative resolution, and
  removal path. L2i.0d.1 must first admit
  the key-service adapter structure; L2i.0f must then prepare the exact unique
  port/repository/SQLite/adapter file slots; L2i.0g.0 freezes only the typed port
  contract; L2i.0g.1 implements and tests replay/membership/decommission
  provider behavior; L2i.0g.1a then implements and reviews the minimal concrete
  adapter open/seal, commit-authorization, resolution, and decommission-fence
  behavior; L2i.0g.2 only then freezes repository/SQLite behavior and its
  executable dev-only composition target; and L2i.0h implements bounded
  repository rekey/recovery before production composition is dispatchable. The
selected adapter is owner-local and remains
`app/hr/adapters/draft/record-encryption-key-service` /
`hr-record-encryption-key-service-draft` while its port is draft.

The HR error type is closed as
`CommitFenceError::{GenerationNotActive, AuthorizationDenied,
AuthorizationUnresolved, CommitBindingMismatch, RepositoryEpochStale,
RepositoryAdmissionEpochStale, RepositoryDecommissioning, ResolutionConflict,
ProviderUnavailable, ProviderCorrupt}`. A successful
resolution is exactly `CommittedBeforeFence` or `AbortedBeforeCommit`; neither
is inferred from a timeout, disconnect, clock, or missing row. Pending-page
results are capped by the L2i.0d accepted item/byte limits and carry an opaque
continuation; duplicate pages and resolutions are idempotent, while a skipped,
reordered, oversized, or non-progressing continuation fails closed.

The provider state machine is
`Active -> Draining | EmergencyDraining -> Revoked`. `authorize_commit`,
`AcquireReplayGenerationSetV1`, `derive_idempotency_locator_v1`, membership CAS,
decommission admission fences, and those transitions share one provider-side
linearization order. Authorization
is allowed only in `Active` and returns an opaque single-use receipt bound to
the repository epoch, transaction id, generation, and commit binding. A
rotation/revocation request that wins first denies authorization. An
authorization that wins first remains pending and orders that one commit before
the transition barrier; the transition cannot become `Revoked` until the
receipt is idempotently resolved committed or aborted. Resolution never expires
into an assumed outcome. Repository-epoch acquisition and unresolved-receipt
enumeration participate in the same provider authority: acquiring epoch N+1
fences N before the new writer may classify N's pending receipts.

One generation is active for new seals. Normal rotation has one global
per-keyring CAS rule: it may transition `Active(G)` to
`Active(G+1) + Draining(G)` only when the keyring has no other `Draining` or
`EmergencyDraining` generation and the caller's exact membership snapshot/version
is still current. The successful CAS freezes that snapshot, disallows enrollment,
removal, and rejoin until terminal retirement, and binds it to both the rotation
fence and every zero-reference receipt. A normal `G+2` request while G is
draining, has an incomplete rekey, durable ciphertext/locator/non-replay-index
reference, unresolved earlier authorization, missing snapshot-member receipt,
missing zero-reference receipt, or missing provider retirement receipt returns
`NormalRotationBlocked` and changes nothing. The provider may issue the G
retirement receipt only after it verifies one terminal receipt from every member
instance in the frozen snapshot, each bound to that snapshot/fence/source
generation and reporting zero ciphertext, locator, and non-replay-index
references plus zero unresolved authorizations, and its own unresolved count is
zero; only then may it CAS G to
`Revoked` and admit a new normal rotation. A partitioned repository cannot be
omitted: it blocks retirement and G+2 rather than becoming removable.
Draining G issues no seals or commit authorizations. Emergency drain immediately
denies new seal/open/authorization/replay admissions, returns no replay set, and
withdraws affected readiness; it also blocks normal rotation until the source is
recovered and retired under the same rule. Source loss, stale matrix/fence,
rekey/replay races, and provider loss therefore return their typed refusal
without zero-match reservation. Provider resolution remains available only to
settle already ordered receipts, which may acknowledge only after
`resolve_commit` proves `CommittedBeforeFence`. "Immediate" means
admission/readiness withdrawal, not retroactive invalidation. There is no
fallback or silent discard.

### Bounded repository rekey protocol

The high-level `RekeyRepository` operation is owned by
`hr-employment-repository-draft`; its provider-neutral reconciler is an
employment use case and its SQLite implementation composes
`hr-record-encryption-draft`. The key-service adapter has no repository edge
and never enumerates HR rows. L2i.0d must expose an immutable normal-rotation
fence plus bounded discovery of incomplete rotations; a crash after the
provider moves `G` to `Draining` but before SQLite creates its job is therefore
recoverable. Job creation is idempotent on
`(repository_id, rotation_fence_id, source_generation, target_generation)`.
The repository port's exact operations are `begin_or_resume_rekey`,
`scan_rekey_page`, `compare_and_swap_rekey_page`,
`count_generation_references`, `record_revocation_authorized`, and
`complete_rekey`; the use case exposes one bounded `advance_rekey` step. The
record-encryption operations above supply only fence discovery, open/seal/
reindex, checkpoint authentication, unresolved-authorization state, and final
revocation authorization. No operation accepts an unbounded iterator or raw
SQLite/provider type.

The hard v1 work bounds are:

| Resource | Hard maximum |
|---|---:|
| one ciphertext envelope read by rekey | 256 KiB |
| one opaque scan cursor | 512 bytes |
| one authenticated checkpoint | 4 KiB |
| records returned by one page | 64 |
| aggregate ciphertext bytes in one page | 8 MiB |
| pages in one reconciler step | 8 |
| records in one reconciler step | 512 |
| aggregate ciphertext bytes in one step | 64 MiB |
| encryption-provider calls in one step | 2,048 |
| consecutive CAS restarts at one page cursor | 3 |

All counters and products use checked `u64` arithmetic. A page request above a
maximum is rejected; SQLite queries one extra row only to determine whether a
continuation is needed and never returns that extra row. A first record above
the per-record or page byte ceiling is corrupt rather than an excuse for a
non-progressing empty page. A step reaching any pages/items/bytes/calls limit
returns `RekeyYielded` with its last durable checkpoint. There is no unbounded
loop, recursive retry, or retry while holding a SQLite transaction.

The durable checkpoint contains format version, job and rotation-fence ids,
frozen membership snapshot id/version and repository member-instance id,
repository id/epoch, source and target generations, phase, the exclusive
`(logical_table_tag, opaque_row_identity)` cursor, page/item/byte/provider-call
counters, the consecutive-CAS counter for that cursor, and a provider-
authenticated checkpoint binding. Phases are
`Scanning -> ZeroReferenceObserved -> RevocationAuthorized -> Complete`; no
phase or cursor may move backward. On boot the writer first acquires the new
exclusive repository epoch, validates the checkpoint binding and provider
fence, and either resumes the exact next page or returns a typed refusal.

For one page, SQLite deterministically scans every sensitive table and every
generation-scoped idempotency-locator column (plus any non-replay field-index
column) in `(logical_table_tag, opaque_row_identity)` order for
`source_generation`. Outside a transaction the adapter opens each observed
envelope under the source generation, validates associated data, seals the
identical semantic plaintext under the active target with a new provider nonce,
and recomputes the target-generation idempotency locator from the stored logical
slot—not from canonical-request bytes—plus any non-replay field indexes under
their own contracts. It then starts one `BEGIN IMMEDIATE` transaction and
CAS-checks every row against row identity, observed revision, source generation,
envelope commitment, and old locator/index set. The transaction atomically
replaces ciphertext, generation, locators/indexes and revision for the entire
page and advances the checkpoint. A
single mismatch aborts the whole page and leaves its prior checkpoint durable.
The reconciler retries that same cursor in stable row order; each conflict
durably increments the cursor's counter, and the fourth attempt returns
`RekeyCasRetryExhausted` until a new operator-admitted run, rather than skipping
or spinning.

Replay and rekey share that single SQLite writer order. A replay first obtains
its provider-authenticated V1 generation set and derives every one-to-two V1
idempotency locators through `derive_idempotency_locator_v1` before
`BEGIN IMMEDIATE`;
inside that transaction it validates epoch, lease, digest, fence, and membership
snapshot before either zero-match reservation or authenticated open. A page rekey
enters the same writer before it replaces an idempotency row's source-generation
locator with its target-generation locator. If replay wins, rekey observes its
committed row/revision on the next scan; if rekey wins, the already-derived
target-generation V1 locator locates the same row. The global no-overlap CAS returns
at most active G+1 plus draining G; G+2 is refused until G is zero-reference,
all frozen-snapshot members have terminal receipts, and G is revoked. A rotation
that wins before a new request invalidates a stale matrix rather than allowing
a reservation; a request that wins authorization before the transition has its
receipt resolved before the transition barrier. Consequently a retry immediately
before, during, or after page CAS either returns the original outcome or a typed
unavailable/corruption result, never a second employee, lifecycle, idempotency,
or outbox effect. Response loss and hard close repeat the complete matrix lookup
after exclusive-epoch recovery. If a draining source is lost before its row
rekeys, the provider cannot authenticate the matrix/lease, or a locator/row
binding diverges, readiness withdraws and `ReplaySourceUnavailable`,
`ReplayGenerationSetUnavailable`, or `IdempotencyLocatorDivergence` is returned;
a new reservation is forbidden. Source revocation occurs only after terminal
count includes every source locator column at zero, so post-revoke
retry locates the rekeyed target row without a cross-generation token.

After an end-of-keyspace page, one SQLite transaction repeats the complete
source-generation reference count across ciphertext and idempotency-locator
columns plus any non-replay field-index columns.
Because the provider drain fence forbids new source-generation seals and commit
authorizations, a zero count is monotonic under the fenced repository epoch.
That transaction writes a provider-authenticated `ZeroReferenceReceiptV1` bound
to the terminal checkpoint, repository sequence, source generation, frozen
membership snapshot id/version, and member-instance id; it attests zero durable
references and zero unresolved authorizations. The provider may CAS the source
to `Revoked` only when it has that exact receipt from every snapshot member and
its own earlier authorization count is zero. A crash after provider revocation
but before the SQLite completion row is recovered by matching the immutable
provider receipt; a membership/snapshot mismatch remains corruption.
`ReferencesRemain` never advances the phase.

The closed rekey result/error vocabulary is
`RekeyResult::{Progressed, Yielded, Complete}` and
`RekeyError::{PageLimitExceeded, RecordTooLarge, CursorInvalid,
CheckpointInvalid, CheckpointStale, RepositoryEpochStale,
RotationFenceStale, SourceGenerationUnavailable,
TargetGenerationUnavailable, EnvelopeAuthenticationFailed,
CanonicalFormatUnsupported, StagedDescriptorInvalid, RekeyCasConflict,
RekeyCasRetryExhausted, ReferencesRemain, AuthorizationUnresolved,
RepositoryUnavailable, ProviderUnavailable, ProviderCorrupt}`. None is mapped
to success, conflict-free replay, key fallback, or completed revocation.

The replay result/error vocabulary is closed as
`ReplayLookupError::{ReplayGenerationSetUnavailable, ReplayGenerationSetMalformed,
ReplayFormatMatrixInvalid, IdempotencyLocatorLimitExceeded,
IdempotencyLocatorCollision, IdempotencyLocatorDivergence,
ReplaySourceUnavailable, ReplayLeaseStale, RepositoryDecommissioning,
RotationFenceStale, NormalRotationBlocked}`. A collision, stale lease/fence,
source loss, malformed/oversized set, incompatible format matrix, provider loss,
or source-generation/locator mismatch never falls through to zero-match creation.
Contract and real-file recovery evidence must independently exercise Cargo and
  Buck full-preimage vectors; every active-only and active-plus-draining V1 set;
  V1 replay after response loss, rotation/page CAS, hard close, rekey, and fresh
  restart; same-slot changed-canonical conflict; encrypted-canonical plaintext
  equality under different nonce/generation; ciphertext/tag/associated-data
  tampering; attempted
  G+2 during G drain; emergency drain/source loss; stale matrix/fence/membership
  snapshot; concurrent enrollment/removal/rejoin/partition; and replay versus
  rekey immediately before/during/after page CAS and zero-reference revocation.
  Each schedule proves the same semantic retry returns its original outcome, a
  changed canonical request returns `IdempotencyConflict`, or a closed typed
  refusal, and no second effect commits.

Before a first cohort, readiness is false while any normal rekey job is
incomplete. For an already-routed cohort, normal rotation may keep reads ready
only while source opens, target seal/open/index/authorization, authenticated
matrix acquisition, repository progress, and declared progress SLO are healthy;
all new writes use the target. Emergency drain, corrupt/stale checkpoint,
exhausted CAS progress, source/target loss, stale/malformed matrix, or
repository/provider outage withdraws the affected cohort. Software rollback is
allowed before a production rotation only by
removing the unrouted additive files/migrations and scratch databases. Once
migration `0003` opens any non-scratch database or a rotation fence is admitted,
rollback means a schema N/N+1
reader that preserves and resumes the job; it may not delete the checkpoint,
downgrade either preimage format, reactivate a generation with target
references, or undo revocation. Normal rotation starts only after every epoch-
eligible process reads the additive schema and current preimage formats.

Contract evidence injects unique sentinels into every sensitive field, commits
to a real SQLite file, checkpoints and copies the backup, and proves neither
artifact contains a sentinel. It hard-closes every encryption, SQL, commit,
rotation-CAS, and reply boundary; reopens with a fresh process and provider
client; verifies authenticated reads and idempotent replay; detects ciphertext,
tag, nonce, associated-data, idempotency-locator, and non-replay field-index
tampering; rotates under concurrent
reads/writes; and exercises normal plus emergency revocation and provider loss
before boot and mid-transaction. It separately races provider authorization,
SQLite commit, provider resolution, rotation/drain, hard close, and exclusive-
epoch recovery in every order, including a kill after authorization but before
the SQLite receipt becomes durable. It injects duplicate/missing/reordered and
limit/limit-plus-one pending pages and a stale repository epoch. Success exposes
either the one committed, resolved authenticated value or no acknowledged
effect. Partial plaintext, unkeyed request equality, nonce reuse, mixed state
without generation metadata, acknowledgement without a resolved receipt,
completed revocation with a pending earlier receipt, or fallback material is
failure.

</data_at_rest>

<error_model>

## Stable failure classes

| Class | Examples | Mutation/disclosure |
|---|---|---|
| validation | malformed identifier/date/evidence, invalid checklist, changed payload on reused key | none |
| unauthenticated | missing, invalid, expired, or unbound principal proof | none |
| forbidden | PDP deny, tenant mismatch, absent legal basis, stale/conflicting overlay | none |
| conflict | employee version mismatch, duplicate identity, changed canonical plaintext at an idempotency locator, commit-resolution conflict, bounded rekey page CAS conflict | none |
| unavailable | SQLite busy/full/unopenable, Packs/Policy/IAM/Audit/key/runtime-context precondition unavailable, commit authorization unresolved, selected adapter unhealthy, key generation draining/revoked, source/target key outage, membership partition, decommission pending, rekey retry exhaustion | none acknowledged; eligible required-authority failure burns availability budget |
| internal/corrupt | schema/preimage incompatibility, corrupt stored outcome, invalid canonical request or staged descriptor, stale/corrupt rekey cursor/checkpoint/fence/membership snapshot, ciphertext/tag/nonce/generation/commit-binding mismatch, impossible state | fail closed; readiness false |

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
- same-key/same-canonical-request replay returns byte-equivalent semantic outcome
  and does not duplicate employee, lifecycle, audit/outbox, workflow, or payroll
  intent across active-only, active-plus-draining, response-loss, page-rekey,
  hard-close, and restart schedules;
- same-key/different-canonical-request replay reaches the same generation-scoped
  locator and returns conflict without mutation or a second reservation;
- domain, authorization, overlay, encryption/key, and adapter failures preserve
  no partial state, plaintext persistence, or sensitive disclosure;
- schema N/N+1 open, migrate, reopen, and supported rollback boundaries are
  explicit.
- canonical-request and staged-descriptor schema N/N+1 reader behavior over
  fixed V1 bytes, exact bounds, semantic default/reordering equivalence,
  changed-field conflict, current V1-only format admission, and rejection of
  unknown, omitted, duplicated, or reordered same-version fields are explicit;
- bounded rekey pages atomically replace envelope plus all idempotency locators
  and non-replay field indexes,
  advance no checkpoint on CAS failure, resume from the last committed cursor,
  and produce a snapshot/member-instance-bound zero-reference receipt before
  normal revocation;
- enrollment, duplicate enrollment, stale membership CAS, removal, rejoin, and
  partition preserve an immutable rotation snapshot; G+2 stays refused until each
  exact snapshot member produces its terminal receipt and the provider has zero
  unresolved authorizations;
- decommission first writes the local intent/admission epoch, then provider
  fence, each bounded ciphertext/locator/non-replay-index scan checkpoint,
  terminal all-zero observation, authenticated proof, immutable known-input
  pre-Remove plan, provider membership removal or provider HandoffReady/
  handoff-persisted/Begin-plan/`Retiring`/Complete-plan/terminal receipt,
  post-terminal disposition plan/disposition, and post-storage completion
  plan/completion.
  Authorization/commit immediately before and after each edge, changed-operation
  replay, plan/receipt/count mismatch, `NotStarted`/abort-tombstone/delayed-Begin,
  recovery-response-id substitution, changed scoped-id/request-digest and
  handoff/authenticator tamper, response loss, partition, crash/restart, local
  drain/delete/quarantine fault,
  stale live-generation digest, concurrent rotation, abort/resume, last-member
  retirement, and rejoin prove no durable reference can commit after the
  observation, provider terminal transition, or local completion and no recovery
  can invent an id or change disposition;
- an eligible request whose required provider is unavailable preserves no mutation
  or disclosure and increments the availability denominator, authority-failure,
  and error-budget signals until recovery or router-withdrawal acknowledgement.

SQLite fault injection uses a real file and a fresh process or connection after
each interruption. Failpoints cover after `BEGIN`, idempotency insertion,
employee write, lifecycle write, audit/outbox write, before `COMMIT`, after
`COMMIT` before response, and migration boundaries. Every case closes without
graceful cleanup, reopens the database, checks invariants, and performs an
idempotent replay. In-memory tests are semantic reference evidence, not crash
durability evidence.

Commit-fence fault injection pauses before/after `authorize_commit`, before/
after SQLite commit, and before/after `resolve_commit`; concurrently requests
normal rotation and emergency drain, kills the process, then takes an exclusive
repository recovery epoch. The fresh process must resolve exactly one stored
receipt state. A provider transition cannot report `Revoked` while an earlier
receipt is unresolved, and an unresolved or provider-unavailable receipt cannot
produce an acknowledgement.

Rekey fault injection pauses before/after enrollment, membership snapshot,
incomplete-rotation discovery, job
creation, page scan, source open, target seal, idempotency-locator derivation,
page CAS,
checkpoint commit, terminal reference count, provider revoke, and local
completion. It then hard-closes all connections, constructs a new repository
and provider client, fences the older epoch, and resumes. Exact/limit-plus-one
items, bytes, pages, calls, cursor and checkpoint sizes; stale epochs/fences;
three conflicts plus the refused fourth attempt; full/busy media; source/target
key and provider loss; duplicate/missing/partitioned/rejoining snapshot members;
and nonzero ciphertext, locator, or non-replay-index references all produce the
closed result/error vocabulary without skipping a row, advancing a failed
checkpoint, silently removing a member, or reporting `Revoked`.

</conformance_and_faults>

<observability>

## Signals and SLO qualification

The facade measures admitted request latency, result class, policy/overlay
generation age, selected adapter, transaction phase/latency, replay/conflict,
outbox lag/redelivery, reopen/migration result, rekey phase/checkpoint age,
bounded rows/bytes/pages/provider calls, CAS conflicts, remaining-generation
references, membership snapshot/version/refusal, typed rekey refusal, saturation,
and the availability `eligible_total`, `good_total`,
`required_authority_failure_total`, `error_budget_burn`, readiness transition,
and router-withdrawal-acknowledgement signals. The denominator is decided at
request admission and cannot be retroactively reduced after a provider recovers;
only a caller-caused validation/unauthenticated/forbidden result determined by an
available authority is excluded. Cardinality is bounded; employee, person,
evidence, and idempotency values are not labels.

Production implements `hr-runtime-context-draft` only through
`hr-runtime-context-oyatie-draft`, an HR-owned consumer adapter for the accepted
Cell trusted-interval and Observability signal facades. Its time value is
`[earliest, latest]` plus source generation and uncertainty; it is not a scalar
wall-clock timestamp. If the interval straddles a policy expiry, overlay
effective boundary, key/legal effective boundary, the operation returns typed
`TimeUncertain`. Provider outage returns `RuntimeContextUnavailable`; system/
process time and discard-to-log are not fallbacks. Signal emission is bounded,
correlation-safe, and contains no HR identity or payload.

Readiness is false when the selected durable adapter cannot open or commit, its
schema is outside the supported window, the active encryption key generation
cannot seal/open/authorize/resolve or is draining/revoked, trusted interval or
bounded telemetry health is unavailable, policy authority is unusable, or a
  required pre-ack audit path cannot satisfy the request class, the repository is
  not an admitted current membership instance, its local/provider decommission
  fence is pending or active, or its frozen rotation snapshot cannot be
  authenticated. It is also false
for a first cohort with an incomplete rekey job and for a routed cohort whose
normal rekey checkpoint exceeds the PRD 60-second objective, exhausts its CAS
  budget, loses either generation/provider/repository, or detects a required-
  authority outage. The outage's eligible failures continue to burn availability
  until recovery or router withdrawal is acknowledged; readiness alone does not
  reclassify them. Health does not
claim durable, network, or downstream capability merely because pure domain
tests pass. The PRD SLO remains unqualified until these signals and the declared
load envelope are exercised in promotion evidence.

</observability>
