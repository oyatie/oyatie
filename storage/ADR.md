---
doc_class: Owner-ADR
owner: storage
status: Accepted
date: 2026-08-25
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Storage decisions in force

This file specializes ADR-0719 for `storage/`. It is current owner law, not a
claim that the target implementation has landed.

<current_state>

## Evidence at the protected base

| Surface | What exists | Maturity |
|---|---|---|
| `core/domain` | Typed bucket, object, volume, filesystem, archive, snapshot, residency, encryption, and provider-port models backed by an in-memory catalog | Contract/reference only; oversized and structurally stale |
| `core/object-store-kernel` | Tenant-scoped BLAKE3 CAS, caller-supplied WORM/audit fields, bounded payload traits, conformance suite, and in-memory store | Syntactic/self-asserted reference model; not trusted-clock, audit-receipt, persistent, or distributed enforcement |
| `ports/object-api`, `ports/block-api` | Rust request validation, caller-constructed authorization projection checks, idempotency, status/error projection | Syntactic reference boundaries; no verified PDP provenance and not the canonical proto/Connect facade |
| `adapters/s3`, `adapters/oci` | Validated provider command/receipt projections | No network or durable backend execution |
| Deployment/SLO artifacts | No storage-owned deployment or SLO source is loaded yet | Future artifacts must be reconciler- or IR-generated |

</current_state>

<boundary>

## Decision: bytes, not records or apps

- **achieves:** one owner for byte durability and no shadow database or Drive
  product inside the cloud substrate.
- **origin:** the old storage PRD mixed object, block, file, archive, backup,
  end-user media, SQL metadata, and provider manifests.
- **rule:** `storage/` MUST own durable object/CAS bytes and MAY expose an
  EBS-class block facade only after its separate promotion; it MUST NOT own
  relational/analytical records, search, Drive, mail, imaging, recordings, or
  wall-clock identity.
- **ensure:** dependencies and review reject SQL/query engines and app business
  models in storage core; app cores reach storage only through their own blob
  port and an adapter to the sold facade.
- **overturn_when:** a founder-accepted owner-boundary decision replaces this
  allocation and updates the affected owners in the same change.

</boundary>

<owned_engine>

## Decision: owned Rust core, compatibility at adapters

- **achieves:** a self-contained service whose correctness, formats, and
  operational lifecycle are controlled in-tree without making a vendor fork the
  product.
- **origin:** SeaweedFS, external S3/OCI backends, and NativeLink are useful
  compatibility and migration references, but none supplies Oyatie's full
  authority, isolation, upgrade, and proof model merely by being embedded or
  rewritten.
- **rule:** the destination engine MUST be implemented as owned Rust behind
  stable storage ports; third-party stores MAY exist only as bounded adapters,
  migration sources, or differential-test oracles and MUST be removable without
  changing the canonical contract.
- **ensure:** core crates have no provider SDK or foreign-runtime dependency;
  conformance runs against the owned engine and each retained adapter; the plan
  contains an explicit external-backend adapter retirement gate without
  retiring the supported S3 wire facade.
- **overturn_when:** measured evidence shows an external engine satisfies every
  owner requirement with lower lifecycle risk and a same-wave decision records
  licensing, control, migration, and exit guarantees.

</owned_engine>

<topology>

## Decision: cells and separate roles

- **achieves:** horizontal growth with bounded blast radius and independent
  scaling of request compute, authority, capacity, and repair.
- **origin:** one process or one globally uniform cluster couples parser bugs,
  compaction, repair, metadata leadership, and device failure.
- **rule:** storage MUST be cell-based and ship one signed distribution with
  independently deployable gateway, metadata, placement, data, and repair roles;
  development MAY run them together, while production role isolation remains the
  default.
- **ensure:** capacity and failure tests scale or kill each role independently;
  a cell has explicit node, tablet, repair, and quorum limits; adding capacity
  normally adds devices or cells rather than enlarging one global consensus
  domain without bound.
- **overturn_when:** failure-injection and unit-cost evidence proves a different
  topology bounds equal or less damage while preserving independent elasticity.

</topology>

<authority_and_consistency>

## Decision: consensus authority, gossip hints, cached routing

- **achieves:** no controller RPC on ordinary payload I/O without allowing stale
  routers or network partitions to create two writers.
- **origin:** a flat consistent-hash or gossip ring answers candidate placement,
  not drain state, capacity, failure domains, ownership generation, or fencing.
- **rule:** authoritative membership, placement maps, metadata tablets, handoff,
  and fencing MUST be versioned and consensus-backed; gossip MAY disseminate
  suspicion and telemetry but MUST NOT grant ownership; gateways MUST cache maps
  and storage nodes MUST reject stale write epochs.
- **ensure:** partition tests prove only the current generation commits; handoff
  publishes after copy, flush, and verification; normal GET/PUT traces contain
  no global-controller hop.
- **overturn_when:** a formally specified alternative proves single-writer
  safety, ordered listing, conditional mutation, and stale-router fencing under
  the same partition model.

</authority_and_consistency>

<residency_authority>

## Decision: installed pack selects storage residency law

- **achieves:** every byte placement and movement obeys the tenant's current
  jurisdiction and storage policy without making a git file or gateway hint
  runtime authority.
- **origin:** device-aware placement alone can replicate or repair into a
  forbidden region; ADR-0719 makes the central pack-id the install fact and the
  storage owner responsible for its own overlay.
