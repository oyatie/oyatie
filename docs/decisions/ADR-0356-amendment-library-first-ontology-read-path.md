---
id: ADR-0356
status: Rejected
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-privacy
  - council-security
  - ops-sre-reliability
  - ops-compliance
  - axis-ontology
  - axis-policy-engine
  - axis-workflow-engine
  - axis-audit-chain
  - axis-tenancy
  - axis-intelligence
supersedes: []
amends:
  - ADR-0257-ontology-object-type-versioning-deprecation-handshake.md
superseded_by: []
related:
  - ADR-0005-eventing-backbone-outbox-pattern.md
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0050-outbox-to-kafka-pattern.md
  - ADR-0055-glossary-ontology-not-object-graph.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0106-ontology-architecture.md
  - ADR-0122-ontology-terminology-fold.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0141-workflow-ontology-read-path-direct.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-service-mesh-cilium.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0353-amendment-library-first-network-opt-in-clarification.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - ADR-0355-amendment-library-first-network-opt-in-clarification.md
  - ADR-NNNN-library-first-credential-sidecar
related_specs:
  - /specs/microservices/ontology.json
  - /specs/knowledge-graph-schema.json
  - /specs/ontology-schema-revision-format.json
  - /specs/ontology-deprecation-handshake.json
  - /specs/platform-architecture.json
  - /specs/tenant-model.json
related_memory:
  - feedback_workflow_objectgraph_adapter_layer
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_quality_performance_scalability_bar
  - feedback_bominal_inheritance_precedence
  - feedback_glossary_ontology_not_object_graph
  - feedback_cedar_as_universal_gate
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_clean_architecture_requirements
doc_class: Architecture-Decision-Record-Amendment
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: promotion-gate-fix-library-first-symmetry-2-of-2
amendment_anchor: F4-LV-6 / F4-AP-1 / F-ANTI-3
enforcement_status: advisory-until-ontology-read-client-library-lands
enforced_by:
  - oya gate validate ontology-read-library-first-default
  - oya gate validate ontology-read-network-opt-in-cedar-gated
  - oya gate validate no-unnecessary-ontology-read-service-hop
  - oya gate validate ontology-read-library-only-failure-perimeter
  - oya gate validate ontology-read-projection-coherence
  - oya gate validate ontology-write-always-network-via-service
  - oya gate validate ontology-read-credential-sidecar-coherence
---

# ADR-0356: Amendment — Library-First Ontology Read-Path Clarification

## Status

Proposed — 2026-05-20.

This is an **amendment** to ADR-0257 (Ontology Object Type Versioning &
Deprecation Handshake, 2026-05-20). It does not supersede ADR-0257; it
clarifies the **delivery shape** and **runtime call topology** of the
Ontology read-path so the substrate does not, by accident, re-introduce
the universal-mediator pattern that ADR-0145 retired and that the
ADR-0255 and ADR-0246 amendments closed for Intelligence and Policy-
Engine respectively. ADR-0257 is the appropriate base ADR because it
governs the *versioned read contract* (per its `amends` of ADR-0145:
"clarifies versioned read contract") and because no standalone "ADR-
0278 Ontology read-path doctrine" exists in the keystone bundle as of
2026-05-20 — the F4 verdict catalogues the gap as F4-LV-6 / F4-AP-1 /
F-ANTI-3 and recommends the amendment land against the closest existing
Ontology ADR. ADR-0257 is that ADR: it is Proposed in the keystone
bundle, it explicitly amends ADR-0145 (the doctrine being protected),
and it governs the read-side schema semantics that the library-first
read path will compose against.

ADR-0141 (Workflow + Ontology read-path direct; 2026-05-18) was the
prior attempt at the same concern and was Superseded by ADR-0145. The
present amendment restores ADR-0141's read/write-split intent inside
ADR-0145's "no universal mediator" frame, but extends it from "direct
gRPC to Ontology µservice" to "library-first in-process projection
with network opt-in" so the read path achieves library-shape parity
with Intelligence (ADR-0255 amendment) and Policy-Engine (ADR-0246
amendment).

The amendment is filed as a **promotion-gate fix (2 of 2)** per the
keystone-bundle 2026-05-20 synthesis §5.13 (F4 library-first symmetry).
The defect it prevents — `microservices/ontology/` becoming the
platform-wide read-mediator for every cross-µservice entity query — is
structurally identical to the pre-ADR-0145 universal-mediator shape
that F-ANTI-1 surfaced for Intelligence, F-ANTI-2 surfaced for Policy-
Engine, and F-ANTI-3 surfaced for Ontology.

Enforcement is `advisory-until-ontology-read-client-library-lands`. CI
lanes that enforce this amendment promote to BLOCKER once:

1. The `oya-shared-ontology-read-*` crate family is scaffolded per §D-2
   below with the **library-mode-default** projection embedded
   in-process, and at least one µservice (pilot: `microservices/social/`
   or equivalent cross-product read consumer) is demonstrated to consume
   the library without making a gRPC call to `microservices/ontology/`
   on the read path.
2. The `oya-check-no-unnecessary-ontology-read-service-hop` static
   analysis lane is authored and exercised against the pilot reference
   path.
3. The `tenants` table includes the `ontology_read_mode` enum attribute
   described in §D-5 below (per-tenant override of the library-first
   default), surfaced in the ADR-0244 tenant DDL.
4. The Ontology µservice retains its **write-path** authority unchanged
   (every CREATE / UPDATE / DELETE / TOMBSTONE goes through the µservice
   per ADR-0141 §Decision and ADR-0257 §D-4 deprecation handshake) and
   exposes its read endpoints as the opt-in network surface for read
   callers under `network_only` tenant mode or cross-cell read scopes.
5. The Ontology µservice publishes per-tenant per-cell CDC streams
   (Kafka topic `ontology.entity.{cell_id}.{tenant_id}` + a separate
   schema-revision-events topic) that library subscribers consume to
   materialise local projections.
6. ADR-0257 §D-3 + §D-4 are annotated with a forward-pointer to this
   amendment so any reader lands here before forming a read-path
   runtime-topology mental model.
7. The reference architecture diagram in
   `docs/architecture/ontology-substrate-read-path-runtime-topology.md`
   is drawn (new file) to show the library-first read path as the
   default solid edge, the network hop (cross-cell read + `network_only`
   tenant opt-in) as a dashed opt-in edge, and the write path as an
   always-network solid edge to the µservice.
