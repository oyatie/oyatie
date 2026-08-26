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
| record encryption | seal/open sensitive values, derive tenant/key-scoped blind indexes and opaque commit bindings, and linearly authorize/resolve repository commits against key-generation transitions without exposing provider types |
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
commit authorization, and bounded rekey progress. The repository port owns the
canonical request and staged-write descriptor byte formats below. The adapter
asks the record-encryption port for a repository, tenant, operation,
idempotency-key, schema, format, and key-generation-scoped blind index and
persists the canonical request only inside authenticated ciphertext. It never
computes or stores an unkeyed request digest. The SQLite adapter's only runtime dependencies
are `hr-employment-repository-draft`, `hr-record-encryption-draft`, and
`rusqlite.workspace = true`. Its only dev-dependencies are
`hr-employment-repository-memory-draft` and `tempfile.workspace = true`;
recovery targets use the real SQLite adapter and `tempfile`, never the memory
oracle or `:memory:`.
The logical idempotency key is:

```text
(tenant_id, operation_kind, idempotency_key)
```

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

The sole V1 replay purpose is `CanonicalRequestReplayV1`, encoded as the
fixed `u16be(1)` `purpose_tag`. It means only “key-generation-scoped equality
for this exact HR canonical request in its logical replay slot”; it is not a
field label, a caller input, a provider-selected value, or a reusable
cross-generation equality token. The purpose is already versioned by both its
name and the V1 domain below. No other V1 purpose tag is admitted. This is the
single law for every `blind_index` caller, including the repository port,
SQLite adapter, rekey worker, Cargo vectors, and Buck vectors.

The blind-index full preimage is exactly the literal ASCII domain bytes
`hr.people.blind-index/canonical-request/v1\0` (no length prefix), followed by
the following nine components in this order: `u16be(1)` purpose tag;
`u32be(length) || repository_id` (at most 256 opaque bytes, no normalization);
`u32be(length) || tenant_id` (at most 8 KiB, its validated canonical ASCII
bytes unchanged); `u16be(operation_kind)`; `u32be(length) || idempotency_key`
(at most 8 KiB, its validated canonical ASCII bytes unchanged);
`u32be(request_schema_version)`; `u16be(canonical_request_format_version)`;
`u64be(key_generation)`; and `u32be(length) || CanonicalRequestV1` (at most
256 KiB, its already-valid exact bytes unchanged). There is no component count
because the count is permanently nine for this V1 preimage; variable component
lengths, integer widths, order, domains, bounds, and normalization are exactly
those stated here. Checked `u64` accounting rejects an overlong component or a
complete preimage over 288 KiB before any PRF call. The accepted L2i.0d PRF
authenticates these bytes without parsing or rewriting them.

V1 writers remain the only writers until a later accepted V2 decision records
a format barrier. An N+1 reader MUST read V1 plus its own V2; an N reader seeing
V2 returns `CanonicalFormatUnsupported` and never guesses equality. Before any
row lookup, `ReplayGenerationSetV1` supplies an authenticated, bounded matrix
of admitted canonical-format versions for each active/draining generation. One
typed semantic command is encoded once for every distinct format in that matrix,
then a candidate is derived for every `(generation, format)` matrix entry. Thus
stored format/version discovery is not circular: a V2 retry derives the V1
candidate that can locate a retained V1 row before the row selects the V1
reader for the in-memory comparison. The v1 matrix permits at most two sorted
distinct formats, at most two sorted entries per generation, at most two
generations, and therefore at most four derivations. A missing/duplicate/
unreadable format, a format absent from its generation's authenticated matrix,
or more than these bounds is a closed replay refusal. A V3 writer cannot enter
a cohort until every repository under the keyring has durably crossed the V1
compatibility-retirement barrier: no retained V1 canonical request or
generation-scoped V1 blind index remains, all epoch-eligible readers support
V2/V3, and the provider has issued the matching retirement receipt. Golden
vectors fix every byte above and prove transport-field reordering and
absent/default-equivalent optionals produce the same bytes, while every changed
semantic field produces different bytes.

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
| 9 | `blind_index_key_generation` | `u64be` |
| 10 | `blind_index` | fixed-width provider-authenticated bytes |
| 11 | `effect_count` | `u16be`; exactly four in v1 |
| 12 | `effects` | one length-delimited concatenation of exactly four effect entries |

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
prefix), `u16be(8)` component count, then exactly eight ascending-tag
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

