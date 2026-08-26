---
doc_class: Owner-ADR
owner: data
status: Accepted
date: 2026-08-26
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Data decisions in force

This file specializes ADR-0719 for `data/`. It records current owner law and
the destination contract. It is not evidence that the destination engine or
its scale properties have landed.

<current_state>

## Evidence at D1a

| Surface | What exists | Maturity |
|---|---|---|
| PostgreSQL command path | Rust command contracts plus a SQLx adapter owning one `PgPool`; IAM and Tenancy have live PostgreSQL-backed stores and RLS tests | Transient compatibility path |
| PostgreSQL CI | One PostgreSQL 16 service exercises transaction and tenant-RLS behavior | Single service; no product sharding or cell failover |
| Citus probe | The SQLx live harness can optionally call `create_distributed_table` when `OYATIE_BACKBONE_REQUIRE_CITUS` is enabled | Opt-in probe; disabled by the normal live workflow and not horizontal-scale evidence |
| Classification contract | `data/ports/classification` exact-re-exports the established classification values and parsers from `data/core/data-boundary-kernel`; Network and Storage already consume the port | Compatibility bridge only; the dependency still points port-to-legacy-core; its 94 other direct package consumers partition into 68 non-app classification-only, nine app classification-only, 15 non-app mixed, one app mixed, and one purpose-only consumer |
| OLAP | In-memory OLAP reference behavior and a ClickHouse-shaped adapter whose emitted errors still expose the provenance label `IP-003 deferred` | Contract/scaffold only; no live ClickHouse store; the numbered runtime wording is explicit D1b-N residue |
| Analytics facade | Configuration and boot validation; the listener is explicitly deferred | Not a served Data product |
| Placement debt | Ontology packages and transactional-outbox packages remain under `data/` | They are not in the target Data charter and move only through separately reviewed owner lanes |

The repository's current relational database is PostgreSQL 16 through SQLx.
It does **not** horizontally scale as shipped. A `home_cell` or `shard_key`
field, a connection pool, and an optional Citus probe do not implement tablet
placement, consensus, fencing, split/move, repair, or rebalancing. No current
Data SLO or owned-engine availability claim follows from green unit or live-RLS
tests.

</current_state>

<boundary>

## Decision: records, not bytes or products

- **achieves:** one cloud owner for durable record processing without turning
  Data into object storage, search, or an application suite.
- **origin:** the current tree mixes record helpers with ontology product code
  and transactional-outbox code, while ADR-0719 assigns those concerns to
  Foundry and Bus.
- **rule:** `data/` MUST own durable records engines: OLTP, OLAP, and record-
  processing pipelines. It MUST NOT own object/CAS bytes, ontology, Pages,
  Grid, Workshop, search/SERP, RAG, a BI application, or a private clock.
  Foundry ontology packages MUST move to `app/foundry`; outbox delivery
  packages MUST move to `bus/` through separately versioned owner changes.
  Shared ontology values MUST be defined by an implementation-free Foundry
  port; Application MUST bind that port or the sold Foundry facade, never a
  Foundry/Data core. The Bus outbox port MUST remain free of SQL, database,
  Gateway, and delivery-runtime implementation; those dependencies belong in
  provider-matching adapters behind agreed ports. An app MUST NOT replace a
  dependency on any cloud capability's core with an in-process dependency on
  that capability's port or implementation adapter. This applies to Data,
  Gateway, Bus, and every other cloud capability without a tenant-zero
  exception. App business types and substrate needs belong to app-owned ports;
  one app-owned adapter translates them to the sold Connect/protobuf facade and
  another app-owned adapter targets a declared commodity backend. The cloud
  adapter may consume only the versioned sold-facade client generated from the
  canonical proto, never a cloud Rust core, port, or implementation adapter.
  Compatibility aliases inside an app MAY preserve source spelling but MUST
  resolve only to that app's package.
