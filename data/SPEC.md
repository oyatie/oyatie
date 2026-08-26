---
doc_class: Owner-SPEC
owner: data
status: Target
date: 2026-08-26
---

# Data behavior and contract

<maturity>

The behavior below is the normative destination. Current PostgreSQL/SQLx,
in-memory OLAP, ClickHouse scaffolding, and analytics boot code do not satisfy
it. A stage becomes available only when its implementation, fault evidence,
SLO signals, and independent promotion review land.

Transfer compatibility does not weaken owner boundaries: shared ontology
types are defined by the agreed Foundry port, and outbox contract/domain,
PostgreSQL command construction, SQLx drain, and Gateway-envelope conversion
remain separate Bus faces. Compatibility aliases may preserve caller spelling
but never resolve to another application's core. An app compatibility alias
cannot resolve to any cloud Rust core, port, or implementation adapter,
including Data, Gateway, or Bus. The app defines its own portable contract; one
app-owned adapter consumes only the generated client of the sold
Connect/protobuf facade, while a separate app-owned commodity adapter
implements the same port. Every Data adapter identifies and implements a
matching Data port; no orphan adapter becomes a contract by convention. Agreed
ports contain no mutable store, recording executor, transport normalization,
or backend behavior.

</maturity>

<operational_identity>

## Semantic compatibility-residue states

Until the corresponding adapter or listener is implemented, the existing
compatibility surfaces continue to fail closed but expose semantic identities:

| Surface | Required semantic identity |
|---|---|
| ClickHouse adapter operations | `clickhouse_adapter_unavailable` plus the attempted operation |
| Data export usecase | `data_export_unavailable` with object-storage and change-stream adapter absence as detail |
| Tenant quota reconciliation | `tenant_quota_reconciliation_unavailable` with the missing policy adapter as detail |
| Tenant-bootstrap process boot | `analytics_tenant_bootstrap_unrouted` with tenant-lifecycle source and ClickHouse reconciliation absence as detail; write the exact bounded status to stderr, write nothing to stdout, exit 78, open no listener, and publish no readiness |
| Analytics process boot | `analytics_listener_unrouted` with authenticated Connect-listener absence as detail; exit nonzero and publish no readiness |

The identity and detail are bounded static strings: no request, tenant,
credential, endpoint, or adapter payload is interpolated. Compatibility error
variants remain unchanged in D1b-N; only their operator-facing content changes.
Tests assert the complete semantic identity, prove the old `IP-*` tokens are
not emitted, and retain decision references only in comments, rustdoc, and
package metadata. The rename neither installs an adapter/listener nor changes
the current no-production-readiness claim. Both binaries' current exit-zero
“boot complete” behavior is not a contract: D1b-N replaces it with their
distinct fail-closed exits so neither an unrouted listener nor an unrouted
tenant bootstrap can signal success. The tenant-bootstrap process emits
exactly
`analytics-tenant-bootstrap-app: analytics_tenant_bootstrap_unrouted (tenant lifecycle event source and ClickHouse reconciliation are unavailable)\n`
on stderr and exits 78; its process test proves that endpoint, user, password,
tenant, request, and environment values cannot appear in either output stream.

</operational_identity>

<topology>

## Runtime roles

```text
global tenant/table directory
              |
              v
         bounded cell
  query/transaction compute pool
        |       cached tablet map
        v               |
  metadata and placement quorum
        |               |
        +---- tablet consensus groups ----+
                         |                 |
                  durable data nodes   OLAP projections
                         |                 |
                   repair/compact     pipeline workers
```

- The global directory resolves a tenant/database to a home cell and carries a
  monotonic ownership generation. It is not in the normal transaction path.
- Compute parses contracts, plans queries, enforces deadlines/admission, and
  coordinates transactions; it owns no durable record authority.
- Metadata authority publishes schemas, ranges, tablet replicas, placement
  epochs, drain state, and fencing through consensus-backed revisions.
- A tablet is a bounded range with three or five consensus replicas over MVCC
  state and a durable local record engine.
- OLAP and pipeline roles consume committed change ordinals and publish derived
  generations; they do not accept source-record authority.
- Gossip may distribute suspicion, latency, utilization, and cache hints. It
  cannot add a voter, assign a tablet, advance an epoch, or fence an owner.

</topology>

<identity_and_partitioning>

## Record identity

A logical record is addressed by at least:

```text
tenant_id
database_id
table_id
primary_key
schema_revision
```

Committed versions carry:

```text
commit_ordinal
transaction_id
request_fingerprint
tablet_id
tablet_epoch
value or tombstone
classification and encryption-key reference
audit binding
```

The commit ordinal is assigned by the authoritative engine and orders versions
inside its defined transaction domain. Wall time is observational metadata and
never record identity.

## Tablet maps

- Initial placement is deterministic from a consensus-published map containing
  failure-domain topology, device class, weight, capacity, and ownership epoch.
- Range boundaries preserve ordered scans. Hash routing may select a database
  or placement group but cannot replace the ordered tablet index.
- A split or move copies a snapshot, streams later log entries, verifies state,
  publishes a higher epoch atomically, and only then makes the prior owner
  reclaimable.
- A request bearing a stale directory, tablet, schema, or ownership revision is
  rejected with a typed retryable error before mutation.

</identity_and_partitioning>

<transaction_protocol>

## Commit states

```text
NEW -> PREPARED -> COMMITTED
  \       |
   +------v
       ABORTED
```

1. The facade authenticates, validates canonical context, derives the v1
   request fingerprint, obtains a PDP decision bound to that digest, and admits
   the request against tenant and cell budgets.
2. Compute resolves the home cell and tablet from cached versioned maps.
3. The tablet validates tenant, schema, idempotency fingerprint, read set,
   write set, and current epoch before `PREPARED` can exist.
4. The consensus group replicates the transaction record and durable engine
   barrier required by the selected profile.
5. One authoritative transition assigns the commit ordinal and makes the write
   visible. Replies are sent only after the commit and any required pre-ACK
   audit receipt are durable.
6. Retrying the same idempotency key and fingerprint returns the same outcome;
   reusing the key with another fingerprint fails.

Reads select one committed MVCC revision and never expose `PREPARED` state.
Single-tablet transactions are the first executable stage. Until a cross-tablet
protocol is promoted, a request spanning tablets fails before preparation. The
target multi-tablet protocol is serializable within one cell and must define
coordinator recovery, participant fencing, deadlock/timeout behavior, and
atomic visibility. Transactions do not span cells.

</transaction_protocol>

<request_authority>

## Canonical request fingerprint v1

The caller never supplies an authoritative fingerprint. After transport
authentication and canonical identifier validation, Data derives it before
PDP evaluation; the PDP request and durable Audit receipt bind the derived
digest. The byte grammar is independent of protobuf so alternate protobuf
encoders, unknown fields, field order, and compatibility facades cannot alter
idempotency.

The fingerprint is SHA-256 over this exact frame (all integers unsigned big
endian):

```text
domain_len:u8 = 30
domain:[u8;30] = "oyatie.data.records.request.v1"
separator:u8 = 0
version:u8 = 1
operation:u8
field_count:u16
repeated field_count times:
  tag:u8 | type:u8 | length:u32 | value:[u8;length]
```

Fields occur once in strictly increasing tag order; missing required,
duplicate, out-of-order, unknown-tag, unknown-type, wrong-width, trailing, or
overlong input is invalid rather than normalized. Types are `ASCII=1`,
`U64=2`, `DIGEST32=3`, and `ENUM8=4`; `U64` is exactly eight bytes and
`DIGEST32` exactly 32. ASCII identifiers are 1..256 bytes from `0x21..0x7e`,
contain no separator or whitespace, and are compared byte-for-byte; alternate
Unicode normalization is neither accepted nor collapsed. The idempotency key
uses the same grammar with a 128-byte maximum. Primary/range keys and values
remain opaque bytes and are never Unicode-normalized.

| Tag | Field |
|---:|---|
| `0x01` | verified tenant ID (`ASCII`) |
| `0x02` | verified principal ID (`ASCII`) |
| `0x03` | idempotency key (`ASCII`) |
| `0x04` | database ID (`ASCII`) |
| `0x05` | table ID (`ASCII`) |
| `0x06` | schema revision (`U64`) |
| `0x07` | home-cell ID (`ASCII`) |
| `0x08` | tablet ID (`ASCII`) |
| `0x09` | ownership epoch (`U64`) |
| `0x10` | durability code (`ENUM8`: local oracle `1`, three-voter durable `2`, five-voter durable `3`) |
| `0x11` | ordered read-set digest (`DIGEST32`) |
| `0x12` | ordered write-set digest (`DIGEST32`) |
| `0x13` | ordered condition-set digest (`DIGEST32`) |
| `0x14` | range-start digest (`DIGEST32`) |
| `0x15` | range-end digest (`DIGEST32`) |
| `0x16` | direction (`ENUM8`: ascending `1`, descending `2`) |
| `0x17` | requested page-record maximum (`U64`) |
| `0x18` | requested page-byte maximum (`U64`) |
| `0x19` | authenticated continuation-state digest (`DIGEST32`) |
| `0x1a` | snapshot commit ordinal (`U64`) |

Operation codes and exact tag sequences are: transaction `1` =
`01..09,10,11,12,13`; point read `2` = `01..09,11`; first scan `3` =
`01..09,14,15,16,17,18`; resumed scan `4` =
`01..09,14,15,16,17,18,19,1a`. A scan's absent range endpoint digest is
`SHA-256(0x00)`; a present endpoint digest is
`SHA-256(0x01 || length:u32 || opaque_bytes)`. Requested page maxima must be
nonzero and at or below both public hard maxima before fingerprinting.

Set digests are SHA-256 over this complete, byte-exact frame. Every integer is
unsigned big-endian; concatenation means no implicit delimiter, varint, or
padding.

```text
domain_len:u8 = 33
domain:[u8;33] = "oyatie.data.records.operations.v1"
separator:u8 = 0
version:u8 = 1
set_kind:u8                         # read=1, write=2, condition=3
count:u32
repeated count times, in caller transaction order:
  ordinal:u32 | item_kind:u8 | key_length:u32 | key:[u8;key_length] | variant
```

`count` is the number of items supplied for that set and is at most the
transaction-wide remaining `MAX_TRANSACTION_OPERATIONS`; its first item has
`ordinal=0`, and each later
item has the preceding ordinal plus one. `item_kind` is `1` for read, `2` for
write, and `3` for condition and MUST equal `set_kind`; a typed collection that
mixes kinds, has a missing/repeated/skipped ordinal, or has a count/order
disagreement is rejected before hashing. `key_length` is `1..=MAX_PRIMARY_KEY_BYTES`;
`key` is opaque bytes, is neither decoded nor normalized, and its stated length
must exactly match the following bytes.

The `variant` is exactly one of the following; all widths are fixed and no
variant has an omitted field:

| Item | Exact variant bytes | Required values |
|---|---|---|
| read | `expected_version_present:u8 | expected_version:u64` | presence is `0` (absent) or `1` (present); absent encodes `expected_version=0`, present encodes the exact version. |
| write | `value_kind:u8 | value_length:u32 | value_digest:[u8;32]` | kind is `1` value or `2` tombstone. Value uses `1..=MAX_RECORD_VALUE_BYTES` exact source bytes and their SHA-256. Tombstone encodes length `0` and SHA-256 of empty bytes. |
| condition | `predicate:u8 | operand_length:u32 | operand_digest:[u8;32]` | predicate is `1` absent, `2` present, `3` version-equal, or `4` value-digest-equal. Absent/present encode length `0` and SHA-256 of empty bytes; version-equal encodes length `8` and SHA-256 of its unsigned-big-endian `u64`; value-digest-equal encodes length `32` and SHA-256 of its exact 32-byte digest. |

Before hashing, checked arithmetic verifies every length, collection count, and
total preimage is no more than `MAX_REQUEST_FINGERPRINT_FRAME_BYTES`; unknown
set/item/presence/value/predicate codes, wrong typed operand width, a
tombstone with a value, trailing bytes, or an alternate Unicode/byte
normalization fails. Independent implementations may share input fixtures and
constants but MUST use separate framing/accumulation code. Golden vectors cover
each variant, all zero/empty forms, permutation and normalization refusal,
truncation/unknown code, exact limits, plus-one limits, the `4,241,449`-byte
worst-case set preimage and its one-byte overflow, and N/N+1 count/order. The
independent encoders also inject `u64::MAX` into every length/count accumulator
and prove the typed overflow refusal occurs before allocation or digest output.

## Authenticated scan continuation v1

The public continuation is raw opaque bytes, at most 6,022 bytes. Its outer frame is:

```text
domain_len:u8 = 33
domain:[u8;33] = "oyatie.data.records.scan-token.v1"
separator:u8 = 0
version:u8 = 1
algorithm:u8 = 1                 # AES-256-GCM
key_generation:u64
nonce:[u8;12]                    # nonce_lease_id:u32 || checked counter:u64
ciphertext_length:u32
ciphertext:[u8;ciphertext_length]
tag:[u8;16]
```

AAD is the complete header through `ciphertext_length` (exactly 61 bytes);
unknown version/algorithm, a ciphertext length above 5,945, truncation, or trailing bytes
fails before an AEAD operation. The plaintext has its own, independently
framed grammar--it is not a bare reuse of the request frame:

```text
domain_len:u8 = 33
domain:[u8;33] = "oyatie.data.records.scan-state.v1"
separator:u8 = 0
version:u8 = 1
field_count:u16 = 19
repeated exactly 19 times:
  tag:u8 | type:u8 | length:u32 | value:[u8;length]
```

