---
id: ADR-0397
title: "Pulsar 4.x + Oxia canonical event-bus (reconstructed record)"
status: Superseded
date: 2026-06-12
authority: founder
owner: council-architecture
planning_impact: true
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0005, ADR-0011, ADR-0132, ADR-0195, ADR-0476, ADR-0478, ADR-0479, ADR-0481, ADR-0482, ADR-0510, ADR-0536, ADR-0537, ADR-0557]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0397 — Pulsar 4.x + Oxia canonical event-bus

> **RECONSTRUCTION.** This record was cited before it was written. Seven governed surfaces
> cite "ADR-0397 Pulsar 4.x + Oxia canonical event-bus" (ADR-0476, ADR-0478, ADR-0479,
> ADR-0481, ADR-0482, ADR-0557, and `specs/master-plan-sequencing.json` wave 15-ZG), but no
> decision file ever existed at this number — audit register H-19
> (`docs/audit/initial-sweep-2026-06-06/00-MASTER-CONTRADICTION-REGISTER.md`) flagged the
> dangling edge. Reconstructed 2026-06-12 from the citing context only (leader+founder call:
> backfill the record at the cited number, do not rewrite history; FRIC-1781430000). Every
> normative sentence below traces to a citing site, to an on-disk predecessor decision, or to
> the dated founder directives quoted in §Founder directive. Where the citers are silent,
> this record says so and leaves the point to founder ratification.
> **Founder ratification required — door: one-way, pending founder.** Status stays Proposed
> until ratified.

## Status

Proposed — reconstructed 2026-06-12. The citing ADRs (all Accepted, dated 2026-05-28) treat
the decision as already made "this session" (ADR-0557 §Context), so the substance below was
operative-in-fact from 2026-05-28; only the record was missing.

## Context (as established by the citers)

- ADR-0005 (Accepted, 2022) made Kafka the canonical eventing backbone with the
  transactional-outbox pattern.
- ADR-0195 (Accepted 2026-05-18) introduced Apache Pulsar (4.2.x) as the log-broker substrate
  for the stream-processing tier, with KoP (Kafka-on-Pulsar) wire-compat for Kafka-protocol
  consumers.
- ADR-0557 (Accepted 2026-05-28; originally misfiled as a duplicate ADR-0377 and renumbered
  per FRIC-1781390000) records: "ADR-0397 (this session) then confirmed Pulsar 4.x + Oxia as
  the canonical event-bus, superseding any competing choice", and executes the standalone-
  Kafka retirement on that basis. Research basis: `docs/research/kafka-reeval-2026-05-28.md`.
- `specs/master-plan-sequencing.json` wave 15-ZG lists "ADR-0397 Pulsar 4.x + Oxia canonical
  event-bus" as a dependency of the Kafka-retirement wave.

## Decision (assumed by the citers)

Each clause names the citing source it is reconstructed from.

### D1 — Pulsar 4.x is the sole canonical event-bus and log-broker substrate

The cluster runs Apache Pulsar 4.x as the single canonical event-bus and log-broker
substrate, superseding any competing eventing-substrate choice (ADR-0557 §Context "confirmed
Pulsar 4.x + Oxia as the canonical event-bus, superseding any competing choice" and
§Decision 1 "the sole canonical event-bus and log-broker substrate"; sequencing 15-ZG scope
"Pulsar 4.x + Oxia is the sole canonical event-bus/log-broker"). Kafka-wire clients are
served via the KoP proxy; the standalone-Kafka retirement mechanics are owned by ADR-0557,
not by this record.

### D2 — Oxia is the metadata/coordination store of the canonical posture

Every citation of this record pairs Pulsar 4.x with Oxia in title position ("Pulsar 4.x +
Oxia canonical event-bus" — ADR-0557 §Related, sequencing 15-ZG). In Pulsar 4.x, Oxia is the
horizontally-scalable metadata-store option that replaces the ZooKeeper metadata role. The
citers state the pairing but do not spell out Oxia's deployment scope (see §Assumed by citers
vs NOT yet decided).

### D3 — Relationship to ADR-0195 and ADR-0557