- **ensure:** new Data core packages model records, transactions, queries,
  projections, or dataset transforms; dependency review rejects app-domain,
  generic blob, search, and broker behavior in Data core. Transfer review
  proves all reverse consumers leave old cores, every adapter name identifies
  the port and backend it implements, and Cargo/Buck enforce port-to-consumer
  rather than port-to-core edges. Cross-owner census explicitly separates
  cloud-capability consumers from apps, rejects every app-to-cloud Rust edge,
  and app lanes prove both sold-facade-client and commodity-adapter conformance
  before removing an illegal cloud edge.
- **overturn_when:** a founder-accepted owner-boundary decision updates every
  affected owner's four law files in the same change.

</boundary>

<owned_engine>

## Decision: owned Rust records engine, removable adapters

- **achieves:** a self-contained database capability whose correctness,
  formats, scaling, repair, and upgrade lifecycle are controlled in-tree.
- **origin:** PostgreSQL and ClickHouse are useful compatibility references,
  but running or wrapping them does not implement the ADR-0719 owned-stack or
  cell-authority destination.
- **rule:** the destination records engine MUST be owned Rust behind stable
  Data ports. PostgreSQL, Citus, ClickHouse, SQLite, and other engines MAY be
  compatibility adapters, migration sources, or differential-test oracles;
  none MAY be canonical metadata, transaction, or analytical authority. An
  agreed Data port MUST contain contract values, errors, and traits only;
  concrete stores, executors, transport normalization, and in-memory oracles
  MUST live in matching core, adapter, facade, or test-oracle faces before port
  promotion.
- **ensure:** core has no foreign database client or runtime dependency;
  parameterized conformance runs against the owned engine and retained
  adapters; removing an adapter leaves the canonical contract unchanged.
  Port-promotion review inspects the complete source cone and rejects concrete
  `BTreeMap` stores, SQL execution, transport status projection, or mutable
  recording implementations behind the agreed face.
- **overturn_when:** measured evidence proves an external engine satisfies the
  complete authority, isolation, format, upgrade, and exit contract, and a
  same-wave founder decision records licensing and migration guarantees.

</owned_engine>

<topology>

## Decision: bounded cells and separated runtime roles

- **achieves:** horizontal growth and bounded blast radius while compute,
  metadata authority, durable record capacity, and repair scale independently.
- **origin:** one PostgreSQL pool or one globally uniform cluster couples query
  load, consensus, compaction, repair, and failure into one scaling unit.
- **rule:** Data MUST be cell-based and ship one signed Rust distribution with
  independently deployable gateway/query compute, metadata/placement, tablet
  data, and repair/rebalance roles. Query compute MUST be stateless with
  respect to durable records. Cells MUST have explicit capacity and quorum
  bounds; production MUST NOT depend on one unbounded global consensus group.
- **ensure:** tests scale and kill roles independently; adding query capacity
  does not move data, adding tablets does not require gateway replacement, and
  cell-loss exercises cannot create a second writer.
- **overturn_when:** fault and unit-cost evidence proves a simpler topology
  preserves the same elasticity, isolation, and recovery bounds.

## Decision: consensus authority, cached routing, fenced tablets

- **achieves:** no authority RPC on every query without allowing stale routing
  or network partitions to create two writers.
- **origin:** consistent hashing and gossip choose candidates but do not decide
  ownership generations, drain state, capacity, or safe handoff.
- **rule:** tablet maps, membership, ownership, placement, split/move, and
  fencing MUST be versioned and consensus-backed. Compute MAY cache maps and
  gossip MAY carry health hints; neither MAY grant write authority. Tablet
  nodes MUST reject stale epochs.
- **ensure:** deterministic partition tests prove one committing owner per
  generation; handoff publishes only after copy, durable verification, and an
  atomic epoch change; normal point operations avoid a global-controller hop.
- **overturn_when:** a formally specified alternative proves equivalent
  single-writer, stale-router, split/move, and recovery properties.

</topology>

<consistency_and_time>

## Decision: cell-local transactions and ordinal versions

- **achieves:** precise relational consistency without inventing a second
  clock or hiding WAN coordination behind a global timestamp.
- **origin:** ADR-0719 assigns `Now() -> Interval` to Cell and states that Data
  versionstamps are engine commit ordinals, not wall time.