The count, tags, component order, lengths, fixed widths, domains, bounds, and
normalization above are part of the V1 byte contract. Components cannot be
omitted, duplicated, reordered, padded, or normalized; fixed-width payloads
with any other length fail `CommitBindingFormatInvalid` before provider
authorization. Checked `u64` accounting rejects a complete preimage over 96
KiB. The provider treats the complete bytes as opaque. SQLite stores descriptor
format/version and binding with its receipt. N/N+1 reader and writer barriers
match `CanonicalRequestV1`; an unsupported descriptor keeps recovery and
readiness closed. Independent Cargo and Buck encoders share only typed semantic
inputs and fixed vector fixtures, never a preimage encoder: both assert exact
full-preimage bytes (including domain, count, tags, lengths, and payloads), not
only descriptor bytes. Their vectors cover every component, changed purpose,
N/N+1 dispatch, a reordered or omitted outer component, and every exact and
limit-plus-one component and total bound; deliberately divergent typed input or
encoder output must fail the independent-vector comparison.

The stored entry binds the blind-index bytes and generation, encrypted canonical
request and outcome bytes/version, canonical and descriptor format versions,
repository epoch, commit binding, authorization receipt, and commit generation.
Its blind-index row is located only by the V1 generation-scoped candidate index;
there is no stable cross-generation equality locator and no cleartext replay
key. Processing is:

1. Validate syntax, tenant binding, verified authorization, and overlay
   generation without mutation.
2. Evaluate the deterministic domain command.
3. Encode the typed semantic command in every distinct format admitted by the
   provider-authenticated, repository-epoch-bound `ReplayGenerationSetV1`.
   The exact `AcquireReplayGenerationSetV1` result contains the active generation
   and, only during normal drain, its immediately prior draining generation;
   sorted generation state, lease, rotation-fence id, authenticated per-
   generation format matrix, and opaque generation-scoped PRF authority are
   all in that one result. The provider returns neither a stable locator nor an
   arbitrary historical generation. A missing, revoked, malformed, stale, or
   oversized set is `ReplayGenerationSetUnavailable` and admits no mutation.
4. Before opening SQLite, derive one `CanonicalRequestReplayV1` candidate for
   every returned `(generation, format)` entry using the exact full preimage
   above and that entry's opaque PRF authority. The hard bound is two
   generations × two formats = four derivations and four fixed-width
   candidates; the complete matrix is derived before any candidate is selected.
   Begin one `BEGIN IMMEDIATE` SQLite writer transaction, validate repository
   epoch, lease, matrix digest, and rotation-fence id, and query only these
   candidates. The query reads at most five rows solely to distinguish zero,
   one, and multiple matches; it opens no row while multiple candidates match.
5. Zero matches may reserve a new entry only with the lease's active-generation
   candidate for the active writer format. Exactly one match must authenticate
   and open its envelope using recorded generation and associated data, select
   the request's candidate bytes for that row's recorded format, then constant-
   time compare complete canonical plaintext bytes in memory. Ciphertext bytes,
   including nonce/tag/randomized ciphertext, are never compared for equality
   and no plaintext is persisted. Equal bytes return the original outcome;
   unequal bytes return `IdempotencyConflict` without mutation. More than one
   candidate row is `ReplayCandidateCollision`; a matrix/candidate/row format
   divergence is `ReplayCandidateDivergence`; an unreadable source, generation/
   index mismatch, or lost source key is `ReplaySourceUnavailable`. Every one
   fails closed and releases the transaction without a new business effect. No
   code reads an entry to discover a generation before locating it.
