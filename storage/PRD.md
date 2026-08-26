---
doc_class: Owner-PRD
owner: storage
status: Active
last_interviewed: 2026-08-25
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Storage product requirements

<product_boundary>

`storage/` is the cloud's durable-bytes capability. It sells one owned object and
content-addressable service through native Connect and supported S3-compatible
facades, provides the pipeline CAS, and may sell an EBS-class block service
after the object engine clears its production gates.

Archive lifecycle is object behavior; volume snapshots are block behavior only
within a separately promoted block service. Storage does not own relational or
analytical records (`data/`), an EFS-like file-service product, mixed
cross-product aggregates, end-user Drive, mail, imaging, recording, search, or
a wall-clock consistency service. Apps use a blob port and treat the Oyatie
storage facade as one adapter among S3-compatible and on-premises alternatives.

</product_boundary>

<users>

- Cloud tenants need durable object bytes, conditional mutation, ordered list,
  versioning, retention, lifecycle, replication, and tenant-visible usage.
- S3 clients need a supported compatibility facade with explicit coverage and
  version policy, not a migration-only endpoint or a second storage model.
- First-party apps need the same sold facade and identity/policy path as any
  external tenant; they do not link storage core crates in-process.
- Pipeline needs tenant-scoped CAS for build inputs and artifacts.
- Operators need bounded cells, automated repair, safe drain, online upgrade,
  disaster recovery, and exact durability-risk telemetry.

</users>

<requirements>

## Object and CAS

- Provide tenant-scoped buckets, immutable object versions, multipart upload,
  conditional writes, lexicographically ordered listing, range reads, deletion
  markers, lifecycle, retention, legal hold, and idempotent retries.
- Keep logical object identity independent from physical placement. CAS identity
  is tenant plus BLAKE3 digest; ordinary object identity is tenant, bucket, key,
  version, and generation. Wall time is metadata, never ownership.
- Acknowledge a write only after the configured durability policy is satisfied
  and an authoritative metadata commit makes the version visible.
- Prevent cross-tenant deduplication by default. Encryption, accounting,
  deletion, and equality observations remain tenant scoped.

## Scale and failure containment

- Partition the service into bounded cells. A global directory maps a bucket to
  a home cell; normal object I/O stays off the global control path.
- Let compute/gateway roles scale independently from persistent metadata and
  data roles. Capacity growth must not require scaling all roles together.
- Use versioned, consensus-backed placement and metadata authority. Gateways
  cache deterministic maps; gossip carries health hints, never write ownership.
- Fence stale writers and publish ownership/layout changes only after data is
  copied, durably verified, and atomically committed.
- Resolve the installed tenant pack-id and signed, content-addressed
  storage-overlay revision before any new placement or movement. Placement,
  replication, repair, drain, erasure conversion, and cross-cell migration fail
  closed on missing, stale, expired, or jurisdiction-incompatible authority.
- Detect, prioritize, and repair under-replication, corruption, misplacement,
  and incomplete layout transitions without unbounded repair storms.

## Interfaces and portability

- The canonical semantic contract is protobuf over the platform Connect/H3
  gateway. A supported S3-compatible facade translates that same transaction
  model and remains a sold surface; external S3-compatible storage backends are
  removable migration adapters, not the internal model or a second truth.
- Ship one signed, reproducible Rust distribution with independently deployable
  gateway, metadata, placement, data, repair, and development roles.
- Require no third-party PostgreSQL, etcd, Kafka, Redis, or repair coordinator
  as storage state authority. Normal operation still consumes owned platform
  IAM/PDP, packs, audit, trusted-clock, and optional KMS/HSM services through
  fail-closed ports; telemetry and archival integrations remain optional.
- Keep buffered file I/O, direct `io_uring`, and optional SPDK behind a storage
  backend port. Select a backend by measured hardware profile, not doctrine.

## Security, isolation, and operations

- Authenticate at the gateway and authorize every request through the normal
  fail-closed IAM/policy path before storage mutation or disclosure. Decisions
  carry verifiable issuer, policy revision, audience, expiry, and request binding;
  caller-constructed allow lists are never authorization proof.