- **rule:** committed OLTP transactions MUST be linearizable within their home
  cell and use engine commit ordinals for version identity. Data MUST consume
  the Cell interval API, MUST NOT expose a private `Now()`, and MUST NOT claim a
  global commit time. The `commit_wait` adapter MUST remain available but MUST
  stay IR-disabled for v1 NTP unless measured epsilon and an accepted SLO make
  waiting preferable to restart.
- **ensure:** clock widening, skew, and adapter-switch tests preserve ordering;
  stale epochs fail before mutation; no persisted key or version derives its
  identity from wall time.
- **overturn_when:** a measured clock plant and founder-accepted decision
  replace ADR-0719's time or cross-cell consistency contract.

</consistency_and_time>

<bounded_contract>

## Decision: public resource bounds are contract semantics

- **achieves:** predictable constant work and fail-closed overload behavior
  before an owned engine or public facade can be exposed.
- **origin:** prose requiring bounded requests, bytes, collections,
  concurrency, and in-flight memory left adapters free to choose incompatible
  maxima, unchecked arithmetic, and allocation order.
- **rule:** records-contract v1 MUST freeze request and response/result hard
  byte/count/concurrency maxima, accounting units, checked-overflow behavior,
  validation order, response-credit backpressure, and stable refusal identities
  before semantic implementation. A mutating transaction MUST size and reserve
  its complete result before `PREPARED`; it MUST NOT commit and then discover
  that the result cannot be represented within the contract. An adapter MAY
  configure a lower admitted profile but MUST NOT raise a hard maximum or
  acknowledge work before all applicable bounds and authority evidence pass.
- **ensure:** `SPEC.md` is the single v1 bounds table; D1c contract suites run
  exact-limit, limit-plus-one, malicious length/count, arithmetic-overflow,
  decode/encode-amplification, result-sizing, response-credit saturation, and
  in-flight-byte matrices through Cargo and Buck for every adapter.
- **overturn_when:** a versioned Data contract replaces the limits with
  measured values and preserves bounded work, stable compatibility/refusal,
  tenant isolation, and an explicit migration window.

</bounded_contract>

<request_authority>

## Decision: server-derived request identity and authenticated continuation

- **achieves:** one replay, idempotency, authorization, and scan-resumption
  identity that cannot be chosen, confused, or extended by a caller.
- **origin:** an unversioned `request_fingerprint` and an unspecified
  continuation token leave field order, normalization, tenant/snapshot
  binding, integrity, expiry, and key rollover to each adapter.
- **rule:** records-contract v1 MUST derive the request fingerprint on the
  trusted side from the canonical, domain-separated frame in `SPEC.md`; body
  claims are never authority. Scan continuation MUST be an opaque,
  authenticated, expiry-bounded capability bound to tenant, principal, query,
  snapshot, tablet epoch, and hard limits. Unknown, retired, or unavailable
  key generations, uncertain time, tamper, and foreign-context replay MUST
  fail closed with the frozen public error mapping. Protobuf is a wire mapping,
  not the fingerprint grammar, and no protobuf accounting claim is executable
  until its D-33 schema/toolchain/codegen gate has landed in Cargo and Buck.
- **ensure:** independent canonical-frame encoders produce the same golden
  digests; permutation, normalization, tamper, cross-tenant, cross-snapshot,
  expiry-boundary, replay, key-roll, unknown-field, and Cargo/Buck codegen-
  parity campaigns run before any routed facade.
- **overturn_when:** a versioned records contract replaces both grammars while
  preserving idempotency compatibility, bounded work, tenant/snapshot
  confinement, an explicit rollover window, and downgrade refusal.

</request_authority>

<record_security>

## Decision: owner-local security ports, encrypted durable state, gated providers

- **achieves:** default-deny authorization, durable audit, and tenant-bound
  encryption without importing another capability's core or treating a test
  double as production authority.
- **origin:** generic references to IAM, Audit, Secrets/KMS, rotation, and
  zeroization do not identify a lawful dependency graph, ciphertext format,
  key-generation fence, recovery rule, or readiness withdrawal condition; the
  currently visible provider packages are not accepted Data consumption faces.