- **rule:** storage MUST resolve the tenant's installed pack-id through the
  packs port and ingest the matching signed, content-addressed storage overlay;
  authoritative state MUST bind pack-id, overlay revision/digest, jurisdiction,
  and validity generation. Placement, replication, repair, drain, erasure
  conversion, and cross-cell migration MUST fail closed when that authority is
  unknown, stale, expired, or incompatible.
- **ensure:** every placement and handoff receipt names the admitted pack-id and
  overlay digest; simulations inject missing, rolled-back, expired, and
  jurisdiction-conflicting overlays and prove that no new physical placement is
  published.
- **overturn_when:** a five-field cross-owner decision replaces central pack-id
  selection and still provides one runtime jurisdiction fact, storage-local
  policy ownership, signed provenance, revocation, and fail-closed movement.

</residency_authority>

<layout_and_durability>

## Decision: immutable bytes and transactional layouts

- **achieves:** repairable, evolvable data placement without changing object
  identity or risking in-place conversion loss.
- **origin:** direct physical pointers and ad hoc replica-to-erasure conversion
  make partial failure ambiguous and turn cleanup faults into data loss.
- **rule:** object metadata MUST point to stable logical chunk identifiers whose
  immutable physical layouts carry generations; replication-to-erasure changes
  MUST write, flush, verify, reconstruct-check, and atomically publish a new
  layout before old replicas become collectible; end-to-end checksums, scrub,
  anti-entropy, and risk-prioritized repair are REQUIRED.
- **ensure:** fault injection at every transition leaves either the prior layout
  authoritative or the verified new layout authoritative; cleanup failure leaks
  space rather than losing data; repair receipts expose remaining tolerance.
- **overturn_when:** a new on-disk protocol proves equal atomicity, recovery, and
  format-evolution properties under power loss and silent corruption.

</layout_and_durability>

<interfaces_and_security>

## Decision: one owned contract and one security path

- **achieves:** compatibility without dual truth and dogfood without privileged
  shortcuts.
- **origin:** HTTP-shaped Rust libraries, provider-backend semantics, and
  in-process app links can each become a second source of truth; permanent
  cross-runtime FFI places memory ownership in the payload path. ADR-0719 also
  accepts S3 as a sold storage facade, not merely a migration backend.
- **rule:** the canonical semantic contract MUST be versioned protobuf over the
  platform Connect/H3 gateway; storage MUST also sell and support an
  S3-compatible facade derived from that same transaction model. The S3 wire
  facade MUST NOT become a second metadata model or be confused with removable
  external S3-compatible backend adapters. Every caller, including first-party
  apps and pipeline, MUST use a sold facade with normal authentication, verified
  PDP provenance, quota, metering, pre-ACK audit where required, and trusted
  cell-clock enforcement; a permanent per-payload foreign-runtime FFI boundary
  MUST NOT exist.
- **ensure:** protocol conformance has one semantic source and exercises both
  Connect and S3; default-deny and forged-proof tests run before handler
  mutation; tenant #0 uses no private endpoint or skip-PDP flag; external
  backend removal leaves both sold facades, core, and callers unchanged.
- **overturn_when:** a same-wave contract decision proves a second wire surface
  is necessary and specifies compatibility, authorization, retirement, and
  ownership without dual-write or dual-truth behavior.

</interfaces_and_security>

<io_and_qos>

## Decision: pluggable I/O and asynchronous fairness

- **achieves:** hardware specialization without binding correctness to one
  kernel/device stack, while noisy tenants and repair cannot monopolize queues.
- **origin:** `io_uring` is not universal kernel bypass; SPDK has dedicated-host
  costs; sleeping inside I/O loops creates head-of-line blocking.
- **rule:** buffered/POSIX, direct `io_uring`, and optional SPDK MUST sit behind
  one backend contract; adoption MUST follow end-to-end p99.9 and unit-cost
  evidence; admission MUST be asynchronous and hierarchical, with distinct
  foreground, replication, repair, scrub, encoding, and compaction queues.
- **ensure:** identical conformance and power-loss suites run per backend;
  overload rejects before unbounded allocation; device benchmarks include repair
  and compaction rather than isolated sequential throughput.
- **overturn_when:** a supported hardware fleet converges on one backend and
  measured evidence shows removing the abstraction reduces risk without losing
  portability or isolation.

</io_and_qos>

<migration>

## Decision: prove, shadow, then cut over

- **achieves:** a ground-up core without a big-bang data migration or
  compatibility regression.
- **origin:** rewriting protocol, metadata, payload I/O, repair, and deployment
  simultaneously would erase the only behavioral oracle and make rollback
  undefined.
- **rule:** migration MUST land in staged, reversible increments: freeze the
  contract, build the owned local engine, add cell authority and layouts, shadow
  compatibility traffic, migrate bucket cohorts, then retire adapters; each
  stage MUST define success, failure, rollback, and fault evidence.
- **ensure:** `PLAN.md` orders the stages; dual-read comparisons are bounded and
  non-authoritative; cutover is cohort-scoped; old data remains readable until
  verified publication and rollback expiry.
- **overturn_when:** an independently reviewed migration plan demonstrates a
  smaller sequence with equivalent rollback and data-loss bounds.

</migration>

## Rejected destinations

- A monolithic all-role production process.
- A gossip or CRDT namespace as sole metadata/ownership authority.
- Direct object-key hashing with no ordered listing index or placement epoch.
- Embedded LSM durability presented as distributed consistency.
- SeaweedFS, OCI, NativeLink, or an external S3 backend as the permanent product
  identity or metadata authority.
- Global cross-tenant deduplication by default.
- `io_uring` or SPDK as proof of end-to-end scale.
- Erasure conversion that mutates physical pointers without a versioned commit.