- Use internal mTLS, per-tenant envelope encryption, rotatable key references,
  immutable audit receipts for privileged operations, a trusted cell-clock
  source for lease and retention expiry, and zeroized secret material. An
  irreversible or privileged operation that requires audit does not acknowledge
  until the audit receipt is durably persisted and bound to its idempotency key.
- Tenant deletion freezes new writes, honors retention and legal hold, proves
  coverage of every metadata version and chunk, performs authorized
  crypto-erasure, and only then permits asynchronous physical reclamation.
- Enforce hierarchical asynchronous admission by cell, tenant, bucket,
  operation class, IOPS, bandwidth, concurrency, and in-flight bytes. Reject
  overload early with retryable behavior instead of sleeping in device loops.
- Support format negotiation, mixed-version rolling upgrades, point-in-time
  metadata recovery, offline consistency checks, scrub, drain, and cell-loss
  runbooks before production promotion.

</requirements>

<promotion_targets>

These are target objectives, not claims about the present in-memory libraries.

| Objective | Production target | Required evidence |
|---|---:|---|
| Standard-class object durability | 99.999999999% annual | Fault model, scrub/repair history, restore drills |
| Home-cell GET/PUT availability | 99.99% monthly | Admitted-load SLI excluding caller faults |
| Regional multi-cell availability | 99.999% monthly after failover phase | Cell-loss exercise and routing evidence |
| Acknowledged-write RPO inside the configured quorum | 0 | Power-cut and quorum-loss campaign |
| Async cross-region replication RPO | at most 300 seconds | Lag SLI and region-loss drill |
| In-cell process/node recovery time | at most 300 seconds | From authoritative fault declaration to admitted GET/PUT meeting the home-cell SLO |
| Home-region cell-loss recovery time | at most 900 seconds | From source-cell fencing decision to a replacement cell accepting fenced reads and writes |
| Region-loss recovery time for replicated buckets | at most 3,600 seconds | From region-loss declaration to a promoted home region accepting fenced reads and writes |
| Warm in-cell first byte, object at most 1 MiB | p99.9 at most 100 ms at 80% admitted load | Hardware-profile benchmark |
| Critical durability-risk detection | p99 at most 60 seconds | Corruption and replica-loss injection |

RTO starts only after the named promotion authority declares the fault; fault
detection latency is measured separately. RTO ends only when a higher
cross-cell ownership epoch is committed, the old writer is externally fenced or
its signed lease plus clock-uncertainty window has expired, the destination
passes read/write probes at admitted load, and the operator-visible recovery
record is durable. The 300-second async RPO bounds which replicated writes may
be absent after region loss; it does not relax split-brain fencing.

</promotion_targets>

<success_and_failure>

Success means the owned Rust engine serves the canonical facade, survives the
declared failure model, automatically converges after faults, clears the target
SLIs, and can remove every transitional storage backend without changing an app
core.

Failure includes any of the following: two authorities accept writes for one
generation; an acknowledged object becomes unreadable within the configured
failure tolerance; stale routing bypasses fencing; LIST omits a committed key;
retention or legal hold can be bypassed; a stale or unknown pack overlay permits
byte movement; a privileged action acknowledges before audit persistence; one
tenant can starve another; repair amplifies an outage; an upgrade requires
stopping a cell; or a facade/backend adapter becomes the product's source of
truth.

Required adversarial campaigns cover process death at every write state, power
loss around durable flush, asymmetric partitions, stale maps, full devices,
partial writes, silent corruption, simultaneous rack loss and repair, noisy
tenants, clock jumps, mixed-version upgrades, and metadata snapshot restore.
They also cover an old home cell that remains live but unreachable during
promotion, LIST pagination across tablet split/move, Complete-versus-Abort and
legal-hold races, and tenant-deletion proof gaps.

</success_and_failure>

<non_goals>

- Building a relational database, OLAP engine, vector store, EFS-like file
  service, Drive product, or global active-active namespace with unspecified
  conflict semantics.
- Forking a third-party object store as the permanent core, or putting a
  per-payload Go/Rust FFI boundary in the destination data path.
- Claiming production durability, horizontal scale, or hyperscaler readiness
  from an embedded LSM, a benchmark, an erasure-coding library, or Rust alone.

</non_goals>