Tags occur once in the stated increasing order. Types are `ASCII=1`, `U64=2`,
`DIGEST32=3`, `ENUM8=4`, `BYTES=5`, and `I64=6`; `U64` is exactly eight
unsigned-big-endian bytes, `DIGEST32` exactly 32 bytes, `ENUM8` exactly one
byte, and `I64` exactly eight two's-complement-big-endian bytes. ASCII is
`1..=256` bytes from `0x21..0x7e` with no separator or whitespace; BYTES is
`0..=MAX_PRIMARY_KEY_BYTES` opaque bytes. No Unicode normalization is accepted.
An unknown/duplicate/out-of-order tag or type, wrong width, missing field,
noncanonical length, or trailing byte fails; no field is ignored or defaulted.
The 19 fields are exactly: `0x01` tenant `ASCII`, `0x02` principal
`ASCII`, `0x03` database `ASCII`, `0x04` table `ASCII`, `0x05` schema revision
`U64`, `0x06` home cell `ASCII`, `0x07` tablet `ASCII`, `0x08` ownership epoch
`U64`, `0x09` snapshot commit ordinal `U64`, `0x0a` original request
fingerprint `DIGEST32`, `0x0b` range-start digest `DIGEST32`, `0x0c` range-end
digest `DIGEST32`, `0x0d` direction `ENUM8`, `0x0e` page-record maximum `U64`,
`0x0f` page-byte maximum `U64`, `0x10` last emitted key `BYTES`, `0x11`
`issued_earliest_ms` `I64`, `0x12` `issued_latest_ms` `I64`, and `0x13`
`expires_at_ms` `I64`. The interval
must satisfy `issued_earliest <= issued_latest <= expires_at`. TTL is at most
15 minutes from `issued_latest_ms`; acceptance requires
`current.earliest >= issued_earliest` and `current.latest <= expires_at`.
Direction is `1` ascending or `2` descending; page-record and page-byte maxima
are nonzero and no greater than `MAX_SCAN_PAGE_RECORDS` and
`MAX_SCAN_PAGE_BYTES` respectively. Range digests use the same present/absent
encoding as the canonical request fingerprint. A first-page `last emitted key`
is the zero-length BYTES value; a resumed page has its exact prior opaque key.
Boundary-straddling or widened intervals return `TimeUncertain`. The token
supplies no new tenant, principal, limit, map, schema, or snapshot authority.

The continuation bounds are derived rather than rounded. The maximum plaintext
is exactly `38` fixed header bytes + `19 * 6` field prefixes + `6 * 256`
identifier bytes + `5 * 8` U64 bytes + `3 * 32` digest bytes + `1` direction
byte + `4,096` last-key bytes + `3 * 8` I64 bytes = `5,945` bytes. AES-GCM
does not expand ciphertext, and the outer header/tag adds `61 + 16 = 77`
bytes, so `MAX_SCAN_CONTINUATION_TOKEN_BYTES = 5,945 + 77 = 6,022`. All
summands use checked `u64` arithmetic before a `usize` conversion; each
summand's exact maximum, its plus-one input, and overflow are separate golden
vectors. A valid 4-KiB primary key is therefore representable in a resumed
continuation, not a nominally legal request that cannot be continued.

Keys have purpose `ScanContinuation` and states `EncryptActive`,
`DecryptOnly`, `RetirePending`, or `Revoked`. Exactly one generation is
encrypt-active; prior decrypt-only generations survive at least maximum TTL
plus the bounded request drain. Retirement first rejects new decrypt leases,
waits for issued leases to drain, and then is fenced by a durable Audit receipt.
Secrets durably reserves a unique `NonceLeaseId` range per generation before
returning it. Data durably creates the exclusive checkpoint before first use
and advances `next` with the linearizable CAS/fsync transition before
submitting that nonce to AEAD; on any uncertain recovery it burns the entire
range and withdraws readiness.
Unknown or revoked generations,
authentication failure, malformed/cross-context input, and foreign replay all
map publicly to `ScanContinuationError::Invalid`; expiry, uncertain time,
unavailable snapshot, stale epoch, key-service outage, and crypto outage map
respectively to `Expired`, `TimeUncertain`, `SnapshotUnavailable`,
`StaleEpoch`, `KeyServiceUnavailable`, and `CryptoUnavailable`. No error
reveals whether another tenant, principal, key generation, or snapshot exists.
Replaying an intact token is read-only and deterministically replays the same
snapshot/page; it cannot mutate or advance authority and remains subject to
ordinary admission.

Independent plaintext/frame implementations require golden/KAT vectors,
permutation, exact/plus-one plaintext and outer-token boundaries,
truncation/unknown-tag/type refusal, header/AAD/ciphertext/tag tamper,
cross-tenant/principal/query/tablet/snapshot replay, expiry-boundary,
interval-widening, active-to-decrypt-only rollover, revoked/unknown generation,
N/N+1 nonce/count vectors, concurrent allocation/duplicate refusal, nonce
exhaustion, every crash/restore point, and repeated-token campaigns.

</request_authority>

<wire_contract>

## Canonical protobuf/Connect mapping

The public package is `data.records.v1` at
`data/facade/proto/data/records/v1/records.proto`. It is non-executable until
D1c-WG records an accepted generator/runtime and D1c-WS establishes the schema,
descriptor, fixed `OUT_DIR` products, Cargo/Buck generator inputs, and parity
canaries. D1c-C freezes only engine-neutral semantic values; D1c-WC owns
protobuf decode, normal-form validation, size-only encoding, streaming
encoding, and their tests. D4 consumes this frozen face and does not invent the
first schema.

Records v1 forbids maps, groups, extensions, and recursive messages. Tags are
never reused; request unknown fields, duplicate singular fields, non-minimal
varints, out-of-tag-order fields, and trailing bytes fail canonical validation.
Repeated-field order is semantic. The normal-form encoder emits known fields
in ascending tag order, elides only schema-declared absent optionals, and
rejects rather than preserves unknowns. Frame bytes mean the complete
uncompressed length-delimited protobuf message; optional transport compression
does not change any limit. The independent size-only and bounded streaming
encoders may share generated message/descriptor types and constants, but no
accumulator or traversal implementation. Their exact-byte equality is a
promotion invariant in both Cargo and Buck.

</wire_contract>

<time_contract>

## Cell interval consumption

- Data calls the agreed Cell `Now() -> Interval { earliest, latest }` port for
  lease, expiry, policy, and externally observed timestamp fields.
- Versionstamps and snapshot identities remain commit ordinals even when a
  tighter PTP or GNSS-backed interval becomes available.
- `commit_wait` is an adapter selected by cell IR. The NTP v1 profile keeps it
  off. Enabling it requires a measured epsilon SLO and a versioned cell change;
  an in-flight interval retains the bound with which it was issued.
- Clock rollback, widening, or uncertainty can cause a typed retry/refusal but
  cannot reorder committed ordinals or permit two owners.

</time_contract>

<durable_engine>

## Persistence and authority

The durable engine uses encrypted append-only transaction/log records,
checksummed immutable runs or segments, generation-bound manifests,
compaction, snapshots, and explicit durable barriers. Encryption and canonical
AAD validation occur through the Data record-protection/key ports before and
after local persistence; an embedded LSM may implement local persistence, but
it supplies neither distributed ownership, consensus, nor key authority.

A local write is not acknowledged merely because it reached userspace or the
operating-system page cache. Recovery validates framing, ciphertext envelope,
AAD/tag, key generation, checksums, generation monotonicity, schema
compatibility, and manifest reachability before serving. Recovery consumes
ciphertext plus key references and never persists reconstructed plaintext or a
raw key. Ambiguous, undecryptable, revoked-key, or corrupt state is quarantined
rather than guessed, and readiness remains withdrawn.

The hot-path persistence boundary remains a D1c cross-owner decision. Until it
is accepted, implementations may define an owner-local draft abstraction but
must not depend on Storage's draft provider port or claim a stable shared
contract. Backup/snapshot export may use the sold Storage facade through an
agreed Data adapter without making object storage the transaction authority.

</durable_engine>

<olap_and_pipelines>

## Derived generations

A committed change envelope contains at least:

```text
tenant, table, primary key
source tablet and commit ordinal
schema revision
operation and value/tombstone checksum
transaction and idempotency identity
```

- Projection checkpoints bind an inclusive source-ordinal vector, schema
  revision, output generation, manifest checksum, and predecessor.
- Consumers apply duplicate envelopes idempotently. A missing predecessor,
  ordinal gap, incompatible schema, or checksum mismatch stops publication and
  exposes a repairable error.
- Columnar segments and pipeline outputs are immutable. Publication is one
  compare-and-swap from the prior generation after every output is durable and
  verified; cleanup failure leaks space instead of removing the visible
  generation.
- Backfill uses a bounded snapshot plus subsequent change replay and converges
  at an explicit barrier before activation.

</olap_and_pipelines>

<security_and_isolation>

## Provider boundary and production authority

Data owns five implementation-free use-case ports:

```text
PolicyClient        authorize(canonical request context) -> PolicyReceipt
AuditSink           append_pre_ack(canonical event) -> DurableAuditReceipt;
                    append/get challenge-bound publication high-water anchors
RecordKeySource     reserve/rotate/retire opaque key uses and nonce leases -> KeyUseLease;
                    bootstrap-reacquire an authorized decrypt operation -> ReacquiredOpenLease
RecordProtection    digest/seal/open using RecordKeySource operation handles, explicit purpose,
                    lease identity, and envelope -> result
ArtifactPublicationCoordinator
                    cell-local durable publication tuple, pin/member, receipt-history,
                    safe-GC, reconciliation, and read-snapshot operations -> result
```

The first four draft homes are respectively `data/ports/draft/policy-client`,
`data/ports/draft/audit-sink`, `data/ports/draft/record-keys`, and
`data/ports/draft/record-protection`. The fifth is the Data-owned, cell-local
`data/ports/draft/artifact-publication` port: its core is
`data/core/artifact-publication-domain`, its one durable implementation is
`data/adapters/draft/artifact-publication-cell`, and
`data/facade/records-app` is the sole production composition owner. It is not
a sold cross-owner provider, object-store adapter, or in-memory authority.
Data provider adapters for the first four ports are
`data/adapters/draft/policy-client-policy`,
`data/adapters/draft/audit-sink-audit`,
`data/adapters/draft/record-keys-secrets`, and
`data/adapters/draft/record-protection-secrets`. The provider adapters cannot
implement content until their provider owners accept the exact sold faces reserved
by D1c-KG: `policy/ports/check` (`policy-check`),
`audit/ports/emission` (`audit-emission`, including the publication high-water
authority), and
`secrets/ports/kms-use` (`secrets-kms-use`), or amend this SPEC with one exact
replacement each. The current `iam/ports/policy-cedar-api`,
`audit/ports/emission-api`, `secrets/ports/kms`, and
`secrets/ports/kms-api` do not satisfy this edge: they expose implementation or
core-backed/internal shapes, and Data MUST NOT depend on them or any provider
`core/**` package.

`secrets-kms-use` is the only planned Data key/AEAD boundary. Before its Data
adapters land, the provider owner must accept these v1 operations and their
typed Cargo/Buck faces: `ReserveNonceRange(tenant, purpose, requested_count,
minimum_generation, request_fence) -> KeyUseLease`,
`AcquireOpenHandle(tenant, purpose, KeyBootstrapLocatorV1, request_fence,
recovery_authorization) -> ReacquiredOpenLease`,
`Seal(OpaqueSealHandle, tenant, purpose, generation, fence_sequence,
nonce_lease_id, counter, envelope_associated_data, plaintext) ->
ciphertext_and_tag`, and `Open(OpaqueOpenHandle, tenant, purpose,
KeyGenerationBinding, envelope) -> plaintext`. `AcquireOpenHandle` is owned
only by `RecordKeySource` and the KK adapter: it parses no plaintext, verifies
the bootstrap locator against the provider catalog and current authorization,
and returns `ReacquiredOpenLease { binding, opaque_open_handle }`. The KX
adapter consumes that typed lease and maps only `Seal` and `Open`; it neither
reacquires a handle nor interprets a provider locator. Thus there is one
acquisition authority and one `data-record-protection-draft ->
data-record-keys-draft` type edge, mirrored in Cargo and Buck.

`KeyBootstrapLocatorV1` is the bounded, pre-decryption recovery reference
carried in every durable envelope header. Its exact canonical grammar is:

```text
domain_len:u8 = 35
domain:[u8;35] = "oyatie.data.record.key-bootstrap.v1"
separator:u8 = 0 | version:u8 = 1 | field_count:u16 = 10
repeated exactly field_count times in increasing-tag order:
  tag:u8 | type:u8 | length:u32 | value:[u8;length]
where each listed tag occurs exactly once:
  01 tenant:ASCII(1..=256)                 06 provider_contract_revision:U64
  02 purpose:ENUM8(1..=6)                  07 provider_catalog_id:DIGEST32
  03 canonical_key_id:ASCII(1..=256)       08 provider_generation_locator:BYTES(1..=512)
  04 key_generation:U64                    09 recovery_policy_digest:DIGEST32
  05 fence_sequence:U64                    0a provider_locator_authenticator:AUTH32
```

`BYTES=5` and `AUTH32=6` are permitted only in this bootstrap grammar.
`provider_locator_authenticator` is exactly
`HMAC-SHA-256(provider_catalog_authentication_key, canonical bytes through
field 09)`, generated and verified only by the provider catalog; no Data value
contains that authentication key. The header is
`1 + 35 + 1 + 1 + 2 = 40` bytes, ten field prefixes add `60`, and the largest
values add `256 + 1 + 256 + 8 + 8 + 8 + 32 + 512 + 32 + 32 = 1,145`; therefore
`MAX_KEY_BOOTSTRAP_LOCATOR_BYTES = 1,245`. The provider catalog authenticates
`provider_locator_authenticator`, resolves the opaque locator, and checks all
tenant/purpose/key/generation/fence/revision/catalog/policy fields before it
returns a handle. A count other than ten, unknown/duplicate/omitted/out-of-
order tag, wrong type/width, noncanonical ASCII, bad HMAC, or truncation/
trailing byte is `KeyBootstrapMalformed` or `KeyBootstrapIntegrityInvalid`;
Data checks this frame, its bound header/AAD context, and all lengths before
allocation, acquisition, Open, or publication, but never interprets the opaque
locator or authenticator.

`KeyGenerationBinding` remains encrypted Data recovery state containing the
provider-defined generation reference, its revision, and integrity; it is not
the sole restart reference and is never required to obtain the first post-crash
open handle. `KeyUseLease` contains a matching binding, bootstrap locator,
exactly one `NonceLeaseId:u32`, `start:u64`, `end_exclusive:u64`, `not_after`,
reservation integrity, and a non-serializable `OpaqueSealHandle`. An
`OpaqueSealHandle` or `OpaqueOpenHandle` has no byte accessor and cannot be
cloned, displayed, serialized, logged, persisted, or supplied to another
tenant/purpose/generation/fence. Data persists neither handle.

