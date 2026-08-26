---
doc_class: Owner-SPEC
owner: storage
status: Target
date: 2026-08-25
---

# Storage behavior and contract

<maturity>

The target below is normative design. The protected tree currently supplies
typed Rust contracts and in-memory references only. A feature is not available
until its implementation and promotion evidence land.

</maturity>

<topology>

## Roles and authority

```text
global bucket directory
        |
        v
bounded cell
  gateway pool ---- cached map ---- placement quorum
       |                                 |
       +---------- metadata tablets -----+
                         |
                  storage data nodes
                         |
                repair/scrub workers
```

- The consensus-backed global directory maps `(tenant, bucket)` to a home cell
  and publishes a signed, time-bounded cross-cell ownership lease containing
  the ownership epoch, home cell, allowed recovery cells, pack/overlay digest,
  lease deadline, and clock-uncertainty bound. Cells cache that record, so the
  directory is not in the normal payload path.
- The placement quorum owns membership, device state, failure-domain rules, map
  epochs, drain, and fencing.
- Metadata is range-partitioned by `(tenant, bucket, key, version_sort_key)` so
  ordered LIST does not require a fleet-wide hash scan. Each tablet is a three-
  or five-replica consensus group over MVCC and a durable local engine.
- Data nodes append immutable extents and accept writes only for current epochs.
- Gossip distributes health suspicion, latency, and utilization hints; it never
  changes authoritative ownership.

</topology>

<identity_and_records>

## Logical records

An object version records at least:

```text
tenant, bucket, key, version_id, generation
state: PREPARED | COMMITTED | DELETING
size, etag semantics, client checksums, internal checksum
logical chunk manifest, layout generation
encryption key reference, retention, legal hold
creation revision, idempotency token, request fingerprint
audit requirement, audit receipt id/digest or NOT_REQUIRED
```

A multipart upload records:

```text
upload_id, tenant, bucket, key, initiating principal
state: OPEN | COMPLETING | COMMITTED | ABORTING | ABORTED
request fingerprint, provisional quota, retention/legal-hold intent
parts: part_number -> checksum, size, logical chunks, receipt generation
completion generation, audit receipt, expiry
```

A deletion record moves monotonically through:

```text
LIVE -> DELETE_PREPARED -> TOMBSTONED -> GC_ELIGIBLE -> PURGED
```

Versioned deletion creates a committed delete marker; it never silently mutates
an older immutable version. Physical bytes become `GC_ELIGIBLE` only when no
live version, reader lease, retention rule, legal hold, snapshot, or rollback
generation references them.

An audit-gated operation records:

```text
operation_id, idempotency_key, request fingerprint
tenant, principal, action, target generation/range
verified decision digest, policy revision
state: AUDIT_PENDING | AUDIT_RECEIPTED | BOUND
durable receipt id/digest/generation
```

The audit authority persists receipts idempotently by operation id. Storage
records the verified receipt binding in its authoritative transaction; a retry
with the same operation id and fingerprint reuses the receipt, while a changed
fingerprint conflicts.

A tenant deletion records:

```text
deletion_id, tenant, idempotency key, request fingerprint
tenant authority epoch, pack/policy/key revisions
snapshot id, barrier revision/vector, coverage-manifest digest
held/retained disposition, key-generation set, erasure receipts
state: REQUESTED | WRITE_FROZEN | SNAPSHOTTED | DISPOSITIONED |
       ERASURE_PREPARED | CRYPTO_ERASED | PROOF_PUBLISHED |
       RECLAIMING | COMPLETE
```

Every transition is a durable compare-and-swap against the same deletion id,
authority epoch, and snapshot lineage. A changed fingerprint conflicts instead
of starting another deletion.

A logical chunk maps to one immutable layout generation:

```text
REPLICATED -> EC_PENDING -> EC_ACTIVE
```

Physical addresses never become the public object identity. CAS uses
`(tenant_id, blake3(payload))`; ordinary objects use bucket/key/version and a
generation. Identical bytes in two tenants do not imply shared storage or keys.

</identity_and_records>

<write_protocol>

## PUT

1. Gateway authenticates the principal, evaluates policy, and reserves tenant
   admission capacity.
2. Gateway resolves the metadata tablet and placement map from caches.
3. The tablet records a `PREPARED` version with an idempotency token and current
   placement epoch.
4. Gateway streams chunks directly to selected data nodes.
5. Data nodes checksum, append, durably flush according to policy, and return
   generation-bound receipts.