ADR-0195 introduced Pulsar as the stream-processing-tier log-broker; this record confirms
Pulsar+Oxia as canonical for the whole eventing substrate; ADR-0557 then retires standalone
Kafka via KoP wire-compat, depending on both (ADR-0557 §Context; sequencing 15-ZG
`depends_on`). The broker-agnostic streaming semantics of ADR-0005 (transactional outbox,
at-least-once delivery, consumer-group fanout, CloudEvents envelope, Protobuf payload,
per-tenant/per-cell partitioning) carry forward unchanged (ADR-0557 §Decision 4; sequencing
15-ZG scope).

### D4 — Capabilities the citing ADRs build on

The citers assume the canonical event-bus provides:

- **Multi-tenant topics and namespaces** — Kafka topics map to Pulsar persistent topics under
  a tenant namespace (ADR-0557 §Decision 2); multi-tenancy primitives are a named unlock
  (ADR-0557 §Consequences).
- **Cross-region geo-replicated topics** — oya-identity replicates session state "via Pulsar
  (ADR-0397) cross-region topics" (ADR-0476 §D5).
- **Typed-event consumption at scale** — oya-meter consumes typed CloudEvents from Pulsar on
  `usage.{tenant}.{resource}.{action}.v1` topics (ADR-0479 §D2, §Integration).
- **Event fan-out for billing** — oya-billing uses Pulsar "for billable-event stream
  fan-out"; all billing state changes emit Pulsar events; oya-meter usage events arrive via
  Pulsar (ADR-0478 §D1, §D2, §D3).
- **Feature-flag plane eventing** — ADR-0481 binds oya-flags to this record in its `related`
  front-matter (no body-level assumption to reconstruct).
- **Tiered storage, geo-replication, schema registry** — named Pulsar feature unlocks
  (ADR-0557 §Consequences).

### D5 — Transitional substrate, not the terminal shape

ADR-0482 (Tier 3, 24–60 months) names "Apache Pulsar (ADR-0397)" as the incumbent that the
bespoke `oya-events` substrate supersedes at "multi-tenant durable event bus parity at FAANG
scale", bridged by Pulsar parallel-run with protocol-compat ingress. ADR-0536 §D-13 rules
Pulsar "the validated launch-primary broker, consumed only through a thin owned Rust client
interface (ADR-0510 transitional-behind-interface)". This record therefore canonizes Pulsar
4.x + Oxia for the transitional window only — see §Owned-stack posture.

## Founder directive (2026-06-12, verbatim, post-reconstruction)

Relayed during the reconstruction lane, distinct from the reconstructed-from-citers content
above; these are dated founder instructions, not citer assumptions.

Directive 1:

> "pulsar oxia transient stack with replacement rust stack already planned and architected.
> make sure to adopt patterns that will allow far better scaling, better performance,
> decoupled, idempotent, reliable etc all the modern best practices and cutting
> edge/trailblazing methods."

Directive 2:

> "try to approach from where the bottleneck is for pulsar oxia and kafka, what their biggest
> complaints are and look to address those in elegant research backed methods. decouple
> things that could be the hotspot like hyperscalers would."

Consequences of the directives for this record:

1. Pulsar 4.x + Oxia are EXPLICITLY ADR-0510 transient — adapter-absorbed, never the port
   shape. The replacement Rust stack is already planned and architected: the owned messaging
   port is `libs/oya-messaging-substrate-kernel` (ADR-0536 §D-13; the G009 owned-messaging
   vertical), with the transactional-outbox producer edge in
   `libs/oya-shared-transactional-outbox-kernel` /
   `libs/oya-shared-transactional-outbox-dispatch-app`. Those surfaces are the destination
   authority this record defers to; nothing here re-architects them.
2. The mandated-patterns section (§Mandated patterns) and the bottleneck-driven requirements
   (§Bottleneck-driven design requirements) are load-bearing: port/contract shapes must
   encode them NOW so the Rust cutover inherits them for free.
3. Where a citing ADR's assumption conflicts with these patterns, the founder directive wins
   and the conflict is noted. Audit of the seven citing sites found no conflict: no citer
   assumes global ordering or an exactly-once transport promise.

## Owned-stack posture