- **rule:** Data MUST own implementation-free policy-client, audit-sink,
  record-key, and record-protection ports. Concrete provider adapters MUST
  depend only on those ports and separately accepted provider-owned sold ports
  or facades, never provider core or internal API packages. Every durable
  record, WAL entry, segment, snapshot, repair copy, and migration artifact
  MUST use the versioned AEAD envelope and AAD in `SPEC.md`; plaintext or
  unaudited fallback is forbidden. Key issue, nonce-range lease, rotation,
  revocation, re-encryption, restore, and zeroization MUST be fail closed and
  observable. No process may publish a route or readiness until real provider
  conformance receipts and an encrypt-active generation are present.
- **ensure:** the D1c security gate names the exact provider decisions and
  Cargo/Buck graph before structure; known-answer, wrong-AAD, nonce-reuse,
  tamper, KMS/PDP/Audit outage, rotation/revocation race, crash-resume,
  ciphertext-only recovery, and secret-remanence campaigns precede the D4
  route join.
- **overturn_when:** accepted provider contracts and a versioned cryptographic
  migration replace these ports or format while retaining tenant isolation,
  durable pre-ACK evidence, rollback fencing, and recoverability.

### Amendment — opaque KMS bootstrap, classified WAL, monotonic artifact root, and pre-use nonce reservation

- **achieves:** removes the contradiction between rejecting raw Data key
  material and requiring locally held AES key bytes, makes opaque KMS recovery
  executable after a cold restart, binds every legal mixed-class WAL without
  splitting its atomic transaction, makes the published artifact root monotonic
  and GC-safe across restore/failover, and makes AES-GCM nonce reuse impossible
  across a crash boundary.
- **origin:** the first D1c draft both prohibited a raw key/provider client in
  a Data contract and described a Data-visible zeroizing 32-byte key lease;
  it also advanced the nonce checkpoint only before acknowledgement, after a
  ciphertext could already exist. The next draft made an encrypted
  `KeyGenerationBinding` the sole provider reference, which cannot reacquire an
  Open handle needed to decrypt the record containing it; its artifact commit
  root was prose rather than an authenticated frame. The next repair left WAL
  with a caller-selectable-looking single class for a legal mixed transaction,
  allowed a retained valid head to replace a newer one after restore, and let GC
  collect verified pre-CAS objects. The subsequent review proved that an
  ordinary losing CAS could still retain a nonterminal pin forever and that the
  presumed trusted coordinator had no executable Data owner, term-rollover, or
  serving-path shape.
- **rule:** `secrets-kms-use` is the required accepted provider face for an
  opaque, tenant-and-purpose-bound AEAD handle. Data contracts carry only
  non-serializable operation handles, encrypted opaque key-generation binding,
  and a bounded authenticated `KeyBootstrapLocatorV1` in the envelope header.
  The bootstrap locator, not encrypted binding alone, authorizes provider
  catalog resolution of the exact tenant/purpose/key/generation/fence/revision
  before the first post-restart Open. KK/`RecordKeySource` is the sole owner of
  `AcquireOpenHandle`; KX/`RecordProtection` consumes its typed lease and maps
  only Seal/Open over the direct Cargo/Buck port edge. Data MUST NOT expose raw
  key bytes or a provider client. Every legal purpose uses the exact
  19-field `ContextAadV1` grammar and a purpose-valid plan: record is exactly
  one entry at at most 4 MiB, WAL is exactly one entry at at most 16 MiB, and
  only aggregate purposes may use 1..4,096 entries. Record tag `09` is its
  canonical primary-key binding; WAL tag `09` is the digest of one
  bounded ordered transaction-class summary. A uniform WAL carries its sole
  DataClass/revision; a mixed WAL carries WAL-only `(0,0)` plus that exact
  summary digest, never a default/rank/split. The binding digest must agree in
  AAD, plan, final manifest, and sealed canonical
  `ArtifactCommitRecordV1`; the summary transaction identity/ordinal must agree
  in every role AAD and the sealed commit. Each artifact becomes visible only
  through the 78-byte head plus a durable
  immutable-context/monotonic-generation/fence anchor and a fresh Audit
  high-water receipt. Data owns the cell-local `ArtifactPublicationCoordinator`
  at `data/ports/draft/artifact-publication`,
  `data/core/artifact-publication-domain`, and
  `data/adapters/draft/artifact-publication-cell`, composed only by
  `data/facade/records-app`: it alone owns tuple CAS, pin/member/decision/
  receipt history, safe-GC epoch, successor takeover, and the authenticated
  Audit receipt callback. A durable
  logical-epoch pin plus its coordinator-only local-CAS/decision proof protects
  every verified data/manifest/commit object through CAS, Audit retry, and
  finalization. A normal losing CAS has a bounded terminal
  superseded/released path; only genuinely undecidable recovery is retained
  under bounded pin/byte/backlog admission. Reads use a fresh validated
  in-cell snapshot rather than an Audit RPC on every hit, and a changed tuple,
  term, or expiry fails closed. GC never guesses around an expired or crashed
  pin. `NonceLeaseId:u32` is the sole
  nonce identity. The provider MUST durably reserve a disjoint range before
  returning it; one exclusive lease owner MUST atomically CAS+fsync `next` past
  a counter before submitting that nonce to Seal. If recovery cannot prove the
  bootstrap/binding, ownership, publication high-water, or latest nonce state,
  it burns/quarantines the affected authority and withdraws readiness. KMS owns
  raw-key lifetime and zeroization proof.