6. For a zero-match reservation, seal every sensitive employee,
   lifecycle, request/outcome, and audit/outbox value through
   `hr-record-encryption-draft`, derive required blind indexes, and stage the
   authenticated envelopes plus the idempotency entry in the same transaction.
7. Enumerate that exact staged row set, encode `StagedWriteDescriptorV1`, derive
   one opaque commit binding over it, and call
   `authorize_commit` with tenant, the current repository-epoch lease, a stable
   commit-authorization id, key generation, blind-index identity, and that
   binding. Store the returned authorization receipt inside the same SQLite
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
and keyed blind indexes. Employee, person, manager, evidence, policy, pack,
lifecycle, canonical request, stored outcome, and audit/outbox payload values
are sensitive and must never be stored as cleartext. The
final `hr-record-encryption-draft` port exposes bounded `seal`, `open`,
`blind_index`, `commit_binding`, `acquire_repository_epoch`,
`acquire_replay_generation_set`,
`list_unresolved`, `authorize_commit`, and idempotent `resolve_commit`
plus `begin_normal_rotation`, `list_incomplete_rotations`, `bind_rekey_checkpoint`,
`verify_rekey_checkpoint`, and `authorize_zero_reference_revocation` operations
over HR-owned values. Its envelope contains an
algorithm identifier, provider/key reference, monotonically ordered key
generation, unique nonce, ciphertext, and authentication tag; provider or
cipher types do not cross the port. A blind index is a fixed-width keyed PRF
output scoped exactly to the detailed
`(repository, tenant, operation kind, idempotency key, schema version,
canonical-format version, key generation)` order plus the fixed
`CanonicalRequestReplayV1 = u16be(1)` purpose tag and
`hr.people.blind-index/canonical-request/v1\0` domain above, with the canonical-
request bytes as the PRF message. The exact full preimage is the nine-component
SPEC grammar, not an adapter-specific field label. Length-prefixing and domain separation are canonical; it
permits equality only inside that logical replay slot and never permits cross-
tenant, cross-operation, or cross-idempotency-key matching. The accepted
primitive, encoding, and width are decision-gated at L2i.0d.

`AcquireReplayGenerationSetV1` is the sole executable provider-truth edge for
replay. Its exact request is
`{ repository_id: OpaqueBytes<=256, repository_epoch: u64, replay_contract: u16be(1) }`;
its successful result is
`ReplayGenerationSetV1 { repository_id, repository_epoch, lease_id:
OpaqueBytes<=128, matrix_digest: FixedBytes, rotation_fence_id:
OpaqueBytes<=128, keyring_id: OpaqueBytes<=128, entries }`. `entries` is one
or two ascending, distinct `ReplayGenerationAuthorityV1` values:
`{ generation: u64, state: Active | Draining, canonical_formats: [u16; 1..=2],
replay_prf_authority: OpaqueBytes<=256 }`. The set has exactly one `Active`, at
most one `Draining`, no `Revoked`/`EmergencyDraining` entry, at most two distinct
format versions across all entries, and at most four `(generation, format)`
derivations. `blind_index` accepts only one returned `replay_prf_authority` for
one matrix entry and the already-fixed full preimage; it cannot derive a
candidate for a caller-selected generation or format. The closed error is
`ReplayGenerationSetError::{RepositoryEpochStale, RotationFenceStale,
LeaseStale, GenerationSetMalformed, GenerationSetTooLarge,
GenerationStateInvalid, FormatMatrixInvalid, NormalRotationBlocked,
ProviderUnavailable, ProviderCorrupt}`. The result is provider-authenticated
and repository/epoch/fence-bound; the SQLite repository calls this port before
lookup, validates it inside its writer transaction, and has no alternate cache,
database narration, or provider-internal import. The key-service adapter
implements this call against its accepted facade but has no repository runtime
edge.