On restart, restore, or a `DecryptOnly` generation, Data first parses the
envelope and bootstrap locator without allocating ciphertext/plaintext, checks
that locator fields equal the envelope AAD/header context, obtains a fresh
`recovery_authorization`, and calls the RecordKeySource-owned
`AcquireOpenHandle`. The provider rechecks authorization, catalog integrity,
tenant, purpose, key ID, generation, fence, revision, generation state, and
revocation before returning the fresh operation-local binding/handle. Rotation
emits a new bootstrap locator for the new generation while old valid
DecryptOnly locators remain usable; restore requires the authenticated catalog;
revocation invalidates acquisition. Source loss, a malformed/corrupt/foreign
locator, unavailable catalog/provider, policy denial, revision/fence/context
mismatch, or revoked generation returns respectively
`KeyBootstrapMalformed`, `KeyBootstrapIntegrityInvalid`,
`KeyBootstrapContextMismatch`, `KeyBootstrapCatalogUnavailable`,
`KeyBootstrapAuthorizationDenied`, or `KeyGenerationUnavailable`, quarantines
the affected recovery, and withdraws readiness. Ciphertext alone never causes
a raw-key, stale-binding, or alternate-provider fallback.

The provider durably records the disjoint reservation before returning the
lease and rejects a Seal/Open whose handle, tenant, purpose, binding, bootstrap
locator, fence, nonce-lease identity, counter range, or key state does not
match. KMS owns raw-key storage, cryptographic key lifetime, and zeroization
evidence; Data's source/type and serialization scans prove that no raw-key-
shaped value exists in any Data contract, receipt, event, envelope, error,
log, or async state.

`PolicyReceipt` binds tenant, principal, operation, resource/range,
request fingerprint, issuer, audience, policy revision, decision, issued Cell
interval, expiry, and receipt integrity. Deny, absence, mismatch, stale
revision, expiry, or Policy outage fails before data-dependent work.
`DurableAuditReceipt` binds the same fingerprint plus policy receipt digest,
transaction/snapshot identity, result class, key generation where applicable,
and durable Audit sequence/digest. `PublicationHighWaterReceipt` is a separate
challenge-bound durable Audit receipt with the exact anchor/head/context fields
specified below; it is the only accepted external freshness witness for a
publication locator. A cell-local publication read snapshot is only a
validated, expiring derivative installed by the trusted coordinator; it is
never a caller-supplied or independent freshness authority. Mutations,
key-state transitions, tenant deletion, export, restore,
publication-anchor advancement, and policy-designated privileged disclosures do
not acknowledge or become visible without their required receipt; an Audit
outage aborts before commit/visibility. Contract fixtures and known-answer
vectors may test these value types, but no fake, in-memory double, or reference
oracle can satisfy production composition, route publication, or readiness.

## Ciphertext envelope and key lifecycle

Every durable value/tombstone payload, WAL/log body, segment block, snapshot,
repair copy, and migration artifact is sealed before provider I/O as
`CiphertextEnvelopeV1`:

```text
domain_len:u8 = 26
domain:[u8;26] = "oyatie.data.record.aead.v1"
separator:u8 = 0
format_version:u8 = 1
algorithm:u8 = 1                         # AES-256-GCM
purpose:u8                                # record=1, wal=2, segment=3,
                                          # snapshot=4, repair=5, migration=6
key_id_length:u16 | key_id:[u8;length]    # canonical ASCII, 1..256 bytes
key_generation:u64
nonce:[u8;12]                             # nonce_lease_id:u32 || counter:u64
bootstrap_locator_length:u16 | bootstrap_locator:[u8;length]
aad_length:u32 | aad:[u8;length]
ciphertext_length:u64 | ciphertext:[u8;length]
tag:[u8;16]
```

`bootstrap_locator` is exactly one `KeyBootstrapLocatorV1`, `1..=1,245` bytes.
The AEAD associated-data input is `EnvelopeAssociatedDataV1`: the exact bytes
from `domain_len` through and including `ciphertext_length`, with no tag or
ciphertext bytes. The plaintext length is known before Seal, so the length is
included in the authenticated input. `aad` within that input is the distinct
19-field `ContextAadV1` below. Thus an alteration of algorithm, purpose, key,
generation, nonce, bootstrap locator, context AAD, or ciphertext length fails
the provider AEAD verification; changing ciphertext or tag likewise fails.
Unsupported format, algorithm, purpose, key state, bootstrap length, AAD/context
mismatch, tag failure, truncation, or trailing bytes fails before plaintext
decode or publication.

`ContextAadV1` is byte-exact and distinct for every purpose. All integers are
unsigned big-endian and the only accepted field encodings in this grammar are
`ASCII=1`, `U64=2`, `DIGEST32=3`, and `ENUM8=4`.

```text
domain_len:u8 | domain:[u8;domain_len] | separator:u8 = 0 | version:u8 = 1
field_count:u16 = 19
repeated exactly field_count times, tags strictly increasing:
  tag:u8 | type:u8 | length:u32 | value:[u8;length]
```

The purpose/domain table is closed: record=`1` and
`"oyatie.data.record.aad.record.v1"` (32 bytes), wal=`2` and
`"oyatie.data.record.aad.wal.v1"` (29), segment=`3` and
`"oyatie.data.record.aad.segment.v1"` (33), snapshot=`4` and
`"oyatie.data.record.aad.snapshot.v1"` (34), repair=`5` and
`"oyatie.data.record.aad.repair.v1"` (32), migration=`6` and
`"oyatie.data.record.aad.migration.v1"` (35). The fields are exactly
`01` tenant ASCII, `02` database ASCII, `03` table ASCII, `04` tablet ASCII,
`05` ownership epoch U64, `06` schema revision U64, `07` classification code
ENUM8, `08` classification revision U64, `09` classification-binding digest
DIGEST32,
`0a` commit ordinal U64, `0b` transaction ID ASCII, `0c` artifact generation
U64, `0d` key ID ASCII, `0e` key generation U64, `0f` artifact role ENUM8,
`10` artifact chunk ordinal U64, `11` artifact chunk count U64, `12` artifact
total plaintext bytes U64, and `13` artifact-plan digest DIGEST32.
Tenant/database/table/tablet/key ID are canonical ASCII `1..=256`; transaction
ID is canonical ASCII `1..=256` when present and exactly zero bytes when empty;
fixed-width values have their stated width. The outer envelope purpose must
select exactly this AAD's purpose-domain row, and `key ID` and key generation
exactly match the envelope header. Every tag `01..13` occurs exactly once: a decoder
rejects a field count other than 19 before looking at a field, and rejects an
unknown, duplicate, omitted, out-of-order, wrong-type, wrong-width, trailing,
or purpose-inapplicable field before allocation, Open, or publication.

The classification ENUM8 conversion is closed and is not Rust declaration
order or a `repr` cast. KC implements this exact total match and refuses every
other DataClass byte: `Public=1`, `InternalOnly=2`, `PiiIdentifying=3`, `PiiSensitive=4`,
`Phi=5`, `Pci=6`, `PipaArticle23=7`, `Children=8`, `Financial=9`, `Usage=10`,
`Secret=11`, `Audit=12`, `PiiQuasiIdentifier=13`,
`FinancialRegulatedCredit=14`, `BehavioralTenantProduct=15`,
`BehavioralAds=16`, `DeclaredPreference=17`, `SearchQuery=18`, and
`SensitivePipaArticle23=19`. Compatibility aliases remain distinct source
variants and retain the listed byte; a future source variant requires a new
versioned AAD grammar rather than an unassigned codepoint. `WalTransactionMixed=0`
is a separate WAL-only control sentinel defined below; it is not a DataClass,
cannot be emitted by a classification port, and is invalid outside a mixed WAL
summary with revision zero.

The following table is the complete purpose-applicability rule. `real` means a
nonzero/current value, `zero` means the listed fixed-width all-zero value or a
zero-length transaction ID, and `phase` is the exact nonzero artifact control
defined immediately after the table. Nothing is omitted.

| tag | field | record | WAL | segment | snapshot | repair | migration |
|---|---|---|---|---|---|---|---|
| `01` | tenant | real | real | real | real | real | real |
| `02` | database | real | real | real | real | real | real |
| `03` | table | real | real | real | real | real | real |
| `04` | tablet | real | real | real | real | real | real |
| `05` | ownership epoch | real | real | real | real | real | real |
| `06` | schema revision | real | real | real | real | real | real |
| `07` | classification code | real | summary | derived | derived | derived | derived |
| `08` | classification revision | real | summary | derived | derived | derived | derived |
| `09` | classification-binding digest | record key | WAL summary | zero | zero | zero | zero |
| `0a` | commit ordinal | real | real | zero | zero | zero | zero |
| `0b` | transaction ID | real | real | zero | zero | zero | zero |
| `0c` | artifact generation | real | real | real | real | real | real |
| `0d` | key ID | real | real | real | real | real | real |
| `0e` | key generation | real | real | real | real | real | real |
| `0f` | artifact role | phase | phase | phase | phase | phase | phase |
| `10` | artifact chunk ordinal | phase | phase | phase | phase | phase | phase |
| `11` | artifact chunk count | phase | phase | phase | phase | phase | phase |
| `12` | artifact total plaintext bytes | phase | phase | phase | phase | phase | phase |
| `13` | artifact-plan digest | phase | phase | phase | phase | phase | phase |

For `derived`, every contributing record must have the same exact
`(classification_code, classification_revision)` pair. The aggregate uses that
pair; an empty or mixed pair is rejected as `AggregateClassificationMixed`
before plan construction, and mixed data must be partitioned into separate
artifacts. This is the aggregate-classification rule--there is no rank-based,
best-effort, or caller-chosen aggregate label.

`record key` is exactly `SHA-256(0x01 || primary_key_length:u32 ||
primary_key)` over the opaque canonical record primary-key bytes. `WAL summary`
is exactly `SHA-256(WalTransactionClassificationSummaryV1)` below. They are
purpose-specific meanings of the same fixed-width tag: a record never accepts a
WAL summary, a WAL never accepts a record-key binding, and every aggregate
purpose requires 32 zero bytes. The outer purpose/domain and the plan,
manifest, and commit equality rules below make a cross-purpose digest
substitution fail before Open or publication.

`WalTransactionClassificationSummaryV1` freezes one WAL artifact to the full
ordered mutation list of exactly one committed, single-tablet transaction. A
cross-tablet transaction remains unsupported and fails before `PREPARED`; it
does not split one atomic transaction into separately headed WAL artifacts. A
read-only transaction has no WAL artifact. A WAL summary with no durable
record, metadata, or control mutation is invalid rather than an empty default:

```text
domain_len:u8 = 39
domain:[u8;39] = "oyatie.data.record.wal-class-summary.v1"
separator:u8 = 0 | version:u8 = 1
transaction_id_length:u16 | transaction_id:ASCII(1..=256)
commit_ordinal:u64
entry_count:u16 = 1..=1,024
repeated exactly entry_count times in ascending entry_ordinal order:
  entry_ordinal:u32 | entry_kind:u8 | subject_digest:[u8;32] |
  classification_code:ENUM8(1..=19) |
  classification_revision:u64(1..=u64::MAX)
```

`entry_kind` is `1` record mutation, `2` metadata mutation, or `3` control
mutation. The canonical list is record mutations in the original ordered
write-set order, then metadata mutations by bytewise canonical metadata
identifier, then control mutations by bytewise canonical control identifier;
`entry_ordinal` is exactly `0..entry_count-1`. A record `subject_digest` is
`SHA-256(0x01 || primary_key_length:u32 || primary_key)`; metadata and control
subjects are respectively `SHA-256(0x02 || identifier_length:u32 ||
canonical_metadata_identifier)` and `SHA-256(0x03 || identifier_length:u32 ||
canonical_control_identifier)`. Each record uses its validated exact DataClass
pair. Each metadata/control item must carry an exact validated DataClass pair
from its authoritative schema/control definition; there is no inherited,
ranked, caller-chosen, or default classification. Unknown kinds, missing or
duplicate subject/ordinal, a noncanonical identifier, an alias spelling in
place of the canonical ENUM8 conversion, a zero/unknown pair, or a summary not
equal to the actual canonical WAL body is `WalClassificationMalformed` before
allocation, provider acquisition, Seal, persistence, or `PREPARED`.

Admission derives this expanded durable-mutation list before `PREPARED` and
refuses an expansion above `1,024` entries as `WalClassificationMalformed`;
one request operation cannot create a legal hidden 1,025th metadata or control
entry. The summary `transaction_id` and `commit_ordinal` are byte-for-byte the
same as tags `0b` and `0a` of every data, final-manifest, and commit envelope's
`ContextAadV1`, and as fields `1a` and `19` of the sealed commit below. Thus a
record artifact's tag `09` commits its one canonical primary key while a WAL
artifact commits both its exact transaction identity and all of that
transaction's classified mutations.

The summary's distinct pairs determine the WAL `summary` cells. If all entries
have one pair, tags `07`/`08` and the plan/manifest/commit carry that pair. If
there are two or more pairs, those fields are exactly
`WalTransactionMixed=0`/revision `0`; byte `0` is a WAL-only control sentinel,
not a DataClass and is forbidden for every record or aggregate purpose. In
both cases tag `09` and the plan/manifest/commit binding digest are the exact
summary digest. Thus the fixed AAD grammar stays at 19 fields while a mixed
transaction has one unique authenticated classification binding. A source
compatibility alias is converted to its listed canonical DataClass byte before
the summary is built; it cannot create another serialized class identity.