Pulsar is a Java/JVM system and Oxia is a Go system: both are ADR-0510 transitional,
adapter-absorbed, never load-bearing in any port shape (owned_stack_policy, root
`CLAUDE.md`; founder directive 2026-06-09 — ports are designed for the owned stack: trait
shapes model the W5 destination, adapters absorb transient infra). The owned messaging
destination (G009 vertical) is modeled by `libs/oya-messaging-substrate-kernel` (ADR-0536
§D-13): queue/stream/bus trichotomy as three single-concern surfaces (ADR-0132) over ONE
substrate, at-least-once transport + transactional outbox + consumer-side idempotency =
effectively-once processing, per-key ordering only, seekable cursors, two loss classes
(ADR-0537). Cutover litmus, applied to every port this record touches: "would this trait
change at Rust-stack cutover?" — the answer must be no; if yes, the boundary is wrong and
must be redrawn at the adapter.

## Mandated patterns (founder directive 1 — load-bearing)

The port/contract surfaces over the transitional Pulsar substrate MUST encode, so the owned
Rust destination inherits them with zero trait change:

1. **Idempotent producers + consumer-side idempotency keys** — at-least-once delivery with
   dedup as the reliability model; no exactly-once API surface (ADR-0536 §D-13 rejected
   global exactly-once promises; `oya-messaging-substrate-kernel` delivery contract).
2. **Transactional outbox for DB-coupled emission** — already shipped:
   `libs/oya-shared-transactional-outbox-kernel` + `libs/oya-shared-transactional-outbox-dispatch-app`
   (+ sqlx adapter, poller, worker, tokio runtime apps); carried forward from ADR-0005 per
   ADR-0557 §Decision 4.
3. **Decoupled schema-evolution contracts** — event topic + schema contracts live as
   AsyncAPI surfaces in the contract registry (ADR-0011), CloudEvents envelope + Protobuf
   payload carried broker-agnostic (sequencing 15-ZG scope).
4. **Partition/key design for horizontal scale** — ordering is promised per-key only
   (`MessageKey` in the kernel); per-tenant/per-cell partitioning carries forward
   (sequencing 15-ZG scope).
5. **Backpressure-aware consumption** — credit-based flow control at the protocol level
   (reactive-streams semantics; Pulsar's consumer-permit model is the transitional
   implementation).
6. **Dead-letter + replay semantics** — seekable cursors (kernel stream surface) and DLQ
   policy as first-class contract state, not broker configuration folklore.
7. **Ordered-per-key, never global ordering** — no contract may promise cross-key or
   cross-topic ordering (kernel delivery contract).

## Bottleneck-driven design requirements (founder directive 2)

Requirements ON the owned Rust destination, derived from the documented bottlenecks of the
three systems this record canonizes transitionally. Encoded now so ports/contracts model
them; the full destination architecture remains the G009 lane's own decision record. Each
remedy carries its research precedent.

### Kafka's documented pain → required remedies