`NormalRotationBlocked` carries
`{ source_generation: u64, reason: NormalRotationBlockReason }`, where
`NormalRotationBlockReason::{DrainingGenerationPresent, EmergencyDrainPresent,
IncompleteRekey, DurableReferencesRemain, UnresolvedAuthorization,
ZeroReferenceReceiptMissing, RetirementReceiptMissing, SourceUnavailable}`.
`ReplayCandidateCollision` carries the bounded matched-row count; a candidate
or format matrix exceeding four derivations/five row reads returns
`ReplayCandidateLimitExceeded`; `ReplayCandidateDivergence` carries only
generation/format identifiers and never ciphertext or plaintext. These are
closed typed outcomes, not diagnostic strings or retry-to-create hints.

`BeginNormalRotationV1` is the provider CAS operation:
`{ keyring_id: OpaqueBytes<=128, expected_active_generation: u64,
expected_rotation_fence_id: OpaqueBytes<=128 }`. It returns exactly
`NormalRotationResult::{Started { active_generation, draining_generation,
rotation_fence_id }, Blocked { source_generation, reason }}` or
`NormalRotationError::{RotationFenceStale, GenerationNotActive,
ProviderUnavailable, ProviderCorrupt}`. `Started` is possible only for the
one permitted `G -> G+1` transition; a G+2 attempt reports the typed `Blocked`
result and does not change the provider state. This operator, not repository
narration, owns global keyring transition truth.

`CommitBinding` is a fixed-width provider-authenticated value over the exact
eight-component outer preimage and canonical staged-write descriptor specified
above, domain-separated by tenant, repository, epoch, authorization id, schema,
and key generation. Neither it nor `CommitAuthorizationId` is an unkeyed request
digest: the authorization id is an opaque repository-unique transaction
identity, and neither value is a telemetry label or cross-slot equality token.

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
blind-index PRF, exact `AcquireReplayGenerationSetV1` request/result/error and
signature/lease validation, per-generation canonical-format matrix and
compatibility-retirement barrier, global normal-rotation CAS/refusal semantics,
generation and commit-authorization linearization semantics, bounded unresolved-
receipt enumeration, repository-epoch fencing, recovery/administrative
resolution, and removal path. L2i.0f must then prepare the exact
unique files, L2i.0g must freeze commit semantics into HR port/repository/
SQLite content, and L2i.0h must implement bounded repository rekey/recovery
before the selected adapter behavior or production composition is dispatchable. The
selected adapter is owner-local and remains
`app/hr/adapters/draft/record-encryption-key-service` /
`hr-record-encryption-key-service-draft` while its port is draft.

The HR error type is closed as
`CommitFenceError::{GenerationNotActive, AuthorizationDenied,
AuthorizationUnresolved, CommitBindingMismatch, RepositoryEpochStale,
ResolutionConflict, ProviderUnavailable, ProviderCorrupt}`. A successful
resolution is exactly `CommittedBeforeFence` or `AbortedBeforeCommit`; neither
is inferred from a timeout, disconnect, clock, or missing row. Pending-page
results are capped by the L2i.0d accepted item/byte limits and carry an opaque
continuation; duplicate pages and resolutions are idempotent, while a skipped,
reordered, oversized, or non-progressing continuation fails closed.

The provider state machine is
`Active -> Draining | EmergencyDraining -> Revoked`. `authorize_commit`,
`AcquireReplayGenerationSetV1`, and those transitions share one provider-side
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
`EmergencyDraining` generation. A normal `G+2` request while G is draining, has an
incomplete rekey, durable ciphertext/blind-index reference, unresolved earlier
authorization, missing zero-reference receipt, or missing provider retirement
receipt returns `NormalRotationBlocked` and changes nothing. The provider may
issue the G retirement receipt only after it verifies every registered
repository's terminal zero-reference receipt and its own unresolved count is
zero; only then may it CAS G to `Revoked` and admit a new normal rotation.
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
repository id/epoch, source and target generations, phase, the exclusive
`(logical_table_tag, opaque_row_identity)` cursor, page/item/byte/provider-call
counters, the consecutive-CAS counter for that cursor, and a provider-
authenticated checkpoint binding. Phases are
`Scanning -> ZeroReferenceObserved -> RevocationAuthorized -> Complete`; no
phase or cursor may move backward. On boot the writer first acquires the new
exclusive repository epoch, validates the checkpoint binding and provider
fence, and either resumes the exact next page or returns a typed refusal.