For `zero`, U64 is eight zero bytes, DIGEST32 is 32 zero bytes, and transaction
ID has length zero. For `phase`, the decoder first applies the purpose-specific
plan cardinality below, then requires one common checked plan digest. A record
or WAL data envelope has `artifact_role=1` (`single`) with ordinal zero and
count exactly one; neither purpose may use role `2` (`chunk`). An aggregate
purpose uses role `1` iff its count is one and role `2` iff its count is at
least two, with data ordinals `0..count-1`. Every final-manifest envelope uses
role `3` with ordinal exactly `count`; every commit envelope uses role `4`
(`commit_record`) with ordinal exactly `count+1`. Record and WAL artifacts are
therefore one-entry canonical plans, not an exception or an optional plan. Any
other role, ordinal, count, total, summary, or phase combination is
purpose-inapplicable and rejected before allocation, provider work, Seal,
persistence, or publication.

`aad_length` is the exact `ContextAadV1` frame length, at most
`MAX_RECORD_AAD_BYTES`; `ContextAadDigest = SHA-256(aad)` is bound in the final
manifest and commit record below. The precise satisfiable maxima are:

| purpose | formula | maximum | plus-one refusal |
|---|---|---:|---:|
| record | `37 + 114 + (6 * 256) + 72 + 64 + 2` | 1,825 | 1,826 |
| WAL | `34 + 114 + (6 * 256) + 72 + 64 + 2` | 1,822 | 1,823 |
| segment | `38 + 114 + (5 * 256) + 72 + 64 + 2` | 1,570 | 1,571 |
| snapshot | `39 + 114 + (5 * 256) + 72 + 64 + 2` | 1,571 | 1,572 |
| repair | `37 + 114 + (5 * 256) + 72 + 64 + 2` | 1,569 | 1,570 |
| migration | `40 + 114 + (5 * 256) + 72 + 64 + 2` | 1,572 | 1,573 |

The domain/header contribution is `1 + domain_len + 1 + 1 + 2`; the 19 fixed
field prefixes total `114`; record/WAL have six bounded ASCII fields while the
four aggregate purposes have five because transaction ID is empty; the nine
U64s total `72`, two digests `64`, and two ENUM8 values `2`. Hence every legal
v1 AAD is at most 1,825 bytes, within the 4,096-byte hard ceiling. The
classification-binding digest replaces a fixed 32-byte digest rather than
adding a field, so the independently checked purpose maxima remain
`1,825/1,822/1,570/1,571/1,569/1,572`. The summary itself has a `310`-byte
fixed portion plus `46` bytes per entry, so
`MAX_WAL_CLASSIFICATION_SUMMARY_BYTES = 310 + (1,024 * 46) = 47,414`; entry
`1,025` (`47,460` bytes if constructed) and a `47,415`th trailing byte are
both rejected before allocation or `PREPARED`. Independent encoder KATs cover
each table-row exact maximum and its listed plus-one case (a 257-byte otherwise
canonical ASCII field), all zero/real/summary/phase combinations, empty,
uniform, mixed, metadata, control, alias, and cross-purpose WAL cases, and
`u64::MAX` checked-length overflow. Each refusal happens before allocation,
handle acquisition/Open, Seal, persistence, or publication.

`CiphertextEnvelopeV1` seals one bounded artifact chunk, not an unbounded
stream. `ciphertext_length` is checked before allocation and is at most 4 MiB
for record/tombstone purpose and at most 16 MiB for WAL, segment, snapshot,
repair, and migration purpose; the ciphertext is the same length as its
AES-GCM plaintext. Only aggregate-purpose artifacts may be pre-split into
multiple such chunks. Record and WAL are each exactly one envelope and bind
their one zero-based ordinal, count, total, role, and plan digest in AAD; no
decoder accumulates more than one accepted data chunk.

For every artifact, including a one-entry record or WAL artifact,
`ArtifactPlanV1` is the exact pre-encryption frame whose SHA-256 is the AAD
plan digest:

```text
domain_len:u8 = 35
domain:[u8;35] = "oyatie.data.record.artifact-plan.v1"
separator:u8 = 0 | version:u8 = 1
purpose:u8 | artifact_generation:u64 | classification_code:u8 |
classification_revision:u64 | classification_binding_digest:[u8;32] |
chunk_count:u64 | total_plaintext_bytes:u64
repeated exactly chunk_count times, ordinal order 0..chunk_count-1:
  ordinal:u64 | plaintext_length:u64
```

Every ordinal is exactly its zero-based position; the classification pair and
binding digest equal the applicable `ContextAadV1` values; and checked addition
of entry lengths must equal the total. The decoder then applies this closed
purpose law before allocating a plan entry, acquiring a provider handle,
Sealing, persisting, or publishing:

| purpose | `chunk_count` | entry and total law |
|---|---:|---|
| record | exactly `1` | ordinal `0`; `0..=4,194,304` bytes (`4 MiB`) |
| WAL | exactly `1` | ordinal `0`; `1..=16,777,216` bytes (`16 MiB`) |
| segment, snapshot, repair, migration | `1..=4,096` | every entry is at most `16 MiB`; total is at most `min(64 GiB, chunk_count * 16 MiB)` with checked arithmetic |

A record tombstone is the only legal zero-length record entry; a WAL is
nonempty because its canonical durable mutation list is nonempty. Thus a
record/WAL count of two is invalid even though the aggregate grammar admits a
count of two. For record the binding digest is its record-key digest, for WAL
it is its summary digest (including the mixed sentinel rule), and for
aggregates it is zero. The fixed frame is
`1 + 35 + 1 + 1 + 1 + 8 + 1 + 8 + 32 + 8 + 8 = 104` bytes, so its exact maximum
is `104 + (4,096 * 16) = 65,640` bytes. A `65,641`-byte trailing form and count
`4,097` (`65,656` bytes if constructed) are both invalid. Independent encoders
accept record count `1` at exactly `4,194,304` bytes and WAL count `1` at
exactly `16,777,216` bytes, reject record/WAL count `2` and their respective
`4,194,305`/`16,777,217` totals, accept aggregate count `4,096` at exactly
`64 GiB`, and reject aggregate count `4,097`, total `64 GiB + 1`, and every
checked multiply/add overflow. A count of one is canonical only for the single
data envelope plus its final-manifest and commit envelopes; it is not an
omitted plan. The final manifest is a separately sealed role-`3` envelope for
every count whose plaintext is this exact
`ArtifactFinalManifestV1` frame:

```text
domain_len:u8 = 39
domain:[u8;39] = "oyatie.data.record.artifact-manifest.v1"
separator:u8 = 0 | version:u8 = 1
purpose:u8 | artifact_generation:u64 | classification_code:u8 |
classification_revision:u64 | classification_binding_digest:[u8;32] |
key_id_length:u16 | key_id:[u8;key_id_length] |
key_generation:u64 | chunk_count:u64 | total_plaintext_bytes:u64 |
plan_digest:[u8;32] | final_manifest_aad_digest:[u8;32]
repeated exactly chunk_count times, ordinal order 0..chunk_count-1:
  ordinal:u64 | plaintext_length:u64 | ciphertext_digest:[u8;32] |
  serialized_envelope_digest:[u8;32] | context_aad_digest:[u8;32]
```

The fixed portion is `438` bytes and each entry is `112` bytes, so its exact
maximum is `438 + (4,096 * 112) = 459,190` bytes. A `459,191`-byte trailing
form and count `4,097` (`459,302` bytes if constructed) are invalid. The final-
manifest envelope's own `ContextAadDigest` must equal
`final_manifest_aad_digest`; its classification-binding digest must equal the
plan and every role envelope's tag `09`; each entry's digest must equal the
actual data envelope's `ContextAadDigest`. This is not a claim about an
unframed manifest: every digest is concrete bytes in the stated canonical
frame.

The only publishable artifact root is a sealed role-`4`
`ArtifactCommitRecordV1`. It is a canonical, bounded, versioned tagged frame:

```text
domain_len:u8 = 37
domain:[u8;37] = "oyatie.data.record.artifact-commit.v1"
separator:u8 = 0 | version:u8 = 1 | field_count:u16 = 26
repeated exactly field_count times in increasing-tag order:
  tag:u8 | type:u8 | length:u32 | value:[u8;length]
where each listed tag occurs exactly once:
  01 artifact_locator_id:ASCII(1..=256)       0d key_generation:U64
  02 tenant:ASCII(1..=256)                    0e fence_sequence:U64
  03 database:ASCII(1..=256)                  0f bootstrap_locator_digest:DIGEST32
  04 table:ASCII(1..=256)                     10 chunk_count:U64
  05 tablet:ASCII(1..=256)                    11 total_plaintext_bytes:U64
  06 purpose:ENUM8(1..=6)                     12 plan_digest:DIGEST32
  07 ownership_epoch:U64                      13 final_manifest_envelope_digest:DIGEST32
  08 schema_revision:U64                      14 final_manifest_aad_digest:DIGEST32
  09 classification_code:ENUM8(1..=19; WAL mixed only=0) 15 commit_aad_digest:DIGEST32
  0a classification_revision:U64              16 predecessor_commit_envelope_digest:DIGEST32
  0b artifact_generation:U64                  17 commit_sequence:U64
  0c key_id:ASCII(1..=256)
  18 classification_binding_digest:DIGEST32
  19 commit_ordinal:U64
  1a transaction_id:ASCII(0..=256)
```

The header is `42` bytes and 26 prefixes add `156`. Seven ASCII values add at
most `1,792`, ten U64 values `80`, two ENUM8 values `2`, and seven DIGEST32
values `224`, so `MAX_ARTIFACT_COMMIT_RECORD_BYTES = 42 + 156 + 1,792 + 80 +
2 + 224 = 2,296`; a `2,297`-byte trailing form is invalid. `transaction_id` is
real for record/WAL and exactly empty for aggregate purposes; `commit_ordinal`
is real for record/WAL and zero for aggregates. The commit plaintext is encrypted/authenticated in a
`CiphertextEnvelopeV1` whose purpose, key ID/generation, bootstrap locator,
classification/binding digest, artifact generation, count, total, plan digest,
role, and ordinal agree with the record. Its `ContextAadDigest` equals field
`15`; its tags `09`, `0a`, and `0b` equal fields `18`, `19`, and `1a`; the actual
bootstrap bytes hash to field `0f`; field `13` hashes the complete serialized
final-manifest envelope; and field `14` equals both the final-manifest plaintext
field and the final-manifest envelope's actual context AAD. The final manifest,
plan, and all data envelopes must agree with the record on every common field,
including the WAL summary or record-key binding digest and the record/WAL
transaction identity. A count other than 26, unknown/duplicate/omitted/out-of-order
tag, wrong type/width, noncanonical ASCII, bad enum, truncation, or trailing
byte is `ArtifactCommitMalformed` before any head mutation or publication. An
initial commit uses sequence zero and an all-zero predecessor;
a later commit uses exactly its predecessor sequence plus one and its
serialized-commit-envelope digest. No committed field is advisory.

The linearizable storage key is `(tenant, artifact_locator_id)`. Its exact
CAS value is `ArtifactCommitHeadV1`:

```text
domain_len:u8 = 35
domain:[u8;35] = "oyatie.data.record.artifact-head.v1"
separator:u8 = 0 | version:u8 = 1 | commit_sequence:u64 |
serialized_commit_envelope_digest:[u8;32]
```

It is exactly 78 bytes. The head is an authenticated-candidate pointer, not
freshness authority: readers recompute its digest over the fetched complete
commit envelope, Open that envelope, and require the sealed commit's
tenant/locator/sequence and all bindings below before treating anything as
published.

`ArtifactPublicationContextV1` freezes the identity that no later publication
may substitute. It is canonical ASCII and not a caller-provided routing hint:

```text
domain_len:u8 = 41
domain:[u8;41] = "oyatie.data.record.publication-context.v1"
separator:u8 = 0 | version:u8 = 1
artifact_locator_id_length:u16 | artifact_locator_id:ASCII(1..=256)
tenant_length:u16 | tenant:ASCII(1..=256)
database_length:u16 | database:ASCII(1..=256)
table_length:u16 | table:ASCII(1..=256)
tablet_length:u16 | tablet:ASCII(1..=256)
purpose:u8
```

Its maximum is `44 + (5 * (2 + 256)) + 1 = 1,335` bytes and
`publication_context_digest = SHA-256(ArtifactPublicationContextV1)`. The
five ASCII values and purpose are exactly the identically named commit fields;
an unknown purpose, malformed length, noncanonical ASCII, trailing byte, or a
digest mismatch is `ArtifactPublicationContextInvalid` before object lookup.

The Data-owned `ArtifactPublicationCoordinator` is the only authority that
stores the exact 78-byte head and this fixed `ArtifactPublicationAnchorV1` in
one linearizable durable CAS tuple under `(tenant, artifact_locator_id)`. Its
`artifact-publication-cell` adapter commits tuple, pin, member, decision,
receipt, accepted-high-water history, safe-GC, and read-snapshot transitions
in the same cell/tablet consensus log; neither object storage nor a wall-clock
metadata value is an authority. Its exact authenticated port operations are
`AcquirePin`, `RenewPublicationPin`, `PutPinnedObject`, `BindPin`,
`CompareAndSwapAnchor`, `FinalizePublicationAudit`, `TakeOverAndReconcile`,
`AcquireTerminalRecoveryLease`, `RecoverTerminalPin`, `ReconcilePin`,
`AbandonPin`, `AdvanceSafeGcEpoch`, `EnumerateNonterminalPins`,
`ReadAcceptedAnchorHistory`,
`ReadPublicationSnapshot`, `ReadPublicationTerminalOutcome`, and the Audit-only
`ResolveLocalPublicationCasReceipt`. A publisher receives only a scoped work
lease; Audit receives only a facade-issued callback capability; no operation
accepts a caller-supplied tuple, receipt, member list, history, or current-head
claim:

```text
domain_len:u8 = 40
domain:[u8;40] = "oyatie.data.record.publication-anchor.v1"
separator:u8 = 0 | version:u8 = 1
publication_context_digest:[u8;32]
head:[u8;78]                           # exact ArtifactCommitHeadV1 bytes
artifact_generation:u64
ownership_epoch:u64
fence_sequence:u64
```

