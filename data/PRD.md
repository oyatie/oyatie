---
doc_class: Owner-PRD
owner: data
status: Active
last_interviewed: 2026-08-26
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Data product requirements

<product_boundary>

`data/` is the cloud's durable-records capability. It will sell an owned Rust
records service spanning OLTP, OLAP, and record-processing pipelines, with
independently scalable compute, metadata, tablet data, and repair roles inside
bounded cells.

It does not own object/CAS bytes (`storage/`), ontology and Foundry UX
(`app/foundry`), queue/bus/outbox delivery (`bus/`), software-delivery
pipelines (`pipeline/`), search/RAG (`intelligence/`), or BI applications.
Shared ontology consumers bind an implementation-free Foundry port or sold
facade, never Foundry/Data core. The Bus outbox port contains portable contract
semantics only; SQL, transaction batches, protocol conversion, and delivery
runtimes remain provider-matching adapters behind agreed ports.
First-party apps do not link the core, Rust ports, or implementation adapters
of any cloud capability in-process; Data, Gateway, and Bus are examples, not an
exception list. Each app owns its business/substrate port, with one app-owned
adapter for the generated client of the sold Connect/protobuf facade and at
least one app-owned commodity adapter. A source-compatibility alias may target
that app-owned package only; it cannot launder any cloud Rust package into an
app graph.

The present repository does not yet supply that product. PostgreSQL 16/SQLx is
the live relational compatibility path and is not horizontally scaled as
shipped. ClickHouse is a deferred adapter scaffold. These are current-state
facts, not destination endorsements.

</product_boundary>

<users>

- Cloud tenants need strongly consistent transactional records with isolation,
  conditional mutation, indexes, schema evolution, backup, and recovery.
- Analytical users need columnar projections, bounded freshness, reproducible
  queries, tenant quotas, and lineage back to committed records.
- Dataset pipelines need durable jobs, idempotent replay, checkpointing,
  generation publication, and no partially visible outputs.
- First-party apps need portable record ports and ordinary authenticated Data
  adapters, not direct links to Data core or a privileged tenant-zero path.
- Operators need bounded cells, explicit capacity, safe drain/rebalance,
  automated repair, rolling upgrades, disaster recovery, and risk telemetry.

</users>

<requirements>

## Transactions and records

- Provide tenant-scoped keyspaces, typed schemas, indexes, serializable
  transactions within a cell, conditional mutation, idempotency, snapshots,
  change streams, and point-in-time recovery.
- Use an engine commit ordinal as the versionstamp. Consume Cell's trusted
  interval for policy, lease, and observation needs; never use wall time as
  transaction identity or claim a global commit timestamp.
- Acknowledge a write only after the selected durability profile and the
  authoritative commit record are durable. An unsupported profile fails before
  mutation.
- A D1c WAL artifact represents the complete ordered durable mutation list of
  one committed single-tablet transaction. A read-only transaction has no WAL;
  an empty or unclassified record, metadata, or control mutation fails before
  `PREPARED`. Uniform and mixed classifications are bound by the canonical
  summary in `SPEC.md`, not chosen, ranked, defaulted, or split into separately
  visible transaction fragments.
- Reject cross-cell transactions until a separately specified protocol can
  preserve explicit semantics; no stage may silently weaken them to eventual
  consistency.

## Horizontal scale and failure containment

- Partition records into range-addressed tablets within bounded cells. Split,
  move, replicate, and rebalance tablets online behind monotonic ownership
  epochs and stale-writer fencing.
- Scale stateless query/transaction compute independently from tablet storage,
  metadata authority, compaction, and repair.
- Keep ordinary transactions off a global controller. A global directory may
  locate the home cell but may not become the per-query serialization point.
- Model host, rack, zone, device class, capacity, and drain state rather than
  treating node membership as a flat ring.
- Bound admission, queues, background work, and repair so one tenant or failed
  rack cannot consume a cell without limit.
- Treat v1 request/key/value/transaction/collection/decode/concurrency and
  result/response-frame/encode-allocation/in-flight-byte maxima as observable
  contract semantics. Enforce checked arithmetic, response-credit
  backpressure, and the ordered validation/refusal matrix in `SPEC.md`; a
  deployment profile may lower but never raise those hard bounds. A mutating
  transaction sizes and reserves its complete response before preparation so
  it cannot commit and then fail a deterministic result bound.