- **ensure:** `SPEC.md` fixes the satisfiable byte formulas, closed AAD
  codepoints/field count/purpose maxima (record 1,825 and migration 1,572),
  `47,414`-byte WAL summary, purpose-specific record/WAL count-one and
  total-plus-one plan vectors while retaining the `65,640`-byte aggregate plan,
  `459,190`-byte manifest, `2,296`-byte commit, 177-byte anchor, 200-byte
  local-CAS receipt, 266-byte pin-decision, and 203-byte pin,
  request/receipt/error fields, and bootstrap/acquire/reserve/pin-renew/
  takeover/CAS/Audit/seal/persist/publish/release/ACK/reacquire crash matrix.
  `PLAN.md` puts KC's
  canonical envelope before any
  persistence content, makes KS (and transitive S/WS) a prerequisite of every
  provider structure lane, inserts the coordinator structural/content lanes
  before Audit callback, durable persistence, and records-app readiness, and
  gives KK--not KX--the sole acquisition mapping over the direct Cargo/Buck
  edge. Conformance scans reject raw-key-shaped Data
  values and test malformed/tampered/replayed/truncated/substituted frames,
  mixed/uniform/metadata/control WAL summaries, H0-after-H1 restore replay,
  immutable-context/generation/fence regressions, A/B/N-writer CAS loss and
  successor takeover, coordinator epoch re-attestation, cell-local snapshot
  refresh, pin/GC races, stale/idempotent CAS, every crash boundary, concurrent
  allocation, N/N+1 counter/chunk, lease burn, bootstrap catalog source loss,
  restart/rotation/revocation, and refusal paths.
- **overturn_when:** an accepted replacement provider/publication contract
  proves equally opaque tenant-bound operations, independently authenticated
  cold recovery, purpose-total mixed-transaction classification, rollback-proof
  monotonic publication, terminal conflict reconciliation, bounded GC
  reachability, cell-local freshness, durable non-reuse, raw-key containment,
  zeroization evidence, and the same crash/rotation refusal coverage.

</record_security>

<operational_identity>

## Decision: semantic operational names, provenance-only decision identifiers

- **achieves:** operators and callers can understand a Data process, error,
  test, or log without consulting the historical decision-number index.
- **origin:** four compatibility cones contain five live runtime emissions or
  assertions of `IP-002`, `IP-003`, `IP-004`, `IP-013`, or `IP-015`: the
  ClickHouse adapter, analytics usecase, tenant-bootstrap library,
  tenant-bootstrap process, and analytics process. Those identifiers record
  provenance but do not describe the failed operation.
