---
doc_class: Owner-PLAN
owner: storage
status: Active
date: 2026-08-25
---

# Storage remaining work

<baseline>

## What has landed

- Typed Rust domain contracts for bucket/object and block metadata, residency,
  encryption binding, object lock, provider receipts, and idempotent boundaries.
- A tenant-scoped BLAKE3 CAS contract, bounded payload traits, caller-supplied
  WORM/audit fields, conformance suite, and in-memory reference store. The
  authorization, clock, and audit inputs are self-asserted test models today.
- S3 and OCI command/receipt projections.
- P0a retired the unconsumed NativeLink fleet deployment and obsolete laptop-lab
  runbook bundle, hand-authored Helm/OpenSLO files, and their deleted-corpus BUCK
  loader. No replacement deployment or SLO source is claimed until a storage
  reconciler or SLO IR consumes it.
- P0b established one primary `storage-domain` core package, moved the HTTP
  and provider projections into owner-local draft adapters, split OCI object
  and block projections, and adopted sorted compile-time item scanners. The old
  CAS and HTTP package identities remain thin re-export shims. S3 and combined
  OCI retain behavior-equivalent compatibility packages at D-8-valid paths for
  the advertised support window.
- P0c moved the object/block backend traits, request and receipt types, and
  behavior-equivalent validation into `ports/draft/provider`. Every retained
  S3/OCI backend adapter now consumes that port, while `storage-domain`
  preserves its public source surface through compatibility re-exports. The
  draft port's foreign core-model edges remain explicit P0 debt.
- P0d introduced the agreed `cell-location` value-contract port and routed both
  storage core and the provider draft through it. The port re-exports Cell's
  existing validated `RegionCode`, `AzCode`, and `CellId` identities so the
  cross-owner import changes without copying models or changing behavior. The
  Cell engine remains their defining crate during its compatibility window;
  direct data/KMS/compute/residency core edges remain P0 debt.

None of that is a persistent distributed engine, a network facade, automated
repair, or production durability evidence.

</baseline>

<sequence>

## P0 — Structural storage cleanup

Class: structural; do not mix behavior.

- **Landed in P0b:** reconciled the two implementation-bearing core crates into
  one primary `core/domain` package. The deprecated
  `core/object-store-kernel` identity is now a re-export-only compatibility
  shim.
- **Landed in P0c:** moved provider traits, request/receipt DTOs, validation,
  and compatibility errors out of `core/domain` into the owner-local
  `ports/draft/provider` seam. `storage-domain` re-exports the existing public
  names during the P0 support window, and every S3/OCI provider adapter depends
  on the new seam rather than the core package. This is not P0 completion: the
  draft port still imports foreign core models and the primary core retains
  non-provider foreign edges.
- **Landed in P0d:** replaced every direct storage-to-Cell core dependency with
  the agreed `cell-location` port. The exact validated identity types remain
  source- and behavior-compatible through re-export; this does not claim the
  Cell engine's internal defining-crate inversion complete. Data-boundary, KMS,
  compute-resource, and residency imports remain direct foreign-core debt.
- Remove direct cross-owner core imports from storage core, ports, and adapters.
  Reconcile required data-boundary, KMS, compute, and residency types onto
  agreed provider ports plus storage-owned adapters; do not copy their domain
  models or claim the current `path = .../core/...` edges are legal. Cell
  location identity now uses its agreed port.
- Rename legacy `cloud-*`, `*-api`, provider, and explicit `[lib]` fossils to the
  ADR-0719 grammar; split every touched hand-written file above 300 lines.
- **Landed in P0a:** removed the hand-authored Helm/OpenSLO residue, retired the
  NativeLink fleet YAML and its obsolete laptop-lab runbook bundle, and deleted
  their stale corpus-only BUCK loader after proving no build, runtime, policy,
  SLO controller, or reconciler consumes them.
- **Landed in P0b:** quarantined HTTP-shaped object/block behavior and provider
  projections under `adapters/draft`, split OCI object and block packages,
  retained old public package identities for the compatibility window, removed
  explicit `[lib]` identity/path overrides from destination packages, and
  split touched Rust sources to the 300-line budget without behavior drift.
- Rehome filesystem, archive, backup, and restore product models that fall
  outside ADR-0719 D-14 behind their eventual owning facades. P0 retains
  deprecated compatibility re-exports/shims and identical behavior for every
  currently public storage type; it does not delete a public contract. Final
  removal requires a separately scoped, explicitly versioned deprecation and
  consumer-migration change after the advertised support window. Object
  recovery internals remain storage behavior, not a separately sold product.