For one page, SQLite deterministically scans every sensitive table and every
generation-scoped blind-index column in `(logical_table_tag,
opaque_row_identity)` order for `source_generation`. Outside a transaction the
adapter opens each observed envelope under the source generation, validates
associated data, seals the identical semantic plaintext under the active target
with a new provider nonce, and recomputes every target-generation blind index
from the stored authenticated `CanonicalRequestV1` when applicable. It then
starts one `BEGIN IMMEDIATE` transaction and CAS-checks every row against row
identity, observed revision, source generation, envelope commitment, and old
blind-index set. The transaction atomically replaces ciphertext, generation,
blind indexes and revision for the entire page and advances the checkpoint. A
single mismatch aborts the whole page and leaves its prior checkpoint durable.
The reconciler retries that same cursor in stable row order; each conflict
durably increments the cursor's counter, and the fourth attempt returns
`RekeyCasRetryExhausted` until a new operator-admitted run, rather than skipping
or spinning.

Replay and rekey share that single SQLite writer order. A replay first obtains
its provider-authenticated matrix and derives every one-to-four candidate
indexes before `BEGIN IMMEDIATE`; inside that transaction it validates epoch,
lease, digest, and fence before either zero-match reservation or authenticated
open. A page rekey enters the same writer before it replaces an idempotency
row's source-generation indexes with target-generation indexes. If replay wins,
rekey observes its committed row/revision on the next scan; if rekey wins, the
already-derived matrix includes the target format/generation candidate and
locates the same row. The global no-overlap CAS returns at most active G+1 plus
draining G; G+2 is refused until G is zero-reference and revoked. A rotation
that wins before a new request invalidates a stale matrix rather than allowing
a reservation; a request that wins authorization before the transition has its
receipt resolved before the transition barrier. Consequently a retry immediately
before, during, or after page CAS either returns the original outcome or a typed
unavailable/corruption result, never a second employee, lifecycle, idempotency,
or outbox effect. Response loss and hard close repeat the complete matrix lookup
after exclusive-epoch recovery. If a draining source is lost before its row
rekeys, the provider cannot authenticate the matrix/lease, or format candidates
diverge from a located row, readiness withdraws and `ReplaySourceUnavailable`,
`ReplayGenerationSetUnavailable`, or `ReplayCandidateDivergence` is returned;
a new reservation is forbidden. Source revocation occurs only after terminal
count includes every source candidate-index column at zero, so post-revoke
retry locates the rekeyed target row without a cross-generation token.