The anchor is exactly `43 + 32 + 78 + 24 = 177` bytes. Genesis requires a
fresh authenticated `HighWaterAbsent` receipt, commit sequence `0`, all-zero
predecessor, and artifact generation `1`. Every later CAS verifies the same
context digest, the predecessor digest and `sequence=prior+1` (refusing
`u64::MAX`), `artifact_generation > prior`, `ownership_epoch >= prior`, and
`fence_sequence > prior`; a changed ownership epoch must also equal the current
authoritative tablet lease. The `(ownership_epoch, fence_sequence)` pair is
strictly lexicographically increasing: it is either the same epoch with a
higher fence or a higher epoch with a higher fence; fencing never resets. The current key
binding, key generation, bootstrap fence, schema/classification binding, count,
total, plan, final-manifest, and AAD digests must agree with the desired sealed
commit and all reachable objects. A changed context, nonincreasing generation
or fence, epoch regression, missing current tablet lease, sequence/generation/
fence exhaustion, or key-transition refusal is respectively
`ArtifactPublicationContextMismatch`, `ArtifactGenerationRegression`,
`ArtifactOwnershipEpochRegression`, `ArtifactFenceRegression`,
`ArtifactCommitSequenceExhausted`, `ArtifactGenerationExhausted`,
`ArtifactFenceExhausted`, or `ArtifactKeyTransitionRefused`; no overflow,
overwrite, or inferred repair is allowed.

`AuditSink` is also the separately accepted durable anti-rollback authority.
Its D1c-KG contract must provide idempotent
`AppendPublicationHighWater(anchor, local_cas_receipt)` and challenge-bound
`GetPublicationHighWater(tenant, artifact_locator_id,
publication_context_digest, freshness_challenge:[u8;32])`. A high-water receipt
binds the challenge, context digest, complete anchor digest/head, sequence,
generation, ownership epoch, fence, durable Audit ordinal, provider revision,
and expiry/integrity. It advances only to the exact next anchor, returns the
same receipt for the same anchor, rejects an equal sequence with different
bytes or any lower value, and retains the latest anchor (or a terminal locator
tombstone) independently of object-store backup/restore. `HighWaterAbsent` is
valid only for genesis and only for its challenge. Audit unavailability,
integrity/freshness failure, a foreign context, or a local head/anchor different
from the returned high-water is `ArtifactPublicationAnchorUnavailable`,
`ArtifactPublicationAnchorInvalid`, or `ArtifactHeadRollbackDetected`; it
quarantines the locator and withdraws affected admission/readiness rather than
guessing that an older valid head is current.

`LocalPublicationCasReceiptV1` is the coordinator-only durable proof for the
otherwise cross-authority CAS-to-Audit gap. It is a canonical bounded record in
the same trusted linearizable coordinator, never an object-store blob or
caller-provided assertion:

```text
domain_len:u8 = 45
domain:[u8;45] = "oyatie.data.record.publication-cas-receipt.v1"
separator:u8 = 0 | version:u8 = 1
pin_id:[u8;32] | publication_context_digest:[u8;32]
expected_anchor_digest:[u8;32] | desired_anchor_digest:[u8;32]
coordinator_epoch:u64 | cas_index:u64 | gc_epoch:u64
```

It is exactly `48 + (4 * 32) + (3 * 8) = 200` bytes. The successful atomic
tuple-CAS is the only operation that creates it; the coordinator durably binds
it to the current authenticated tuple and makes it retrievable only through its
authenticated coordinator port by `(pin_id, desired_anchor_digest)`.
`coordinator_epoch` is the originating durable consensus term, not a
requirement that Audit contact that departed leader. On failover the current
term's `ResolveLocalPublicationCasReceipt` atomically checks the immutable
receipt bytes, context, desired tuple, original `cas_index`, and retained
decision history, then returns the original receipt together with an
authenticated re-attestation containing its digest, original epoch, current
epoch, and current applied index. It cannot re-attest a receipt whose tuple is
not the recorded desired tuple or whose decision was not `CAS_SUCCEEDED`.
Audit accepts an older originating epoch only through that current,
facade-authenticated resolution; it compares the resolved receipt to the
supplied anchor before accepting `AppendPublicationHighWater`. Supplied receipt
bytes, a missing/mismatched resolution, a stale coordinator callback
capability, or a receipt whose current-term re-attestation fails is
`ArtifactPublicationAnchorInvalid`. The coordinator retains a receipt for every
`COMMITTING` pin and current anchor, so crash-after-CAS/before-Audit recovery
can retry after a coordinator failover without guessing or weakening fencing.

On a successful `AppendPublicationHighWater`, the facade passes the
provider-authenticated receipt through `FinalizePublicationAudit`; the Cell
adapter verifies its callback capability and atomically records one canonical
`PublicationAcceptedAnchorHistoryV1` row before it can finalize the pin:

```text
domain_len:u8 = 47
domain:[u8;47] = "oyatie.data.record.publication-audit-history.v1"
separator:u8 = 0 | version:u8 = 1
publication_context_digest:[u8;32] | anchor_digest:[u8;32] |
predecessor_anchor_digest:[u8;32] | local_cas_receipt_digest:[u8;32] |
audit_receipt_digest:[u8;32] | audit_ordinal:u64 | accepted_gc_epoch:u64
```

The row is exactly `50 + (5 * 32) + 16 = 226` bytes. It names the complete
already-validated anchor and receipt rather than duplicating their variable
frames. Rows are immutable, context-scoped, ordered by the anchor's sequence,
and retained through every nonterminal pin/receipt that could need them; a
history lookup authenticates each predecessor-to-successor link and cannot
manufacture, reorder, skip, or substitute an Audit acceptance. The row is the
coordinator's durable accepted-high-water index, not a new source of truth:
Audit remains the authority for its receipt and `GetPublicationHighWater`.
After the last nonterminal pin/receipt that names a history prefix reaches a
safe terminal release, compaction retains the one current-anchor checkpoint and
removes the older prefix only after the same safe-GC proof; history therefore
does not grow with every published generation.
`FinalizePublicationAudit` validates that the returned high-water is the
pin's desired current anchor, persists this row, records `COMMITTED`, and
detaches members/releases the pin in one consensus transition. Thus a later
publisher never has to infer whether a predecessor reached Audit.

For every pin that reaches a CAS decision, the coordinator also writes exactly
one immutable `PublicationPinDecisionV1` consensus record. It has a `49`-byte
header (`domain_len=46`,
`"oyatie.data.record.publication-pin-decision.v1"`, separator, version),
five 32-byte digests (`pin`, context, expected, desired, observed tuple), a
one-byte decision, a 32-byte local-receipt digest (zero unless successful),
and `decision_index`, `coordinator_epoch`, and `decision_gc_epoch` U64s; its
exact size is `49 + 160 + 1 + 32 + 24 = 266` bytes. `NOT_CAS=1` means the
coordinator proved this pin never entered its atomic CAS (an explicit safe
abandon records expected; a stale unattempted pin records the newer observed
tuple), `CAS_SUCCEEDED=2` means the atomic tuple changed to desired and names
the exact local receipt, and `CAS_LOST=3` means the tuple was observed at the
recorded different anchor without mutation. The record is coordinator-authenticated storage,
never an object-store object or caller assertion. It is the durable proof that
distinguishes a pin which never CASed, one that successfully CASed and was
later superseded, and an ordinary losing racer.

Before the first artifact object is persisted, a publisher durably acquires an
`ArtifactPublicationPinV1` from that coordinator. The pin is not a best-effort
GC grace period and uses logical consensus GC epochs, never wall time:

```text
domain_len:u8 = 37
domain:[u8;37] = "oyatie.data.record.publication-pin.v1"
separator:u8 = 0 | version:u8 = 1
pin_id:[u8;32] | publication_context_digest:[u8;32]
expected_anchor_digest:[u8;32] | desired_anchor_digest:[u8;32]
lease_fence_sequence:u64 | lease_epoch:u64 | acquired_gc_epoch:u64 |
expires_after_gc_epoch:u64 | expected_object_count:u16 | state:u8
```

It is exactly `40 + (4 * 32) + (4 * 8) + 2 + 1 = 203` bytes.
`pin_id` is coordinator-minted, never caller-chosen, and is exactly the
cell's durable globally unique 24-octet allocation namespace followed by its
strictly increasing `pin_allocation_index:u64`. The consensus allocator writes
the next representable index and the new pin in one transaction; a committed
index is never decremented, reused, or restored from backup. The allocation
high-water is retained independently of released-pin safe-GC compaction, so a
deleted pin or terminal lease can never name a later pin. Exhaustion rejects
before pin or object allocation with `ArtifactPublicationPinAllocationExhausted`.
`expected_anchor_digest` is all zero only for genesis; `desired_anchor_digest`
is all zero only in `OPEN=1`. The states are `OPEN=1`, `BOUND=2`,
`COMMITTING=3`, `COMMITTED=4`, `ABORTED=5`, `CONFLICTED=6`,
`SUPERSEDED=7`, and `RELEASED=8`. `OPEN`, `BOUND`, `COMMITTING`, and
`CONFLICTED` are nonterminal; the others are terminal, but only `RELEASED`
has atomically detached its member rows. `lease_epoch` and
`lease_fence_sequence` are the publisher-ownership pair and are not the
artifact fence: the immutable desired anchor carries the artifact's own fence.
A lawful `CAS_LOST` is never a `CONFLICTED` state: it has the atomic terminal
path below. `CONFLICTED` is reserved for a damaged/missing authenticated
decision or other genuinely undecidable recovery and is therefore subject to
the bounded quarantine/admission rules, never a normal loser retry.
Every publisher action compares both ownership values; a successor replaces
both atomically, so a stale owner cannot renew, put, bind, CAS, finalize,
release, or retry Audit. The count is exactly
`chunk_count + 2`, therefore `3..=4,098`: data ordinals
`0..chunk_count-1`, final-manifest ordinal `chunk_count`, and commit ordinal
`chunk_count+1`. Each
`PutPinnedObject(pin_id, ordinal, immutable serialized envelope bytes)` atomically
persists the bytes at their SHA-256 content address and writes the unique pinned
member `(pin_id, ordinal) -> serialized_envelope_digest || put_gc_epoch`; it
cannot expose an unpinned object to GC. Duplicate, skipped, wrong-digest, or
out-of-range membership is `ArtifactPublicationPinInvalid` before verification.

Pin acquisition atomically records the current coordinator publisher lease
epoch/fence, the exact expected anchor digest, and an
`expires_after_gc_epoch` strictly above the current logical GC epoch (refusing
arithmetic exhaustion). `RenewPublicationPin(pin_id, lease_epoch,
lease_fence_sequence, new_expires_after_gc_epoch)` is linearizable and succeeds
only for the current owner while the pin is `OPEN`, `BOUND`, or `COMMITTING`;
it must preserve pin, context, expected/desired-anchor, and membership bytes
and strictly advance expiry beyond the current GC epoch but never beyond
`acquired_gc_epoch + MAX_PUBLICATION_PIN_RENEWAL_EPOCHS`. Let that checked
inclusive upper bound be `H`; acquisition refuses
`ArtifactPublicationPinRenewalHorizonExceeded` before object persistence if the
addition cannot be represented. Every Put, verify-to-Bound, tuple CAS, finalize, and
release compares the same current work-lease/fence and an unexpired logical
epoch. `TakeOverAndReconcile` accepts a facade-authorized successor after the
old pair is expired, unavailable, or fenced only while `current_gc_epoch < H`;
in one consensus transaction it strictly advances both ownership values,
preserves immutable pin/context/member bytes and its original acquisition
epoch, and sets a replacement expiry strictly above the current epoch and at
most `H`. Thus at `H-1` a normal successor may end at `H`, while at `H` no
ordinary renewal or work takeover can manufacture an impossible expiry and
both return `ArtifactPublicationTerminalRecoveryRequired`.

At or after `H`, `AcquireTerminalRecoveryLease` writes one durable, fenced,
non-renewable `TerminalRecoveryLeaseV1` consensus row only when the current
term proves the old work pair expired, unavailable, or fenced:

```text
domain_len:u8 = 51
domain:[u8;51] = "oyatie.data.record.publication-terminal-recovery.v1"
separator:u8 = 0 | version:u8 = 1
pin_id:[u8;32] | publication_context_digest:[u8;32] |
expected_anchor_digest:[u8;32] | desired_anchor_digest:[u8;32] |
old_lease_fence_sequence:u64 | old_lease_epoch:u64 |
terminal_lease_fence_sequence:u64 | terminal_lease_epoch:u64 |
coordinator_epoch:u64 | issued_gc_epoch:u64
```

It is exactly `54 + (4 * 32) + (6 * 8) = 230` bytes. The creation transaction
strictly advances the pin's ownership fence/epoch and binds the row to the
current coordinator term; an unavailable or fenced terminal holder can be
replaced only by a newer fenced terminal row, never renewed or converted into a
work lease. There is exactly one **active** terminal row per nonterminal pin:
a takeover atomically replaces that bounded relation with a higher fenced row
rather than appending a recovery list. The pin expiry remains `H`.
`RecoverTerminalPin` accepts only the exact current durable terminal row and
never extends that expiry. Its operation type is deliberately not coercible to
`PublicationWorkLease`, so `AcquirePin`, renewal, Put, Bind, tuple CAS, rebase,
or any new publication cannot accept it. It may only read the pin's
authenticated decision, local receipt, accepted-anchor history, and fresh
high-water; resolve/re-attest and append Audit for that *already successful*
exact CAS receipt; and terminalize the existing pin. Each invocation performs
one bounded reconciliation pass and adds no member, desired tuple, or
publication attempt. If those existing proofs are insufficient it returns
`ArtifactPublicationRecoveryQuarantined`; a later newer fenced terminal row
may retry the same lookup/append without growing state.

Every terminal completion uses the coordinator/Cell adapter's internal
`CommitTerminalRelease` consensus operation; it is not a separately callable
publisher capability. In one transaction it validates or writes the immutable
CAS decision and terminal cause, records `released_gc_epoch`, sets the pin to
`RELEASED`, detaches every member, and deletes the exact active
`TerminalRecoveryLeaseV1` row. A terminal recovery invocation additionally
requires equality with its complete terminal-lease row; a normal owner release
requires its current work lease and cannot race a newer terminal owner. The
transaction is all-or-nothing: a failed cleanup leaves the old nonterminal pin
and its one current 230-byte terminal row, never a `RELEASED` pin with a live
terminal row. No terminal-lease row or terminal-lease tombstone represents a
released pin. A stale, foreign, superseded, or deleted terminal credential is
therefore rejected as `ArtifactPublicationTerminalRecoveryLeaseLost` before
and after safe-GC compaction. Expiry fences an old publisher from further work,
but never authorizes GC to collect a nonterminal pin or lets any successor
change its expected/desired tuple.