- Keep `object-store-kernel` as a deprecated package-identity re-export only
  while P1 introduces the sold facade. Remove any remaining direct application
  call sites or guidance; app cores own blob ports and adapters rather than
  linking this storage core.

Success: path-layout admission and exact storage crate tests pass; no behavior
is claimed changed; the owner has one primary engine, explicit ports/adapters,
and no cross-owner core-to-core dependency.

Failure: any unversioned behavior or public type drifts, a compatibility re-
export changes results, an imported owner model is copied, an illegal core edge
remains, or removed residue is still consumed.

Rollback: revert only the structural moves/renames and restore consumed
artifacts; no data format or runtime route changes in this stage.

Fault evidence: negative fixtures inject a forbidden cross-owner core edge, an
oversized touched file, and a stale consumed artifact and prove admission fails;
before/after contract and crate-test results remain identical.

## P1 — Freeze the canonical contract

Class: behavioral contract.

- Author versioned object/CAS protobuf under the storage facade grammar.
- Define bucket, version, multipart, conditional mutation, ordered listing,
  retention/legal hold, checksums, streaming, idempotency, and typed errors.
- Freeze the multipart Complete-versus-Abort and delete/tombstone/tenant-delete
  state machines, bucket-wide LIST barrier/vector/lineage token, pack-overlay
  authority, cross-cell ownership epoch, trusted-clock, verified PDP decision,
  and pre-ACK audit-receipt fields from `SPEC.md`.
- Freeze a parameterized durability policy with an explicitly non-production
  single-node development profile. Production replicated/erasure profiles state
  their required receipt quorum; an implementation that cannot satisfy the
  selected profile refuses before mutation.
- Make the in-memory reference implement the contract's semantic state machine
  and publish one parameterized backend conformance suite. Its authorization,
  clock, audit, and durability proofs remain explicitly test-only and cannot
  satisfy production capability profiles or promotion evidence. Translate
  existing Rust HTTP-shaped boundaries only as compatibility fixtures, then
  retire them.
- Model block APIs separately; do not let block scope delay object/CAS.
- Derive and support the sold S3 facade from the same protobuf semantic model;
  keep external S3-compatible backends on a separate removable adapter port.

Success: protobuf compatibility tests plus native and S3 SDK conformance pass;
no extra JSON/REST SSOT; forged or expired authorization, clock, pack, and audit
evidence fails before handler mutation.

Failure: either facade admits semantics absent from the shared model, mutation
races are undefined, or a caller-constructed proof reaches business logic.

Rollback: keep the new packages unrouted and retain the old reference fixture;
published versions are never rewritten, only superseded by a compatible version.

Fault evidence: malformed/unknown frames, Complete-versus-Abort, legal-hold,
stale LIST token, missing pack, forged PDP, uncertain clock, and audit outage
tests produce stable fail-closed errors and no mutation.

## P2 — Owned single-node durable engine

Class: behavioral engine.

- Implement immutable segments, checksums, WAL/manifest recovery, tenant key
  binding, durable flush semantics, orphan collection, and local snapshots in
  Rust.
- Put the local metadata engine behind an internal abstraction. Select one
  canonical production LSM only after power-cut, corrupted-WAL, compaction, and
  upgrade tests; embedded storage is persistence, not distributed authority.
- Run the conformance suite against buffered I/O first.
- Expose only the P1 `development-single-node` durability profile in this stage.
  It is marked non-production in protocol, configuration, telemetry, and test
  evidence. A request selecting 3x replication, erasure coding, or any
  production profile returns a typed durability-unavailable error before a
  `PREPARED` record or physical write; P4 is the first stage that may qualify
  and acknowledge those profiles.

Success: parameterized conformance proves the development profile's crash and
power-cut contract without exposing an uncommitted version or losing an
acknowledged local write; every production profile is refused without mutation;
snapshot/restore and format upgrade work. This is not production durability.

Failure: page-cache persistence is mistaken for durable flush, recovery trusts
corrupt WAL/manifest state, or an acknowledged version is absent after restart.

Rollback: retain the prior format reader and manifest generation; disable the
new writer before its format barrier and restore from the last verified snapshot.

Fault evidence: kill/power-cut injection before and after every flush/publication
edge, partial writes, full devices, corrupt WAL/manifests, N/N+1 format tests,
and negative conformance proving a 3x request cannot mutate or acknowledge.