8. The Slice-2 sidecar key-holder primitive (ADR-NNNN-library-first-
   credential-sidecar) lands and is referenced by the Ontology read-
   path library's credentialed external-LLM ontology enrichment path
   per §D-2 (when an enrichment step calls a provider LLM that requires
   a provider-BYOK credential, the credential is held in the sidecar, not
   in the read-path library's main process).

Until those eight items land, validators emit findings without failing
CI. Post-bootstrap, the lanes promote to BLOCKER.

## Date

2026-05-20.

## Context

### F-ANTI-3: the F4-architecture finding that triggered this amendment

The 2026-05-20 multispectrum-review v2.4.0 F4-Architecture verdict
(`evidence/debate/keystone-bundle-2026-05-20-F4-architecture-r1.json`)
issued finding **F4-LV-6 / F4-AP-1** (catalogued as F-ANTI-3 in the
keystone-bundle idea-refine deep-dive):

> The library-first / network-opt-in amendment to ADR-0255 closes
> F-ANTI-1 for Intelligence. The same anti-pattern threat applies to
> ADR-0246 (Policy-Engine — F-ANTI-2) and to Ontology (F-ANTI-3). …
> ADR-0245 §D-3.A classifies ontology as substrate-data; ADR-0145
> already said "Ontology is a SUBSTRATE for cross-µservice read
> queries, not a GATEWAY" but the bundle does not lock the read-
> replica-per-cell + materialised-view + CDC-update + per-product-
> subset-binding pattern as defaults. F-ANTI-3 in the idea-refine
> deep-dive recommended an ontology read-path doctrine; this
> ADR is not in the keystone bundle. Without it, every cross-product
> Ontology read becomes a synchronous network hop and the universal-
> gateway pathology emerges via Ontology.

The finding is not a defect in ADR-0257's *intent*. ADR-0257's intent
is to lock down the Ontology Object Type schema-evolution surface so
that cross-µservice consumers can pin to a versioned schema. That
intent is preserved by this amendment. The defect is in the unspecified
**read-path delivery shape**: ADR-0257 establishes versioned-read
contracts but does not specify whether the read traverses a gRPC hop
to `microservices/ontology/` per request or whether the caller's
process holds a materialised projection that satisfies the read
in-process.

This amendment forecloses the per-request gRPC reading as the default
for entity reads and establishes the library-link reading (with local
CRDT-materialised projection) as the canonical delivery shape for
**reads**, while preserving the µservice's authority over **writes**
(ADR-0257's deprecation handshake, ADR-0141's write-path orchestration,
and ADR-0145's audit/tracing invariants are unaffected).

### Why F-ANTI-3 is structurally identical to F-ANTI-1 and F-ANTI-2

F-ANTI-1 (Intelligence), F-ANTI-2 (Policy-Engine), and F-ANTI-3
(Ontology read-path) share the same shape. The shape is:

1. **A substrate concern** that requires *consolidation* of cross-
   cutting state (provider-adapter inventory for Intelligence; fragment
   registry for Policy-Engine; canonical entity write authority + schema
   registry for Ontology).
2. **A per-call hot-path step** that is the substrate's primary user-
   visible work (LLM dispatch for Intelligence; Cedar evaluation for
   Policy-Engine; entity read for Ontology).
3. **An implementation ambiguity** between "the per-call work happens
   in-process in the caller via a library backed by a local
   materialised projection" vs "the per-call work traverses a gRPC hop
   to the substrate µservice."

The hyperscaler reference for Ontology read-path resolves the ambiguity
to library-in-process with a locally materialised projection backed by
a CDC stream:

- **Palantir Foundry caller-side ontology projections.** Foundry's
  Ontology service exposes a **bulk projection API** plus a **CDC
  event stream**; consumer applications (AIP Logic, AIP Threads,
  Code Workbook, Workshop) maintain in-application projections that
  serve reads in-process. The Ontology service is the canonical write
  authority + schema registry, not a per-read mediator. This is the
  direct precedent ADR-0145 cited but did not specify in delivery
  terms.
- **AWS S3 Express One Zone client SDKs** (2024 GA). Express One Zone
  caches account-scoped metadata at the client SDK; reads against
  cached metadata satisfy in-process without an S3 control-plane
  round-trip. The shape — central service for canonical state + client
  SDK for per-read locality — is the Ontology shape.
- **DynamoDB DAX** (Accelerator) — caller-side cache layer with CDC
  invalidation; per-read latency at single-digit microseconds because
  reads do not hit DynamoDB's primary endpoint.
- **Google Spanner client SDK with TrueTime-aware staleness reads** —
  caller-side staleness reads against a local snapshot; the central
  service is the write authority.
- **Stripe.js + Stripe client SDK** — caches account-scoped metadata
  (products, prices, customers' minimum-projection fields) in the
  caller's process; full reads still hit the API, but the common case
  (validating an account's status during a checkout) is served from
  the cache.

The same six failure modes ADR-0255 amendment and ADR-0246 amendment
each enumerated apply to a µservice-mediated read default for Ontology:

1. **SLO ceiling.** Every cross-µservice read becomes bounded by
   Ontology's availability. Because the read path satisfies >60% of
   the cross-µservice latency budgets per ADR-0141 §Context, the
   ceiling propagates to most user-visible product surfaces.
2. **Failure perimeter.** A regional outage of `microservices/ontology/`
   in cell X cascades to every cross-µservice read in cell X. The
   blast radius covers every product that consumes another product's
   canonical entities.
3. **Latency tax.** Every cross-µservice read adds a network round-trip
   (caller → ontology → caller). At cell-internal latency budgets
   (~2-10 ms one way over the mesh) plus Ontology's own query cost
   (~5-30 ms for a typed entity fetch), the tax is 20-60 ms per read.
   The `social-post fan-out` p99 budget of 1 s (per
   `docs/standards/cross-microservice-latency-budget.md`) breaks if
   every Person, Task, Document, Recording, Attachment, and Comment
   entity is fetched via gRPC.
4. **Capacity coupling.** Ontology's capacity becomes the platform
   capacity for cross-µservice reads. Sizing errors at Ontology become
   sizing errors everywhere.
5. **Observability inversion.** The natural span hierarchy for a
   cross-µservice read is `caller → ontology-read`. The library
   collapses this to a single in-process span.
6. **Distributed monolith.** Ontology's schema-evolution cadence
   becomes coupled to every consumer. Schema changes require
   coordinated deploys across N callers. ADR-0257's deprecation
   handshake exists precisely to manage this risk, but the handshake
   is more honestly applied when consumers hold local projections than
   when they per-request-fetch.

These six failure modes are exactly the modes ADR-0145 §Context cited
as the reason to retire the universal-mediator pattern. Re-introducing
them under the Ontology label is not acceptable.

### What library-first means for Ontology reads specifically

The pattern is well-precedented for materialised projections. The
canonical embedded usage looks like:

```rust
use oya_shared_ontology_read::{OntologyProjectionClient, QueryBuilder};

// At service startup: open a long-lived projection client. The
// client subscribes to the CDC stream for its tenant scope + the
// entity types it depends on, and materialises a local CRDT-backed
// projection.
let projection = OntologyProjectionClient::open()
    .for_tenant("acme-corp")
    .subscribe_object_types(&[
        "person.v3",
        "task.v7",
        "document.v2",
    ])
    .start()
    .await?;

// At read time: query the local projection. No network call.
let person = projection
    .get_object::<Person>("person.v3", person_id)
    .await?;  // <-- in-process; ~50-500 µs.

let recent_tasks = projection
    .query::<Task>(QueryBuilder::for_type("task.v7")
        .filter("assigned_to", person_id)
        .order_by("due_at", Order::Asc)
        .limit(20))
    .await?;  // <-- in-process; ~1-5 ms for indexed query.
```

The projection is a **per-process materialised view** keyed by tenant
and Object Type version. The materialisation primitive is a CRDT
(specifically: per-Object-Type Last-Writer-Wins map keyed by
(`entity_id`, `schema_revision`) with semantically-merged property
values per the per-property merge strategy declared in the Object Type
schema). The CRDT property gives the projection the merge-safety it
needs to ingest CDC events out of order, deduplicate retransmissions,
and converge across pod restarts.

The Ontology µservice continues to be the **canonical write authority
+ schema registry**. The library does *not* mediate writes; writes
always traverse the µservice (per §D-4 below). The library mediates
*reads* via the local projection that converges toward the µservice's
canonical state via CDC.

### What ADR-0145 actually said about Ontology (re-stated)

ADR-0145 Invariant 3:

> **Ontology projection invariant.** µservices that own canonical
> entities (Person, Task, Document, Recording, etc.) MUST project them
> into Ontology for cross-µservice queryability. Ontology IS the
> canonical READ substrate for cross-µservice entity data.
>
> But: Ontology is a SUBSTRATE, not a GATEWAY. µservices may also call
> each other directly via mTLS gRPC for transactional/synchronous
> needs; Ontology query is the preferred path for cross-µservice
> entity reads where latency budget permits.

This amendment formalises Invariant 3's "SUBSTRATE, not a GATEWAY"
language by specifying that the read substrate is *consumed via
library-with-local-projection*, not *via per-request gRPC mediation*.
The substrate's authority over canonical entity state is preserved;
its mediator role is foreclosed.

The amendment also recovers and supersedes ADR-0141 (Superseded by
ADR-0145). ADR-0141's "read path direct" intent was correct; ADR-0145
broadened the doctrine to "no universal mediator" and superseded ADR-
0141 because the read-path-direct framing was too narrow. The present
amendment extends ADR-0141's intent within ADR-0145's frame: reads are
library-first with local projection; writes traverse the Ontology
µservice (and Workflow Engine when sagas apply).

### Why a CRDT projection (not a Postgres replica, not a Kafka changelog reader)

Three named alternatives to "in-process CRDT projection" were
considered:

- **Per-process Postgres read replica.** Each caller process opens a
  read replica of Ontology's canonical Postgres. Pros: zero schema
  drift; full SQL queryability. Cons: connection pool exhaustion at
  ~200 caller pods × 3 replicas × ~10 connections each = 6000
  connections per cell vs Postgres connection-pool budget of ~1000;
  no per-tenant scoping at the storage layer; cross-pod write
  invalidation requires Postgres logical replication slots that don't
  scale to this fan-out. Rejected.
- **Per-process Kafka changelog reader without merge primitive.**
  Each caller reads the CDC topic from the beginning and applies
  events to a local KV store. Pros: simple. Cons: out-of-order delivery
  + duplicate redelivery + property-level merge conflicts (two
  concurrent updates to the same entity's same property) produce
  read-your-writes anomalies; no convergence guarantee across pods
  that crash and resume from different offsets; per-tenant filtering
  requires per-tenant Kafka topics (which Ontology already publishes
  per §D-2 below, mitigating part of this concern). Rejected for the
  convergence-guarantee gap.
- **In-process CRDT projection** (CHOSEN). Per-Object-Type LWW map
  keyed by `(entity_id, schema_revision)` with semantically-merged
  property values per the per-property merge strategy declared in the
  Object Type schema. Convergence is automatic: any two library
  instances that have consumed the same CDC event set will hold
  identical projections regardless of consumption order. Out-of-order
  delivery and duplicate redelivery are absorbed by the CRDT merge
  function. The schema-revision pinning (per ADR-0257 §D-3) maps
  cleanly onto the CRDT's per-revision key partition: callers that pin
  to `person.v3` see only `(entity_id, v3)` keys, ignoring `v4` and
  newer projections until they bump their pinned revision.

### Why writes always traverse the µservice (asymmetry is intentional)

The asymmetry between reads (library-first) and writes (µservice-only)
is deliberate and matches the hyperscaler precedent:

- **Palantir Foundry.** Ontology *mutations* (Object Type schema
  changes, Action invocations that create/update/delete) flow through
  Workflow orchestration to Foundry's Ontology service. *Reads* are
  served from caller-side projections.
- **Google Spanner.** Writes flow through Spanner's central transaction
  coordinator. Reads (with appropriate staleness) are served from
  zone-local replicas with caller-side staleness tolerance.
- **AWS DynamoDB DAX.** Writes flow through DynamoDB. Reads are served
  from DAX's caller-adjacent cache layer with CDC invalidation.
- **Stripe.** Writes (creating customers, charging cards) flow through
  api.stripe.com. Reads of cached account state are served from the
  client SDK's local cache.

ADR-0257's deprecation handshake (ACTIVE → DEPRECATED → TOMBSTONED) is
a write-path concern — it governs how Object Type schema *evolves*,
which is a mutation at the schema level. The handshake therefore
remains routed through the Ontology µservice unchanged; this amendment
does not weaken it. ADR-0141's "write-path orchestrated" intent is
preserved unchanged: every state-changing inter-µservice call
(CREATE / UPDATE / DELETE; any operation that emits an audit row; any
operation that crosses a Cedar admission boundary on `Action::write_*`)
MUST flow through the Ontology µservice for entity writes and through
Workflow Engine for orchestrated write sagas. The library exposes no
write API.

## Decision

### D-1. Ontology Substrate reads are library-first

The Ontology Substrate's **per-call read surface** is delivered as a
library by default. The canonical entry point is the
`oya-shared-ontology-read-*` crate family. Every caller (every
µservice's read handler that fetches a cross-µservice entity, every
Workflow Engine step that reads an Object, every Foundry workflow that
queries an Ontology projection, every product page-render that joins
entities across µservices) links the library and calls the library's
in-process `get_object(...)` / `query(...)` API against the local
materialised CRDT projection.

The library, not a µservice, is the user-visible surface of the
Ontology read-path for the default consumer.

The Ontology µservice's bounded contexts (per ADR-0106 architecture +
ADR-0257 deprecation handshake) split across read and write surfaces:

| BC | Read-path delivery | Write-path delivery |
|---|---|---|
| Object Type registry | **In-process bundle** (library holds compiled schema set; schema-revision events refresh the bundle). | **In the µservice** (schema authoring, deprecation handshake, tombstone state machine). |
| Object instance store | **In-process CRDT projection** (library holds per-tenant per-Object-Type LWW map). | **In the µservice** (canonical write authority; Postgres + Citus shard on `(tenant_id, object_type_version, entity_id)`). |
| Action invocation | **N/A** (Actions are state-changing; not on the read path). | **In the µservice** (per ADR-0172 Ontology Action Receipt canonical shape; emits ActionReceipt audit row). |
| Function library | **In-process** (library composes Functions against the local projection). | **In the µservice** (Function authoring, version pinning per ADR-0257 §D-3). |
| Link store | **In-process projection** (links are entities of type `Link::<from_type, to_type>`; same CRDT projection). | **In the µservice** (canonical write authority). |
| Vector property store | **In-process projection with vector index** (HNSW or IVF built locally from CDC events). | **In the µservice** (canonical write authority + vector-index rebuild coordination). |
| Geo property store | **In-process projection with R-tree index** (built locally from CDC events). | **In the µservice**. |
| Time-series property store | **In-process projection** (small windows held locally; deep history opt-in via network read). | **In the µservice**. |
| Ciphertext property store | **In-process projection** (ciphertext blobs held locally; decryption credentials per Slice-2 sidecar). | **In the µservice**. |
| Struct property store | **In-process projection** (nested structs serialised in CRDT property values). | **In the µservice**. |
| CDC publisher | **In the µservice** (publishes per-tenant per-cell topics). | N/A. |
| Schema-revision publisher | **In the µservice** (publishes schema-revision events). | **In the µservice**. |
| Deprecation handshake | **N/A** (write-path concern). | **In the µservice** (per ADR-0257 §D-3). |
| Tombstone state machine | **N/A** (write-path concern). | **In the µservice**. |
| Audit emission | **In the caller's process** (per ADR-0145 Invariant 1) for read sampling rows; **in the caller's process** (signed via Slice-2 sidecar) for write rows. | **In the caller's process** (canonical seal emission per ADR-0145 Invariant 1). |

The library is the unit of consumption for reads. The Ontology
µservice is the unit of consumption for writes.

### D-2. What the library performs in-process

The `oya-shared-ontology-read-*` library performs **all** of the
following work in the caller's own process:

| Concern | In-process responsibility |
|---|---|
| Object fetch by ID | `projection.get_object::<T>(object_type, entity_id)` returns the LWW-merged property bundle from the local CRDT map. ~50-500 µs typical. |
| Filtered query | `projection.query::<T>(QueryBuilder)` evaluates the predicate against the in-process projection index. ~1-5 ms typical for indexed queries; ~10-50 ms for full-scan filters over 100k entities per tenant per Object Type. |
| Schema-revision pinning | Library exposes only the Object Type versions the caller declared at startup. Newer revisions visible in CDC events are still consumed but exposed under their own version key; unpinned versions are not visible to caller queries. |
| CRDT merge | Per-property merge function dispatches to per-Object-Type-declared strategy (LWW, set-union, counter-add, custom). Convergence guaranteed across out-of-order CDC delivery. |
| CDC subscription | Long-lived Kafka consumer subscribed to `ontology.entity.{cell_id}.{tenant_id}` topics for each tenant the library serves. Per-tenant per-Object-Type filter applied at the consumer to reduce in-process memory pressure. |
| Schema-revision-event consumption | Separate Kafka consumer subscribed to `ontology.schema-revision.{cell_id}`. New revisions trigger compiled-schema-bundle refresh in-library. Schema events are <100 KB and propagate sub-second. |
| Cedar gate on read | In-process call to `oya-shared-policy-engine-client-sdk::evaluate(...)` (per ADR-0246 amendment) for read admission. The Cedar fragment `ontology-read-admission.cedar` permits the read based on principal + Object Type + tenant scope + data class. Library-first by default per ADR-0246 amendment. |
| Tenant-scope enforcement | Library refuses any read whose `(principal, requested_tenant)` is not authorised by the local Cedar evaluation. Cross-tenant reads require explicit Cedar opt-in fragment. |
| Audit emission (read-sampled) | Sampled per-tenant per-day read-summary rows (per ADR-0141 §"Audit-chain row optional, sampled") emitted from the caller's process via `oya-shared-audit-chain-client` + Slice-2 sidecar key-holder. Sample rate per Cedar fragment (e.g., `audit.ontology_read.sampling_rate = 0.001` for high-volume reads). |
| OTel propagation | In-process call to `oya-shared-tracing-client`. Span: `ontology.read`. No artificial mediator span. |
| Brown-out signal | Library emits ADR-0176 brown-out signal locally when projection staleness exceeds the per-tenant freshness target (default 5 s for hot Object Types; declared per Object Type in the schema). |
| Decision cache | Optional LRU cache of `(query_hash, schema_revision) → result_set` with short TTL (default 5 s) for repeated reads within a request scope. In-process. |
| Read-staleness floor | Library refuses reads whose required staleness is tighter than the local projection can guarantee (e.g., a caller requesting strict read-your-writes for an entity it just wrote via the µservice must wait for the corresponding CDC event to arrive; the library exposes a `wait_until_at_least(write_lsn)` primitive to coordinate). |
| Credentialed enrichment (external-LLM) | When a read query requires enrichment via an external LLM (e.g., embedding similarity search calls a provider-BYOK LLM provider), the LLM credential is held by the **Slice-2 sidecar key-holder** (ADR-NNNN-library-first-credential-sidecar); the read library calls into the sidecar via UDS to perform the LLM call. The read library's main process never holds the LLM credential beyond the immediate UDS call. |

All fourteen concerns happen in the caller's process or via non-
blocking async stream consumption. No synchronous network hop to
`microservices/ontology/` is required for any of them on the default
read path.

### D-3. The library composes reads in-process against the materialised projection

After the library has performed its in-process work (projection
lookup, CRDT merge, Cedar gate green, audit row emitted via sidecar
co-located key-holder, OTel span open), the caller receives the result
directly — the library returns the result; the caller's request
continues on its normal path.

The read topology is:

```
caller process
  │
  │ (in-process)
  │  oya-shared-ontology-read-* library
  │   ├─ projection lookup         (in-process; 50 µs)
  │   ├─ CRDT merge (per-property)  (in-process)
  │   ├─ schema-revision check     (in-process; bundle compiled at refresh)
  │   ├─ Cedar gate                (in-process via policy-engine library)
  │   ├─ audit-emit (sampled)      (in-process via UDS to sidecar key-holder
  │   │                              per Slice-2 ADR-NNNN-library-first-
  │   │                              credential-sidecar)
  │   ├─ OTel span open            (in-process)
  │   └─ result returned           (Object instance or query result-set)
  │
  │  (background, non-blocking)
  │   ├─ Kafka consumer  ──◄── ontology.entity.{cell_id}.{tenant_id}
  │   │                                                     ▲
  │   │                                                     │ CDC stream
  │   ├─ Kafka consumer  ──◄── ontology.schema-revision.{cell_id}
  │   │                                                     ▲
  │   │                                                     │ schema events
  │   ▼
  │  microservices/ontology/  (canonical write authority + CDC publisher)
```

There is **no synchronous network hop to `microservices/ontology/`**
on the read path. The CDC consumer is asynchronous and non-blocking;
its purpose is to keep the local projection fresh, not to serve any
individual read.

This is the **default** path. It is what ≥99% of reads take unless the
caller has explicitly opted in to the network-side read features
described in §D-4 + §D-5.

### D-4. The Ontology µservice exists for writes + canonical schema + CDC publishing + opt-in reads

`microservices/ontology/` continues to exist, and its bounded-context
split is preserved. What this amendment changes is the **runtime
responsibility surface** for reads. Writes are unaffected — they
remain mediated by the µservice. Specifically:

| Concern | Why it cannot live in the library | Network-side responsibility |
|---|---|---|
| Canonical entity write authority | Writes require the deprecation handshake state machine + the audit-row admission gate + the Cedar permit + the schema-revision validation. None of these can run safely in a caller's process because cross-caller serialisation + idempotency keys + saga compensation require central coordination. | Object instance store BC + Action invocation BC retain their full µservice surface for writes. Every CREATE / UPDATE / DELETE / TOMBSTONE flows through the µservice. |
| Schema authoring + deprecation handshake | Schema-revision lifecycle (ACTIVE → DEPRECATED → TOMBSTONED) per ADR-0257 §D-3 + §D-4 is a coordinated cross-µservice state machine. Cannot be distributed. | Object Type registry BC + Schema-revision publisher BC retain authority. |
| CDC publication | The CDC stream is the canonical source-of-truth for projection state. Publishing must happen exactly once per write transaction and must be globally ordered per (tenant, Object Type, entity). | CDC publisher BC owns the per-tenant per-cell Kafka topics. |
| Schema-revision distribution | Schema events (new ACTIVE revision; DEPRECATED grace start; TOMBSTONED) must reach every library instance within sub-second. | Schema-revision publisher BC owns the topic. |
| Cross-cell read federation | A query that spans multiple cells (e.g., a federated view of `task.v7` instances across all cells the tenant has been sharded to per ADR-0248 §D-7 shuffle sharding) cannot be served from a single cell's projection. The library only holds the local cell's projection. | Ontology µservice's `query-federated` endpoint coordinates the cross-cell fan-out via async sub-queries to peer cells' Ontology instances. Opt-in for callers that genuinely need cross-cell reads. |
| Cross-tenant read coordination | Cross-tenant reads (per ADR-0244 tenant-as-universal-scoping-primitive; permitted only under explicit Cedar opt-in) require a coordinator that can validate the cross-tenant permit + emit the cross-tenant audit row + observe the cross-tenant rate limit. | Ontology µservice's `query-cross-tenant` endpoint coordinates; library does not participate in cross-tenant reads. |
| Untrusted-tier read mediation | A caller in an untrusted cell tier (per ADR-0248 §D-7) cannot be trusted to hold the projection in-process without leaking it or returning stale entities to its consumer. For these callers, reads must go to a centrally-attested read endpoint. | Ontology µservice's `Read` gRPC endpoint serves untrusted-tier callers and `network_only` tenant mode callers per §D-5. |
| Bulk export / DSAR cascade | Bulk export of all of a tenant's entities (DSAR cascade per ADR-0257 §References; export to data warehouse) requires a coordinated scan + bulk-write to the export sink. Not in scope for per-call reads. | Ontology µservice's `bulk-export` endpoint coordinates. |
| Audit-stream consumer | Per-call read-sample audit rows emit at the caller's library; cross-cell aggregate read views require a rollup. | Subscribe to the audit-chain stream's `OntologyRead` rows; aggregate. Read-only consumer; not a per-call participant. |
| Vector index rebuild coordination | When a Vector property type undergoes index rebuild (e.g., HNSW parameter change), the rebuild requires globally-coordinated re-embedding + index swap. Cannot be distributed safely. | Vector property store BC owns the rebuild coordinator. |

None of the ten concerns above is on the **synchronous per-call
default-tenant read path**. They are control-plane and batch concerns
(except for the opt-in cross-cell / cross-tenant / untrusted-tier
paths, which are explicit escalations). The runtime topology preserves
the static-stability property: when the Ontology µservice is
unavailable, default-tenant per-call reads continue to function
(degraded only in the cross-cutting sense — new writes do not propagate
until the µservice recovers; projection becomes increasingly stale;
cross-cell federation falls back to local-only).

**Writes are explicitly out of scope for the library.** This amendment
does not weaken the write-path orchestration. Every write traverses
the µservice. Workflow Engine remains the orchestration substrate for
multi-step write sagas. ADR-0257's deprecation handshake remains
authoritative.

### D-5. Callers opt in to network-side Ontology reads per Cedar policy and per tenant attribute

Most read callers default to the library-only path. A caller that
**needs** network-side read coordination opts in per call (or per
audience, or per tenant) via two surfaces: a per-tenant attribute and
a per-Cedar fragment.

**Per-tenant attribute opt-in (new in this amendment).** The `tenants`
table (per ADR-0244) gains a column governing the tenant-wide read
mode:

```sql
ALTER TABLE tenants
    ADD COLUMN ontology_read_mode ontology_read_mode_t NOT NULL DEFAULT 'library_first';

CREATE TYPE ontology_read_mode_t AS ENUM (
    'library_first',
    'network_only',
    'library_first_with_freshness_floor'
);
```

Semantics:

- `library_first` (default): the library serves reads from the local
  projection. This is the canonical mode for the overwhelming majority
  of tenants.
- `network_only`: every read for this tenant goes via gRPC to the
  Ontology µservice's `Read` endpoint. This is the mode for tenants
  in untrusted-cell-tier deployments, tenants whose compliance pack
  mandates centralised read posture (e.g., financial-services
  tenants under packs requiring real-time-strict consistency), or
  tenants whose entity-volume is small enough that local
  projection overhead exceeds the per-read network cost.
- `library_first_with_freshness_floor`: library serves reads from the
  local projection, but if the projection's freshness is older than a
  pack-declared threshold (e.g., 1 s for trading-tenant Object Types),
  the library falls back to the network endpoint. This is the mode
  for compliance packs that require maximum freshness guarantees
  without giving up the static-stability property entirely.

**Per-call Cedar opt-in.** A Cedar fragment in `ontology/fragments/
baseline/` governs the opt-in decision:

```cedar
permit (
    principal,
    action == Ontology::Action::"NetworkSideRead",
    resource is Tenant
) when {
    resource.ontology_read_mode == "network_only" ||
    (resource.ontology_read_mode == "library_first_with_freshness_floor"
        && context.projection_age_milliseconds > resource.freshness_floor_milliseconds)
};
```

Cross-cell reads have their own fragment:

```cedar
permit (
    principal,
    action == Ontology::Action::"CrossCellRead",
    resource is QueryScope
) when {
    resource.cell_count > 1 &&
    context.federated_query_authorised == true
};
```

Cross-tenant reads have their own fragment:

```cedar
permit (
    principal,
    action == Ontology::Action::"CrossTenantRead",
    resource is QueryScope
) when {
    resource.tenant_set.size() > 1 &&
    principal in resource.cross_tenant_collaborators
};
```

The caller's library checks the relevant Cedar fragment; permit routes
to the network endpoint (or to the federated coordinator); forbid (or
NotApplicable) routes to the in-process projection.

This per-policy opt-in keeps the network side as an *explicit*
escalation, not an *implicit default*. The hyperscaler shape that
ADR-0145 established is preserved.

### D-6. Default is library-only reads; network-side is opt-in; writes are always network

The library defaults to **local-only in-process reads** for every
caller whose tenant `ontology_read_mode = library_first` and whose
read scope is single-cell + single-tenant.

The default's properties:

1. The Ontology µservice has zero per-read network traffic from the
   caller for default reads.
2. The Ontology µservice's availability does not bound the caller's
   read availability.
3. The caller's latency budget does not include an Ontology round-
   trip for default reads.
4. The caller's failure perimeter for reads does not include the
   Ontology µservice.

Opt-in's properties (for callers under `network_only` or
`library_first_with_freshness_floor` modes, or for cross-cell /
cross-tenant scopes, or for untrusted-tier callers):

1. The library performs a gRPC `Read` (or federated/cross-tenant
   query) RPC before returning.
2. The RPC is on the synchronous path; the caller's latency budget
   includes one extra round-trip per read.
3. The caller's failure perimeter includes the Ontology µservice
   *for reads*. If the RPC fails open (per Cedar fallback policy),
   the caller proceeds with the stale local projection; if it fails
   closed, the caller fails the read.

Writes' properties (unchanged from ADR-0141 + ADR-0145 + ADR-0257):

1. Writes always traverse the Ontology µservice.
2. Multi-µservice write sagas always traverse Workflow Engine.
3. Each write emits its own audit-chain seal at the **calling**
   service (per ADR-0145 Invariant 1).
4. ADR-0257's deprecation handshake gates all schema-changing writes.

### D-7. ADR-0145 alignment statement

ADR-0145 Invariant 3 stated: **Ontology is a SUBSTRATE for cross-
µservice read queries, not a GATEWAY.**

This amendment makes that doctrine explicit. The Ontology library is
parallel in shape to:

- The `oya-shared-intelligence-client-*` library family (per ADR-0255
  amendment). The Intelligence µservice does not mediate every LLM
  call; the library does the dispatch in-process.
- The `oya-shared-policy-engine-client-*` library family (per ADR-0246
  amendment). The Policy-Engine µservice does not mediate every Cedar
  evaluation; the library does the evaluation in-process.
- The `oya-shared-audit-chain-client` library. The audit-chain µservice
  does not mediate every seal; the library emits seals directly.
- The `oya-shared-tracing-client` library. Tempo does not mediate every
  span; the library propagates and emits OTel directly.

Ontology reads join this family. The library is the per-call read
surface; the µservice is the canonical write authority + schema
registry + CDC publisher + opt-in coordinator surface.

The ADR-0145 three invariants apply unchanged:

- Invariant 1 (audit): caller emits its own read-sample audit row via
  library + Slice-2 sidecar key-holder.
- Invariant 2 (tracing): caller emits its own span hierarchy; no
  artificial mediator span.
- Invariant 3 (ontology projection): caller projects canonical entities
  into Ontology via its own writes (per ADR-0145 unchanged); reads
  from Ontology via library projection (new doctrine, this amendment).

**The "no universal mediator" doctrine remains intact.** This amendment
does not introduce a new mediator; it removes the implicit mediator
that ADR-0257's read-path could be read as introducing (and that ADR-
0141's "direct gRPC to Ontology" framing originally accepted before
being superseded by ADR-0145). The library-first reading is the
canonical shape; the µservice retains writes + schema + CDC +
coordination.

If a future amendment to this amendment proposes to re-introduce a per-
call gRPC mediator for reads under any label, that amendment must
explicitly overturn ADR-0145 Invariant 3. This amendment's existence
documents the position that such an overturning would have to clear.

### D-8. SLO + failure perimeter consequences

The library-first read default produces three operational properties:

1. **Read SLO ceiling is removed.** The platform-wide SLO for cross-
   µservice reads is bounded by the caller's own SLO, not by Ontology's
   SLO. If the caller is 99.95% available, cross-µservice reads inherit
   that SLO. Ontology's SLO bounds *write* availability and CDC
   *freshness*, not per-read availability.
2. **Failure perimeter contracts.** When the Ontology µservice is down,
   per-call default reads continue to function from the local
   projection. Degradation is limited to: writes block (correctly —
   writes require the µservice); projection becomes stale (callers
   read entities as of the last CDC event); cross-cell federation falls
   back to local-only; cross-tenant reads fail closed (correctly —
   cross-tenant reads require coordinator approval).
3. **Read latency tax is removed.** The default read path adds ~50-500
   µs of in-process projection lookup + ~50-500 µs of Cedar evaluation,
   compared to the ~5-50 ms network round-trip + ~5-30 ms Ontology
   query that the µservice-mediated reading would have imposed. For
   ADR-0141 §Context's per-hop budget analysis, the saving is 30-150 ms
   per cross-µservice read.

These three properties are required to honor the hyperscaler-bar
quality target (`feedback_quality_performance_scalability_bar`) for
cross-µservice read functionality across the portfolio.

## Alternatives

### Alternative 1 — Keep ADR-0257 as-is (status quo without this amendment)

**Description.** Accept ADR-0257's versioned-read-contract language at
face value without specifying delivery shape. Implement reads as a
gRPC endpoint on `microservices/ontology/`. Every cross-µservice read
from every caller issues a gRPC `Read` to Ontology.

**Pros.**

1. Single binary to test and operate for the read surface.
2. Schema-revision pinning is centralized.
3. CDC complexity is avoided at consumers.
4. Strong read-your-writes guarantees because every read hits the
   canonical store.

**Cons.**

1. **Re-introduces the universal-mediator anti-pattern.** F4-LV-6 /
   F4-AP-1 / F-ANTI-3 expressly catalogue this risk.
2. **SLO ceiling.** Every cross-µservice read inherits Ontology's
   availability.
3. **Failure perimeter.** A regional Ontology outage cascades to every
   cross-µservice read.
4. **Latency tax.** 30-150 ms per read per ADR-0141 §Context.
5. **Capacity coupling.** Ontology becomes the platform capacity for
   reads.
6. **Distributed monolith.** Coordinated deploys across N consumers
   for any schema change.

**Rejected.** This is exactly the anti-pattern ADR-0145 retired and
that the ADR-0255 + ADR-0246 amendments closed for Intelligence and
Policy-Engine. The defect ADR-0145 closed in PR #143 is the same defect
this alternative would re-open for Ontology reads.

### Alternative 2 — Per-process Postgres read replica

**Description.** Each caller process opens a read replica of Ontology's
canonical Postgres. Per-call reads are SQL queries against the
in-process replica.

**Pros.**

1. Zero schema drift.
2. Full SQL queryability.
3. Strong consistency at replica's freshness.

**Cons.**

1. Connection pool exhaustion.
2. No per-tenant scoping at the storage layer.
3. Cross-pod write invalidation requires Postgres logical replication
   slots that don't scale to this fan-out.
4. Per-process Postgres is operationally heavy.

**Rejected.** Postgres replication doesn't scale to the per-caller-
process fan-out required.

### Alternative 3 — Library-first CRDT projection with CDC subscription (CHOSEN)

**Description.** Per §D-1 through §D-8 above. The library holds a per-
tenant per-Object-Type CRDT projection; CDC subscription keeps it
fresh; reads serve from the projection in-process. The µservice retains
writes + schema + CDC publishing + opt-in coordination.

**Pros.**

1. **Aligns with ADR-0145.** Direct calls are the default; coordination
   is opt-in. Same shape as Intelligence client + Policy-Engine client.
2. **Read SLO ceiling removed.** Cross-µservice read SLO is bounded by
   caller, not by Ontology.
3. **Failure perimeter contracts.** Ontology µservice outage does not
   block reads.
4. **No read latency tax** on the default path.
5. **Hyperscaler-shape parity.** Palantir Foundry caller-side ontology
   projections, AWS S3 Express One Zone client SDK, DynamoDB DAX,
   Google Spanner client SDK with staleness reads.
6. **Writes remain coordinated** where they need to be (no weakening
   of ADR-0141 + ADR-0145 + ADR-0257 write-path doctrine).
7. **CRDT convergence guarantees** absorb out-of-order CDC delivery +
   duplicate redelivery + pod restart resync.

**Cons.**

1. **Library memory footprint.** Each caller pod holds its tenant
   subset's projection in-process. Mitigated by per-tenant per-Object-
   Type subscription filtering: a caller only subscribes to the types
   it reads. Typical pod holds ~10-50 MB of projection per tenant for
   the most common 5-10 Object Types.
2. **CDC topic count.** Per-tenant per-cell topics multiply Kafka
   topic count. Mitigated by topic-per-tenant being a managed
   abstraction over Kafka partitions; the actual topic-count is
   per-cell × N partitions, which Kafka scales to.
3. **Schema-revision discipline.** Consumers must subscribe to schema
   events and bump pinned revisions when ACTIVE → DEPRECATED →
   TOMBSTONED progression happens. Mitigated by ADR-0257's
   deprecation handshake giving consumers ≥12-month grace per
   revision.
4. **Eventual consistency for reads.** Reads see projection state as
   of the last CDC event consumed, not as of the most recent write.
   Mitigated by `wait_until_at_least(write_lsn)` primitive for
   strict read-your-writes scenarios, and by the
   `library_first_with_freshness_floor` mode for tenants that need
   tighter freshness.

**Accepted.** This is the shape that honors ADR-0145 Invariant 3,
preserves hyperscaler-bar SLO, and retains ADR-0257's write-path
authority.

## Consequences

### Positive

1. **ADR-0145 universal-mediator retirement is preserved on the
   read-path axis.** F-ANTI-3 (F4-LV-6 / F4-AP-1) is closed in writing
   before any caller code is authored against ADR-0257's read-path.
   The "no universal mediator" doctrine remains intact for the third
   of three structurally identical risk axes (Intelligence closed by
   ADR-0255 amendment; Policy-Engine closed by ADR-0246 amendment;
   Ontology read-path closed by this amendment).
2. **Hyperscaler-bar SLO for cross-µservice reads is reachable.**
   Caller SLO bounds read SLO; Ontology SLO bounds write SLO and CDC
   freshness independently.
3. **Static stability per Hamilton 2007.** The read data path does not
   depend on the control path being up. Ontology µservice can be
   under maintenance, rolling-restarted, or in a regional outage
   without blocking reads.
4. **Latency budget unchanged from baseline.** The default read path
   is in-process projection lookup; ADR-0141 §Context's 30-150 ms-per-
   hop saving is recovered.
5. **Diagnostic locality.** When a read fails, the failure is visible
   in the caller's OTel span as `ontology.read` child of `caller`. No
   artificial mediator span.
6. **Per-µservice scaling preserved.** Reads scale with caller pod
   count; Ontology µservice scales independently with write QPS + CDC
   publish QPS.
7. **Write-path orchestration retained.** ADR-0141 + ADR-0145 + ADR-
   0257 write doctrine is unchanged.
8. **Schema-revision pinning enabled.** Per ADR-0257 §D-3, callers pin
   to specific Object Type revisions. Library exposes only pinned
   revisions; CDC keeps them current.

### Negative

1. **Library version pinning + projection memory footprint.** Operators
   must roll library updates across the workspace; pod sizing must
   account for projection memory. Mitigated by per-tenant per-Object-
   Type subscription filtering.
2. **CDC topic scale.** Per-tenant per-cell topics multiply count.
   Mitigated by Kafka topic+partition scaling.
3. **Eventual-consistency read semantics.** Reads see CDC-as-of state,
   not write-as-of state. Mitigated by the `wait_until_at_least(...)`
   primitive and by `library_first_with_freshness_floor` mode.

### Consequences for ADR-0145's "no universal mediator" doctrine

ADR-0145's load-bearing doctrine is **explicitly preserved** by this
amendment. The amendment introduces no new mediator. Specifically:

1. **Direct service-to-service calls remain the default for reads.**
   Caller process serves reads from its own materialised projection.
   Ontology µservice is not in the read path.
2. **Writes traverse the µservice (unchanged from ADR-0141 + ADR-0145
   + ADR-0257).** This is not a new mediator; it is the canonical
   write authority that ADR-0145 already established as the substrate
   for canonical entity state.
3. **The three weaker invariants apply unchanged.** Audit emission
   (Invariant 1): caller emits its own read-sample seal via library +
   sidecar key-holder. Tracing (Invariant 2): caller emits its own
   span hierarchy. Ontology projection (Invariant 3): the projection
   is now materialised in the caller's process for reads; the canonical
   write authority remains in the µservice.
4. **Workflow remains opt-in for write sagas.** ADR-0145's "use
   Workflow when ..." rubric is unchanged. Reads do not flow through
   Workflow.
5. **Service-mesh substrate is unaffected.** mTLS handshakes per ADR-
   0148 Cilium happen for the Ontology µservice surfaces that remain
   (writes, schema authoring, opt-in reads, federated queries) — not
   for per-call default reads.

If a future amendment to this amendment proposes to re-introduce a
per-call gRPC mediator for default reads under any label, that
amendment must explicitly overturn ADR-0145 Invariant 3's "SUBSTRATE,
not a GATEWAY" doctrine. This amendment's existence documents the
position that such an overturning would have to clear.

### Operational

1. **Authoring sequence.** `oya-shared-ontology-read-*` crates are
   scaffolded with the library-mode-default projection embedded
   **before** the Ontology µservice's `Read` gRPC endpoint is exercised
   against any caller. The µservice's runtime surface for reads is
   scoped to opt-in callers from day one. If a future requirement
   genuinely needs centralized per-call default reads, that requirement
   amends this amendment with a fresh ADR.
2. **CI lane authoring.** The `oya-check-no-unnecessary-ontology-read-
   service-hop` lane scans for unconditional
   `OntologyClient::read_via_service(...)` calls (or equivalent gRPC
   client invocations) that are not gated by a per-tenant
   `ontology_read_mode != library_first` check or a cross-cell /
   cross-tenant Cedar permit. Failures block merge.
3. **Documentation updates.** ADR-0257 §D-3 + §D-4 gain forward-pointers
   to this amendment. The new reference architecture diagram at
   `docs/architecture/ontology-substrate-read-path-runtime-topology.md`
   is authored.
4. **Brown-out signal authoring.** The brown-out signal per ADR-0176
   is emitted by the *library* (local projection-staleness signal) and
   by the *µservice* (CDC-publish-lag signal). Both signals are
   observable; they do not conflict.
5. **Cell-µservice load is reduced.** Per ADR-0148 Cilium Service Mesh,
   mTLS handshake count for Ontology reads drops by ~99% on the
   library-first default. SPIFFE-ID issuance budget at cell-µservice
   eases correspondingly.
6. **Service-mesh egress.** Ontology µservice egress (CDC consumption,
   schema-revision subscription, opt-in read gRPC) flows through Cilium
   per ADR-0148. NetworkPolicy permits library → µservice for these
   specific endpoints only.
7. **Multi-cell deployment.** Each cell publishes its own per-tenant
   CDC topics. Library subscribers in cell X consume cell X's topics.
   Cross-cell reads go through the µservice's federation endpoint
   (opt-in).
8. **Failure-mode runbook.** A new runbook at
   `docs/operators/ontology-substrate-read-path-failure-modes.md`
   enumerates: CDC consumer lag (library detects; caller surfaces
   staleness warning if older than per-tenant threshold); CRDT merge
   conflict (logged; LWW resolution applied per Object Type schema);
   cell-local Ontology µservice down (library proceeds local-only;
   writes block; alarms on the µservice itself); cell-wide network
   partition to CDC topic (library circuit-breaks CDC consumption;
   stale-projection warnings escalate); pod restart (library re-
   subscribes CDC from saved checkpoint + replays from cluster
   compacted-topic snapshot). The runbook is referenced from ADR-
   0257 §F and from the keystone-bundle 2026-05-20 synthesis §5.9
   runbook coverage gate.

## Implementation surface

### Library crates (workspace `crates/`)

The library is delivered as a family of crates. The crate naming
follows the established `oya-shared-{substrate}-{role}-*` pattern:

| Crate | Layer (per ADR-0105) | Responsibility |
|---|---|---|
| `oya-shared-ontology-read-domain` | domain | Caller-facing types: `ObjectTypeVersion`, `EntityId`, `Query`, `QueryBuilder`, `QueryResult`, `Projection`, `OntologyReadError`. Pure types; no I/O. |
| `oya-shared-ontology-read-kernel` | kernel | Trait `OntologyProjectionClient` with `get_object`, `query`, `wait_until_at_least`. Trait `CdcSubscriber`. Trait `SchemaRevisionSubscriber`. Pure traits; no concrete adapter. |
| `oya-shared-ontology-read-projection-app` | app | Default composition of in-process CRDT projection + CDC consumer + schema-revision consumer + Cedar gate + audit-emit + OTel propagation. The library's default `OntologyProjectionClient` impl. **This crate is the library-first default.** |
| `oya-shared-ontology-read-crdt` | adapter | CRDT primitives: per-Object-Type LWW map keyed by `(entity_id, schema_revision)`; per-property merge strategy dispatch. |
| `oya-shared-ontology-read-cdc-consumer` | adapter | Kafka consumer for `ontology.entity.{cell_id}.{tenant_id}` topics; per-Object-Type filter; CRDT-event apply. |
| `oya-shared-ontology-read-schema-consumer` | adapter | Kafka consumer for `ontology.schema-revision.{cell_id}`; compiled-schema-bundle refresh. |
| `oya-shared-ontology-read-cedar-gate` | adapter | Cedar gate evaluator wrapper; uses `oya-shared-policy-engine-client-sdk::evaluate(...)` per ADR-0246 amendment. |
| `oya-shared-ontology-read-vector-index` | adapter | In-process HNSW or IVF vector index built from CDC events for Vector property type queries. |
| `oya-shared-ontology-read-geo-index` | adapter | In-process R-tree geo index for Geo property type queries. |
| `oya-shared-ontology-read-time-series-index` | adapter | In-process time-series index (small windows held locally). |
| `oya-shared-ontology-read-audit-emit` | adapter | Read-sample audit emission via `oya-shared-audit-chain-client` + Slice-2 sidecar key-holder UDS call for signing. |
| `oya-shared-ontology-read-network-opt-in` | adapter | Optional crate for callers that opt in to network-side reads (per-tenant `network_only`, cross-cell, cross-tenant, untrusted-tier). Wraps `Read`, `QueryFederated`, `QueryCrossTenant` gRPC against the Ontology µservice. |
| `oya-shared-ontology-read-credentialed-enrichment` | adapter | Bridge to Slice-2 sidecar key-holder for provider-BYOK LLM enrichment calls (e.g., embedding similarity queries that route through a tenant's provider-BYOK LLM provider). Per ADR-NNNN-library-first-credential-sidecar pattern. |
| `oya-shared-ontology-read-sdk` | sdk | High-level Rust SDK exposing `projection.open()`, `get_object`, `query`, `wait_until_at_least`. **This is the public façade callers depend on.** |

Each crate is independently version-pinned in the workspace. Callers
depend on `oya-shared-ontology-read-sdk`; the SDK transitively pulls
the projection-app + CRDT + CDC consumer + schema consumer + Cedar
gate + audit-emit by default. Vector/Geo/Time-series adapters are
optional dependencies added by callers that read those property types.
The `network-opt-in` and `credentialed-enrichment` crates are optional
dependencies added only by callers in untrusted-cell-tier deployments,
tenants under `network_only` mode, or read paths that perform external-
LLM enrichment.

### Rust trait + module surface

The canonical projection trait:

```rust
// crates/oya-shared-ontology-read-kernel/src/lib.rs
use async_trait::async_trait;
use crate::domain::{
    EntityId, ObjectTypeVersion, OntologyReadError, Query, QueryResult, WriteLsn,
};

#[async_trait]
pub trait OntologyProjectionClient: Send + Sync {
    /// Fetch a single object by ID from the local projection.
    async fn get_object<T: DeserializeOwned>(
        &self,
        object_type: &ObjectTypeVersion,
        entity_id: &EntityId,
    ) -> Result<Option<T>, OntologyReadError>;

    /// Run a query against the local projection.
    async fn query<T: DeserializeOwned>(
        &self,
        query: Query,
    ) -> Result<QueryResult<T>, OntologyReadError>;

    /// Wait until the local projection has consumed at least the
    /// specified write LSN. Used by callers requiring strict read-
    /// your-writes semantics across the library + µservice boundary.
    async fn wait_until_at_least(
        &self,
        write_lsn: WriteLsn,
        timeout: Duration,
    ) -> Result<(), OntologyReadError>;

    /// Report current projection age for observability + brown-out
    /// signalling.
    fn projection_age(&self, object_type: &ObjectTypeVersion) -> Duration;
}

/// Network-opt-in selector for reads. Parallel in shape to the
/// Policy-Engine selector per ADR-0246 amendment.
#[async_trait]
pub trait OntologyReadPathSelector: Send + Sync {
    async fn select_path(
        &self,
        query: &Query,
    ) -> Result<ReadPath, OntologyReadError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadPath {
    LibraryInProcess,
    NetworkViaMicroservice,
    NetworkFederatedCrossCell,
    NetworkCrossTenant,
}
```

The library-first composition root:

```rust
// crates/oya-shared-ontology-read-projection-app/src/lib.rs
use crate::crdt::CrdtProjection;
use crate::cdc::CdcConsumer;
use crate::schema::SchemaRevisionConsumer;
use crate::cedar_gate::ReadAdmissionGate;

pub struct LibraryFirstProjectionClient {
    projection: Arc<CrdtProjection>,
    cdc_consumer: Arc<CdcConsumer>,
    schema_consumer: Arc<SchemaRevisionConsumer>,
    cedar_gate: Arc<ReadAdmissionGate>,
    selector: Arc<dyn OntologyReadPathSelector>,
    network_client: Option<Arc<dyn OntologyProjectionClient>>,
    audit_emitter: Arc<dyn AuditEmitter>,
    tracer: Arc<dyn Tracer>,
}

#[async_trait]
impl OntologyProjectionClient for LibraryFirstProjectionClient {
    async fn get_object<T: DeserializeOwned>(
        &self,
        object_type: &ObjectTypeVersion,
        entity_id: &EntityId,
    ) -> Result<Option<T>, OntologyReadError> {
        let _span = self.tracer.start_span("ontology.read.get_object");

        // Step 1: Cedar admission.
        self.cedar_gate.permit_read(object_type, entity_id).await?;

        // Step 2: path selector.
        let query = Query::single(object_type.clone(), entity_id.clone());
        let path = self.selector.select_path(&query).await?;

        if path != ReadPath::LibraryInProcess {
            let network = self.network_client
                .as_ref()
                .ok_or(OntologyReadError::NetworkPathSelectedButCrateNotWired)?;
            return network.get_object(object_type, entity_id).await;
        }

        // Step 3: library-first default. Read from projection.
        let entity = self.projection
            .get::<T>(object_type, entity_id)
            .await?;

        // Step 4: sampled audit emission.
        self.audit_emitter
            .emit_sampled_read(object_type, entity_id)
            .await
            .ok(); // non-blocking

        Ok(entity)
    }

    async fn query<T: DeserializeOwned>(
        &self,
        query: Query,
    ) -> Result<QueryResult<T>, OntologyReadError> {
        let _span = self.tracer.start_span("ontology.read.query");

        // Path selection + Cedar gate as in get_object.
        self.cedar_gate.permit_query(&query).await?;
        let path = self.selector.select_path(&query).await?;

        if path != ReadPath::LibraryInProcess {
            let network = self.network_client
                .as_ref()
                .ok_or(OntologyReadError::NetworkPathSelectedButCrateNotWired)?;
            return network.query(query).await;
        }

        // Library-first: evaluate against projection's in-process index.
        let result = self.projection.evaluate_query::<T>(&query).await?;

        self.audit_emitter
            .emit_sampled_query(&query, result.len())
            .await
            .ok();

        Ok(result)
    }

    async fn wait_until_at_least(
        &self,
        write_lsn: WriteLsn,
        timeout: Duration,
    ) -> Result<(), OntologyReadError> {
        self.cdc_consumer.wait_until_at_least(write_lsn, timeout).await
    }

    fn projection_age(&self, object_type: &ObjectTypeVersion) -> Duration {
        self.projection.age_for(object_type)
    }
}
```

The default network-opt-in selector:

```rust
// crates/oya-shared-ontology-read-projection-app/src/selector.rs
pub struct DefaultOntologyReadPathSelector {
    tenant_attribute_cache: Arc<TenantAttributeCache>,
    cell_trust_provider: Arc<dyn CellTrustProvider>,
    cedar_evaluator: Arc<dyn PolicyEvaluator>,
}

#[async_trait]
impl OntologyReadPathSelector for DefaultOntologyReadPathSelector {
    async fn select_path(
        &self,
        query: &Query,
    ) -> Result<ReadPath, OntologyReadError> {
        // Cross-tenant scope: always network.
        if query.tenant_scope().is_cross_tenant() {
            return Ok(ReadPath::NetworkCrossTenant);
        }

        // Cross-cell scope: always network (federated).
        if query.cell_scope().is_cross_cell() {
            return Ok(ReadPath::NetworkFederatedCrossCell);
        }

        let tenant = self.tenant_attribute_cache
            .get(query.tenant_id())
            .await?;

        // Tenant explicitly opted into network-only.
        if tenant.ontology_read_mode == OntologyReadMode::NetworkOnly {
            return Ok(ReadPath::NetworkViaMicroservice);
        }

        // Freshness-floor fallback.
        if tenant.ontology_read_mode == OntologyReadMode::LibraryFirstWithFreshnessFloor {
            let age = query.estimated_projection_age();
            if age > tenant.freshness_floor {
                return Ok(ReadPath::NetworkViaMicroservice);
            }
        }

        // Untrusted-tier mediation.
        let cell_trust = self.cell_trust_provider.trust_tier_for_local_cell().await?;
        if cell_trust == CellTrustTier::Untrusted {
            return Ok(ReadPath::NetworkViaMicroservice);
        }

        // Default: library in-process.
        Ok(ReadPath::LibraryInProcess)
    }
}
```

### Opt-in µservice surface (`microservices/ontology/`)

The µservice retains its full bounded-context set unchanged in
*responsibility*. What this amendment changes is which surfaces are
*default-consumed by callers*. The runtime endpoints split:

| µservice endpoint | Purpose | Caller-facing path |
|---|---|---|
| `POST /v1/entities/{type_version}` | Write: create an Object instance. | Always-network (writes always go through µservice). |
| `PATCH /v1/entities/{type_version}/{id}` | Write: update an Object instance. | Always-network. |
| `DELETE /v1/entities/{type_version}/{id}` | Write: delete (TOMBSTONE) an Object instance. | Always-network. |
| `POST /v1/actions/{action}` | Invoke an Action (write semantics per ADR-0172). | Always-network. |
| `POST /v1/schema/revisions` | Author / publish a new Object Type schema revision. | Always-network; schema authoring path. |
| `POST /v1/schema/deprecation-handshake` | Drive ACTIVE → DEPRECATED → TOMBSTONED transitions per ADR-0257 §D-3. | Always-network. |
| `GET /v1/entities/{type_version}/{id}` | Single-entity read. | **Opt-in only.** Caller library calls when tenant `ontology_read_mode != library_first` OR cell trust tier is untrusted. |
| `POST /v1/query` | Filtered query. | **Opt-in only.** Same gating. |
| `POST /v1/query/federated` | Cross-cell federated query. | **Opt-in only.** Selector returns `NetworkFederatedCrossCell` for cross-cell scope. |
| `POST /v1/query/cross-tenant` | Cross-tenant query. | **Opt-in only.** Selector returns `NetworkCrossTenant` for cross-tenant scope. |
| `GET /v1/schema/bundle/{cell_id}` | Library polls / pulls the canonical schema-bundle. | Non-call-path; refresh-time. |
| `GET /v1/cdc/topic/{cell_id}/{tenant_id}` | Library subscribes to CDC stream (gRPC streaming or Kafka topic-discovery endpoint). | Non-call-path; subscription setup. |
| `POST /v1/bulk-export` | DSAR cascade / data warehouse export. | Always-network; bulk path. |

The default per-read endpoints (`GET /v1/entities/...`, `POST /v1/query`)
are **deliberately retained but marked opt-in only**. The CI lane
`oya-check-no-unnecessary-ontology-read-service-hop` enforces this.

### Cedar fragments (`ontology/fragments/baseline/`)

Four Cedar fragments govern the library + µservice boundary for reads:

1. `ontology-read-admission.cedar` — governs the in-process read
   admission decision (this is the baseline + per-Object-Type permit
   set that every caller's library evaluates against).
2. `ontology-network-side-read-opt-in.cedar` — governs the opt-in
   decision per §D-5 (tenant `network_only`, freshness-floor fallback,
   untrusted-tier mediation).
3. `ontology-cross-cell-read.cedar` — governs cross-cell federated
   queries.
4. `ontology-cross-tenant-read.cedar` — governs cross-tenant queries.

The four fragments are part of the baseline bundle that every library
instance loads at startup (via the Policy-Engine library per ADR-0246
amendment).

## Verification

### CI lanes

Seven advisory-until-bootstrap lanes promote to BLOCKER after the
items in §Status are complete:

1. **`oya-check-ontology-read-library-first-default`** (static
   analysis). Scans the workspace for any caller that issues an
   Ontology `Read` / `Query` µservice RPC on the per-call read path
   without a corresponding tenant `ontology_read_mode != library_first`,
   cross-cell, cross-tenant, or untrusted-tier declaration. Flags
   violations.
2. **`oya-check-ontology-read-network-opt-in-cedar-gated`** (static
   analysis). Confirms that every network-side read site is gated by a
   Cedar evaluation of `ontology-network-side-read-opt-in.cedar` (or
   the cross-cell / cross-tenant fragments). Flags ungated RPCs.
3. **`oya-check-no-unnecessary-ontology-read-service-hop`** (integration
   test). Runs the pilot reference workflow end-to-end and asserts
   that zero gRPC calls hit `microservices/ontology/` on the per-call
   default read path.
4. **`oya-check-ontology-read-library-only-failure-perimeter`** (chaos
   test). Brings down `microservices/ontology/` in a test cell; asserts
   that reads through the library continue to succeed for default
   callers; asserts that writes correctly fail-closed (writes always
   require the µservice).
5. **`oya-check-ontology-read-projection-coherence`** (unit + integration
   test). Confirms that the library's in-process projection converges
   to the µservice's authoritative state within the per-tenant
   freshness target. Asserts no entity-level divergence between
   projection and µservice canonical store after CDC catch-up.
6. **`oya-check-ontology-write-always-network-via-service`** (static
   analysis + integration test). Confirms that the library exposes no
   write API; every write call is routed through the µservice gRPC.
7. **`oya-check-ontology-read-credential-sidecar-coherence`** (integration
   test). Confirms that the library's credentialed-enrichment path (when
   used) routes the LLM credential through the Slice-2 sidecar key-
   holder (per ADR-NNNN-library-first-credential-sidecar) and never
   holds the LLM credential in the read library's main process beyond
   the immediate UDS call.

### Manual verification gates

1. The reference architecture diagram at
   `docs/architecture/ontology-substrate-read-path-runtime-topology.md`
   shows the library-first read path as the default solid edge, the
   network read hop as a dashed opt-in edge, and the write path as an
   always-network solid edge.
2. ADR-0257 §D-3 + §D-4 frontmatter is annotated with a forward
   pointer to this amendment.
3. The `tenants` migration ships the `ontology_read_mode` column per
   the follow-up ADR-0244 tenant DDL extension (tracked separately in
   §F-1).
4. The pilot reference workflow is documented in Appendix B as the
   canonical worked example.

## Migration

### F-1. ADR-0244 tenant DDL extension required (follow-up, not edited here)

This amendment introduces a new column on the `tenants` table:

```sql
CREATE TYPE ontology_read_mode_t AS ENUM (
    'library_first',
    'network_only',
    'library_first_with_freshness_floor'
);

ALTER TABLE tenants
    ADD COLUMN ontology_read_mode ontology_read_mode_t NOT NULL DEFAULT 'library_first';

ALTER TABLE tenants
    ADD COLUMN freshness_floor INTERVAL
        NOT NULL DEFAULT '5 seconds'::INTERVAL;
```

The column **is not added in this amendment**. The follow-up extends
ADR-0244 §D-3 (tenant DDL). Tracked as keystone-bundle synthesis §5.13
follow-up "library-first symmetry promotion-gate fix tenant DDL
extension."

### F-2. CDC topic provisioning

Ontology µservice authors per-tenant per-cell Kafka topics
`ontology.entity.{cell_id}.{tenant_id}` + the cell-wide schema-
revision topic `ontology.schema-revision.{cell_id}`. Tracked as a
follow-up infrastructure ADR.

### F-3. Migration path for existing callers

All existing callers default to library-first. ADR-0141's existing
direct-gRPC read paths are migrated to the SDK façade in a sweep PR.
The SDK selects library-first by default.

### F-4. Schema-revision deprecation handshake coherence

ADR-0257's deprecation handshake remains authoritative. Library
consumers receive DEPRECATED notifications via the schema-revision
topic and adjust pinned revisions per the ≥12-month grace.

### F-5. Coverage of ADR-0257 §Status items

ADR-0257's bundling status items remain in effect. This amendment
adds the requirement that the pilot read-path consumption is
**in-process projection**, not gRPC mediation. The integration test
under §Verification 3 above is the gating evidence.

## References

- **ADR-0141** — Workflow + Ontology read-path direct (2026-05-18;
  Superseded by ADR-0145). The original read-path-direct intent this
  amendment recovers under ADR-0145's frame.
- **ADR-0145** — Inter-microservice communication reform (2026-05-18).
  Invariant 3 "Ontology is a SUBSTRATE for cross-µservice read
  queries, not a GATEWAY" is the doctrine this amendment formalises
  in delivery terms.
- **ADR-0150** — Cedar policy engine. The library-first pattern this
  amendment extends.
- **ADR-0176** — Brown-out degradation signal. Library emits local
  projection-staleness signal; µservice emits CDC-publish-lag signal.
- **ADR-0211** — In-house tech stack preference.
- **ADR-0212** — Buildability doctrine.
- **ADR-0242** — `oyatie` is a tenant.
- **ADR-0243** — Cedar as universal gate.
- **ADR-0244** — Tenant as universal scoping primitive. The new
  `ontology_read_mode` enum extends the tenant DDL (per F-1).
- **ADR-0245** — Substrate-vs-product layering. Ontology remains a
  substrate (substrate-data per §D-3.A); the library shape is the
  consumption surface for reads.
- **ADR-0246** — Policy-Engine Substrate Promotion.
- **ADR-0353-amendment-library-first-network-opt-in-clarification** —
  The Policy-Engine library-first amendment; this Ontology amendment
  is its structural twin for the read-path axis. Together (with the
  ADR-0255 amendment) the three amendments close all three structurally
  identical universal-mediator risks.
- **ADR-0247** — Self-hosting / self-modification doctrine. Foundry
  workflows read Ontology projections in-process via the library on
  the default path.
- **ADR-0248** — Amazon-shape cellular architecture. The untrusted-tier
  mediation path in §D-4 references ADR-0248 §D-7 cell-trust-tier
  taxonomy.
- **ADR-0255** — Intelligence as two-layer AI substrate.
- **ADR-0355-amendment-library-first-network-opt-in-clarification** —
  The Intelligence library-first amendment.
- **ADR-0257** — Ontology Object Type Versioning & Deprecation
  Handshake. Base ADR amended by this document.
- **ADR-NNNN-library-first-credential-sidecar** (Slice-2, number
  pending assignment) — Sidecar key-holder primitive for audit-signing
  key + provider-credential isolation. Referenced by this amendment's
  §D-2 audit-emission and credentialed-enrichment rows.
- **Palantir Foundry Ontology** product documentation — Caller-side
  ontology projections; bulk projection API; CDC event stream;
  consumer applications (AIP Logic, AIP Threads, Code Workbook,
  Workshop) maintain in-application projections. Direct precedent.
- **AWS S3 Express One Zone client SDKs** (2024 GA) — Caller-side
  account-scoped metadata cache; reads against cached metadata satisfy
  in-process. Reference shape.
- **AWS DynamoDB DAX** — Caller-adjacent cache layer with CDC
  invalidation; per-read latency at single-digit microseconds.
- **Google Spanner client SDK with TrueTime-aware staleness reads** —
  Caller-side staleness reads against a local snapshot.
- **Stripe client SDK** — Caches account-scoped metadata in caller's
  process.
- **Google Zanzibar** (Pang et al., USENIX ATC 2019) — Distributed
  authorization with caller-side cached relations + central namespace
  service.
- **James Hamilton 2007 LISA** — Static stability principle.
- **AWS Builder's Library** — "Static stability using Availability
  Zones" (2020).
- **AWS Builder's Library** — "Avoiding Cascading Failures" (2019).
- **AWS Builder's Library** — "Avoiding overload in distributed
  systems."
- **Martin Fowler 2014** — "Microservices and the First Law of
  Distributed Object Design."
- **Google SRE Workbook Chapter 11** — Managing load.
- **CRDT survey (Shapiro et al., 2011)** — Conflict-free Replicated
  Data Types as the convergence primitive for distributed projections.
- **`feedback_workflow_objectgraph_adapter_layer`** — Memory retired
  by ADR-0145; this amendment restores its read-path-direct intent
  within ADR-0145's frame.
- **`feedback_quality_performance_scalability_bar`** — Hyperscaler-
  grade performance + horizontal scalability bar.
- **`feedback_no_silent_regression`** — Linus-style protection of
  public contracts.
- **`feedback_autonomous_implementation_artifacts`** — Long-term goal
  of autonomous masterplan implementation; library version pinning
  per-caller preserves this.
- **`feedback_glossary_ontology_not_object_graph`** — Terminology
  precedent.
- **F4-Architecture verdict** —
  `evidence/debate/keystone-bundle-2026-05-20-F4-architecture-r1.json`
  finding F4-LV-6 / F4-AP-1 / F-ANTI-3. Authority for this amendment.
- **Keystone-bundle 2026-05-20 synthesis** —
  `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.13
  promotion-gate fix library-first-symmetry-2-of-2.

## Appendix A — Hyperscaler-pattern attribution

| Reference | Library | Network coordination | Shape |
|---|---|---|---|
| **Palantir Foundry Ontology** | AIP Logic / AIP Threads / Code Workbook / Workshop hold in-application projections. | Ontology service is canonical write authority + schema registry + CDC publisher. | Library-first reads; central writes + CDC. Direct precedent. |
| **AWS S3 Express One Zone** (2024 GA) | Client SDK caches account-scoped metadata. | S3 control plane for canonical state. | Library-first cached reads; central writes. |
| **AWS DynamoDB DAX** | Caller-adjacent cache layer; CDC invalidation. | DynamoDB canonical store. | Library-first cached reads; central writes. |
| **Google Spanner client SDK** | Caller-side staleness reads against local snapshot. | Spanner central transaction coordinator for writes. | Library-first staleness reads; central writes. |
| **Stripe client SDK** | Caches account-scoped metadata in caller's process. | api.stripe.com for canonical state. | Library-first cached reads; central writes. |
| **Google Zanzibar** (USENIX ATC 2019) | Caller-side cached relation tuples. | Zanzibar central namespace service for writes + relation publishing. | Library-first reads; central writes + namespace authoring. |
| **OpenTelemetry collector** (per ADR-0145 Invariant 2) | In-process SDK. | Tempo for storage. | Library-first; central storage. |
| **`oya-shared-audit-chain-client`** (per ADR-0145 Invariant 1) | Library. | Audit-chain µservice for canonical Merkle storage. | Library-first emission; central storage. |
| **`oya-shared-intelligence-client-*`** (per ADR-0255 amendment) | Library family. | Intelligence µservice for coordination. | Library-first; the structural twin (Intelligence). |
| **`oya-shared-policy-engine-client-*`** (per ADR-0246 amendment) | Library family. | Policy-Engine µservice for fragment authoring + distribution. | Library-first; the structural twin (Policy-Engine). |

The convergence is unambiguous. Every reference at the hyperscaler bar
uses the library-first read pattern with central write authority +
CDC distribution. Ontology reads join the pattern.

## Appendix B — Worked example: Social post fan-out reads Person and Task entities via the library

This appendix walks through a cross-µservice read from
`microservices/social/` rendering a post that joins Person entities
(authored by `microservices/iam/`) and Task entities (authored by
`microservices/task/`) into a single rendered post.

### Setup

- **Caller principal:** `acme-corp.user.bob@acme-corp` (per ADR-0242
  tenant doctrine).
- **Action:** `Social::Action::RenderPostFeed`.
- **Resources:** 1 Post entity, 5 Person entities (post author +
  4 commenters), 3 Task entities (referenced in the post body).
- **Cell:** `cell-us-east-1` (Tier 3 data-plane cell;
  `cell.trust_tier == standard`).
- **Tenant attribute:** `acme-corp.ontology_read_mode = library_first`
  (default).
- **Cedar fragments active:** `ontology-read-admission.cedar`,
  `social-post-feed-read.cedar`.

### Step-by-step

1. **Caller constructs the page-render request.** Social's HTTP
   handler receives `GET /v1/feed/{user_id}` and decides to fetch the
   most recent Post + its Person + Task references.
2. **Library opens an OTel span.** `ontology.read.page-render`.
3. **Library evaluates Cedar gate.** In-process call to
   `oya-shared-policy-engine-client-sdk::evaluate(...)` (per ADR-0246
   amendment) with action `Ontology::Action::ReadObject` and resource
   `post.v4 + person.v3 + task.v7`. Cedar returns Permit.
4. **Library evaluates read-path selector.** Single-cell, single-
   tenant, `ontology_read_mode = library_first`. Returns
   `ReadPath::LibraryInProcess`.
5. **Library reads from projection.** `projection.get_object::<Post>(
   "post.v4", post_id)` returns the Post in ~80 µs from the in-process
   CRDT projection. Library then fan-out reads Person + Task entities
   via the projection.
6. **Library emits sampled audit row.** Sampling rate per Cedar
   fragment `audit.ontology_read.sampling_rate = 0.001`; the current
   read falls outside the sample (no row emitted on this request).
7. **Library closes the OTel span.** Span ends with status `OK`.
8. **Caller renders the page.** Social's handler returns the rendered
   post to its HTTP client.

### What did NOT happen

- No gRPC call from `microservices/social/` to
  `microservices/ontology/`.
- No `OntologyRead` audit seal emitted by Ontology's SPIFFE-ID.
- No artificial `ontology-mediator` span in the OTel trace.
- No SLO contribution from Ontology's availability to the page-render
  per-call success budget.

### What happened separately (asynchronously)

- The CDC consumer continued processing `ontology.entity.cell-us-east-1.acme-corp`
  Kafka topic events in the background; projection freshness remained
  within the per-tenant 5-second target.
- The schema-revision consumer continued watching
  `ontology.schema-revision.cell-us-east-1`; no new revisions during
  this request.

### Hypothetical opt-in variant: cross-cell read

If the caller had requested a federated view across cells (e.g., a
user whose tenant is sharded across cell-us-east-1 + cell-eu-west-1):

- Step 4 selector returns `ReadPath::NetworkFederatedCrossCell`.
- Steps 5-6 are replaced by a gRPC `POST /v1/query/federated` call to
  `ontology.cell-us-east-1.svc`; the µservice fans out sub-queries to
  peer cells and aggregates.
- Step 7 emits an audit row carrying `evaluation_path =
  network_federated_cross_cell` for cross-cell observability.

### Hypothetical opt-in variant: `network_only` tenant

If the tenant had `ontology_read_mode = network_only`:

- Step 4 returns `ReadPath::NetworkViaMicroservice` immediately.
- Steps 5-6 are replaced by gRPC `GET /v1/entities/...` calls to
  `ontology.cell-us-east-1.svc`.
- Step 7 audit emission carries `evaluation_path = network_via_microservice`.

### Hypothetical credentialed enrichment variant

If the page-render included an embedding similarity search that
required a provider-BYOK LLM call (e.g., "find posts semantically similar
to this one"):

- Library calls into the Slice-2 sidecar key-holder via UDS to perform
  the LLM call. The sidecar holds the tenant's LLM credential; the
  read library's main process never sees the credential beyond the UDS
  call.
- Per ADR-NNNN-library-first-credential-sidecar, RCE in the read
  library's main process does not expose the LLM credential.

### Why this matters

This worked example demonstrates that the canonical cross-µservice
read workflow in the platform (a Social page-render joining Person and
Task entities) does **not** require `microservices/ontology/` to be up.
The platform's read capability — the property that cross-µservice
reads proceed without an additional mediator hop — is preserved across
Ontology µservice outages, upgrades, schema migrations, and even
across regional Ontology partitions. Writes correctly require the
µservice (preserving canonical write authority), but reads do not.
That is the static-stability guarantee Hamilton 2007 prescribed, that
ADR-0145 codified for the platform's inter-µservice surface, that the
ADR-0255 amendment extended to AI-mediated functionality, that the
ADR-0246 amendment extended to authorization-mediated functionality,
and that this amendment now extends to read-mediated functionality.

ADR-0145's "no universal mediator" doctrine and Invariant 3's "Ontology
is a SUBSTRATE, not a GATEWAY" are intact and are now actively defended
on the third of three structurally identical risk axes.

---

*End of amendment.*