Publication admission is bounded before pin acquisition or object persistence.
`MAX_PUBLICATION_NONTERMINAL_PINS_PER_LOCATOR=8`,
`MAX_PUBLICATION_NONTERMINAL_PINS_PER_TENANT_PER_CELL=64`,
`MAX_PUBLICATION_NONTERMINAL_PINS_PER_CELL=256`,
`MAX_PUBLICATION_PIN_RENEWAL_EPOCHS=1,024`,
`MAX_PUBLICATION_AUDIT_APPEND_ATTEMPTS_PER_LEASE=8`,
`MAX_PUBLICATION_REBASE_ATTEMPTS_PER_REQUEST=3`, and
`MAX_PUBLICATION_RECONCILIATION_BATCH=64` with
`MAX_PUBLICATION_RECONCILIATION_BACKLOG=256`. A live terminal row is charged
one-for-one to its nonterminal pin: the exact active limits are
`MAX_PUBLICATION_ACTIVE_TERMINAL_RECOVERY_LEASES_PER_LOCATOR=8`,
`MAX_PUBLICATION_ACTIVE_TERMINAL_RECOVERY_LEASES_PER_TENANT_PER_CELL=64`, and
`MAX_PUBLICATION_ACTIVE_TERMINAL_RECOVERY_LEASES_PER_CELL=256`, or exactly
`8 * 230 = 1,840`, `64 * 230 = 14,720`, and `256 * 230 = 58,880` durable
bytes. `AcquireTerminalRecoveryLease` increments those counters in its same
consensus transaction and refuses a saturated or inconsistent relation as
`ArtifactPublicationTerminalRecoveryLeaseQuotaExceeded`; `CommitTerminalRelease`
decrements it while deleting the row. Thus the original horizon is the pin's
checked `H`, not an accumulating terminal-row retention horizon: after a
successful release the durable terminal-row count and bytes are exactly zero
for that pin, while an undecidable pin consumes at most one 230-byte row inside
the declared nonterminal limits. A maximum-envelope byte overhead is
`1 + 26 + 1 + 1 + 1 + 1 + 2 + 256 + 8 + 12 + 2 + 1,245 + 4 + 1,572 + 8 + 16 =
3,156` bytes. This is deliberately the aggregate-purpose maximum: migration
has the largest legal aggregate AAD (`1,572`), while the record-only `1,825`
AAD is purpose-inapplicable. Likewise the general commit maximum remains
`2,296`, but every aggregate commit has the mandatory empty `transaction_id`,
so its exact maximum is `2,296 - 256 = 2,040`. Therefore one maximum aggregate
pin's serialized member bytes are exactly `64 GiB + 459,190 + 2,040 + (4,098 *
3,156) = 68,732,871,254`. The coordinator rejects a locator above
`8 * 68,732,871,254 = 549,862,970,032` pinned-member bytes, a tenant/cell above
`64 * 68,732,871,254 = 4,398,903,760,256`, or the whole cell above
`256 * 68,732,871,254 = 17,595,615,041,024`, before allocating a member buffer;
a deployment profile may only lower these maxima. An aggregate attempt carrying
a record/WAL AAD or a nonempty transaction ID is rejected as
`ArtifactPurposeInapplicable`/`ArtifactCommitMalformed` before allocation, key
acquisition, persistence, or publication, rather than being charged against a
larger incompatible quota. The current consensus leader enumerates at most 64
nonterminal pins per bounded reconciliation pass; it admits no new pin for a
saturated locator, tenant, or cell until a terminal release restores capacity.
`ArtifactPublicationPinQuotaExceeded`,
`ArtifactPublicationPinnedBytesExceeded`,
`ArtifactPublicationPinAllocationExhausted`,
`ArtifactPublicationPinRenewalHorizonExceeded`,
`ArtifactPublicationTerminalRecoveryRequired`,
`ArtifactPublicationTerminalRecoveryLeaseQuotaExceeded`,
`ArtifactPublicationContentionExhausted`, and
`ArtifactPublicationReconciliationBacklogExceeded` are typed pre-I/O
refusals, not availability guesses.

The publisher verifies every pinned envelope and binds one immutable desired
anchor only after all `chunk_count+2` members verify. Before it may enter the
one logical tuple CAS attempt, `CompareAndSwapAnchor` atomically requires that
the local current tuple equals its expected anchor, a fresh authenticated Audit
high-water and `PublicationAcceptedAnchorHistoryV1` entry both name that
expected anchor (or the fresh genesis `HighWaterAbsent` case), and no
`COMMITTING` pin has a `CAS_SUCCEEDED` decision whose desired anchor is that
expected anchor. A known missing accepted-history row or successful
`COMMITTING` predecessor writes no CAS decision or tuple mutation and returns
`ArtifactPublicationPredecessorAuditPending`; an unavailable or invalid fresh
high-water instead returns the existing
`ArtifactPublicationAnchorUnavailable`/`ArtifactPublicationAnchorInvalid`.
The bound pin may wait/retry under its still-valid work lease or take the proven
`NOT_CAS` terminal path because the coordinator itself knows no CAS was entered.
Deferred successors never hold the successful predecessor's recovery lock, so
H0-to-H1 crash recovery can make progress and Audit appends remain H1 then H2,
never out of order.

Once the gate passes, a successful attempt atomically changes `BOUND` to
`COMMITTING`, changes the head+anchor tuple, writes the exact
`LocalPublicationCasReceiptV1`, and writes a `CAS_SUCCEEDED` decision. A losing
attempt changes no tuple but in that same consensus transaction writes
`CAS_LOST` with its observed anchor, records the `SUPERSEDED` terminal cause,
sets `released_gc_epoch`, detaches every member, and leaves the pin
`RELEASED`. It therefore cannot be stranded merely because its recorded H1
later becomes the accepted predecessor of H2: the durable loss proof remains
idempotently terminal for every later history lookup. Releasing the pin only
removes its pin reference; it never deletes an object or bypasses the separate
anchored-chain/safe-epoch GC proof below. The one allowed retry is an idempotent
observation that the same desired anchor is already current with the same local
receipt and high-water; it is `ArtifactAlreadyPublished`, not a second CAS. A
publisher that continues after a loss acquires a fresh pin from a fresh snapshot
with a newly monotonic generation/fence. The request may rebase at most three
times, then receives `ArtifactPublicationContentionExhausted`; it cannot
retarget an existing pin or reuse its lease/members for a different desired
tuple.

After successful CAS, the current work owner makes at most eight idempotent
Audit append attempts per work-lease pass; `RecoverTerminalPin` has the same
eight-attempt bound for its one terminal-only reconciliation pass. A crash
before CAS leaves `OPEN` or `BOUND`; a crash after H0-to-H1 CAS but before
Audit leaves H1 `COMMITTING` and is not reader-visible. A successor whose
expected anchor is H1 is refused by the predecessor gate until the current or
fenced terminal recovery resolves/re-attests the H1 receipt, appends H1,
records the accepted-history row, and releases H1; only then can it CAS H1 to
H2. A crash after the Audit receipt is an idempotent
`FinalizePublicationAudit`. `AbandonPin` is permitted only for `OPEN` or
`BOUND` after a fresh expected local tuple and high-water match; it writes
`NOT_CAS`, transitions `ABORTED`, and cannot discard a pin that might have
CASed. The sole additional `BOUND` case is a coordinator-recorded predecessor
gate refusal: it is an equally durable proof that this pin never entered CAS,
so `AbandonPin` may atomically write `NOT_CAS` and release it without waiting
for the predecessor's Audit availability.

The current pin owner through `ReconcilePin`, an authorized normal successor,
or the durable terminal-only recovery holder runs this exact reconciliation
table under the current coordinator term. `H_expected`, `H_desired`, and
`H_other` denote the pin's expected, desired, and a different same-context
anchor; `H_current` is the fresh high-water/local tuple. `accepted ancestor` is
a retained ordered
`PublicationAcceptedAnchorHistoryV1` chain from the named anchor through the
fresh high-water; each link proves an Audit-accepted predecessor.

| Fresh Audit high-water and local tuple | durable pin decision/history | exact action |
|---|---|---|
| `H_expected`, `H_expected` | no decision or `NOT_CAS` | atomically write `NOT_CAS` if needed, transition `ABORTED`, then release members. |
| `H_other`, `H_other` | no decision while the pin is `OPEN` or `BOUND` | the atomic-decision invariant proves this pin never CASed; write `NOT_CAS` with `H_other`, record `SUPERSEDED`, then release members. |
| `H_desired`, `H_desired` | `CAS_SUCCEEDED` with the matching receipt | resolve/re-attest the receipt; validate/persist the H-desired accepted-history row; record `COMMITTED`, then release members. |
| `H_expected`, `H_desired` | `CAS_SUCCEEDED` with the matching receipt | retry the resolved/re-attested H-desired Audit append, persist its accepted-history row, then record `COMMITTED` and release members. |
| `H_current`, `H_current` | `CAS_LOST` recorded H1, where H1 is `H_current` or an accepted ancestor of it | this is already the atomic loss terminal path: preserve the `CAS_LOST`/`SUPERSEDED` history and return idempotent `RELEASED`; H1-to-H2 cannot reopen or quarantine it. |
| `H_other`, `H_other` | `CAS_SUCCEEDED` and `H_desired` is an accepted ancestor of `H_other` | record `COMMITTED` as a successfully published, later superseded pin, then release members. |
| any mismatch, missing authenticated decision/history, changed context, or unavailable fresh high-water | insufficient proof | retain the pin, return `ArtifactPublicationRecoveryQuarantined`, and enqueue bounded successor reconciliation; it cannot be reclaimed or silently reused. |

The coordinator's `CommitTerminalRelease` writes or validates the terminal
cause/decision, `released_gc_epoch`, every member detachment, and deletion of
the active terminal-lease row atomically; the state becomes `RELEASED` only
after that transaction. A crash before commit leaves the original nonterminal
pin and one current terminal row for its exact holder or a higher-fenced
replacement; a crash after commit leaves `RELEASED` with no terminal row even
if the caller lost the reply. The latter is idempotently observable through
`ReadPublicationTerminalOutcome(pin_id, publication_context_digest)` only to a
same-context coordinator capability while the pin decision is retained. Its
only retained result is the existing immutable decision digest plus
`RELEASED`/`released_gc_epoch`; otherwise it returns `NotRetained`, and it
never accepts a terminal credential or mints, renews, or uses terminal
authority. `RecoverTerminalPin` always requires
an exact active row, so its old credential receives
`ArtifactPublicationTerminalRecoveryLeaseLost` in either durable outcome.

GC may reclaim a released member only after its safe logical epoch strictly
exceeds `released_gc_epoch`, it is referenced by neither the current anchored
chain nor a nonterminal pin, and its receipt/decision/history retention is no
longer required. This check applies equally to an atomically lost pin:
`CAS_LOST` releases only its membership, never an object still named by the
H1/H2 anchored chain. A current anchor's receipt is retained; a nonterminal pin
retains its receipt/decision/history; terminal release retains those proofs
through the next safe-GC proof and then deterministically compacts the released
pin/control records. It never compacts an active terminal row because terminal
release already deleted it. The cell retains the independent monotonic
pin-allocation high-water across that compaction; `AcquirePin` cannot recreate
the old ID and `AcquireTerminalRecoveryLease` requires a current nonterminal
pin. Thus a stale terminal credential has no publishing power before or after
compaction and fails `ArtifactPublicationTerminalRecoveryLeaseLost`; a stale
work credential fails `ArtifactPublicationPinLeaseLost`. Thus a normal loser
has a finite terminal path, and an actually undecidable recovery remains safe
but consumes a bounded pin, byte, and reconciliation slot rather than immortal
state. Readiness withdrawal for an undecidable pin is locator-scoped; it does
not make unrelated fresh cell-local publication snapshots unavailable.

At boot, recovery, post-publication, coordinator-term change, or snapshot
expiry, `records-app` performs one bounded `GetPublicationHighWater` challenge
through the real Audit adapter and asks the current coordinator to validate the
returned anchor against its local tuple. On success it installs an in-cell RAM
`PublicationReadSnapshot` containing the immutable context digest, complete
anchor/head digest, Audit ordinal/expiry, coordinator epoch, and applied CAS
index. An ordinary artifact read makes no Audit RPC: it uses that snapshot only
when Cell's current interval is within the receipt expiry, the coordinator term
and tuple still exactly match the snapshot, and the locator has no unresolved
pin. A tuple or term change, expiry, cache miss, or failed local comparison
returns `ArtifactPublicationReadSnapshotStale`, triggers at most one bounded
refresh, and otherwise fails closed; a pre-H1 `H0` snapshot therefore cannot
serve after local H1. This keeps the serving hit path cell-local while treating
Audit as the authoritative boot/recovery/refresh witness required by ADR-0719.

The reader then fetches immutable content-addressed envelopes by their exact
digests, validates the sealed commit and final manifest, and requires every
data ordinal `0..count-1` exactly once in order. It verifies every serialized-
envelope digest, context AAD digest, classification-binding digest, immutable
context, monotonic anchor fields, purpose-specific count/total law, and checked
total. A retained valid `H0` after `H1` is
`ArtifactHeadRollbackDetected`; a missing object is
`ArtifactPublishedObjectMissing`; and a stale pin, context/key/generation/fence
substitution, or truncated/extra/duplicate/reordered frame returns its typed
refusal without falling back to a predecessor and preserves or quarantines the
current state. Independent encoders, model checks, and fault plants cover
N/N+1 counts, all purpose/classification cases, record/WAL count-one and
total-plus-one refusal, aggregate count-4,096 maxima, the exact/plus-one
summary/plan/manifest/commit/pin/decision/history/terminal-lease bounds, and
the independent migration aggregate arithmetic: accept exactly
`64 GiB + 459,190 + 2,040 + (4,098 * 3,156) = 68,732,871,254` and reject its
plus-one plus the locator/tenant/cell quota plus-ones; reject a record
`1,825`-byte AAD or nonempty-transaction `2,296`-byte commit on an aggregate
purpose before allocation. They cover H0-after-H1 crash/restore/failover
replay, H0-to-H1 CAS crash before Audit with a deferred H1-to-H2 CAS, ordered
H1 then H2 Audit acceptance, delayed `CAS_LOST(H1)` after accepted/local H2,
every monotonicity/exhaustion transition, A/B and N-writer races, normal and
terminal-only successor takeover, stale-owner/fence rejection, and GC
interleavings before/after every pin acquire, put, verify, bind, CAS, Audit
append, re-attestation, finalize, release, and ACK. They test normal renewal
at `acquired_gc_epoch+1,023`, terminal-only recovery at `+1,024` and `+1,025`,
checked-horizon overflow, old-work-lease fencing, unrecoverable-proof
quarantine, and that GC retains a lost pin's content while any anchored chain
still names it. For each scope, `N` is its active-terminal-row limit (8, 64,
or 256): tests admit exactly N current 230-byte terminal rows, refuse N+1,
then run N+1 **sequential** `+1,024` horizon recoveries with a crash/restore
and lost-reply case on every cycle. After each successful
`CommitTerminalRelease`, the released pin has zero durable terminal rows and
zero terminal-row bytes; an injected release-cleanup transaction abort leaves
only its one active 230-byte row and a nonterminal pin, then a retry or fenced
successor completes it. The campaigns verify no T1..TN released-row list, no
counter/byte leak beyond 8/64/256 or 1,840/14,720/58,880, stale terminal-token
refusal both before and after safe-GC compaction, and durable
pin-allocation-high-water non-reuse. They crash at each losing-CAS, takeover,
Audit, finalize, release, epoch-rollover, snapshot-refresh, restore, and GC
edge; verify every safely decidable pin becomes terminal, uncertain pins stay
safe and bounded, and no read-hit path needs a remote Audit call.
Malformed frames are rejected before a buffer, acquisition/Open, persistence,
or publication can be reached.

`NonceLeaseId:u32` is the only lease identity in a nonce and in every receipt,
checkpoint, error, envelope, and provider call; the AES-GCM nonce is exactly
`nonce_lease_id:u32 || counter:u64`. `RecordKeySource` returns a tenant/purpose
bound, generation-fenced `KeyUseLease` carrying that one identity, range,
binding, and opaque seal handle. The provider durably reserves a disjoint range
before return. Before first use, one designated local lease owner atomically
creates and fsyncs `NonceLeaseCheckpoint { binding digest, nonce_lease_id,
start, next=start, end_exclusive, reservation integrity, epoch }`.

`AllocateNonce` is the only transition that returns a counter to Seal. It is a
linearizable compare-and-swap on that checkpoint under exclusive lease-owner
identity: if `next < end_exclusive`, it durably replaces `next=x` with
`next=x+1`, fsyncs the replacement, and only then returns `(nonce_lease_id,x,
epoch)` to one caller. A failed CAS retries from durable state; a stale owner,
lost exclusivity, range exhaustion, fence/key state mismatch, or durability
failure returns a typed refusal and never a nonce. The provider independently
rejects a duplicate `(binding, nonce_lease_id, counter)` Seal. Thus two live
workers cannot issue the same nonce even without a crash; after a crash the
unconfirmed allocation is burned rather than replayed. The checkpoint is
durably reserved before first use, and rotation/restore obtains a new owner and
range rather than resurrecting an old checkpoint.

Recovery accepts a lease only when the durable reservation, provider receipt,
binding, owner epoch, and local checkpoint agree. Any missing, corrupt,
rolled-back, source-lost, or uncertain state burns the whole lease through KMS,
quarantines recovery, and withdraws admission/readiness. Typed terminal
outcomes are `NonceLeaseExhausted`, `NonceLeaseOwnershipLost`,
`NonceCheckpointUnavailable`, `NonceCheckpointConflict`,
`NonceReservationInvalid`, and `KeyGenerationUnavailable`; only an explicit
fresh lease can resume encryption. Tests cover concurrent workers and CAS
races, retry, every crash barrier (acquire, provider reserve, checkpoint
create, allocation CAS/fsync, Seal, ciphertext persist, publish, ACK),
exhaustion/wrap, rotation, restore, duplicate-use refusal, and source-loss.
No Data code receives a raw 32-byte key; therefore there is no Data
zeroizing-key buffer to expose or test. Provider conformance proves raw-key
zeroization and Data conformance proves opaque-handle containment and terminal
path refusal.

Key generations transition
`EncryptActive -> DecryptOnly -> RevocationPending -> Revoked`; no reverse
transition exists and exactly one generation per tenant/purpose is
encrypt-active. Rotation first publishes a new active generation and durable
Audit fence, then bounded workers scan a fixed manifest snapshot, decrypt with
the old generation, re-encrypt with a fresh nonce under the new generation,
verify read-back/AAD/tag, acquire a new publication pin, and advance the sealed
head+anchor tuple through the same CAS/Audit-high-water protocol from old to
new. A
durable checkpoint contains snapshot generation, last artifact ID, counts,
byte total, old/new generations, and rolling manifest digest. Crash resumes
after revalidation. A failed conversion leaks the verified old ciphertext;
it never publishes plaintext, mixed bytes, or an unverified replacement.
Retirement is forbidden until inventory, snapshots, backups within the
declared recovery window, repair queues, and all checkpoints prove zero live
old-generation references and the required audit fence is durable.

Revocation is a linearization fence. Entering `RevocationPending(F)` rejects
new leases and causes each cell to withdraw affected admission. An operation
pins its generation/fence lease in `PREPARED`; the key service cannot publish
`Revoked(R)` until durable per-cell drain receipts prove that every lease with
sequence at or below `F` is terminal. Data revalidates immediately before
prepare and records the lease sequence in the commit/Audit record; the final
revocation therefore orders after every acknowledged pre-fence operation and
before every refused post-fence operation. Emergency forced revocation
withdraws readiness, refuses all uncommitted work, and quarantines already
committed ciphertext for explicit recovery rather than decrypting through a
stale lease. After `Revoked(R)`, no new lease, encrypt, decrypt, re-encryption
commit, restore, or repair under that generation may succeed. A fence mismatch
yields `RecordSecurityError::KeyRevoked`.
Other stable failures are `PolicyUnavailable`, `PolicyDenied`,
`PolicyReceiptInvalid`, `AuditUnavailable`, `AuditReceiptInvalid`,
`KeyServiceUnavailable`, `NoEncryptActiveGeneration`, `NonceLeaseExhausted`,
`NonceLeaseOwnershipLost`, `NonceCheckpointUnavailable`,
`NonceCheckpointConflict`, `NonceReservationInvalid`, `KeyGenerationUnavailable`,
`KeyBootstrapMalformed`, `KeyBootstrapIntegrityInvalid`,
`KeyBootstrapContextMismatch`, `KeyBootstrapCatalogUnavailable`,
`KeyBootstrapAuthorizationDenied`, `AggregateClassificationMixed`,
`WalClassificationMalformed`,
`ArtifactPurposeInapplicable`, `ArtifactCommitMalformed`, `ArtifactCommitConflict`,
`ArtifactAlreadyPublished`, `ArtifactCommitHeadInvalid`,
`ArtifactPublicationContextInvalid`, `ArtifactPublicationContextMismatch`,
`ArtifactGenerationRegression`, `ArtifactOwnershipEpochRegression`,
`ArtifactFenceRegression`, `ArtifactCommitSequenceExhausted`,
`ArtifactGenerationExhausted`, `ArtifactFenceExhausted`,
`ArtifactKeyTransitionRefused`, `ArtifactPublicationAnchorUnavailable`,
`ArtifactPublicationAnchorInvalid`, `ArtifactHeadRollbackDetected`,
`ArtifactPublicationPinInvalid`, `ArtifactPublishedObjectMissing`,
`ArtifactPublicationRecoveryQuarantined`, `ArtifactPublicationPinLeaseLost`,
`ArtifactPublicationPinQuotaExceeded`, `ArtifactPublicationPinnedBytesExceeded`,
`ArtifactPublicationPinAllocationExhausted`,
`ArtifactPublicationPinRenewalHorizonExceeded`,
`ArtifactPublicationPredecessorAuditPending`,
`ArtifactPublicationTerminalRecoveryRequired`,
`ArtifactPublicationTerminalRecoveryLeaseLost`,
`ArtifactPublicationTerminalRecoveryLeaseQuotaExceeded`,
`ArtifactPublicationContentionExhausted`,
`ArtifactPublicationReconciliationBacklogExceeded`,
`ArtifactPublicationReadSnapshotStale`, `CryptoUnavailable`,
`CiphertextMalformed`, `AuthenticationFailed`, and `ContextMismatch`. There is
no plaintext, stale-key, unaudited, or best-effort fallback.

Production composition remains unrouted and not ready unless real provider
adapters attest compatible contract revisions, Policy/Audit/KMS are reachable,
an encrypt-active record and continuation generation plus sufficient durable
nonce lease, encrypted `KeyGenerationBinding`, and authenticated
`KeyBootstrapLocatorV1`/provider catalog exist, trusted Cell time is usable,
the Data-owned publication coordinator has a current term and bounded
reconciliation backlog, and every served publication locator has a fresh
matching cell-local read snapshot. An undecidable pin withdraws only that
locator while its bounded successor-reconciliation slot remains; a normal lost
CAS is terminal and cannot keep global readiness withdrawn. The latest
rotation/inventory audit must be within its capacity profile. Loss of any
condition withdraws the affected admission/readiness before accepting new work.
Independent SHA-256 and KMS-AEAD known-answer vectors, independent AAD
encoders, wrong-AAD/tag/tenant/purpose/key tests, N/N+1 final-manifest tests,
nonce duplicate/exhaustion/concurrent-CAS refusal, raw-key-containment and
provider-zeroization evidence, PDP/Audit/KMS outage, rotation/revocation at
every barrier, restart reacquisition for EncryptActive and DecryptOnly,
ciphertext-only restore, bootstrap catalog/locator tamper and source loss,
commit-head substitution/stale-CAS/crash recovery, H0-after-H1 replay across
restore/failover, immutable-context/generation/fence regressions, missing
pinned objects, pin lease/expiry/takeover, A/B/N-writer lost-CAS terminal
release, coordinator epoch re-attestation, Audit callback authentication,
cell-local snapshot expiry/refresh, and GC races at every put/verify/bind/CAS/
Audit/finalize/release barrier, corrupt backup, and locator-scoped
readiness-withdrawal campaigns are mandatory before D4.

## Fail-closed request context

Every mutation or disclosure binds:

```text
tenant and principal
operation and resource range
verified policy issuer/revision/audience/expiry
request and idempotency identity
cell, tablet, and ownership epochs
pack/classification revision where applicable
```

Missing, forged, expired, cross-tenant, or stale context fails before handler
mutation. Tenant zero follows the same route. Internal traffic uses identity-
bound encryption; persisted values, logs, snapshots, repair, and migration
artifacts carry tenant-scoped encryption references and checksums.

Admission is hierarchical by cell, tenant, database, table, operation class,
requests, bytes, concurrency, and in-flight memory. Foreground reads/writes,
replication, repair, compaction, OLAP, and pipeline work use distinct bounded
queues. Overload rejects early with typed retry information; no I/O loop sleeps
while retaining unbounded work.

## Records-contract v1 hard bounds

The following constants are part of the v1 semantic contract. Byte units are
octets; `KiB` and `MiB` are binary multiples. String limits apply to their
canonical UTF-8 bytes, not scalar-value counts. A cell profile may advertise a
lower value, but no implementation may raise these maxima:

| Constant | Hard maximum | Accounted work |
|---|---:|---|
| `MAX_REQUEST_FRAME_BYTES` | 16 MiB | complete encoded request before decode |
| `MAX_CANONICAL_IDENTIFIER_BYTES` | 256 bytes | each tenant, principal, database, table, cell, tablet, transaction, or key identifier |
| `MAX_IDEMPOTENCY_KEY_BYTES` | 128 bytes | one canonical caller retry key |
| `MAX_PRIMARY_KEY_BYTES` | 4 KiB | one opaque primary-key byte string |
| `MAX_RECORD_VALUE_BYTES` | 4 MiB | one encoded value or tombstone payload |
| `MAX_REQUEST_FINGERPRINT_FRAME_BYTES` | 4,241,449 bytes | complete server-derived canonical fingerprint preimage after set values become fixed digests |
| `MAX_SCAN_CONTINUATION_TOKEN_BYTES` | 6,022 bytes | complete authenticated opaque continuation frame |
| `MAX_RECORD_AAD_BYTES` | 4 KiB | complete canonical ContextAadV1 frame; every legal v1 purpose is at most 1,825 bytes |
| `MAX_WAL_CLASSIFICATION_SUMMARY_ENTRIES` | 1,024 | complete expanded durable record/metadata/control mutation list before PREPARED |
| `MAX_WAL_CLASSIFICATION_SUMMARY_BYTES` | 47,414 bytes | complete 1..=1,024-entry canonical WAL mutation-class summary |
| `MAX_ARTIFACT_PLAN_BYTES` | 65,640 bytes | complete purpose-valid plan including classification-binding digest |
| `MAX_ARTIFACT_FINAL_MANIFEST_BYTES` | 459,190 bytes | complete final manifest including classification-binding digest |
| `MAX_ARTIFACT_COMMIT_RECORD_BYTES` | 2,296 bytes | complete sealed commit control frame including classification-binding and transaction-identity fields |
| `MAX_AGGREGATE_ARTIFACT_COMMIT_RECORD_BYTES` | 2,040 bytes | aggregate-only commit: the general maximum less its forbidden 256-byte transaction ID |
| `MAX_AGGREGATE_ENVELOPE_OVERHEAD_BYTES` | 3,156 bytes | migration-purpose envelope: fixed 1,584-byte structural overhead plus the largest legal aggregate AAD (1,572 bytes) |
| `MAX_ARTIFACT_PUBLICATION_CONTEXT_BYTES` | 1,335 bytes | immutable locator context preimage |
| `MAX_ARTIFACT_PUBLICATION_ANCHOR_BYTES` | 177 bytes | fixed durable head/high-water anchor |
| `MAX_ARTIFACT_PUBLICATION_CAS_RECEIPT_BYTES` | 200 bytes | coordinator-only durable CAS-to-Audit proof |
| `MAX_ARTIFACT_PUBLICATION_ACCEPTED_HISTORY_BYTES` | 226 bytes | fixed accepted-high-water history row |
| `MAX_ARTIFACT_PUBLICATION_PIN_DECISION_BYTES` | 266 bytes | immutable coordinator proof of no-CAS, successful CAS, or lost CAS |
| `MAX_ARTIFACT_PUBLICATION_PIN_BYTES` | 203 bytes | fixed durable publication-pin state |
| `MAX_ARTIFACT_PUBLICATION_TERMINAL_RECOVERY_LEASE_BYTES` | 230 bytes | fixed durable, fenced, non-renewable terminal-only recovery authority |
| `MAX_PUBLICATION_ACTIVE_TERMINAL_RECOVERY_LEASES_PER_LOCATOR` | 8 rows | one current 230-byte row for at most each nonterminal locator pin; zero released rows |
| `MAX_PUBLICATION_ACTIVE_TERMINAL_RECOVERY_LEASES_PER_TENANT_PER_CELL` | 64 rows | one current row for at most each nonterminal tenant/cell pin; zero released rows |
| `MAX_PUBLICATION_ACTIVE_TERMINAL_RECOVERY_LEASES_PER_CELL` | 256 rows | one current row for at most each nonterminal cell pin; zero released rows |
| `MAX_PUBLICATION_ACTIVE_TERMINAL_RECOVERY_LEASE_BYTES_PER_LOCATOR` | 1,840 bytes | `8 * 230`; active rows only |
| `MAX_PUBLICATION_ACTIVE_TERMINAL_RECOVERY_LEASE_BYTES_PER_TENANT_PER_CELL` | 14,720 bytes | `64 * 230`; active rows only |
| `MAX_PUBLICATION_ACTIVE_TERMINAL_RECOVERY_LEASE_BYTES_PER_CELL` | 58,880 bytes | `256 * 230`; active rows only |
| `MAX_ARTIFACT_PINNED_OBJECTS` | 4,098 | data envelopes plus final-manifest and commit envelopes |
| `MAX_ARTIFACT_PUBLICATION_PINNED_MEMBER_BYTES` | 68,732,871,254 bytes | one maximum migration aggregate pin: `64 GiB + 459,190 + 2,040 + 4,098 * 3,156` |
| `MAX_ARTIFACT_PUBLICATION_PINNED_MEMBER_BYTES_PER_LOCATOR` | 549,862,970,032 bytes | eight maximum aggregate pins |
| `MAX_ARTIFACT_PUBLICATION_PINNED_MEMBER_BYTES_PER_TENANT_PER_CELL` | 4,398,903,760,256 bytes | 64 maximum aggregate pins |
| `MAX_ARTIFACT_PUBLICATION_PINNED_MEMBER_BYTES_PER_CELL` | 17,595,615,041,024 bytes | 256 maximum aggregate pins |
| `MAX_PUBLICATION_NONTERMINAL_PINS_PER_LOCATOR` | 8 | admission/reconciliation pins for one locator |
| `MAX_PUBLICATION_NONTERMINAL_PINS_PER_TENANT_PER_CELL` | 64 | admission/reconciliation pins for one tenant in one cell |
| `MAX_PUBLICATION_NONTERMINAL_PINS_PER_CELL` | 256 | admission/reconciliation pins across one cell |
| `MAX_PUBLICATION_RECONCILIATION_BACKLOG` | 256 | persisted nonterminal reconciliation work across one cell |
| `MAX_TRANSACTION_OPERATIONS` | 1,024 | reads plus writes plus conditions |
| `MAX_TRANSACTION_LOGICAL_BYTES` | 16 MiB | checked sum of every encoded key, value, condition, and schema reference |
| `MAX_COLLECTION_ITEMS` | 4,096 | any other repeated request collection |
| `MAX_SCAN_PAGE_RECORDS` | 1,000 | records requested or emitted in one page |
| `MAX_SCAN_PAGE_BYTES` | 16 MiB | logical record bytes in one response page before frame overhead |
| `MAX_TRANSACTION_RESULT_ITEMS` | 1,024 | non-scan result entries returned by one transaction |
| `MAX_TRANSACTION_RESULT_LOGICAL_BYTES` | 16 MiB | checked sum of canonical key, value/tombstone, ordinal, status, and metadata bytes across those results |
| `MAX_RESPONSE_FRAME_BYTES` | 32 MiB | complete canonical uncompressed protobuf response frame, including envelope overhead |
| `MAX_DECODE_ALLOCATION_BYTES` | 32 MiB | cumulative heap capacity newly reserved while decoding one request |
| `MAX_RESPONSE_ENCODE_ALLOCATION_BYTES` | 8 MiB | cumulative heap capacity newly reserved by the chunk-reusing response encoder |
| `MAX_CONCURRENT_REQUESTS_PER_TENANT` | 256 | admitted, not-yet-terminal requests in one cell |
| `MAX_IN_FLIGHT_REQUEST_BYTES_PER_TENANT` | 64 MiB | checked sum of declared frame bytes for those requests |
| `MAX_IN_FLIGHT_RESPONSE_BYTES_PER_TENANT` | 64 MiB | checked sum of precomputed response frame bytes reserved but not fully delivered |

All counters and byte accumulators use checked `u64` arithmetic before any
conversion to `usize`, `Duration`, or allocator capacity. Overflow is never
saturation or wraparound. Validation order is observable and fixed:

The fingerprint maximum is an exact satisfiability bound. Its ordered-set
header is `41` bytes. The largest legal item is a write or condition:
`4` ordinal + `1` kind + `4` key length + `4,096` key + `1` variant code +
`4` operand/value length + `32` digest = `4,142` bytes. Therefore
`MAX_REQUEST_FINGERPRINT_FRAME_BYTES = 41 + (1,024 * 4,142) = 4,241,449`.
`MAX_TRANSACTION_OPERATIONS` is transaction-wide: one checked counter deducts
every read, write, and condition, so three independent sets cannot each admit
1,024 items. The exact maximum, maximum-plus-one, and overflowed multiply/add
are refused before hashing, allocation, or policy evaluation. The bound permits
every otherwise legal key and operation-count combination; values are already
represented by their fixed digest.

1. Using the already authenticated transport tenant (never a body-asserted
   tenant), check the concurrency slot and checked addition of the declared
   frame bytes to the in-flight budget; refusal allocates no body buffer.
2. Reject an absent, malformed, or over-limit frame length before body decode.
   A decoder validates every length/count prefix before reserve/copy and never
   reserves from an untrusted count alone.
3. Validate canonical identifier, idempotency-key, continuation-token, and
   primary-key byte lengths, then each value length, then repeated-item counts.
4. Accumulate transaction logical bytes and decode allocation with checked
   addition; reject the first exceeded limit or arithmetic overflow.
5. Validate canonical protobuf normal form, schema, authenticated tenant/
   principal context, and revisions/epochs; derive the canonical fingerprint
   and validate its derived 4,241,449-byte preimage bound; then obtain and verify the Policy
   receipt, establish the required Audit obligation, validate idempotency
   reuse, continuation authority where present, and durability profile, in that
   order. Body-asserted identity is ignored. No `PREPARED` state or
   data-dependent evaluation occurs before all request-side checks pass.
6. Using that authorized immutable MVCC snapshot, but before `PREPARED`, a
   mutating/durable adapter call, or externally visible mutation, run the
   deterministic size-only response pass. In order, it uses checked `u64`
   addition for transaction-result items, transaction-result logical bytes,
   emitted scan records and bytes, the complete canonical response frame, and
   response-encoder allocation. It converts to `usize` only after the relevant
   maximum passes. It then reserves the exact precomputed frame bytes against
   the tenant response-credit budget. The snapshot revision, request
   fingerprint, result digest, computed lengths, and reservation form one
   prepare input; a revision conflict releases the reservation and repeats this
   entire step. Authorization therefore precedes result sizing, while every
   deterministic result refusal precedes mutation.
7. Execution may start only after that reservation. The encoder reuses bounded
   chunks and cannot buffer the complete frame merely because the frame limit is
   larger than its allocation limit. Transport backpressure retains at most the
   reserved credit; cancellation or terminal delivery releases it exactly once.
   A mutating transaction cannot discover a size/allocation refusal after
   commit. A read-only scan stops before the next record would cross its logical
   or frame bound and returns a continuation token without publishing a partial
   record. Optional compression neither changes accounting nor permits a larger
   pre-compression frame.

Stable resource refusals are:

```text
ResourceLimitError::ConcurrentRequests
ResourceLimitError::InFlightRequestBytes
ResourceLimitError::RequestFrameBytes
ResourceLimitError::CanonicalIdentifierBytes
ResourceLimitError::IdempotencyKeyBytes
ResourceLimitError::PrimaryKeyBytes
ResourceLimitError::RecordValueBytes
ResourceLimitError::RequestFingerprintFrameBytes
ResourceLimitError::ScanContinuationTokenBytes
ResourceLimitError::RecordAadBytes
ResourceLimitError::TransactionOperations
ResourceLimitError::TransactionLogicalBytes
ResourceLimitError::CollectionItems
ResourceLimitError::ScanPageRecords
ResourceLimitError::ScanPageBytes
ResourceLimitError::TransactionResultItems
ResourceLimitError::TransactionResultLogicalBytes
ResourceLimitError::ResponseFrameBytes
ResourceLimitError::DecodeAllocationBytes
ResourceLimitError::ResponseEncodeAllocationBytes
ResourceLimitError::InFlightResponseBytes
ResourceLimitError::ArithmeticOverflow
```

Each non-overflow refusal carries `limit`, `observed`, and `unit`.
`ArithmeticOverflow` instead carries the operation and accumulator name because
no truthful observed total exists. Saturation is retryable and all other
request-shape or deterministic result-shape violations are non-retryable until
the request or selected result changes. `InFlightResponseBytes` is retryable
backpressure and cannot consume a transaction ordinal.

The conformance matrix accepts every exact bound when the request and result are
otherwise valid and refuses bound plus one. It covers `u64::MAX` length/count
prefixes, identifier/idempotency/fingerprint/continuation/AAD exact and
plus-one cases, the derived 4,241,449-byte fingerprint and 6,022-byte token
limits, multi-field sum overflow, a small request frame that amplifies
beyond the decode-allocation cap, 1,025 result items, a 16-MiB-plus-one logical
result, a 32-MiB-plus-one encoded response, an encoder-allocation request above
8 MiB, and a tenant response-credit addition above 64 MiB. It also forces operation/
request/result limits in opposite orders, proves authorization refusal wins
before any data-dependent sizing result, and exercises 1,024 valid 4-MiB result
values without permitting roughly 4 GiB of returned data. Independent
size-only and streaming encoders must agree on exact frame bytes while sharing
no accumulation implementation. Scan continuation, transport cancellation,
idempotent retry after committed delivery failure, and encode-amplification
fixtures prove bounded backpressure. Every refusal releases any acquired slot
or byte reservation and leaves records, idempotency, ordinal, audit, and adapter
state unchanged.

</security_and_isolation>

<compatibility_and_cutover>

## Adapter states

```text
ORACLE -> SHADOW -> CUTOVER_READY -> OWNED_AUTHORITY -> RETIRED
                         |
                         v
                    ROLLBACK_FENCED