6. The tablet validates durability receipts. When policy classifies the PUT as
   audit-gated, it creates or resumes `AUDIT_PENDING`, obtains the audit
   authority's durable receipt bound to tenant, principal, action, object
   generation, idempotency key, fingerprint, policy revision, and durability-
   receipt digest, then verifies and records `AUDIT_RECEIPTED`. Audit refusal or
   uncertainty leaves the version invisible and unacknowledged.
7. The tablet atomically commits visibility, the idempotency result, and the
   audit receipt binding (`BOUND`), or the explicit `NOT_REQUIRED` decision.
8. GET and LIST expose only `COMMITTED` versions.
9. An orphan collector reclaims abandoned prepared bytes after the retry and
   reader lease windows expire.

Retries with the same idempotency token and request fingerprint return the same
result. A token reused with a different fingerprint fails with a conflict.
Conditional mutation compares against the authoritative committed generation.
After a crash, recovery looks up any `AUDIT_PENDING` operation by operation id,
verifies an existing durable receipt, and either completes the same atomic
commit or leaves the object uncommitted for retry/collection; it never fabricates
a receipt from a local log or acknowledges physical bytes alone.

## Multipart upload

1. Initiate allocates an `OPEN` upload, binds the principal, policy revision,
   retention intent, idempotency fingerprint, expiry, and provisional quota.
2. UploadPart replaces only the named part generation through compare-and-swap;
   data receipts bind part number, checksum, size, placement epoch, and upload.
3. Complete atomically moves `OPEN -> COMPLETING`; it validates the ordered part
   manifest, conditional object generation, quota, trusted-clock retention,
   legal hold, durability receipts, and required pre-ACK audit receipt.
4. The metadata tablet commits the object version and
   `COMPLETING -> COMMITTED` in one transaction. A retry with the same
   fingerprint returns that committed result.
5. Abort atomically moves `OPEN -> ABORTING -> ABORTED`, releases provisional
   quota, and schedules unreferenced parts for collection. Complete and Abort
   race through the same upload generation, so exactly one can win; the loser
   receives a typed conflict and cannot undo the winner.
6. An expired `OPEN` or recoverably stuck intermediate state is resolved by a
   leased sweeper using the same compare-and-swap transitions. It never infers
   completion from physical part presence.

## DELETE, lifecycle, and tenant deletion

1. A delete or lifecycle worker prepares against the current version generation
   and re-evaluates policy, retention, and legal hold with the trusted cell clock.
2. If the operation is privileged or irreversible, the audit boundary durably
   persists a receipt bound to tenant, principal, mutation, generation,
   idempotency key, and policy revision before storage acknowledges success.
3. The metadata tablet commits a delete marker or tombstone. Retries return the
   same marker; a changed fingerprint or a concurrent legal-hold/retention
   generation fails closed.
4. A collector moves bytes to `GC_ELIGIBLE` and then `PURGED` only after the
   reference, lease, hold, snapshot, and rollback checks succeed. Collection
   failure leaks space, not visibility or retention guarantees.
5. Tenant deletion creates `REQUESTED`, then advances the tenant authority to a
   higher write-frozen epoch before recording `WRITE_FROZEN`. Gateways, metadata
   tablets, repair, lifecycle, and old epochs reject new tenant mutations.
6. At that epoch it obtains a bucket/tablet barrier, persists its snapshot id,
   revision vector, pack/policy/key revisions, and moves to `SNAPSHOTTED`. It
   enumerates every version, chunk, snapshot, hold, retention deadline, and key
   generation into a signed coverage manifest before `DISPOSITIONED`.
7. Held or retained data remains frozen and enumerated until its constraint
   clears. The controller revalidates the unchanged authority epoch, snapshot
   lineage, coverage digest, key set, trusted time, authorization, and durable
   pre-erasure audit receipt before `ERASURE_PREPARED`; no key is erased sooner.
8. KMS erasure is idempotent by deletion id and key generation. Each durable KMS
   receipt advances covered keys monotonically; only a complete receipt set
   permits `CRYPTO_ERASED`. A crash after erasure recovers receipts from the KMS
   idempotency boundary and continues forward while the tenant stays frozen.
9. A signed proof binds the coverage manifest and every erasure/disposition
   receipt before `PROOF_PUBLISHED`. Physical collection then advances through
   `RECLAIMING -> COMPLETE`; completion acknowledges only after its required
   audit receipt is durable.

Cancellation may roll back only before `ERASURE_PREPARED`, by publishing a
higher tenant authority epoch after proving that no erasure occurred. From
`ERASURE_PREPARED` onward the workflow is forward-only: outage or ambiguity
keeps the tenant frozen and retries idempotently rather than restoring access to
possibly erased bytes.