## P3 — Cell metadata, placement, and fencing

Class: distributed control plane.

- Add metadata tablets with consensus, MVCC, ordered key ranges, split/move,
  snapshot, and recovery.
- Implement the bucket-wide LIST barrier, revision vector, tablet-lineage epoch,
  token-history lease, and explicit restart behavior from `SPEC.md`.
- Add placement quorum, hierarchical failure domains, device states, cached map
  epochs, allocation leases, write fencing, drain, and verified handoff.
- Ingest installed pack-id plus signed content-addressed storage overlays and
  bind their revision, digest, jurisdiction, and validity generation into every
  placement/movement receipt; fail closed on missing or stale authority.
- Add the global bucket-to-cell directory without putting it in payload I/O.
- Add signed time-bounded cross-cell ownership leases, forward-only promotion
  epochs, external fencing/lease-expiry gates, and stale-cell refusal.

Success: deterministic partition simulation and model checking prove one
committed writer per generation; GET/PUT traces have no global controller hop;
ordered LIST remains exact across pages and tablet split/move; forbidden
jurisdictions receive no new bytes.

Failure: two cells acknowledge one ownership generation, a stale pack/map writes,
or pagination duplicates/omits a key instead of returning restart-required.

Rollback: publish a higher map/ownership epoch that fences the new authority and
returns cohorts to the prior implementation after verified handoff; never
decrement an epoch or re-enable an unfenced cell.

Fault evidence: asymmetric partitions with the old home still live, clock
uncertainty, stale maps/leases, overlay rollback/expiry/conflict, leader change,
and concurrent write plus LIST split/move histories.

## P4 — Replication, erasure coding, scrub, and repair

Class: durability plane.

- Introduce stable logical chunk IDs and versioned immutable layouts.
- Implement failure-domain-aware replication and the verified replicated-to-EC
  transition in `SPEC.md`.
- Implement scrub, anti-entropy, durability-state evaluation, risk-prioritized
  repair, evacuation, repair leases, and storm budgets.

Success: process, disk, rack, power, partial-write, and silent-corruption
campaigns converge without acknowledged loss inside the configured tolerance;
cleanup faults leak space only; repair telemetry reports remaining tolerance.

Failure: an unverified layout publishes, repair violates pack residency or
exhausts foreground budgets, or cleanup removes the last readable generation.

Rollback: keep replication authoritative, disable new EC publication/repair
classes by consensus policy, and retain old layouts through the extended reader
and rollback lease.

Fault evidence: fail every EC transition edge, corrupt each shard/manifest,
remove disks/racks during reconstruction, inject stale overlays, and saturate
repair while foreground load remains admitted.

## P5 — Gateway, S3 facade, and migration

Class: facade/adapters.

- Serve the canonical Connect/H3 contract through the platform gateway with
  streaming, cancellation, deadlines, authn, fail-closed policy, quotas,
  metering, and audit.
- Build the supported S3 facade as an independent gateway adapter and
  differential-test it against the shared semantic oracle.
- Before any shadow claim, inventory each live source bucket, backend, format,
  policy, key boundary, and owner. Implement an executable, bounded legacy
  backend bridge on the storage backend port; the current command/receipt
  projections are fixtures and do not qualify. If no live source exists, mark
  migration not applicable and use seeded differential corpora instead.
- While the legacy source is authoritative, capture mutations in a durable,
  ordered migration journal, populate the owned engine, and shadow reads without
  serving shadow results. Cut over a bounded cohort by atomically advancing its
  backend authority epoch; there is never dual write authority.
- Journal sequence is monotonic per cohort partition and total per object. Each
  entry binds backend authority epoch, object generation, predecessor,
  idempotency fingerprint, and checksum; a cohort barrier is a vector of durable
  partition watermarks. Replay is idempotent and refuses gaps or reordering.
- After cutover, the owned commit transaction durably appends an owned-to-legacy
  reverse-journal entry before acknowledging every mutation until signed
  rollback expiry. A replication-only credential may apply those entries to the
  fenced legacy source, but cannot grant it client write or read authority.
  Persist and expose owned-committed and legacy-applied watermark vectors.
- The comparison authority is the frozen semantic conformance suite plus
  checksum, version/list, retention, quota, audit, and byte-manifest equality.
  Retain the source and replay journal until a signed rollback expiry.