| Bottleneck (documented complaint) | Required remedy on the destination | Research precedent |
|---|---|---|
| Broker-disk coupling: partition data lives on broker disks, so rebalance = data movement, scaling is expensive, recovery is slow | Disaggregated log: object-storage-native tiering with a local NVMe write buffer; storage scales independently of compute | WarpStream (diskless Kafka-protocol-on-S3, stateless agents); AutoMQ (stateless brokers over object storage + WAL buffer); Confluent Freight clusters (direct-to-object-store tier) |
| Consumer-group stop-the-world rebalances + partition rigidity | Subscription/cursor state as first-class transactional state DECOUPLED from the log (the Pulsar cursor model done right), plus cooperative incremental assignment | Pulsar managed-cursor/subscription model; Kafka KIP-429 cooperative rebalancing and KIP-848 next-gen consumer protocol (upstream's own admission of the pain) |
| Fragile exactly-once transactions coordinator (zombie fencing, coordinator complexity) | At-least-once + idempotent producers + consumer dedup keys as the reliability model (already mandated, §Mandated patterns 1) | ADR-0536 §D-13 (rejected global exactly-once); AWS SQS at-least-once doctrine |
| Per-partition head-of-line blocking | Per-key ordering with work-queue semantics unified alongside streams (queue+stream unification) | Pulsar's queue+stream thesis (shared/key-shared subscriptions); Apache Iggy's unified model |

### Pulsar's documented pain → required remedies

| Bottleneck (documented complaint) | Required remedy on the destination | Research precedent |
|---|---|---|
| Three-system operational sprawl: brokers + BookKeeper + ZooKeeper/Oxia | Single-binary, shared-nothing design for the destination substrate | Redpanda (single-binary Kafka-compatible, no ZK/JVM); Apache Iggy (single Rust server) |
| BookKeeper double-write (journal + entry log) ⇒ write amplification; catch-up/backlog reads interfere with the hot write path | Single-write-path storage engine with read isolation: tiered/backlog reads served from object storage, never touching the hot write path | WarpStream/AutoMQ object-store read path; Pulsar tiered-storage offload (the mitigation pattern, made structural) |
| Java GC tail latency across every component (broker, BookKeeper, ZK) | Rust no-GC implementation + thread-per-core/shared-nothing execution + io_uring/zero-copy I/O | Redpanda on Seastar (thread-per-core, C++); ScyllaDB (same model vs JVM Cassandra); Apache Iggy — the Rust streaming system, PRIMARY reference research for the owned destination |

### Oxia/metadata pain → required remedies

| Bottleneck (documented complaint) | Required remedy on the destination | Research precedent |
|---|---|---|
| Metadata as the scaling wall: ZooKeeper's documented ceiling is Oxia's raison d'être — and Oxia itself is Go, young, and still a SEPARATE system to operate | Align with the repo's existing quorum doctrine: multi-Raft ranges within cells, no single consensus group for substrate state, no cross-cell quorum (founder directive 2026-06-10; ADR-0536 control-plane posture) | CockroachDB multi-Raft ranges; Oxia's own shard-per-range design notes (the problem statement, not the destination implementation) |
| Full consensus where consensus is overkill (coordination hotspot) | Epoch-fenced object-store CAS (conditional writes) for coordination state that does not need a quorum | WarpStream-style object-store coordination; S3 conditional-write fencing lineage |

### Cross-cutting decoupling mandate (the hyperscaler move)

Identify every hotspot and SEPARATE it: metadata from data, write path from read path,
storage from compute, subscription state from the log, control plane from data plane.
Credit-based flow control at the protocol level for backpressure (reactive-streams
semantics). These splits are encoded as port boundaries in the G009 surfaces, so the
transitional Pulsar adapters and the owned Rust destination implement the SAME contracts.

## Assumed by citers vs NOT yet decided

**Assumed by all citers (reconstructed above):** D1–D5.

**Disagreement among citers:** ADR-0195 pins "Apache Pulsar 4.2.x"; ADR-0557 and the
sequencing node say "Pulsar 4.x". This record adopts the looser "4.x" (the form every
ADR-0397 citation uses) and leaves minor-version pinning to founder ratification.

**NOT decided by any citer — left to founder ratification:**

- Oxia deployment scope and the BookKeeper storage-layer posture (the citers pair "Pulsar
  4.x + Oxia" but never describe the storage/metadata topology).
- Whether this record formally supersedes any clause of ADR-0005 or ADR-0195 (supersession
  edges here are deliberately empty; ADR-0557 owns the executed ADR-0005 substrate-clause
  supersession). The H-34/X5 audit tension — ADR-0195's "ClickHouse MV + Kafka Engine"
  default naming vs the Pulsar-only substrate — is acknowledged, not resolved here.
- Capacity, sizing, retention, and per-tenant quota policy for the canonical bus.
- Security/tenancy configuration details beyond the multi-tenant namespace assumption (D4).
- The bespoke `oya-events` cutover criteria beyond ADR-0482's Tier-3 parity statement.

## Consequences

- The seven citing surfaces (ADR-0476/0478/0479/0481/0482/0557 + sequencing 15-ZG) resolve
  to a real decision file with zero retargeting; the decision-crosswalk face gains an
  ADR-0397 row at settle.
- The phantom-citation defect class becomes mechanically detectable: the
  `phantom_decision_citation` lane of GATE-1 (cloud-ci-cross-artifact-agreement) is
  born-blocking frozen-empty for NEW phantom citations (FRIC-1781430000), with the
  pre-existing phantom inventory carried as explicit, ledgered, shrink-only DATA
  (emitted into the decision-crosswalk face as `grandfathered_phantom_ids`; the live
  gate test enforces anti-padding — a healed id must leave the inventory — and a
  decrease-only size ceiling). RED fixture:
  `specs/fixtures/cross-artifact-agreement/tc-XA-bad-phantom-citation.json`.
- Founder ratification (one-way door) either flips this record to Accepted — possibly
  amending the open points above — or retargets the citers; until then the record is the
  honest, minimal closure of the citation graph.
