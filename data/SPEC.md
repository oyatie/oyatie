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

Set digests are SHA-256 over the 33-byte domain
`oyatie.data.records.operations.v1`, a zero separator, version `1`, set kind
(`read=1`, `write=2`, `condition=3`), `count:u32`, then original transaction
order. Every item starts `ordinal:u32 | item_kind:u8 | key_length:u32 | key`.
A read appends `expected_version_present:u8` and, when present, one `u64`. A
write appends `value_kind:u8` (`value=1`, `tombstone=2`), original
`value_length:u32`, and SHA-256 of the exact value bytes (the tombstone digest
is SHA-256 of empty bytes). A condition appends predicate code
(`absent=1`, `present=2`, `version_equal=3`, `value_digest_equal=4`), operand
length `u32`, and SHA-256 of the exact operand; predicates without an operand
use zero length and SHA-256 of empty bytes. Counts, lengths, and ordinals are
checked before hashing. Two independent encoders may share typed inputs and
constants, but not framing/accumulation code.

## Authenticated scan continuation v1

The public continuation is raw opaque bytes, at most 4 KiB. Its outer frame is:

```text
domain_len:u8 = 33
domain:[u8;33] = "oyatie.data.records.scan-token.v1"
separator:u8 = 0
version:u8 = 1
algorithm:u8 = 1                 # AES-256-GCM
key_generation:u64
nonce:[u8;12]                    # lease_id:u32 || checked counter:u64
ciphertext_length:u32
ciphertext:[u8;ciphertext_length]
tag:[u8;16]
```

AAD is the complete header through `ciphertext_length`. Plaintext uses the
same strict tagged-field grammar, extended with `BYTES=5` and `I64=6`, and
contains exactly these increasing tags: `0x01` tenant `ASCII`, `0x02` principal
`ASCII`, `0x03` database `ASCII`, `0x04` table `ASCII`, `0x05` schema revision
`U64`, `0x06` home cell `ASCII`, `0x07` tablet `ASCII`, `0x08` ownership epoch
`U64`, `0x09` snapshot commit ordinal `U64`, `0x0a` original request
fingerprint `DIGEST32`, `0x0b` range-start digest `DIGEST32`, `0x0c` range-end
digest `DIGEST32`, `0x0d` direction `ENUM8`, `0x0e` page-record maximum `U64`,
`0x0f` page-byte maximum `U64`, `0x10` last emitted key `BYTES`, `0x11`
`issued_earliest_ms` `I64`, `0x12` `issued_latest_ms` `I64`, and `0x13`
`expires_at_ms` `I64`. `BYTES` uses its field length and the primary-key
maximum; `I64` is exactly eight two's-complement big-endian bytes. The interval
must satisfy `issued_earliest <= issued_latest <= expires_at`. TTL is at most
15 minutes from `issued_latest_ms`; acceptance requires
`current.earliest >= issued_earliest` and `current.latest <= expires_at`.
Boundary-straddling or widened intervals return `TimeUncertain`. The token
supplies no new tenant, principal, limit, map, schema, or snapshot authority.

Keys have purpose `ScanContinuation` and states `EncryptActive`,
`DecryptOnly`, `RetirePending`, or `Revoked`. Exactly one generation is
encrypt-active; prior decrypt-only generations survive at least maximum TTL
plus the bounded request drain. Retirement first rejects new decrypt leases,
waits for issued leases to drain, and then is fenced by a durable Audit receipt.
Secrets leases a
unique nonce range per generation; Data durably checkpoints
`lease_id/start/next/end`, uses checked increment, never restores a used range,
and withdraws readiness before exhaustion. Unknown or revoked generations,
authentication failure, malformed/cross-context input, and foreign replay all
map publicly to `ScanContinuationError::Invalid`; expiry, uncertain time,
unavailable snapshot, stale epoch, key-service outage, and crypto outage map
respectively to `Expired`, `TimeUncertain`, `SnapshotUnavailable`,
`StaleEpoch`, `KeyServiceUnavailable`, and `CryptoUnavailable`. No error
reveals whether another tenant, principal, key generation, or snapshot exists.
Replaying an intact token is read-only and deterministically replays the same
snapshot/page; it cannot mutate or advance authority and remains subject to
ordinary admission.

Golden, permutation, exact/plus-one, truncation, tag-flip, ciphertext/tag
tamper, cross-tenant/principal/query/tablet/snapshot replay, expiry-boundary,
interval-widening, active-to-decrypt-only rollover, revoked/unknown generation,
nonce exhaustion, crash/restore, and repeated-token campaigns are required.

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

Data owns four implementation-free use-case ports:

```text
PolicyClient        authorize(canonical request context) -> PolicyReceipt
AuditSink           append_pre_ack(canonical event) -> DurableAuditReceipt
RecordKeySource     acquire/rotate/retire keys and nonce leases -> KeyReceipt
RecordProtection    digest/seal/open using explicit purpose and envelope -> result
```