```

- Each cohort identifies its current source, schema, transaction semantics,
  authorization context, authority epoch, ordered migration journal, and
  comparison oracle.
- Shadow reads never serve results. Differences block cutover and retain the
  prior authority.
- Cutover advances one cohort authority epoch after journal catch-up, snapshot
  parity, and rollback preparation. Route caches and both adapters reject stale
  epochs.
- Before rollback expiry, owned commits durably capture enough ordered state to
  restore the prior adapter. Rollback fences both sides, drains in-flight work,
  proves parity at a durable barrier, then advances a higher authority epoch.
- A PostgreSQL wire facade is outside this specification until separately
  accepted. SQLx compatibility does not make the owned engine drop-in wire
  compatible.

</compatibility_and_cutover>

<observability_and_slos>

## Required signals

Telemetry exposes correlation-bound request latency, queue time, transaction
state, tablet-map staleness, consensus latency, commit ordinal, prepared age,
replica tolerance, repair backlog, checksum/media errors, compaction debt,
split/move progress, OLAP freshness, pipeline checkpoint age, rejected load,
logical/physical bytes, and per-tenant unit cost.

The target numeric objectives are in `PRD.md`. Promotion evaluates them during
steady admitted load, noisy-tenant load, repair, compaction, and mixed-version
upgrade. Averages or aggregate throughput cannot substitute for p99.9, recovery,
or isolation evidence.

</observability_and_slos>

<fault_model>

## Required campaigns

- Process death before and after every prepare, durable barrier, commit,
  acknowledgement, projection publication, and authority-epoch transition.
- Lost, duplicated, delayed, and reordered messages; asymmetric partitions;
  stale routers and leaders; voter and rack loss.
- Power loss, partial writes, full devices, corrupt logs/manifests/snapshots,
  silent bit flips, compaction interruption, and repair during foreground load.
- Tablet split/move/rebalance during reads, writes, scans, schema changes, and
  ownership transfer.
- Clock rollback, widening uncertainty, lease expiry, and adapter change.
- Forged/expired policy, cross-tenant keys, revoked encryption references,
  audit outage, deletion with retention, and noisy tenants.
- PostgreSQL migration gaps, duplicate/reordered journal entries, shadow
  divergence, cutover crash, and rollback at every pre-expiry transition.
- N/N+1 protocol and format upgrades, downgrade barriers, quorum-loss restore,
  cell loss, and repeated recovery drills.

Any split brain, acknowledged-write loss inside the declared tolerance,
cross-tenant access, partial visible generation, or silent semantic weakening is
a hard failure regardless of availability or latency results.

</fault_model>