</write_protocol>

<read_and_list>

## GET, range read, and LIST

- GET resolves a committed metadata version, pins its layout generation for the
  reader lease, then reads a verified replica or reconstructs from valid shards.
- Before a privileged GET emits headers, metadata, or the first payload byte,
  it creates or resumes the audit-gated operation and obtains a durable receipt
  bound to the principal, policy revision, request fingerprint, object/layout
  generation, requested range, and operation id. Only a verified receipt moves
  the boundary to `BOUND` and opens the stream. Audit outage/refusal releases
  the reader lease and returns a typed refusal with zero disclosed object bytes.
  Retried range streams reuse the same receipt only when every binding matches.
- Every returned range is verified against the internal checksum tree; corrupt
  sources are quarantined and enqueue repair.
- The first LIST page asks the bucket routing tablet for a bucket-wide read
  barrier. The barrier orders all mutations completed before the request,
  freezes a tablet-lineage epoch for the scan, and obtains a linearizable MVCC
  read index from every intersecting range.
- LIST continuation tokens are authenticated and bind tenant, bucket, ownership
  epoch, prefix, delimiter, bucket barrier revision, tablet-lineage epoch, the
  revision vector `(range_id, read_index, range_generation)`, last key/version,
  and expiry. Later pages scan exactly that vector, not each tablet's newest
  state.
- Splits and moves preserve parent lineage redirects and MVCC history until the
  longest token lease expires. A changed bucket ownership epoch, expired
  history, untranslatable lineage, or invalid token returns a typed
  restart-required error; it never resumes approximately or silently duplicates
  or omits a key. It does not expose a partially committed version.
- Home-cell object mutation and listing provide strong read-after-write
  behavior. Cross-region asynchronous replicas expose their explicit lag and do
  not advertise active-active semantics.

</read_and_list>

<layout_transition>

## Replication-to-erasure transition

1. Seal an immutable segment or chunk group.
2. Read and checksum the source layout.
3. Generate data and parity shards.
4. Write shards across declared failure domains and flush durably.
5. Verify every shard manifest and reconstruct the source checksum.
6. Compare-and-swap the metadata layout generation to `EC_ACTIVE`.
7. Retain old replicas through the reader/rollback grace period.
8. Collect old replicas only after no reader can hold the old generation.

Failure before publication leaves the replicated generation authoritative.
Failure after publication but before cleanup leaks space, never acknowledged
data. Small objects are packed into immutable sealed segments before encoding.

</layout_transition>

<durability_and_repair>

## Durability state machine

```text
healthy -> under_replicated -> reconstructing -> healthy
   |              |                 |
   +----------> degraded <----------+
                     |
          checksum_suspect -> lost
```

`misplaced` and `stale_replica` are repairable side states. The controller
prioritizes by remaining failure tolerance, not object age or queue arrival.
Repair, scrub, lifecycle deletion, compaction, and erasure encoding have bounded
budgets and yield to admitted foreground traffic without starvation. Every
transition emits an auditable reason, owner lease, retry count, and convergence
result.

</durability_and_repair>

<placement_and_fencing>

## Placement

Placement is a deterministic function of object group, map epoch, installed
pack-id, signed storage-overlay revision/digest, jurisdiction, storage policy,
device weights/classes, and hierarchical failure domains. The packs adapter
resolves the installed pack-id; storage ingests and verifies its own
content-addressed overlay. Gateways can calculate candidates locally, but only
the consensus-published map and matching unexpired overlay authority authorize a
write. Storage nodes reject stale map, ownership, or overlay epochs. Placement,
replication, repair, drain, erasure conversion, and cross-cell migration reject
unknown, stale, expired, rolled-back, or jurisdiction-incompatible overlays.
Drain and rebalance use explicit handoff records and do not publish the new
generation before verified copy.

Dead, unreachable, drained, rebuilding, full, and read-only are distinct device
states. A partitioned node cannot promote itself from a health hint.

## Cross-cell promotion and fencing

1. A consensus-backed promotion authority compare-and-swaps the bucket record
   from `ACTIVE(epoch N, cell A)` to `FENCING(epoch N+1, cell A -> cell B)`.
   Operator intent or failure suspicion alone cannot authorize promotion.
2. It revokes cell A through available network/KMS/placement fencing. If A is
   unreachable, promotion waits until A's signed ownership lease deadline plus
   the declared clock-uncertainty window; a cell with unavailable or uncertain
   trusted time fails closed for writes.