Success: SDK and S3 suites cover strong read-after-write/list, conditional
writes, multipart, versions, retention, and legal hold; every source is
inventoried; tenant #0 uses the same facade; backend-adapter removal does not
change callers, the S3 facade, or core.

Failure: an untracked source exists, shadow comparison diverges, journal replay
has gaps, two backends accept authority, or parity is asserted from receipt
shape without executing bytes.

Rollback: before cutover, stop shadowing with no route change. Before the signed
expiry, advance to a higher `ROLLBACK_FENCING` epoch so neither backend accepts
new cohort requests, drain in-flight owned commits, seal the owned-committed
watermark vector, replay until every legacy-applied watermark equals it, and
prove semantic/manifest parity at that vector. Only then may a still higher
epoch grant the legacy backend client authority. Failure keeps the cohort
fenced and retryable; it never enables two authorities or drops post-cutover
writes. Signed expiry closes reverse capture only after its final journal and
parity barrier are durable; afterward normal restore/recovery applies.

Fault evidence: bridge outage, lost/duplicated/reordered forward and reverse
journal entries, crash between owned commit and acknowledgement, checksum and
LIST divergence, partial cohort cutover, stale route caches, and rollback at
every pre-expiry transition while concurrent post-cutover writes are attempted.

## P6 — QoS and hardware backends

Class: performance after correctness.

- Add hierarchical async admission, deadline-aware rejection, weighted device
  queues, and separate foreground/background budgets.
- Add direct `io_uring`; add SPDK only for a dedicated NVMe profile whose
  end-to-end p99.9 latency, throughput per core, and unit cost beat the kernel
  path during repair and compaction.
- Benchmark small metadata reads, small-object append, large streaming,
  multipart completion, range reads, reconstruction, and saturated queues.

Success: noisy-tenant and repair-storm tests preserve admitted foreground SLOs;
every backend passes identical durability/recovery conformance; no busy polling
is imposed on general-purpose cells.

Failure: a hardware backend changes durability semantics, tail latency or unit
cost regresses, overload allocates without bound, or a tenant/background class
starves admitted work.

Rollback: select the last qualified buffered/kernel backend and scheduler policy
through a versioned cell profile; drain incompatible queues before switching.

Fault evidence: noisy tenants, queue saturation, device timeout/reset, partial
direct-I/O completion, repair/compaction storms, and low-utilization SPDK tests.

## P7 — Production operations and block promotion

Class: promotion.

- Land online N/N+1 upgrades, downgrade barriers, format migration, metadata
  point-in-time recovery, cell evacuation, regional failover, capacity
  forecasting, accounting, key rotation, audit export, and incident runbooks.
- Continuously run deterministic simulation, model checks, Jepsen-style
  histories, block/filesystem fault injection, SDK compatibility, and restore
  drills.
- Promote object/CAS against the `PRD.md` objectives. Staff and promote the
  EBS-class block facade only in a separate evidence-backed lane.

Success: object/CAS clears every promotion target and named failure campaign; a
cell or region can be lost and recovered within the declared RPO/RTO; operators
can roll forward or back without stopping the service.

Failure: an upgrade crosses an irreversible barrier without proof, failover
permits split brain or exceeds RPO/RTO, recovery skips retention/audit evidence,
or operations depend on an undocumented external state service.

Rollback: N/N+1 stays inside its declared downgrade barrier; failed region
promotion advances to another ownership epoch rather than reviving stale state;
format cleanup waits until all rollback leases expire.

Fault evidence: mixed-version process and cell loss, quorum loss, region
partition with old writers live, corrupt snapshot restore, KMS/audit outage,
capacity exhaustion, and repeated forward/rollback drills under admitted load.

</sequence>

<ordering_rules>

## Ordering and lane boundaries

1. Correctness and authority precede `io_uring`, SPDK, or broad performance work.
2. Structural mutation is separate from behavioral implementation.
3. Metadata, placement, data layout, facade, and repair changes use disjoint
   files/crates where possible; shared owner-law and workspace files have one
   writer.
4. A stage does not claim completion from unit tests alone. It carries its named
   failure injection, SLO signal, rollback, and independent review.
5. Transitional adapters may extend a migration window but may not become an
   excuse to postpone the owned engine or create a second source of truth.

</ordering_rules>

<next_lane>

The next implementation lane remains P0: reconcile the direct
data/KMS/cell/compute/residency core edges onto agreed provider contracts
without copying foreign models, then rehome filesystem/archive/backup/restore
product models. P1 begins only after those structural obligations merge and
`dev` is refreshed.

</next_lane>