- **rule:** Data executable, process, job, error, test, log, metric, and other
  code-facing names MUST be semantic. Decision identifiers MAY remain in ADR
  citations, comments, rustdoc, and package metadata, but MUST NOT be the
  operator-visible error or status identity. Existing numbered diagnostics are
  compatibility residue and MUST pass through D1b-N's behavior-preserving
  structure-then-content sequence before feature work touches that cone.
- **ensure:** the D1b-N suite exercises every emitted deferred adapter error,
  export refusal, quota-reconciliation refusal, and both process boot statuses
  using semantic identities and exact exit/output contracts; source review
  separately proves that retained `IP-*` text occurs only in provenance-bearing
  comments, documentation, or metadata.
- **overturn_when:** an accepted external protocol requires a stable numbered
  identifier and the same surface retains a semantic operator-facing label
  alongside it.

</operational_identity>

<olap_and_pipelines>

## Decision: one record authority, derived analytical state

- **achieves:** OLTP, OLAP, and dataset transforms scale by workload without
  creating multiple authoritative versions of one record.
- **origin:** a row store, column store, and job engine can otherwise become
  three products with divergent mutation semantics.
- **rule:** the OLTP commit log MUST be the authoritative change order. OLAP
  layouts and pipeline outputs MUST be immutable, checkpointed projections
  published by explicit generation. They MUST NOT acknowledge a source
  mutation or silently become OLTP authority. Cloud software delivery remains
  `pipeline/`; Data pipelines are record/dataset transforms.
- **ensure:** replay, duplicate, gap, and out-of-order tests converge or fail
  closed; projection freshness and lineage are measurable; partial results are
  never published as a completed generation.
- **overturn_when:** an accepted consistency model names another authority and
  proves deterministic recovery, lineage, and rollback across all three roles.

</olap_and_pipelines>

<interfaces_and_migration>

## Decision: canonical contract first; prove, shadow, cut over

- **achieves:** an owned core can replace current stores workload by workload
  without a big-bang rewrite or dual-write ambiguity.
- **origin:** current consumers bind directly to PostgreSQL helpers, while the
  ClickHouse and analytics surfaces are incomplete; replacing protocol,
  persistence, placement, and consumers together would erase the oracle.
- **rule:** Data MUST define one versioned engine-neutral semantic contract and
  migrate in reversible cohorts: preserve the current adapter as oracle,
  shadow reads, durably capture ordered changes, compare results, fence one
  authority epoch, and then cut over. Two stores MUST NOT accept authoritative
  writes for one cohort. Connect/protobuf remains the canonical platform
  contract; PostgreSQL wire compatibility is not implied by the SQLx adapter.
- **ensure:** each cohort records source, schema, policy, authority epoch,
  comparison result, rollback barrier, and expiry; replay is idempotent and
  refuses gaps; old stores remain readable until parity and rollback evidence
  are durable.
- **overturn_when:** an independently reviewed migration protocol proves a
  smaller sequence with equal rollback and acknowledged-write safety.

</interfaces_and_migration>

<cross_owner_decisions>

## Decisions required before D1c

Two surfaces exceed Data-only amendment jurisdiction:

1. The sold wire surface: ADR-0719 makes Connect/protobuf canonical. A supported
   PostgreSQL wire facade requires an explicit founder/facade decision with a
   version, compatibility envelope, authorization path, and retirement policy.
2. Physical persistence: runtime separation does not by itself decide whether
   tablet WAL/segments remain internal to the records engine or consume a
   future agreed Storage contract. D1c cannot depend on
   `storage/ports/draft/provider`; any shared port requires provider-owner and
   architecture acceptance before consumers land.

</cross_owner_decisions>

## Rejected destinations

- PostgreSQL, Citus, CockroachDB, YugabyteDB, ClickHouse, or another database as
  the permanent product identity.
- A big-bang rewrite or permanently co-authoritative dual writes.
- One global cluster, flat gossip ownership, or a consistent-hash ring without
  consensus epochs and fencing.
- An embedded LSM presented as distributed consistency.
- Wall-clock version identity or global commit-time claims.
- OLAP projections or pipeline output as a second mutation authority.
- Ontology/Foundry, search/RAG, object bytes, or Bus delivery in Data core.