3. Cell B verifies the selected pack overlay, replication watermark, and
   recovery manifest, then the authority publishes
   `ACTIVE(epoch N+1, cell B)`. Every gateway, metadata tablet, and data receipt
   must carry epoch N+1; A rejects it as foreign and cannot renew epoch N.
4. Rollback is another forward compare-and-swap to epoch N+2 after the same
   fencing and data-verification protocol. Ownership epochs never decrement and
   an old home is never simply re-enabled.

Fault campaigns keep cell A live but asymmetrically unreachable while promoting
B, skew clocks to the declared bound, restore A after promotion, and prove that
at most one cell can acknowledge a mutation. Recovery RTO ends only after this
protocol and destination read/write probes complete.

</placement_and_fencing>

<wire_contract>

## Canonical and compatibility APIs

- Canonical semantic API: versioned protobuf packages over the platform Connect
  gateway, with streaming bodies, deadlines, cancellation, checksums,
  idempotency, and typed error details.
- Supported S3 facade: a sold gateway surface translating declared S3 semantics
  into that canonical transaction model and security path. Compatibility tests
  cover conditional writes, multipart, versions, ordered list, retention, legal
  hold, errors, and version policy. It is distinct from removable external
  S3-compatible backend/migration adapters.
- Internal role protocol: versioned protobuf with mixed-version negotiation.
  Unknown fields are preserved during the supported rolling-upgrade window.
- Block service: a separately promoted facade with volume create/attach/
  snapshot/delete state machines. Existing Rust block types do not mean the
  service is sold.

The historical `CloudFilesystem` and `StorageTenantCellGuardrail` Rust names
are source-compatibility inventory, not wire contracts or advertised services.
They accept no new consumers and may leave only through a separately versioned
consumer migration or an owner-boundary amendment. They do not define Drive;
Drive owns people-facing file and folder semantics over its own blob port.

There is no additional hand-authored JSON/REST source of truth; the accepted S3
HTTP facade derives from the protobuf semantic model. There is no per-byte
cross-runtime FFI boundary.

</wire_contract>

<io_and_admission>

## I/O and workload isolation

The backend contract provides aligned read, append/write, durable flush,
discard, and health/error classification. Buffered files are the development
baseline; direct `io_uring` and SPDK are hardware profiles that must pass the
same durability and recovery suites.

Admission is hierarchical: cluster, cell, tenant, bucket, operation class, and
device. It limits requests, bytes, IOPS, concurrency, in-flight bytes, metadata
QPS, and multipart state. Device queues distinguish latency reads, writes,
replication, repair, scrub, erasure work, compaction, and deletion. Work that
cannot meet a deadline fails early with a retryable overload response.

</io_and_admission>

<security_and_recovery>

## Security, upgrade, and recovery

- All external operations carry tenant, principal, authorization decision,
  request/correlation ID, and policy revision. A decision is accepted only when
  its issuer signature, audience, tenant/principal/request binding, policy
  revision, and expiry verify; a caller-constructed allowed-surface list is not
  proof. Default deny occurs before data access.
- Data uses per-tenant envelope encryption with versioned key references.
  Retention, legal hold, ownership leases, and credential expiry use the trusted
  cell clock and fail closed when time is unavailable or outside its uncertainty
  bound. They are checked in the authoritative metadata commit.
- Privileged reads cross the audit boundary before any object disclosure;
  irreversible mutations acknowledge only after a durable audit receipt is
  verified and bound to the committed generation and idempotency record. Audit
  recovery follows the durable `AUDIT_PENDING -> AUDIT_RECEIPTED -> BOUND`
  record above. Unavailability is a refusal or resumable uncommitted operation,
  never a deferred best-effort log.
- On-disk records and protocols carry explicit versions. Supported releases run
  mixed N/N+1, with declared rollback barriers and dual-reader windows.
- Metadata snapshots and immutable segment manifests support point-in-time and
  quorum-loss recovery without trusting one local LSM.
- Offline tooling can verify metadata-to-layout reachability, checksums,
  uncommitted orphans, and retention invariants before repair publication.
- Region recovery uses the cross-cell ownership protocol above; snapshots or
  replica lag can satisfy recovery data needs but cannot bypass fencing. The RTO
  and RPO clocks and completion boundaries are those declared in `PRD.md`.

</security_and_recovery>

<observability>

## Required signals

The service exposes queue time by stage, map staleness, quorum latency,
prepared-but-uncommitted count, repair backlog by durability risk, remaining
failure tolerance, media/checksum errors, device tail latency, compaction debt,
erasure-conversion lag, throttling, hot partitions, and logical/physical/
replicated/billable bytes. Promotion SLIs are defined in `PRD.md`.

</observability>