## OLAP and record pipelines

- Derive columnar layouts from a durable ordered change stream; checkpoints
  bind source ordinal, schema version, projection generation, and checksum.
- Publish a pipeline output atomically by generation. Retries are idempotent;
  gaps, reorder, incompatible schema, or lineage loss fail closed.
- Support independent analytical compute and immutable columnar storage without
  making ClickHouse or another adapter the semantic authority.
- Expose freshness, scan bytes, queue delay, spill, checkpoint age, and
  per-tenant unit cost.

## Interfaces and portability

- Define one versioned protobuf semantic contract served through the platform
  Connect gateway. Its schema, generator/runtime choice, generated-output
  identities, descriptor, unknown-field policy, deterministic encoding rules,
  and Cargo/Buck generation parity land through the D-33 structural gate before
  any frame-size, decode, encode, handler, or compatibility claim. A separately
  accepted compatibility facade must translate that model rather than create a
  second transaction truth.
- Ship one signed, reproducible Rust distribution with no mandatory external
  PostgreSQL, ClickHouse, etcd, Kafka, Redis, or proprietary database runtime.
- Keep foreign engines behind adapters with explicit compatibility and removal
  gates. Apps prove portability against at least the in-memory contract oracle
  and one durable adapter.
- Keep agreed ports implementation-free. Concrete stores, recording executors,
  SQL construction/execution, transport normalization, and mutable test
  oracles live in matching core/adapter/facade packages before promotion.
- Name each adapter for the provider port and backend it implements; an adapter
  without a matching port, or one that reaches another owner's core, is not a
  portable Data face.
- Use IAM/PDP, trusted Cell time, packs, audit, secrets/KMS, and telemetry only
  through agreed fail-closed ports. Direct cross-owner core dependencies are
  forbidden.
- App adapters that select an Oyatie cloud SKU consume only the generated
  public-facade client and canonical proto. They never depend on a cloud Rust
  core, port, or provider implementation; commodity adapters implement the same
  app-owned contract and remain independently selectable.
- Use semantic executable, process, job, error, test, log, metric, and status
  names. Decision identifiers remain provenance in citations, comments, and
  metadata; they are not an operator-facing state or refusal identity.

## Security, deletion, and operations

- Authenticate and authorize before record disclosure or mutation. Bind the
  decision to tenant, operation, resource range, policy revision, issuer,
  audience, server-derived canonical request fingerprint, and expiry. A
  caller-supplied fingerprint, tenant, principal, snapshot, or key generation
  is never authority.
- Resume a scan only from the bounded authenticated v1 continuation capability
  in `SPEC.md`. It binds the verified tenant/principal, normalized query,
  snapshot ordinal, tablet/ownership epoch, page maxima, issuance/expiry
  interval, and key generation; tamper, foreign replay, stale snapshots,
  retired keys, and uncertain time fail closed.
- Consume policy, audit, key, and cryptographic behavior only through Data-owned
  implementation-free ports and provider adapters to separately accepted sold
  provider faces. Current IAM, Audit, or Secrets core/internal API packages are
  not an implied dependency, and a fake or fixture cannot satisfy production
  composition or readiness.
