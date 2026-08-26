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
| Analytics process boot | `analytics_listener_unrouted` with authenticated Connect-listener absence as detail; exit nonzero and publish no readiness |

The identity and detail are bounded static strings: no request, tenant,
credential, endpoint, or adapter payload is interpolated. Compatibility error
variants remain unchanged in D1b-N; only their operator-facing content changes.
Tests assert the complete semantic identity, prove the old `IP-*` tokens are
not emitted, and retain decision references only in comments, rustdoc, and
package metadata. The rename neither installs an adapter/listener nor changes
the current no-production-readiness claim. The analytics binary's current
exit-zero “boot complete” behavior is not a contract: D1b-N replaces it with
the fail-closed exit above so an unrouted process cannot signal success.

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

1. The facade authenticates, obtains a verified PDP decision, and admits the
   request against tenant and cell budgets.
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

The durable engine uses append-only transaction/log records, checksummed
immutable runs or segments, generation-bound manifests, compaction, snapshots,
and explicit durable barriers. An embedded LSM may implement local persistence;
it does not supply distributed ownership or consensus.

A local write is not acknowledged merely because it reached userspace or the
operating-system page cache. Recovery validates log framing, checksums,
generation monotonicity, schema compatibility, and manifest reachability before
serving. Ambiguous or corrupt state is quarantined rather than guessed.

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
| `MAX_PRIMARY_KEY_BYTES` | 4 KiB | one opaque primary-key byte string |
| `MAX_RECORD_VALUE_BYTES` | 4 MiB | one encoded value or tombstone payload |
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
3. Validate canonical identifier and primary-key byte lengths, then each value
   length, then repeated-item counts.
4. Accumulate transaction logical bytes and decode allocation with checked
   addition; reject the first exceeded limit or arithmetic overflow.
5. Validate canonical encoding, schema, tenant/context, authorization/audit
   evidence, revisions/epochs, idempotency fingerprint, and durability profile
   in that order. No `PREPARED` state or data-dependent evaluation occurs before
   all request-side checks pass.
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
ResourceLimitError::PrimaryKeyBytes
ResourceLimitError::RecordValueBytes
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
prefixes, multi-field sum overflow, a small request frame that amplifies beyond
the decode-allocation cap, 1,025 result items, a 16-MiB-plus-one logical result,
a 32-MiB-plus-one encoded response, an encoder-allocation request above 8 MiB,
and a tenant response-credit addition above 64 MiB. It also forces operation/
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