After an end-of-keyspace page, one SQLite transaction repeats the complete
source-generation reference count across ciphertext and blind-index columns.
Because the provider drain fence forbids new source-generation seals and commit
authorizations, a zero count is monotonic under the fenced repository epoch.
That transaction writes a provider-authenticated `ZeroReferenceReceipt` bound
to the terminal checkpoint and repository sequence. The provider may CAS the
source to `Revoked` only when that exact receipt verifies and its own earlier
authorization count is zero. A crash after provider revocation but before the
SQLite completion row is recovered by matching the immutable provider receipt;
a mismatch remains corruption. `ReferencesRemain` never advances the phase.

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
ReplayFormatMatrixInvalid, ReplayCandidateLimitExceeded, ReplayCandidateCollision,
ReplayCandidateDivergence, ReplaySourceUnavailable, ReplayLeaseStale,
RotationFenceStale, NormalRotationBlocked}`. A collision, stale lease/fence,
source loss, malformed/oversized set, incompatible format matrix, provider loss,
or source-generation/index mismatch never falls through to zero-match creation.
Contract and real-file recovery evidence must independently exercise Cargo and
Buck full-preimage vectors; every one-to-four candidate matrix; V1 write then
V2 retry after response loss, rotation/page CAS, hard close, rekey, and fresh
restart; changed-purpose conflict; encrypted-canonical plaintext equality under
different nonce/generation; ciphertext/tag/associated-data tampering; attempted
G+2 during G drain; emergency drain/source loss; stale matrix/fence; and replay
versus rekey immediately before/during/after page CAS and zero-reference
revocation. Each schedule proves the same semantic retry returns its original
outcome or a closed typed refusal and no second effect commits.

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
rollback means an N/N+1
reader that preserves and resumes the job; it may not delete the checkpoint,
downgrade either preimage format, reactivate a generation with target
references, or undo revocation. Normal rotation starts only after every epoch-
eligible process reads the additive schema and current preimage formats.

Contract evidence injects unique sentinels into every sensitive field, commits
to a real SQLite file, checkpoints and copies the backup, and proves neither
artifact contains a sentinel. It hard-closes every encryption, SQL, commit,
rotation-CAS, and reply boundary; reopens with a fresh process and provider
client; verifies authenticated reads and idempotent replay; detects ciphertext,
tag, nonce, associated-data, and blind-index tampering; rotates under concurrent
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
| conflict | employee version mismatch, duplicate identity, idempotency blind-index mismatch, commit-resolution conflict, bounded rekey page CAS conflict | none |
| unavailable | SQLite busy/full/unopenable, audit/key/runtime-context precondition unavailable, commit authorization unresolved, selected adapter unhealthy, key generation draining/revoked, source/target key outage, rekey retry exhaustion | none acknowledged |
| internal/corrupt | schema/preimage incompatibility, corrupt stored outcome, invalid canonical request or staged descriptor, stale/corrupt rekey cursor/checkpoint/fence, ciphertext/tag/nonce/generation/commit-binding mismatch, impossible state | fail closed; readiness false |

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
- same-key/same-canonical-request replay returns byte-equivalent semantic outcome and does
  not duplicate employee, lifecycle, audit/outbox, workflow, or payroll intent;
- same-key/different-canonical-request replay returns conflict without mutation;
- domain, authorization, overlay, encryption/key, and adapter failures preserve
  no partial state, plaintext persistence, or sensitive disclosure;
- schema N/N+1 open, migrate, reopen, and supported rollback boundaries are
  explicit.
- canonical-request and staged-descriptor N/N+1 bytes, exact bounds, semantic
  default/reordering equivalence, changed-field conflict, and rejection of
  unknown, omitted, duplicated, or reordered same-version fields are explicit;
- bounded rekey pages atomically replace envelope plus all blind indexes,
  advance no checkpoint on CAS failure, resume from the last committed cursor,
  and produce a zero-reference receipt before normal revocation.

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

Rekey fault injection pauses before/after incomplete-rotation discovery, job
creation, page scan, source open, target seal, blind-index derivation, page CAS,
checkpoint commit, terminal reference count, provider revoke, and local
completion. It then hard-closes all connections, constructs a new repository
and provider client, fences the older epoch, and resumes. Exact/limit-plus-one
items, bytes, pages, calls, cursor and checkpoint sizes; stale epochs/fences;
three conflicts plus the refused fourth attempt; full/busy media; source/target
key and provider loss; and nonzero ciphertext or blind-index references all
produce the closed result/error vocabulary without skipping a row, advancing a
failed checkpoint, or reporting `Revoked`.

</conformance_and_faults>

<observability>

## Signals and SLO qualification

The facade measures admitted request latency, result class, policy/overlay
generation age, selected adapter, transaction phase/latency, replay/conflict,
outbox lag/redelivery, reopen/migration result, rekey phase/checkpoint age,
bounded rows/bytes/pages/provider calls, CAS conflicts, remaining-generation
references, typed rekey refusal, and saturation. Cardinality is
bounded; employee, person, evidence, and idempotency values are not labels.

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
required pre-ack audit path cannot satisfy the request class. It is also false
for a first cohort with an incomplete rekey job and for a routed cohort whose
normal rekey checkpoint exceeds the PRD 60-second objective, exhausts its CAS
budget, or loses either generation/provider/repository. Health does not
claim durable, network, or downstream capability merely because pure domain
tests pass. The PRD SLO remains unqualified until these signals and the declared
load envelope are exercised in promotion evidence.

</observability>