- Encrypt records, WAL/log entries, segments, snapshots, repair copies, and
  migration artifacts with the tenant-bound versioned AEAD envelope and AAD in
  `SPEC.md`. Data uses accepted KMS AEAD operations through opaque,
  non-serializable operation handles; it never receives, serializes, logs, or
  zeroizes raw key bytes. The KMS contract owns raw-key lifetime/zeroization
  evidence. Rotate generations online with a non-reusing `NonceLeaseId` range
  durably reserved and allocated by one exclusive linearizable CAS/fsync owner
  before each use, bounded resumable re-encryption, durable audit,
  ciphertext-only restore, and revocation fencing. Every durable envelope
  carries a bounded authenticated bootstrap locator with the tenant, purpose,
  key, generation, fence, provider revision/catalog, and recovery-policy
  binding needed to reacquire a fresh authorized Open/DecryptOnly handle before
  decrypting the encrypted opaque key-generation binding; Data never persists a
  handle or raw key. `RecordKeySource` is the only acquisition/reacquisition
  owner, while record protection consumes its typed lease for Seal/Open. Each
  artifact has the exact purpose-valid 19-field AAD, final manifest, sealed
  commit record, and atomic CAS head. Record plans are exactly one entry at at
  most 4 MiB; WAL plans are exactly one entry at at most 16 MiB; only aggregate
  artifacts may use the bounded 1..4,096-entry plan. Record primary-key and
  WAL transaction-classification bindings agree through every frame, while WAL
  transaction identity agrees in each role AAD and the sealed commit root. The
  head is current only when its immutable-context, strictly monotonic
  generation/fence anchor matches a durable Audit high-water witness. The
  Data-owned cell-local publication coordinator (`artifact-publication` port,
  core, and cell adapter) owns the tuple, pins, members, durable CAS
  decisions/receipts/accepted-high-water history, safe-GC epochs, normal and
  terminal-only successor takeover, and an authenticated Audit callback;
  `records-app` alone composes its core/adapter with Audit. A successor cannot
  CAS until its expected tuple is locally current, fresh Audit high-water, and
  accepted history with no successful `COMMITTING` predecessor, so recovery
  completes H1 Audit before H2. A normal losing CAS atomically records its
  observed anchor and reaches `SUPERSEDED`/`RELEASED`; later H1-to-H2 advance
  never converts that loss into a permanent quarantine. At the original
  renewal horizon, only a durable, fenced, non-renewable terminal-recovery
  authority may reconcile/release an existing decidable pin or append its
  already-successful Audit receipt; it cannot put, bind, renew, CAS, rebase, or
  publish. There is one current terminal-recovery row per nonterminal pin; a
  fenced successor replaces it rather than accumulating recovery state. The
  coordinator keeps a checked active-row relation and its three scope counters
  identical: absent-to-present is `+1` row/`+230` bytes, exact higher-fenced
  replacement is `+0`/`+0`, exact terminal release is `-1`/`-230`, and normal
  owner release with a serializable proof of no terminal row is `+0`/`+0`.
  `CommitNormalPublicationRelease` handles ordinary CAS loss, Audit finalization,
  and safe abandon; `CommitTerminalRecoveryRelease` requires the complete exact
  terminal row. Underflow, overflow, row/counter mismatch, stale terminal token,
  or a normal release racing a terminal row fails atomically and cannot strand a
  lawful normal loser. The terminal consensus transition atomically validates or
  writes its immutable decision/cause, records `released_gc_epoch`, detaches all
  members, marks `RELEASED`, and applies only its lawful relation branch. It
  leaves no released-row or tombstone authority. The retained decision/receipt/
  history proof makes a lost reply observable until safe GC.

  A bounded versioned `PublicationRecoveryAuthorityV1`, authenticated by the
  independently durable Audit control plane and excluded from all restorable
  Cell pin snapshots, prevents deleted authority from being recreated. It binds
  the exact cell/context bootstrap locator, a fresh 16-byte namespace nonce,
  current and retired-incarnation high-waters, current-incarnation allocation
  high-water, monotonic fence/revision, predecessor digest, and integrity tag.
  The bootstrap locator and authority are fixed canonical 142-byte and 278-byte
  frames; the authority also repeats the authenticated Audit root and
  non-restorable Cell recovery generation from its locator before its integrity
  tag is verified.
  A `pin_id` contains that 16-byte nonce, its 8-byte authority incarnation, and
  its 8-byte allocation index. Before any snapshot import or full cell/device
  recovery accepts a pin, a Cell recovery attestation source-CAS rotates the
  external locator/context incarnation, retires the old one, and only then
  recovery-quarantines every old row. It can then reconstruct only externally
  authenticated current publication state and issue fresh pins in the new
  namespace; an old work or terminal credential can neither take over nor
  recreate a released pin. This rotation, rather than an invalid inference that
  every index below one high-water is retired, closes old-snapshot replay after
  terminal release and safe-GC compaction. A live replay uses only the current
  local authority mirror; ordinary artifact reads remain cell-local. Missing,
  tampered, rolled-back, foreign, or exhausted external authority withdraws
  affected publication admission/readiness, and the read-only terminal-outcome
  query never mints recovery or publication authority. Full Cell loss
  reacquires the bootstrap locator and the independently durable Audit-quorum
  source in that order; absence of either refuses rather than initializing from
  a restored pin snapshot. Active terminal rows are
  charged one-for-one to the existing 8/64/256 locator/tenant-cell/cell
  nonterminal limits and therefore cap at 1,840/14,720/58,880 bytes; no terminal
  row survives a successful release. Only genuinely undecidable recovery remains
  safely quarantined under bounded pin/byte/backlog admission. Aggregate
  publication admission charges
  the aggregate-legal migration maximum only: `3,156` envelope overhead,
  `2,040` empty-transaction commit, and exactly `68,732,871,254` bytes per pin,
  with `549,862,970,032`, `4,398,903,760,256`, and
  `17,595,615,041,024` byte locator, tenant-cell, and cell ceilings. The
  general `2,296`-byte record/WAL commit and `1,825`-byte record AAD are
  rejected for aggregate artifacts. Serving reads use a fresh validated
  in-cell snapshot and never make Audit a per-read dependency; a changed tuple,
  coordinator term, or expiry refreshes or fails closed. GC retains every
  pinned/current chain until its terminal safe-epoch release, so it never
  collects verified publication input or accepts a valid retained older head;
  releasing a losing pin does not permit collection while any anchored chain
  still names the shared content address.
  An uncertain recovered lease/binding/bootstrap/publication state is burned or
  locator-scoped quarantined rather than reused.
  Missing PDP, Audit, KMS, cryptography, active key generation, or trusted-time
  evidence withdraws the affected route and readiness. Plaintext, stale-key,
  unaudited, or fail-open fallback is forbidden.