Their draft homes are respectively `data/ports/draft/policy-client`,
`data/ports/draft/audit-sink`, `data/ports/draft/record-keys`, and
`data/ports/draft/record-protection`. Data provider adapters are
`data/adapters/draft/policy-client-policy`,
`data/adapters/draft/audit-sink-audit`,
`data/adapters/draft/record-keys-secrets`, and
`data/adapters/draft/record-protection-awslc`. The first three cannot be
implemented until their provider owners accept the exact sold faces reserved
by D1c-KG: `policy/ports/check` (`policy-check`),
`audit/ports/emission` (`audit-emission`), and
`secrets/ports/kms-use` (`secrets-kms-use`), or amend this SPEC with one exact
replacement each. The current `iam/ports/policy-cedar-api`,
`audit/ports/emission-api`, `secrets/ports/kms`, and
`secrets/ports/kms-api` do not satisfy this edge: they expose implementation or
core-backed/internal shapes, and Data MUST NOT depend on them or any provider
`core/**` package.

`PolicyReceipt` binds tenant, principal, operation, resource/range,
request fingerprint, issuer, audience, policy revision, decision, issued Cell
interval, expiry, and receipt integrity. Deny, absence, mismatch, stale
revision, expiry, or Policy outage fails before data-dependent work.
`DurableAuditReceipt` binds the same fingerprint plus policy receipt digest,
transaction/snapshot identity, result class, key generation where applicable,
and durable Audit sequence/digest. Mutations, key-state transitions, tenant
deletion, export, restore, and policy-designated privileged disclosures do not
acknowledge without it; an Audit outage aborts before commit/visibility.
Contract fixtures and known-answer vectors may test these value types, but no
fake, in-memory double, or reference oracle can satisfy production composition,
route publication, or readiness.

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
nonce:[u8;12]                             # lease_id:u32 || counter:u64
aad_length:u32 | aad:[u8;length]
ciphertext_length:u64 | ciphertext:[u8;length]
tag:[u8;16]
```

AAD is itself a strict tagged frame, domain-separated by purpose, containing
in increasing tags: tenant, database, table, tablet, ownership epoch, schema
revision, classification code/revision, primary-key digest, commit ordinal,
transaction ID, artifact generation, key ID, and key generation. Fields not
meaningful to an artifact are present with their defined zero/empty value;
they are never omitted. `aad_length` is at most 4 KiB, checked before
allocation, and SHA-256 of the exact AAD is stored in the manifest/commit
record. Header values through `aad` are authenticated. Unsupported format,
algorithm, purpose, key state, length, AAD/context mismatch, tag failure, or
trailing bytes fails before plaintext decode or publication.

`RecordKeySource` returns only a purpose/tenant-bound, generation-fenced
zeroizing key lease carrying `fence_sequence:u64`, `lease_id`, `not_after`, and
a disjoint nonce range `nonce_lease_id:u32,start:u64,end_exclusive:u64`. Data
durably checkpoints the next counter before a ciphertext can be acknowledged,
uses checked increment, and never reuses a range after crash, restore, snapshot
rollback, or ownership transfer. The raw 32-byte key may exist only in the bounded
`record-protection-awslc` call scope, in a non-cloneable buffer whose `Drop` and
all error/cancellation paths are verified to zeroize; it is never logged,
serialized, paged into a request/result, or retained by an async task.

Key generations transition
`EncryptActive -> DecryptOnly -> RevocationPending -> Revoked`; no reverse
transition exists and exactly one generation per tenant/purpose is
encrypt-active. Rotation first publishes a new active generation and durable
Audit fence, then bounded workers scan a fixed manifest snapshot, decrypt with
the old generation, re-encrypt with a fresh nonce under the new generation,
verify read-back/AAD/tag, and CAS the artifact manifest from old to new. A
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
`CryptoUnavailable`, `KeyBufferUnavailable`, `CiphertextMalformed`,
`AuthenticationFailed`, and `ContextMismatch`. There is no plaintext, stale-key,
unaudited, or best-effort fallback.

Production composition remains unrouted and not ready unless real provider
adapters attest compatible contract revisions, Policy/Audit/KMS are reachable,
an encrypt-active record and continuation generation plus sufficient durable
nonce lease exist, trusted Cell time is usable, recovery has no quarantine,
and the latest rotation/inventory audit is within its capacity profile. Loss
of any condition withdraws admission/readiness before accepting new work.
Known-answer AES-GCM/SHA-256 vectors, independent AAD encoders, wrong-AAD/tag/
tenant/purpose/key tests, nonce duplicate/exhaustion, key-buffer remanence,
PDP/Audit/KMS outage, rotation/revocation at every barrier, crash-resume,
ciphertext-only restore, corrupt backup, and readiness-withdrawal campaigns are
mandatory before D4.

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
| `MAX_REQUEST_FINGERPRINT_FRAME_BYTES` | 8 KiB | complete server-derived canonical fingerprint preimage after set values become fixed digests |
| `MAX_SCAN_CONTINUATION_TOKEN_BYTES` | 4 KiB | complete authenticated opaque continuation frame |
| `MAX_RECORD_AAD_BYTES` | 4 KiB | complete canonical AEAD associated-data frame |
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
   and validate its 8-KiB preimage bound; then obtain and verify the Policy
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
plus-one cases, multi-field sum overflow, a small request frame that amplifies
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