- Tenant deletion freezes writes, inventories every table/tablet/projection,
  respects retention holds, proves logical erasure, and only then reclaims
  physical generations.
- Support mixed-version rolling upgrades, format negotiation, snapshot restore,
  offline consistency checking, drain, rebalance, and cell-loss runbooks before
  production promotion.

</requirements>

<service_objectives>

## Target SLOs

These are promotion objectives, not current implementation claims. Measurement
is at the sold facade under the declared production capacity profile.

| Signal | Target |
|---|---|
| Regional OLTP availability | At least 99.999% successful admitted requests per calendar month after multi-cell recovery is promoted |
| Per-cell OLTP availability | At least 99.99% successful admitted requests per calendar month |
| Point-read latency | p99.9 at or below 50 ms under 70% admitted cell load, excluding caller WAN |
| Single-tablet commit latency | p99.9 at or below 100 ms under the same profile |
| In-tolerance node failure | RPO 0; p99 tablet leader recovery at or below 30 seconds |
| Analytical freshness | p99 committed-ordinal to queryable projection at or below 60 seconds for streaming profiles |
| Pipeline admission | p99.9 accepted job to durable queued state at or below 1 second |
| Repair | p99 under-replicated tablet restored within 15 minutes after eligible capacity exists |
| Tenant isolation | Zero cross-tenant disclosure or mutation; a saturated tenant causes less than 2x p99.9 latency for other admitted tenants |
| Authorization and encryption coverage | 100% of routed disclosures and mutations carry accepted policy evidence; 100% of durable record artifacts verify a supported tenant-bound AEAD envelope |
| Key rotation | 100% of eligible artifacts migrate before retirement; revoked-generation encryptions and nonce reuse are zero |

No durability probability, throughput, or clock-epsilon marketing claim is
allowed until the production plant, workload, and fault evidence measure it.
Inside the configured failure tolerance, accepted writes have an RPO objective
of zero and any observed acknowledged-write loss is a promotion failure.

</service_objectives>

<promotion_targets>

Production promotion requires all of the following, not unit-green alone:

- Cell-local serializability and stale-writer fencing survive deterministic
  simulation, model checking, and adversarial process/network histories.
- Acknowledged records survive the declared node, device, rack, and power-loss
  profile; corruption is detected, isolated, repaired, and audited.
- Online split/move/rebalance preserves transaction and scan semantics.
- N/N+1 upgrade, rollback barrier, snapshot restore, and quorum-loss recovery
  are repeatedly exercised.
- PostgreSQL workload migration proves semantic and data parity per cohort with
  one write authority and a tested rollback deadline.
- OLAP and pipeline replay prove no gaps, duplicate publication, partial
  generations, stale-head fallback, GC-dangling committed artifacts, or
  unbounded lag under admitted load.

Success means the owned engine clears these targets and foreign database
adapters can be removed without changing callers. Failure includes split brain,
acknowledged loss, cross-tenant access, silent semantic weakening, unbounded
queues, unverified repair, or a compatibility claim supported only by types.

</promotion_targets>
