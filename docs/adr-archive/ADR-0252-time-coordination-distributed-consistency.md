---
id: ADR-0252
status: Superseded
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - ops-sre-reliability
  - ops-compliance
  - axis-workflow-engine
  - axis-audit-chain
  - axis-observability
  - axis-tenancy
  - axis-identity
supersedes: []
amends:
  - ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md (clarifies HLC + saga timing primitive)
  - ADR-0145-inter-microservice-communication-reform.md (clarifies deadline propagation)
  - ADR-0128-hyperscaler-architecture-invariants.md (formalises INV-IDEMPOTENCY as caller-supplied key)
superseded_by: [ADR-709]
amended_by: [ADR-0350]
related:
  - ADR-0005-eventing-backbone-outbox-pattern.md
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md
  - ADR-0040-metric-gated-rollback.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0140-cedar-policy-enforcement.md (retired; referenced for history)
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-finops-sustainability-tagging.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0222-saga-compensation-portfolio-policy.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/workflow-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/observability.json
  - /specs/time-coordination-model.json
  - /specs/idempotency-key-format.json
  - /specs/hlc-uncertainty-budget.json
related_memory:
  - feedback_time_coordination_distributed_consistency
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_bominal_inheritance_precedence
  - feedback_automate_everything
  - feedback_clean_architecture_requirements
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 11-of-14
purpose: >
  Establish Hybrid Logical Clocks (HLC) as the default clock primitive
  across the platform; reserve TrueTime-style atomic-clock-backed
  clocks for Tier-4 financial-grade cells and IL5+ classified cells;
  mandate Workflow Engine sagas (per ADR-0222) for cross-microservice
  coordination instead of distributed locks; require caller-supplied
  idempotency keys (Stripe pattern) for retry safety; default to
  causal consistency with strict total order opt-in; smear leap
  seconds (Google approach); per-cell cron with jitter — no global
  scheduler. Code never reads a wall clock for ordering decisions;
  code asks the HLC primitive. Locks are an anti-pattern at
  distributed scale and are forbidden outside the narrow exceptions
  enumerated in D-5.
enforcement_status: advisory-until-time-kernel-lands
enforced_by:
  - oya gate validate hlc-integration-coverage
  - oya gate validate no-wall-clock-for-ordering
  - oya gate validate idempotency-key-coverage
  - oya gate validate no-distributed-lock
  - oya gate validate leap-smear-configured
  - oya gate validate per-cell-cron-jitter
  - oya gate validate timeout-deadline-propagation
---

# ADR-0252: Time, Coordination, and Distributed Consistency

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive) landing as a single multispectrum-reviewed PR.
This ADR is keystone #11 of 14.

Enforcement is `advisory-until-time-kernel-lands`. The doctrine is
accepted in text now; the CI lanes promoting to BLOCKER require:

1. `crates/oya-shared-time-kernel/` exists, exposes the HLC primitive,
   and is integrated into ≥ 1 µservice as a reference implementation.
2. `chronyd` + `leap_smear` mode deployed on every Kubernetes node in
   every cell (per node DaemonSet + per-cell verification probe).
3. `microservices/workflow-engine/` saga coordinator (per ADR-0222) is
   the only sanctioned cross-µservice coordination primitive; static
   analysis lane `oya-check-no-distributed-lock` reports zero
   Valkey-SETNX, Postgres advisory-lock, Zookeeper-ephemeral, etcd-
   lease, or in-house lock-server usage outside the §D-5 exceptions.
4. `oya gate validate idempotency-key-coverage` reports ≥ 95% coverage
   across all state-changing actions (bootstrap target; goal 100% by
   post-keystone +90 days).
5. Tier-4 cell hardware procurement (GPS + Cesium atomic clock pairs;
   Microsemi/Spectracom or equivalent) has at least one cell in
   commissioning state, even if not yet in production traffic.

Until those five preconditions land, validators emit findings without
failing CI. Post-bootstrap, the lanes promote to BLOCKER.

## Date

2026-05-20.

## Context

### The structural problem: time in distributed systems is hard

Distributed systems get time wrong constantly. The canonical
enumeration of the failure surface comes from the **Fallacies of
Distributed Computing** (Peter Deutsch + James Gosling, 1994; expanded
by Arnon Rotem-Gal-Oz, 2006). Fallacy #6 reads: *"There is one global
clock."* It is not the most cited fallacy (#1 "The network is
reliable" and #5 "Topology doesn't change" usually win that prize),
but it is arguably the most expensive when violated. David Patterson's
2013 Berkeley lecture series ("CS 162: Operating Systems") explicitly
called the clock-synchronisation assumption "the silent killer of
distributed correctness."

Catalogued production failures that originate from time / coordination
mistakes:

| Year | Incident | Root cause | Cost |
|---|---|---|---|
| 2012-06-30 | **Leap second outage at Reddit, Mozilla, LinkedIn, Yelp, FourSquare, Gawker, StumbleUpon, Tumblr** | Linux kernel + Java `Thread.sleep` looped on `time_t` regression when the kernel inserted the leap second. CPU pegged at 100%; livelock. | Multi-hour outage at every named site |
| 2012-06-30 | **Cloudflare leap second outage** | Same kernel bug; Cloudflare's HTTP termination layer livelocked. | Hours of edge degradation |
| 2015-06-30 | **Cassandra cluster split-brain at multiple production sites** | Wall-clock timestamp used for last-write-wins reconciliation; clock skew between nodes caused writes to be discarded. | Data loss + manual reconciliation |
| 2015-08-13 | **AWS Lambda invocation duplication during US-East-1 partition** | Caller retry logic without idempotency keys; downstream side-effect (S3 PutObject, SQS SendMessage) executed twice. | Customer data corruption + manual cleanup |
| 2017-02-28 | **AWS S3 US-East-1 outage** | Operator-initiated debugging command; secondary cause: clock skew between billing subsystem nodes amplified the recovery time when the S3 metadata service restarted. Patrick Helland's later commentary attributed part of the 4h recovery to "everything that depends on `now()` had to be carefully sequenced." | $150M+ industry-wide losses |
| 2017-10-04 | **Google Cloud Spanner regional latency spike** | TrueTime uncertainty bound exceeded SLO due to GPS antenna degradation at one datacentre; commit latencies extended from ~10ms to ~80ms p99. | Latency SLO breach for Spanner customers |
| 2019-08-23 | **AWS DynamoDB increased error rates US-East-1** | NTP synchronisation issue between availability zones; quorum-based timestamps disagreed; PartiQL queries returned stale reads. | Multi-region degradation |
| 2020-08-30 | **Cloudflare workers outage** | Race condition in distributed configuration update; node-local cache stored the new config under the old timestamp due to NTP skew. | Edge config drift |
| 2021-06-08 | **Fastly outage (took down NYT, Amazon, gov.uk, Reddit, Twitch, GitHub)** | Single-customer config push triggered software bug in Varnish; downstream incident exposed an undeclared cross-service dependency without proper deadline propagation. | $300M+ industry impact |
| 2022-04-21 | **Atlassian outage (Confluence, Jira, OpsGenie)** | Automated cleanup script ran without idempotency keys; deleted production tenants. | 800+ enterprise tenants lost for 14 days |
| 2023-02-20 | **GitHub repository availability issue** | Distributed lock held in Redis cluster; cluster failover left lock as zombie; new operations blocked. | Multi-hour git operation degradation |
| 2024-07-19 | **CrowdStrike kernel-driver outage** | Not directly time-related but exposed the cost of any unsafe global-coordinated change (8.5M Windows machines down). Lesson: per-cell + per-tenant rollout with HLC-tagged versions enables surgical rollback. | $5.4B insured losses estimated |

The pattern is clear: anything that depends on the clock without a
*causal-consistency primitive* and *idempotency keys* eventually
explodes.

### What hyperscalers converged on

The 2008-2025 industry convergence is unambiguous. There are exactly
two production-grade clock primitives at hyperscale, and one
coordination primitive:

**Clock primitives:**

1. **Hybrid Logical Clocks (HLC).** Default for: CockroachDB (every
   transaction since 2015 launch), MongoDB Atlas (causal-consistency
   mode, since 3.6), YugabyteDB (since 2018 launch), Vitess (since
   PlanetScale adoption 2020), TiDB (since 2018), Citus (multi-tenant
   workloads), FoundationDB (post-Apple acquisition 2018), various
   Kafka KIP enhancements. Demirbas + Kulkarni 2014 paper
   ("Logical Physical Clocks and Consistent Snapshot Isolation,"
   OPODIS 2014) is the canonical academic source. Implementation is
   trivial (~150 lines of correct Rust); operational tax is minimal
   (no special hardware); strongest property is *causal consistency
   without external observers*.

2. **TrueTime-style (GPS + Atomic Clock).** Used by: Google Spanner
   (2012-present), Google Cloud Spanner (GA 2017), Amazon Aurora
   DSQL (announced 2024, uses GPS+atomic-pair per "Amazon Time Sync
   Service" microsecond-accurate timekeeping with PTP), Microsoft
   Azure Cosmos DB (limited TT-equivalent for "Strong" consistency
   tier), Yandex YDB. Provides bounded uncertainty interval; enables
   *external consistency* (linearizability across regions). Hsieh et
   al. 2012 OSDI paper ("Spanner: Google's Globally-Distributed
   Database") is the canonical academic source. Operational tax is
   substantial: GPS antennas with sky view, Cesium or Rubidium atomic
   clocks per data centre, PTPv2 protocol distribution, ~$10k-$50k
   per datacentre in hardware. Justifiable only when external
   consistency is a hard product requirement.

**Coordination primitives:**

3. **Sagas + caller-supplied idempotency keys.** The
   compensation-based coordination pattern. Originated in García-
   Molina + Salem's 1987 SIGMOD paper ("Sagas") for long-running
   business transactions; adapted to microservices by Caitie McCaffrey
   (then-Twitter, later Microsoft, 2015 keynote on "Distributed Sagas").
   Workflow engines that codify this: AWS Step Functions (2016+),
   Temporal.io (2019+ — fork of Uber Cadence), Microsoft Durable
   Functions (2017+), Netflix Conductor (2016+ open source).
   Distributed locks are *explicitly named an anti-pattern* in the
   **Google SRE Workbook** (2018, Beyer et al., O'Reilly), Chapter 24
   "Reliable Distributed Systems": "Distributed locks are subtle,
   error-prone, and almost always replaceable with idempotent
   operations and saga compensation. Avoid them."

**Idempotency keys (the coordination primitive's coordination
primitive):**

4. **Caller-supplied idempotency keys.** The Stripe pattern,
   formalised in Brandur Leach's 2014 engineering blog post
   ("Implementing Stripe-like Idempotency Keys in Postgres," later
   expanded in his "Designing Robust APIs" 2017 series). Caller
   generates a unique key per logical operation (UUIDv7, ULID, or
   crypto-random base32); sends key with the request; server stores
   `(key, request_signature, response)` and returns the cached
   response on duplicate. The pattern is now standard in: Stripe,
   PayPal, Square, AWS SDK (all retryable APIs since 2018), Google
   Cloud API gateway (since 2020), Twilio (since 2018), every
   payment processor of note. The RFC draft `draft-ietf-httpapi-
   idempotency-key-header-09` codifies the `Idempotency-Key` HTTP
   header convention.

### What `oyatie` inherits and what's missing

The portfolio already has:

- **ADR-0035** (workflow engine state machine + DAG hybrid) — establishes
  the Workflow Engine as the saga coordinator.
- **ADR-0145** (inter-microservice communication reform) — bans 2PC,
  bans direct cross-µservice writes outside the saga coordinator,
  requires deadline propagation.
- **ADR-0222** (saga + compensating-transaction portfolio policy) —
  makes sagas the only sanctioned shape for cross-µservice writes;
  every step declares `(forward_action, compensation_action,
  idempotency_key_strategy)`.
- **ADR-0128 INV-IDEMPOTENCY** — declares idempotency keys as a
  hyperscaler invariant but does not specify the *format*, *TTL*, or
  *cross-cell replication semantics*.
- **ADR-0009** (cell architecture per tenant per region) — establishes
  per-cell isolation; implicitly assumes per-cell time discipline but
  doesn't specify the primitive.
- **ADR-0049** (cross-region replication + residency) — assumes some
  ordering primitive for cross-region replication but defers
  specification.

What is missing — and what this keystone provides:

1. The **canonical clock primitive** for the platform (HLC default,
   TrueTime opt-in for Tier-4 cells).
2. The **idempotency key format spec** + TTL policy + cross-cell
   replication semantics.
3. The **distributed locks doctrine** (forbidden, with narrow
   enumerated exceptions).
4. The **consistency model default** (causal; strict total order
   opt-in via saga state machine).
5. The **leap second handling policy** (Google smear).
6. The **per-cell cron + jitter doctrine** (no global scheduler).
7. The **clock skew tolerance bounds** (±500ms HLC uncertainty
   default; alert when exceeded).
8. The **time-based feature flag integration** with Cedar (per
   ADR-0243).
9. The **per-µservice integration contract** for HLC consumption.
10. The **replay safety doctrine** for workflow re-execution.
11. The **audit-chain ordering primitive** (HLC on every entry; cross-
    cell merge via gossip).
12. The **transaction isolation default** (Postgres REPEATABLE READ;
    SERIALIZABLE opt-in via Cedar gate).

### Why now (2026-05-20)

Three forcing functions:

- **ADR-0222 saga portfolio policy** (2026-05-18) demands per-step
  idempotency keys, but the platform-wide format spec is undefined.
  Each saga step today negotiates a format ad hoc. Drift is starting.
- **ADR-0240 sovereign-cloud-per-regional-pack** (2026-05-18) implies
  cross-pack causal ordering for audit-chain rollup but doesn't
  specify the primitive.
- **ADR-0241 DR + business-continuity portfolio policy** (2026-05-18)
  declares T1 RTO < 5min, which is only achievable with HLC-tagged
  replication state + idempotency-key-safe replay. Without this ADR,
  the T1 RTO declaration is aspirational.
- **PR-159A `oya git` rename** (2026-05-18) made HLC-tagged ledger
  entries first-class in the agent VCS primitive. Internal git
  operations now carry HLC timestamps; ADR-0252 ratifies that choice.
- **Autonomous masterplan execution** (feedback_autonomous_implementation_artifacts)
  requires workflows that survive arbitrary cell failures and resume
  from durable state. That requires idempotency keys + saga
  compensation + HLC-tagged replay state.

### What this is NOT

This ADR is NOT:

- A demand that every µservice run a GPS receiver. Most µservices use
  HLC; Tier-4 cells run TrueTime; the rest stays HLC.
- A demand that all operations be strictly ordered. Causal consistency
  is sufficient for ≥ 95% of platform operations.
- A retirement of Postgres transactional semantics. Postgres
  transactions remain the within-µservice consistency primitive;
  cross-µservice consistency is via saga compensation.
- A retirement of Kafka / event streams. Kafka topics carry HLC
  timestamps and operate as the inter-µservice event backbone per
  ADR-0005.
- A reinvention of NTP. Each cell still runs `chronyd` (or `ntpd`)
  for *physical-clock discipline*; HLC is the *logical layer* atop.
- A claim that we can avoid time entirely. Some operations (TLS
  certificate expiry, scheduled workflows, JWT exp claims, audit
  retention sunset, lease timeouts) genuinely require physical time;
  for those we use chronyd-disciplined wall clock + uncertainty
  budget, never raw `now()`.

The bright line: **logical/causal ordering is HLC; external
consistency at Tier-4 is TrueTime; wall-clock-for-scheduling is
chronyd-disciplined-with-uncertainty-budget; coordination is sagas +
idempotency keys; locks are forbidden except for the narrow §D-5
exceptions.**

## Decision

### D-1. Hybrid Logical Clocks (HLC) as the default clock primitive

The platform-wide canonical clock primitive is the **Hybrid Logical
Clock** as defined in Demirbas + Kulkarni 2014 ("Logical Physical
Clocks and Consistent Snapshot Isolation," OPODIS). Every µservice's
kernel layer accepts an HLC parameter; every audit-chain entry carries
an HLC timestamp; every saga step's persistence row carries an HLC
timestamp; every Kafka message envelope carries an HLC timestamp;
every cross-µservice gRPC call carries an HLC header.

**Definition.** An HLC timestamp is the pair `(physical_ms, logical_counter)`
where `physical_ms` is the local monotonic-clock-disciplined wall-clock
time in milliseconds since Unix epoch, and `logical_counter` is a
16-bit unsigned counter that increments when multiple events share the
same `physical_ms`. Total size: 64 bits physical + 16 bits logical =
80 bits per timestamp (10 bytes; serialised as 12-byte base64 or
16-byte hex for human-readable logs).

**Update rules:**

```
HLC.now() at node N, receiving message M (which may be None):
  let pt = max(monotonic_wall_clock_now_ms(), N.last_pt)
  let l = N.last_l

  if M is None:  // local event
    if pt > N.last_pt:
      pt' = pt
      l'  = 0
    else:
      pt' = N.last_pt
      l'  = N.last_l + 1
  else:  // receiving M.hlc
    let pt_max = max(N.last_pt, M.hlc.pt, monotonic_wall_clock_now_ms())
    if pt_max == N.last_pt && pt_max == M.hlc.pt:
      l' = max(N.last_l, M.hlc.l) + 1
    elif pt_max == N.last_pt:
      l' = N.last_l + 1
    elif pt_max == M.hlc.pt:
      l' = M.hlc.l + 1
    else:
      l' = 0
    pt' = pt_max

  N.last_pt = pt'
  N.last_l  = l'
  return (pt', l')
```

This is the standard HLC algorithm. Properties:

- **Monotonic per node.** `HLC.now()` never regresses on a single
  node.
- **Captures happens-before.** If event A causally precedes event B
  (B observed a message tagged by A, or B was on the same node as A
  later), then `hlc(A) < hlc(B)` under lexicographic ordering of the
  pair.
- **Bounded divergence from wall clock.** Under standard chronyd
  discipline (≤ 100ms NTP drift per ADR-0241), HLC's physical
  component stays within ±150ms of wall clock; logical component
  rarely exceeds 1-2 increments under normal load.
- **Compact.** 10 bytes per timestamp; serialises into any envelope
  without budget concern.
- **No external observer needed.** Self-stabilises across the network
  via message exchange.

**Implementation.** A single shared crate `crates/oya-shared-time-kernel/`
exposes the HLC primitive:

```rust
// crates/oya-shared-time-kernel/src/hlc.rs

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HlcTimestamp {
    pub physical_ms: u64,
    pub logical: u16,
}

#[derive(Debug)]
pub struct Hlc {
    state: parking_lot::Mutex<HlcState>,
    monotonic_clock: MonotonicClock,
    uncertainty_budget_ms: u32,  // cell-configurable; default 500ms
}

impl Hlc {
    pub fn now(&self) -> HlcTimestamp { /* local event */ }
    pub fn observe(&self, received: HlcTimestamp) -> HlcTimestamp { /* received message */ }
    pub fn current_uncertainty_ms(&self) -> u32 { /* skew metric */ }
}
```

The implementation may be **vendored from CockroachDB's `hlc` crate**
(BSD-3-Clause; well-tested at production scale) or implemented in-
house (~200 lines + tests). The in-house implementation is preferred
per ADR-0211 (in-house tech stack preference); vendoring is a
fallback if in-house implementation is delayed beyond keystone
landing.

**Source citation.** Demirbas, M.; Kulkarni, S. "Logical Physical
Clocks and Consistent Snapshot Isolation." Principles of Distributed
Systems (OPODIS), 2014. DOI 10.1007/978-3-319-14472-6_2. Also
"Logical Physical Clocks." Demirbas + Kulkarni technical report,
2014, arXiv:1407.5208.

### D-2. TrueTime tier for Tier-4 financial-grade + IL5+ cells

The HLC default is sufficient for ≥ 95% of platform operations. For
operations requiring **external consistency** (linearizable across
geographically distributed nodes without external observers), HLC is
insufficient; only an atomic-clock-backed uncertainty-bounded clock
suffices. Per ADR-0251 cell certification levels, these cells are
**Tier-4 financial-grade** and **IL5+ classified-network** cells.

**Qualifying use cases:**

- **Payments settlement** (Tier-4 financial-grade cell). FedNow,
  SEPA Instant, KR financial-trading settlement windows require
  external consistency. Mismatched ordering between participants
  produces regulatory-incident-class outcomes.
- **IL5+ classified-network cells** (DoD IL5, NATO Secret, KR-MILSPEC,
  EU-SECRET). Air-gapped or limited-connectivity environments where
  cross-cell event ordering must be provable without trust in any
  single observer.
- **KR-FSS financial trading** (Korean Financial Services
  Commission-regulated trading cells). Order matching engines require
  external consistency.
- **Audit-chain root-of-trust ceremony cells** (annual key rotation).
  External consistency on key-rotation events prevents replay across
  cells.

**Hardware specification:**

- **GPS receiver:** Microsemi (now Microchip) SyncSystem 4380A; or
  Spectracom (now Orolia, now Safran) SecureSync 2400; or Endrun
  Technologies Praecis Ce. Two-of-three redundancy per cell with
  separate antenna runs and lightning protection.
- **Atomic clock:** Microsemi/Microchip 5071A Cesium beam (primary
  reference; <1×10^-12 frequency drift); or Stanford Research
  Systems FS725 Rubidium (secondary; <5×10^-11 drift); pair per cell.
  Cesium primary required for Tier-4 financial-grade; Rubidium pair
  sufficient for IL5 if Cesium procurement timelines exceed Wave-D
  rollout target.
- **Distribution:** PTPv2 (IEEE 1588-2019) with boundary clocks per
  rack; <100ns synchronisation across cell.
- **Hold-over budget:** Cesium primary → 1µs/day drift if GPS lost;
  Rubidium pair → 100µs/day drift. Cells must survive ≥ 72h GPS
  outage without exceeding TT uncertainty bound of 7ms (Spanner-
  comparable).

**Cost justification:** ~$50k-$150k per cell in initial hardware +
~$15k/yr in calibration/replacement. Procurement budget is folded
into per-cell capital cost; per-call cost attribution charges Tier-4
tenants the amortised hardware cost via the FinOps cost-center model
(per ADR-0174).

**TrueTime API.**

The platform's TrueTime abstraction mirrors Google Spanner's published
API. The same `oya-shared-time-kernel` crate provides:

```rust
// crates/oya-shared-time-kernel/src/truetime.rs

#[derive(Clone, Copy, Debug)]
pub struct TTInterval {
    pub earliest_ms: u64,   // earliest possible wall-clock time
    pub latest_ms: u64,     // latest possible wall-clock time
}

pub trait TrueTime: Send + Sync {
    fn now(&self) -> TTInterval;
    fn after(&self, t_ms: u64) -> bool {
        self.now().earliest_ms > t_ms
    }
    fn before(&self, t_ms: u64) -> bool {
        self.now().latest_ms < t_ms
    }
}

pub struct TrueTimeProvider {
    pub gps_pair: GpsReceiverPair,
    pub atomic_pair: AtomicClockPair,
    pub uncertainty_bound_ms: u32,  // default 7ms; tightened to 1ms when both pairs healthy
}
```

µservices in Tier-4 cells obtain `Box<dyn TrueTime>` from the cell
substrate; µservices in non-Tier-4 cells obtain `Box<dyn HlcProvider>`
which is the HLC primitive. The trait interface allows tests to mock
both.

**Implementation reference.** Google has published the Spanner
TrueTime sketch in the OSDI 2012 paper (Hsieh et al.) and in
subsequent SIGMOD + VLDB papers. There is no open-source TrueTime
reference implementation; the platform implements TrueTime in-house
per ADR-0211, using the GPS + atomic-clock hardware and the published
algorithm. The full implementation is small (~500 lines Rust) but
requires hardware integration glue (GPS NMEA parsing; PTP slave
discipline; hold-over math).

**Source citation.** Corbett, J. C.; Dean, J.; Epstein, M.; et al.
"Spanner: Google's Globally-Distributed Database." 10th USENIX
Symposium on Operating Systems Design and Implementation (OSDI 2012).
ISBN 978-1-931971-96-6. Also published in ACM TOCS 31(3), 2013.

### D-3. Cross-cell event ordering: causal consistency default; strict total order opt-in

For cross-cell event ordering, the platform default is **causal
consistency**, achieved by carrying HLC timestamps with every cross-
cell message. Receivers reorder by HLC; observers see a consistent
causal view; concurrent events (no causal relation) may appear in
different orders to different observers — accepted under causal
consistency.

When **strict total order** is required (e.g., financial-trading
order matching; regulatory event streams that must converge across
all observers to identical ordering), the call opts in via the
Workflow Engine saga state machine. The saga coordinator runs **raft
consensus** in the workflow-engine quorum (3-of-5 per cell-pair; per
ADR-0035 inheritance) to linearize the strict-total-order operations.

**API surface.** µservices declare a consistency requirement per
saga at registration:

```rust
#[derive(Debug, Clone, Copy)]
pub enum ConsistencyTier {
    /// HLC causal ordering only. ~95% of operations.
    Causal,
    /// Saga coordinator linearizes via raft. Slower (raft round trip
    /// adds ~5-30ms p99) but globally consistent.
    StrictTotalOrder,
    /// TrueTime external consistency. Tier-4 cells only.
    ExternalConsistency,
}

impl SagaDefinition {
    pub fn with_consistency(self, tier: ConsistencyTier) -> Self { /* ... */ }
}
```

**Cost.** `Causal` has zero coordination cost beyond message-passing.
`StrictTotalOrder` adds one raft round trip per saga step: 5ms p99
[same-region adjacent AZ] – 30ms p99 [same-region far AZ pair]
[P5..P95 error bars by region-pair] (evidence: modeling note
docs/performance-budgets/truetime-hlc-uncertainty-budget.md §3;
basis: measured cross-AZ RTT 2–5ms, cross-region same-continent
~20ms). Cross-continent `StrictTotalOrder` (~250ms per saga step)
is explicitly FORBIDDEN for real-time workflows; use `ExternalConsistency`
at Tier-4 for cross-continent financial operations instead.
`ExternalConsistency` requires Tier-4 cell + TrueTime; commit-wait
latency is the TT uncertainty bound: +1ms p99 [GPS + atomic clocks
healthy] / +7ms p99 [GPS degraded, Rubidium hold-over] [P5..P95:
0.5ms–10ms] (evidence: docs/performance-budgets/truetime-hlc-uncertainty-budget.md
§2; basis: Corbett et al. OSDI 2012 §4.2 commit protocol).

**Default.** New sagas register at `ConsistencyTier::Causal`. Promotion
to `StrictTotalOrder` requires a Cedar fragment (per ADR-0243) that
declares the action requires it and explains why; multispectrum
review must approve. Promotion to `ExternalConsistency` further
requires the action's saga to be cell-pinned to a Tier-4 cell.

**Audit-chain consequence.** Audit-chain entries always carry HLC.
For cross-cell rollup, the audit-chain merge protocol (per ADR-0028
inheritance + ADR-0010 per-pack overlay) reorders by HLC; the cross-
cell Merkle root is computed over the HLC-sorted union.

### D-4. Idempotency keys (Stripe pattern) as the canonical retry-safety primitive

Every state-changing action carries a caller-supplied idempotency key.
The pattern matches Stripe's 2014 design (Brandur Leach blog) with
oyatie-specific extensions.

**Wire format.**

```
HTTP/gRPC header:    Idempotency-Key: idem_<32-char-base32-encoded>
Internal saga step:  field idempotency_key: IdempotencyKey
Internal audit row:  field idempotency_key: IdempotencyKey
```

**Key format spec.** `idem_<32-char-base32-encoded>` (40 chars total
including the `idem_` prefix). The 32-char body is base32-encoded
(RFC 4648 Crockford alphabet, no padding) of 160 random bits. Per
the canonical format spec at `/specs/idempotency-key-format.json`:

```json
{
  "format": "idem_<32-char-base32>",
  "regex": "^idem_[0-9A-HJKMNP-TV-Z]{32}$",
  "entropy_bits": 160,
  "encoding": "base32-crockford-no-padding",
  "case_sensitivity": "case-insensitive on read; lowercase canonical",
  "generation_strategy": [
    "uuid_v7_then_base32",
    "crypto_random_160_bits_then_base32",
    "ulid_then_base32"
  ],
  "opaque_to_server": true,
  "client_generated": true,
  "max_length_bytes": 40
}
```

**Why this format:**

- **`idem_` prefix.** Self-describing in logs; greppable; prevents
  ambiguity with other ID classes (tenant IDs, user IDs, request IDs).
- **32-char base32 body.** 160 bits of entropy = 2^160 ≈ 1.46×10^48
  possible keys. Birthday-collision probability is negligible (2^80
  keys before 1% collision; the platform will never generate 2^80
  operations).
- **Crockford base32.** Avoids visually-ambiguous characters (I/1,
  0/O); case-insensitive on read; lowercase canonical for logs.
- **Total length 40 chars.** Fits in HTTP header without budget
  concern; readable in logs without truncation; matches Stripe key
  lengths.

**TTL policy:**

| Domain | TTL | Storage |
|---|---|---|
| Default (every state-changing action) | 24 hours | Postgres `idempotency_keys` table per cell |
| Payments (Stripe-equivalent operations) | 7 days | Same table, longer TTL |
| Healthcare (HIPAA-classed actions touching PHI) | 7 days | Same table, longer TTL |
| Financial trading (KR-FSS-classed orders) | 30 days | Same table, longer TTL; regulator-readable |
| Audit-chain emission (already de-duped at chain layer) | N/A | Chain layer Merkle dedup |
| Read-only operations | Not required | N/A |

**Per-tenant scope.** Idempotency keys are scoped to `(tenant_id,
idempotency_key, request_signature_hash)`. Two different tenants may
generate the same `idem_*` key without conflict; same tenant
re-sending the same key with a different request body fails fast
(returns 422 with "idempotency-key-mismatch" code).

**Schema:**

```sql
-- per cell; Citus distributed by tenant_id
CREATE TABLE idempotency_keys (
    tenant_id           TEXT NOT NULL,
    idempotency_key     TEXT NOT NULL CHECK (idempotency_key ~ '^idem_[0-9a-z]{32}$'),
    request_signature_hash BYTEA NOT NULL,
    -- request signature = SHA-256 of (action, resource, canonicalised request body)
    cached_response_blob BYTEA,
    response_status     SMALLINT NOT NULL,
    hlc_inserted_at     BIGINT NOT NULL,    -- HLC physical_ms component
    hlc_logical         SMALLINT NOT NULL,  -- HLC logical component
    inserted_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at          TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tenant_id, idempotency_key)
);

CREATE INDEX idempotency_keys_expires_at_idx
    ON idempotency_keys (expires_at);

SELECT create_distributed_table('idempotency_keys', 'tenant_id');
```

**Request flow:**

```
1. Caller sends action with `Idempotency-Key: idem_<key>`.
2. µservice computes request_signature_hash = SHA256(action || resource || canonicalised_body).
3. µservice queries idempotency_keys for (tenant_id, key):
   - If row found AND request_signature_hash matches: return cached_response_blob (HTTP 200).
   - If row found AND request_signature_hash differs: return 422 "idempotency-key-mismatch".
   - If no row: proceed to action execution.
4. Before action commit, insert idempotency_keys row with HLC timestamp + null cached_response_blob.
5. Execute action.
6. Update idempotency_keys row with cached_response_blob + response_status.
7. Return response to caller.

Edge cases:
- Concurrent retries with same key: Postgres UPSERT with ON CONFLICT
  DO NOTHING; loser of the race waits (poll with backoff up to
  Idempotency-Key-Timeout header value, default 30s) for the winner
  to populate cached_response_blob, then returns it.
- Server crashes between step 4 and step 6: next retry sees row with
  null cached_response_blob; waits up to Idempotency-Key-Timeout;
  if still null after timeout, re-executes the action (the inserted
  row's existence ensures the second execution detects the prior
  partial state). The action's compensation (per ADR-0222) handles
  the partial-state case.
```

**Source citation.** Leach, B. "Implementing Stripe-like Idempotency
Keys in Postgres." Brandur Leach engineering blog, 2014.
https://brandur.org/idempotency-keys. Also: Stripe API Reference,
"Idempotent Requests" section, 2014-2025. IETF draft:
`draft-ietf-httpapi-idempotency-key-header-09`.

### D-5. Distributed locks — AVOID with narrow enumerated exceptions

Distributed locks are an **anti-pattern at distributed scale**. The
**Google SRE Workbook** Chapter 24 ("Reliable Distributed Systems")
states explicitly: "Distributed locks are subtle, error-prone, and
almost always replaceable with idempotent operations and saga
compensation. Avoid them."

The platform forbids distributed locks except for the narrow
exceptions enumerated below.

**Forbidden patterns** (the `oya-check-no-distributed-lock` static-
analysis lane refuses code matching these patterns):

- Valkey/Redis `SETNX` or `SET ... NX EX ...` for cross-µservice
  mutual exclusion lasting > 1 second.
- Zookeeper ephemeral nodes (Zookeeper is not in the platform tech
  stack; this is preemptive).
- etcd lease-based locks for application coordination (etcd is used
  by Kubernetes control plane; application-layer use is forbidden).
- Postgres advisory locks (`pg_advisory_lock`) for cross-µservice
  coordination.
- In-house lock-server µservice (none exists; preemptive).
- Application-layer fence tokens (Martin Kleppmann's "fencing token"
  pattern is acknowledged as the correct way to use a distributed
  lock if you must use one; the platform still prefers saga
  compensation).

**Replacement patterns** (in preference order):

| Coordination need | Replacement |
|---|---|
| Cross-µservice durable coordination | **Workflow Engine saga** (ADR-0222) — declared `(forward, compensation, idempotency_key)` per step |
| In-cell single-process critical section | **Postgres `SELECT FOR UPDATE`** within a single transaction; single-µservice scope |
| In-cell compare-and-swap | **Postgres optimistic locking** with version column (`UPDATE ... WHERE version = $expected`) |
| Sub-second in-cell coordination | **Valkey `SETNX` with EX ≤ 1s and fence token** — narrowly permitted (see exception list) |
| Cross-region leader election | **Raft consensus in Workflow Engine quorum** (already T1 per ADR-0241) |
| Singleton job (e.g., daily report) | **Per-cell cron with leader election among cell-local replicas via Postgres advisory lock within the cell** — narrowly permitted (see exception list) |

**Narrow enumerated exceptions** (the `oya-check-no-distributed-lock`
lane allows these via explicit annotation):

1. **Sub-second in-cell critical sections** (< 1s TTL, scoped to a
   single cell). Use case: hot-cache rebuild coordination among
   multiple replicas of the same µservice within the same cell.
   Allowed primitive: Valkey `SET key value NX EX 1` with fencing
   token. Code must be annotated `#[allow(oya_distributed_lock,
   reason = "sub_second_in_cell_critical_section_only")]` and the
   review checklist asserts the < 1s budget + fence token.

2. **Per-cell singleton-job leader election** (e.g., daily aggregation
   job that should run exactly once per cell). Allowed primitive:
   Postgres `pg_try_advisory_lock` on a well-known integer key,
   held by the leader for the job's duration. Code annotated
   `#[allow(oya_distributed_lock, reason = "per_cell_singleton_leader_election")]`.

3. **Workflow Engine internal coordination** (the engine itself uses
   raft + leader election internally per ADR-0035). This is the
   engine's substrate; not application-layer.

4. **Identity substrate session-token rotation** (Zitadel internal).
   The identity substrate has its own internal coordination per its
   upstream design. Not application-layer.

5. **Bootstrap-time singletons** (e.g., `0001_create_self_tenant.sql`
   migration runs exactly once). Permitted via Postgres migration
   tooling's built-in advisory lock; not application code.

Outside these five exceptions, the lane fails CI on any pattern
matching a distributed-lock primitive. This includes detecting:

- Use of `redis-rs::SETNX`, `redlock-rs`, `bb8-redis::set_nx`, etc.
- Use of `pg_advisory_lock` outside the migration runner and the
  per-cell singleton exception.
- Use of `etcd-client::Lease` for application coordination.
- Use of any third-party "distributed lock manager" crate.

**Source citation.** Beyer, B.; Murphy, N. R.; Rensin, D. K.; et al.
*The Site Reliability Workbook: Practical Ways to Implement SRE.*
O'Reilly Media, 2018. Chapter 24, "Distributed Periodic Scheduling
with Cron." Also: Kleppmann, M. "How to do distributed locking."
2016 blog post (acknowledged for the fencing token concept; the
platform takes Kleppmann's argument further to avoid locks
altogether).

### D-6. Per-cell cron and scheduled workflows with jitter

The platform forbids **global cron** schedulers. Each cell runs its
own scheduling primitive via the Workflow Engine cron API. Schedules
are tenant-scoped: a tenant's workflows are scheduled by their home-
cell's Workflow Engine; cross-cell scheduling is forbidden.

**API:**

```rust
// microservices/workflow-engine/src/cron.rs

pub struct CronSchedule {
    pub schedule_id: ScheduleId,
    pub tenant_id: TenantId,
    pub cell_id: CellId,                       // pinned to home cell
    pub cron_expression: String,                // standard cron syntax
    pub timezone: chrono_tz::Tz,                // per-tenant timezone
    pub jitter_pct: u8,                         // 0..50; default 10
    pub workflow_definition_id: WorkflowDefinitionId,
    pub catchup_policy: CatchupPolicy,          // SkipMissed | RunAll | RunMostRecent
    pub max_concurrent_runs: u8,                // default 1
}

#[derive(Debug, Clone, Copy)]
pub enum CatchupPolicy {
    /// If scheduler missed N runs (e.g., during outage), skip them all.
    SkipMissed,
    /// Run every missed schedule in order.
    RunAll,
    /// Run only the most recent missed schedule.
    RunMostRecent,
}
```

**Jitter spec.** The actual fire time is `scheduled_fire_time +
random(-jitter_pct% × interval, +jitter_pct% × interval)`. Examples:

- 10% jitter on hourly schedule: fires anywhere in the 54-66 minute
  window each hour.
- 50% jitter on daily schedule: fires anywhere in the 12-36 hour
  window (used for hot-spot prevention when many tenants share a
  daily aggregation pattern).
- 0% jitter: fires exactly at scheduled time (forbidden for any
  schedule with > 100 tenants on the same cron expression; CI lane
  `oya-check-cron-jitter-required` enforces).

**Tenant timezone + DST.** Per-tenant timezone stored in the global
tenant directory (per ADR-0244). Scheduler computes next fire time
in the tenant's timezone, accounting for DST transitions per IANA tz
database; converts to UTC + HLC; enqueues into the cell's workflow
runner.

**No global cron.** Each cell schedules its own tenants' workflows.
This is the **Amazon-shape cellular architecture** (per ADR-0248):
no shared scheduler across cells; each cell has its own
WorkflowEngine cron coordinator; failures in one cell's scheduler
do not affect other cells.

**Source citation.** Beyer et al. *The Site Reliability Workbook*
Chapter 24 ("Distributed Periodic Scheduling with Cron") describes
Google's experience moving from a global cron service to per-cell
schedulers. Also: Klein, B. "Cron at scale" (Google SRE blog,
2018).

### D-7. Leap second handling: Google Smear (24-hour linear smear)

The platform uses the **Google Smear** approach to leap seconds.
Origin: Google blog 2008 ("Time, technology and leaping seconds")
and the subsequent paper Geremia + Pascoe + Mills 2011 "A linear
smear approach to leap second handling" (Google internal technical
report, later released).

**Smear definition.** The 24-hour window centred on the leap second
(noon-to-noon UTC on the day of the leap event) is dilated such that
the inserted/deleted second is distributed linearly across all
86,400 seconds in the window. The result: no clock reversal, no
duplicated timestamps, no failed timeouts, no service restart.

**Implementation:**

- Every Kubernetes node runs **chronyd** with `leapsectz` option
  set to `slew` (chronyd's equivalent of Google smear; chronyd
  version ≥ 4.0 supports this natively per the chrony.conf manual).
- The smear is applied to the **wall-clock component** (physical_ms)
  of HLC; the logical component is unaffected.
- Container runtime inherits the host clock; no per-container leap
  smear configuration.
- Tier-4 cells with TrueTime: smear is applied at the TrueTime
  provider layer (the GPS + atomic clock pair's slew rate is
  steered to absorb the leap event over 24h).

**Why this matters:**

- **No clock reversal.** Applications that watch `now()` see
  monotonic progression even across the leap event.
- **No failed timeouts.** Sleep-until and timer-expiration math
  works without the kernel-livelock pattern that caused the
  2012-06-30 outages.
- **No JWT/TLS expiry confusion.** Tokens and certificates issued
  before the leap event remain valid through the smear window;
  expiry checks succeed monotonically.
- **Audit-chain ordering is preserved.** HLC ordering is unaffected
  by the smear (HLC's physical component is the smeared time;
  ordering is preserved).

**Source citation.** Kuhn, M.; Sahoo, P. "Time, technology and
leaping seconds." Google Official Blog, 2011-09-15. Also: AWS:
"Look Before You Leap – The Coming Leap Second and AWS." AWS
official announcement, 2015 + 2016. And: Facebook (Meta) Engineering
blog, "It's Time to Leave the Leap Second in the Past." 2022-07-25.
chronyd documentation: `man chrony.conf` (specifically `leapsectz`
+ `smoothtime` directives).

### D-8. Idempotency key format spec (canonical)

The canonical idempotency key format is exhaustively specified at
`/specs/idempotency-key-format.json` and the schema is reproduced
in §D-4 above. Restated here for emphasis:

- **Prefix:** `idem_`
- **Body:** 32-character Crockford base32 (RFC 4648 with Crockford
  alphabet; no padding)
- **Total length:** 40 characters (5 prefix + 32 body + 3 reserved
  for potential future versioning; current version 32-char body
  occupies 32 of the 35 available characters)
- **Entropy:** 160 bits in the body
- **Opacity:** Server treats the key as an opaque identifier. The
  server does NOT decode or interpret the body.
- **Generation:** Client-side. Approved generators:
  - **UUIDv7 → base32:** Recommended for clients in environments
    with reliable monotonic UUIDv7 generation (most modern SDKs).
    UUIDv7's first 48 bits are millisecond timestamp; remaining
    74 bits are random. Encoding the full 128-bit UUIDv7 as base32
    yields 26 characters; pad with 6 random characters to reach 32.
  - **Crypto-random 160 bits → base32:** Simple fallback. Use
    `crypto.getRandomValues` (Web Crypto), `secrets.token_bytes`
    (Python), `crypto.randomBytes` (Node), `rand::thread_rng`
    + 20-byte buffer (Rust).
  - **ULID → base32:** ULID is already base32; replace
    Crockford-base32 encoding directly (26 chars from ULID + 6
    random chars).
- **Uniqueness scope:** Per `(tenant_id, idempotency_key)`. Two
  tenants may generate the same key without conflict.
- **Mismatch handling:** Same `(tenant_id, idempotency_key)` with
  different `request_signature_hash` returns HTTP 422 with code
  `idempotency-key-mismatch`. Same key with same signature returns
  cached response (HTTP 200 OK with original response body).

**Client SDK helper:**

```rust
// crates/oya-shared-idempotency-key/src/lib.rs

pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 20];  // 160 bits
        rand::thread_rng().fill_bytes(&mut bytes);
        let body = base32::encode(base32::Alphabet::Crockford, &bytes);
        Self(format!("idem_{}", body.to_lowercase()))
    }

    pub fn from_uuid_v7(uuid: uuid::Uuid) -> Self {
        // Encode 128-bit UUID + 32 random bits as 32-char base32.
        let mut bytes = [0u8; 20];
        bytes[..16].copy_from_slice(uuid.as_bytes());
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut bytes[16..]);
        let body = base32::encode(base32::Alphabet::Crockford, &bytes);
        Self(format!("idem_{}", body.to_lowercase()))
    }

    pub fn from_str_validated(s: &str) -> Result<Self, IdempotencyKeyError> {
        let re = regex::Regex::new(r"^idem_[0-9a-z]{32}$").unwrap();
        if !re.is_match(s) {
            return Err(IdempotencyKeyError::InvalidFormat);
        }
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Equivalent TypeScript, Python, Go, Java SDKs ship in
`sdk/<lang>/oya-idempotency-key/`.

### D-9. Cross-cell idempotency

Most idempotency-key lookups stay within a cell (request lands in
home-cell; cell-local Postgres holds the key row). When a call
traverses cells (rare slow-path), the idempotency key is replicated
to the target cell as part of the request envelope.

**Replication semantics:**

- **Outbound call.** When µservice A in cell C1 calls µservice B in
  cell C2 (via the Workflow Engine saga coordinator per ADR-0145),
  the saga step carries the idempotency key. Cell C2's µservice B
  inserts the key into C2's `idempotency_keys` table before action
  execution.
- **Deduplication.** If the saga retries (because the response to
  C1 was lost), C2's µservice B sees the same key, returns the
  cached response. The retry is safe.
- **TTL coherence.** The key is replicated with the originating TTL
  (24h default / 7d payments / etc.); each cell's table independently
  ages out the key when expired.
- **Cross-region.** Per ADR-0049 + ADR-0240, cross-region (cross-
  pack) calls are limited; when they happen, idempotency key
  replication respects sovereign-cloud constraints (the key itself
  is not regulated; the request body's data classes are).
- **No global idempotency store.** Each cell owns its own key table;
  no global Postgres or Valkey for keys. This is the Amazon-shape
  cellular architecture (ADR-0248) applied to idempotency state.

### D-10. Audit-chain ordering: HLC timestamps + cross-cell gossip merge

Every audit-chain entry carries an HLC timestamp. Audit-chain
emission flow:

```
1. State-changing action begins.
2. Action acquires HLC.now() as its "begin" timestamp.
3. Action executes (Postgres transaction, gRPC call, etc.).
4. On commit, action emits audit row with HLC timestamp.
5. Per-cell audit-chain accumulator orders entries by HLC.
6. Per-period (default: per-minute) the cell's audit chain seals a
   batch with Merkle root + Ed25519 signature.
7. Per-period the cell publishes its sealed batch to peer cells via
   gossip protocol.
8. Cross-cell merge orders sealed batches by HLC; computes
   cross-cell Merkle root.
9. Tamper detection: per-cell Merkle chain + cross-cell Merkle root
   attestation. Mismatches alert SEV-2.
```

**Cross-cell merge:** Each cell publishes its `(period_start_hlc,
period_end_hlc, merkle_root, signature)` tuple to peer cells. Peers
order tuples by HLC and verify signatures. The cross-cell Merkle
root is the root of the Merkle tree over the HLC-sorted union of
per-cell sealed batches.

**Why HLC for audit.** Wall-clock-based audit ordering is the
canonical pre-condition for the kind of multi-site forensic
incidents that produced the 2017-02-28 AWS S3 recovery delay.
HLC-ordered audit:

- Survives clock skew between cells without ordering ambiguity.
- Provides a causally-consistent view even when wall clocks disagree.
- Enables replay of cross-cell event streams in a deterministic order.
- Survives leap-second smear without ordering perturbation.

**Source citation.** Per ADR-0010 (audit chain Merkle/Ed25519
inheritance) + this ADR's HLC primitive. Also: Saltzer + Schroeder
1975 "The Protection of Information in Computer Systems"
(foundational principles of audit-log integrity).

### D-11. Time bound — per-action timeout in Cedar context

Every action carries a deadline. The deadline is encoded as a Cedar
context attribute `action.deadline_ms` (absolute HLC timestamp at
which the action must complete or be considered failed). The deadline
is enforced at Cedar evaluation time (per ADR-0243); the policy
engine may forbid actions whose deadline is in the past, whose
deadline is unreasonably distant (e.g., > 1h for a request-response
action), or whose deadline violates a brown-out signal threshold.

**Deadline propagation.** Per ADR-0145, deadlines propagate across
µservice boundaries via the gRPC `grpc-timeout` header. The Workflow
Engine saga coordinator decrements the deadline at each hop by the
hop's measured latency + a safety buffer (default 50ms).

**Brown-out integration.** Per ADR-0176, when a µservice's brown-out
signal is `degraded`, the policy engine may shorten action deadlines
to shed load. When the signal is `outage`, the policy engine forbids
new actions on that µservice.

**Cedar example:**

```cedar
// fragment: baseline/action-deadline-coherence.cedar

forbid (
  principal,
  action,
  resource
)
when {
  // Deadline must be in the future per HLC.
  context.action_deadline_hlc_ms <= context.hlc_now_ms
};

forbid (
  principal,
  action,
  resource
)
when {
  // Deadline cannot be more than 1 hour in the future for synchronous actions.
  context.action_class == "synchronous"
  && context.action_deadline_hlc_ms - context.hlc_now_ms > 3600000
};

forbid (
  principal,
  action,
  resource
)
when {
  // Brown-out: forbid actions when target µservice is in `outage` state.
  context.target_microservice_brown_out_signal == "outage"
};
```

### D-12. Replay safety — idempotency + saga state machine + compensation

Workflow re-execution after failure is safe by construction because:

1. **Each saga step carries an idempotency key** (per D-4). On replay,
   the receiving µservice sees the key, returns the cached response,
   the saga advances without side-effect duplication.
2. **The saga state machine persists progress** (per ADR-0035) with
   HLC timestamps. On engine restart, the engine replays the saga log
   and resumes from the last persisted state.
3. **Compensation actions are declared up-front** (per ADR-0222).
   When a step fails non-recoverably, the engine executes the
   declared compensation in reverse order from the failed step
   backward.
4. **Per-action handlers are idempotent given the key.** Each action
   handler is required to be deterministic given the (idempotency
   key, request signature) pair. The handler may have non-deterministic
   side-effects (e.g., billing a card) but the dedup at the
   idempotency-key layer ensures the side-effect occurs exactly once
   per logical operation.

**Determinism requirement.** A handler is **deterministic given the
idempotency key** if and only if:

- Given the same `(key, request_signature)` pair, the handler
  produces the same response (modulo timing).
- The handler does not depend on `now()` for output (only for
  logging / metrics).
- The handler does not depend on per-process state that varies
  between processes.
- The handler does not depend on RNG seeded from the current time
  (RNG seeded from the idempotency key body is acceptable and
  deterministic).

This requirement is verified at unit-test time by the
`oya-check-idempotency-handler-determinism` lane (which runs the
handler twice with the same input and asserts response equality).

### D-13. Clock skew tolerance bounds

The HLC primitive maintains an **uncertainty budget** per cell.
Default: ±500ms [conservative; actual chronyd-disciplined drift is
≤100ms cross-region under normal conditions; the 500ms bound is ~5–10×
larger than typical worst-case, matching CockroachDB's default]
(evidence: modeling note docs/performance-budgets/truetime-hlc-uncertainty-budget.md
§1; basis: Demirbas-Kulkarni 2014 HLC proof; chronyd RFC 5905 NTP
discipline). Configurable per cell (Tier-4 cells with TrueTime have
±7ms [GPS degraded, Rubidium hold-over] / ±1ms [GPS + atomic healthy];
cells with degraded NTP have escalated bounds).

**Alert thresholds (canonical; also in §D-11):**

| Metric threshold | Alert level | Action |
|---|---|---|
| `uncertainty_ms > 300` p99 sustained 5 min | WARN (SEV-4) | Investigate chronyd; check NTP source reachability; 30+ min lead time before budget breach |
| `uncertainty_ms > 500` | SEV-3 (cell-local) | Investigate NTP / chronyd health |
| `uncertainty_ms > 1000` | SEV-2 (cell-wide) | Brown-out signal `degraded`; new sagas tagged `causal-only` |
| `uncertainty_ms > 5000` | SEV-1 (cross-cell) | Brown-out signal `outage`; saga coordinator pauses cross-cell ops; engineer page |

The 300ms warn threshold (below the 500ms budget) provides ≥20 hours of advance warning
for gradual drift scenarios (typical chronyd drift rate ≤10ms/hour).

**Computation.** Each node's uncertainty is the maximum of:

- chronyd `offset` reading (from `chronyc tracking`)
- Time since last NTP poll × maximum drift rate (default: 50 ppm for
  consumer NTP; 1 ppm for chronyd-disciplined; near-zero for
  TrueTime).

Cell-level uncertainty is the maximum uncertainty across all nodes
in the cell.

**Alert thresholds:**

| Threshold | Alert level | Action |
|---|---|---|
| `uncertainty_ms > 500` | SEV-3 (cell-local) | Investigate NTP / chronyd health |
| `uncertainty_ms > 1000` | SEV-2 (cell-wide) | Brown-out signal `degraded`; new sagas tagged `causal-only` (no strict-total-order ops) |
| `uncertainty_ms > 5000` | SEV-1 (cross-cell) | Brown-out signal `outage`; saga coordinator pauses cross-cell ops; engineer page |
| Tier-4 cell `uncertainty_ms > 10` | SEV-2 (Tier-4 only) | TrueTime degradation; commit-wait extended |
| Tier-4 cell `uncertainty_ms > 100` | SEV-1 (Tier-4 only) | TrueTime outage; cell flagged non-TrueTime until restored |

**Metrics:**

```
oya_hlc_uncertainty_ms{cell="<cell-id>",node="<node-id>"}
oya_hlc_physical_ms{cell="<cell-id>",node="<node-id>"}
oya_hlc_logical_counter{cell="<cell-id>",node="<node-id>"}
oya_truetime_uncertainty_ms{cell="<cell-id>"}  # Tier-4 cells only
oya_truetime_gps_lock_count{cell="<cell-id>"}  # Tier-4 cells only
oya_truetime_atomic_clock_health{cell="<cell-id>"}  # Tier-4 cells only
```

Cell-local Prometheus scrapes these; the per-cell observability
substrate (per ADR-0130 / ADR-0131) rolls up to a portfolio-wide
view.

### D-14. Time-based feature flags

Capabilities that activate at specific times use HLC timestamp +
grace window. Per ADR-0243 (Cedar as universal gate), the activation
check is a Cedar fragment, not imperative code:

```cedar
// fragment: pack/<pack-id>/time-based-activation.cedar

permit (
  principal,
  action == Feature::"<feature-name>",
  resource is User
)
when {
  context.hlc_now_ms > resource.effective_at_hlc_ms
  && context.hlc_now_ms < resource.sunset_at_hlc_ms
};
```

The `effective_at` and `sunset_at` are HLC timestamps stored on the
resource (e.g., a feature flag definition). Cedar evaluates the
permit at each call.

**Grace window.** When `effective_at` is approaching, the policy may
include a grace window (e.g., 5-minute warning) where the feature
emits `pre-activation` audit rows but still returns the pre-feature
behaviour. After `effective_at`, the feature returns the new
behaviour.

**No imperative time-flag SDK.** LaunchDarkly / Flagsmith / etc. are
not adopted (per ADR-0243 §D-13). All feature-flag evaluation,
including time-based activation, lives in Cedar.

### D-15. Per-µservice HLC integration contract

Every µservice's kernel layer accepts an HLC parameter. The shared
crate `oya-shared-time-kernel` provides:

```rust
// crates/oya-shared-time-kernel/src/lib.rs

pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> HlcTimestamp;
    fn observe(&self, received: HlcTimestamp) -> HlcTimestamp;
    fn current_uncertainty_ms(&self) -> u32;
}

pub struct ProductionClock { /* uses HLC with chronyd-disciplined wall clock */ }
pub struct TrueTimeClock { /* Tier-4 cells only */ }
pub struct MockClock { /* for tests; deterministic */ }

impl Clock for ProductionClock { /* ... */ }
impl Clock for TrueTimeClock { /* ... */ }
impl Clock for MockClock { /* ... */ }
```

**Integration pattern (per µservice):**

```rust
// microservices/<ms>/src/kernel.rs

pub struct <Ms>Kernel<C: Clock> {
    clock: Arc<C>,
    // ... other dependencies
}

impl<C: Clock> <Ms>Kernel<C> {
    pub fn new(clock: Arc<C>, ...) -> Self { /* ... */ }

    pub async fn perform_action(&self, request: ActionRequest) -> Result<ActionResponse, ActionError> {
        let ts = self.clock.now();
        // ... use `ts` for audit, idempotency, etc.
    }
}
```

**Test pattern:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use oya_shared_time_kernel::MockClock;

    #[tokio::test]
    async fn perform_action_emits_correct_hlc_timestamp() {
        let clock = Arc::new(MockClock::starting_at_ms(1_700_000_000_000));
        let kernel = <Ms>Kernel::new(clock.clone(), ...);
        // ... assert HLC behaviour
        clock.advance_ms(500);
        // ... continue asserting
    }
}
```

**Integration tests:** Use the real `ProductionClock`. CI environments
run chronyd-disciplined clocks; HLC behaviour is exercised end-to-
end.

**No `SystemTime::now()` in business logic.** The
`oya-check-no-wall-clock-for-ordering` lane refuses any business-
logic code that calls `std::time::SystemTime::now()`,
`chrono::Utc::now()`, `tokio::time::Instant::now()` for ordering
purposes (logging + metrics + observability are exempt; the lane
distinguishes by file path + annotation).

**Uniform interface.** Every µservice integrates the HLC the same
way. Cross-µservice consistency comes from this uniformity.

### D-16. Database transaction isolation

The platform's default Postgres isolation level is **REPEATABLE READ**
(snapshot isolation). For actions requiring **SERIALIZABLE**
isolation (true linearizability within a Postgres database), the
caller opts in via a Cedar gate.

**Default rationale.** REPEATABLE READ provides snapshot isolation,
preventing dirty reads + non-repeatable reads + phantom rows. It
performs well at hyperscale; the Postgres optimiser produces good
plans; concurrent writers see consistent snapshots. ~95% of
operations don't need SERIALIZABLE.

**SERIALIZABLE opt-in.** When SERIALIZABLE is required, the caller
sends the request with context attribute `request.serialisation =
SERIALIZABLE`. The Cedar gate evaluates:

```cedar
permit (
  principal,
  action == DatabaseAction::"BeginSerializable",
  resource is Database
)
when {
  context.action_class == "financial_settlement"
  || context.action_class == "regulatory_emission"
  || context.action_class == "kr_fss_order_match"
};
```

Without the Cedar permit, the request is downgraded to REPEATABLE
READ. With the permit, the action runs at SERIALIZABLE.

**Cross-µservice consistency.** Postgres SERIALIZABLE is a within-
µservice primitive (single database, single transaction). Cross-
µservice consistency comes from saga compensation (per ADR-0222),
not from cross-database SERIALIZABLE (which would require 2PC,
banned per ADR-0145).

**SSI (Serializable Snapshot Isolation) implementation.** Postgres'
SERIALIZABLE is SSI (per Cahill et al. 2008 "Serializable Isolation
for Snapshot Databases," SIGMOD). It is correct + composable + has
predictable performance overhead (~10-20% on conflict-heavy
workloads; ~negligible on read-heavy workloads).

**Source citation.** Cahill, M. J.; Röhm, U.; Fekete, A. D.
"Serializable Isolation for Snapshot Databases." SIGMOD 2008. Also:
PostgreSQL documentation, "Transaction Isolation" chapter (14+).

## Alternatives considered

### Alt-1. Wall clock only (the naïve baseline)

Use `SystemTime::now()` for all ordering, timestamps, and
coordination. NTP discipline for cross-node sync.

**Pros:**

- Zero implementation cost (already available in every standard
  library).
- Familiar to every engineer.
- Compact (8 bytes per timestamp).

**Cons:**

- **Every named outage in §Context.** Wall-clock ordering is the
  root cause of leap-second outages, NTP-skew reordering bugs,
  duplicate-side-effect retries, audit-chain ambiguity.
- **No happens-before capture.** Concurrent events on different
  nodes may receive identical timestamps; ordering is undefined.
- **No causal consistency.** Receivers cannot order events by
  causal precedence; only by wall clock.
- **Leap-second-sensitive.** Without smear or explicit handling,
  leap events corrupt ordering.
- **No uncertainty bound.** A node with a degraded NTP source
  silently produces wrong timestamps with no warning.
- **Cannot replay deterministically.** Replay-based recovery (per
  ADR-0035) requires deterministic ordering; wall clock alone
  doesn't provide it.

**Rejected.** Wall clock alone is the canonical anti-pattern; every
hyperscaler reference has moved past it.

### Alt-2. NTP-synchronized + best-effort (slight upgrade over Alt-1)

Use `SystemTime::now()` with aggressive NTP discipline (chronyd at
< 10ms skew); accept the best-effort ordering; mitigate failures
case-by-case as they occur.

**Pros:**

- Slightly better than Alt-1.
- Familiar.
- No HLC complexity.

**Cons:**

- **Same fundamental issues as Alt-1.** NTP discipline reduces
  skew but doesn't eliminate it. Concurrent events still produce
  identical timestamps.
- **No causal consistency.** Same as Alt-1.
- **Mitigation-by-incident.** Each new failure mode discovered
  produces a one-off fix. The portfolio drifts toward HLC anyway,
  but as ad-hoc patches across µservices.
- **Audit-chain ambiguity.** Same as Alt-1.

**Rejected.** The portfolio would converge on HLC eventually via
incident-driven patches; better to adopt HLC up front.

### Alt-3. TrueTime everywhere (the Spanner-everywhere baseline)

Deploy GPS + atomic clock hardware in every cell; use TrueTime for
all operations.

**Pros:**

- **External consistency** (linearizability across all observers)
  as a default.
- **Tight uncertainty bound** (~1-10ms with healthy hardware).
- **Simpler conceptually.** One primitive, one model.

**Cons:**

- **Cost prohibitive.** ~$50k-$150k per cell × 100+ cells over 5
  years = $5M-$15M in hardware. Plus ~$1.5M-$3M/yr in calibration
  + replacement. Most cells don't need external consistency.
- **Deployment complexity.** GPS antennas with sky view; PTP boundary
  clocks per rack; hold-over math; calibration cadence. Operational
  burden across every cell.
- **Sovereign-cloud constraint.** Some cells deploy in sovereign
  facilities where GPS reception is restricted or atomic-clock
  procurement is regulated. Forces fallback to HLC for those cells,
  which defeats the "everywhere" claim.
- **Marginal value over HLC for most ops.** ≥ 95% of platform
  operations are satisfied by causal consistency; external
  consistency adds no value for them.
- **Power draw.** Cesium atomic clocks draw 20-50W continuously
  per pair; GPS receivers another 5-10W. Per-cell power budget
  increases measurably; sustainability metrics (per ADR-0174) worsen.

**Rejected.** Cost is unjustifiable when ≥ 95% of operations are
satisfied by HLC. The Spanner-everywhere model serves a workload
(global ACID transactions) that the platform does not have at scale.

### Alt-4. Vector clocks (Lamport-style; no physical time)

Use vector clocks (a counter per node, exchanged on every message)
for causal ordering. No physical time component.

**Pros:**

- **Captures happens-before precisely.** Causal relations are
  exactly representable.
- **No clock-skew sensitivity.** No physical time means no NTP
  dependency for ordering.
- **Well-studied.** Lamport 1978 + Fidge 1988 + Mattern 1989.

**Cons:**

- **Size scales with cluster size.** Vector clocks are O(N) where N
  = number of nodes. At 1000+ nodes per cell, vector clocks are
  KB-scale per event; budget-prohibitive in audit-chain + Kafka
  envelopes.
- **No physical time means cannot align with wall clock.** Audit
  queries like "show me events from yesterday 14:00 UTC" don't work
  without physical timestamps.
- **Cannot use for scheduling.** Scheduled workflows need physical
  time; vector clocks don't help.
- **Cannot use for retention sunset, TLS expiry, JWT exp, etc.**
- **Garbage-collection of stale vector entries** is operationally
  complex.
- **HLC subsumes vector clocks** for practical purposes. HLC's
  physical+logical pair captures both causal ordering and physical
  alignment; vector clocks capture only the former.

**Rejected.** Vector clocks are theoretically pure but operationally
unwieldy; HLC strictly subsumes them for production use.

### Alt-5. HLC + TrueTime tier — **CHOSEN**

The selected alternative, fully specified in §Decision.

**Pros:**

- **Matches every named industry reference.** CockroachDB,
  MongoDB, YugabyteDB, Vitess, TiDB, Citus all use HLC.
  Spanner-class workloads use TrueTime.
- **Hyperscaler-grade.** Per feedback_quality_performance_scalability_bar.
- **Cost-efficient.** TrueTime hardware deployed only where justified.
- **Sustainability-conscious.** Atomic clock power draw confined to
  Tier-4 cells.
- **Sovereign-cloud-compatible.** HLC works in any cell; TrueTime
  is opt-in per cell.
- **Causal consistency by default.** Sufficient for ≥ 95% of
  operations.
- **External consistency opt-in.** Available for the operations that
  need it.
- **Saga + idempotency keys complete the picture.** Coordination
  primitive (sagas) + retry-safety primitive (keys) + clock primitive
  (HLC/TT) compose cleanly.
- **Locks are forbidden by construction.** The doctrine plus the
  static-analysis lane prevent the lock anti-pattern from re-
  emerging.
- **Replay safety provable.** Workflow re-execution after failure
  is safe given the idempotency + saga + HLC composition.
- **Closes the leap-second incident class.** Google Smear eliminates
  the family of failures from 2012-06-30.
- **In-house implementation feasible.** HLC ~200 lines; TrueTime
  ~500 lines + hardware glue. Both within the buildability bar
  (ADR-0212).

**Cons:**

- **Bounded one-time integration cost.** Every µservice's kernel
  layer integrates the HLC primitive. ~80-100 µservices in the
  portfolio at maturity; integration is mechanical.
- **Tier-4 hardware procurement lead time.** GPS + atomic clock
  procurement is 6-12 months for the first cell. Mitigation:
  Wave-D rollout schedule begins procurement at keystone landing;
  Tier-4 cells come online over 2026-Q4 — 2027-Q2.
- **Test discipline required.** Tests must use `MockClock` for
  determinism; integration tests use `ProductionClock`. The
  determinism requirement (D-12) is enforced by a CI lane.

**Accepted** as the foundational keystone for time, coordination,
and distributed consistency.

## Consequences

### Positive

1. **Distributed coordination primitive unified.** One HLC, one
   TrueTime (where applicable), one saga coordinator, one
   idempotency-key format. Drift is structurally prevented.
2. **Causal consistency by default.** ≥ 95% of operations work
   without external observers; ordering is provable.
3. **External consistency available.** Tier-4 cells get Spanner-
   class external consistency for the operations that need it.
4. **Idempotency keys eliminate retry-side-effect bugs.** Stripe-
   pattern dedup prevents the 2015-08-13 AWS Lambda duplication
   class.
5. **Locks forbidden by construction.** Static analysis prevents
   the lock anti-pattern; saga compensation replaces it.
6. **Leap seconds handled.** Google Smear eliminates the
   2012-06-30 outage class.
7. **Per-cell cron + jitter prevents hot-spotting.** No global
   scheduler; cell failures don't cascade.
8. **Replay safety provable.** Workflows resume from durable HLC-
   tagged state across arbitrary cell failures.
9. **Audit-chain ordering survives clock skew.** HLC + cross-cell
   gossip merge produces deterministic ordering for forensics +
   regulators.
10. **Hyperscaler-shape achieved.** Matches Spanner, CockroachDB,
    MongoDB, YugabyteDB, AWS Step Functions, Temporal, Stripe.
11. **Autonomous masterplan execution enabled.** Workflows survive
    cell failures via the idempotency + saga + HLC composition.
12. **Time-based feature flags live in Cedar.** Per ADR-0243,
    activation is policy, not code.
13. **Time bounds enforced at Cedar evaluation.** Deadlines are
    policy attributes; expired deadlines forbid action.
14. **Cell-tier sustainability tracking.** TrueTime power draw is
    confined to Tier-4 cells; non-Tier-4 cells don't pay the carbon
    cost.

### Negative

1. **HLC integration cost.** Every µservice's kernel layer
   integrates HLC. ~80-100 µservices; integration is mechanical but
   bounded.
2. **Test-time discipline required.** `MockClock` for tests;
   `ProductionClock` for integration. The `oya-check-no-wall-clock-
   for-ordering` lane enforces.
3. **Tier-4 hardware procurement.** GPS + atomic clock pairs lead
   time ~6-12 months. Wave-D rollout schedule absorbs.
4. **Sub-millisecond uncertainty for non-Tier-4 cells unattainable.**
   HLC's physical component is bounded by chronyd discipline (~10-
   100ms typical). For operations that need < 1ms uncertainty, the
   cell must be Tier-4.
5. **Saga coordinator becomes hot-path dependency.** Per ADR-0222
   the coordinator is already declared T1 in ADR-0241. Its
   availability constrains cross-µservice coordination.

### Operational

1. **New CI lanes:**
   - `oya-check-hlc-integration-coverage` (advisory until kernel
     lands; BLOCKER post)
   - `oya-check-no-wall-clock-for-ordering` (static analysis lane)
   - `oya-check-idempotency-key-coverage` (advisory; goal 100% by
     post-keystone +90 days)
   - `oya-check-no-distributed-lock` (static analysis lane)
   - `oya-check-leap-smear-configured` (chronyd config audit lane)
   - `oya-check-per-cell-cron-jitter` (workflow registration audit
     lane)
   - `oya-check-timeout-deadline-propagation` (gRPC header audit
     lane)
   - `oya-check-idempotency-handler-determinism` (handler unit-test
     lane)
2. **New µservice / crate surfaces:**
   - `crates/oya-shared-time-kernel/` — HLC + TrueTime primitives.
   - `crates/oya-shared-idempotency-key/` — key generator + validator.
   - `microservices/workflow-engine/src/cron.rs` — per-cell cron
     primitive (extends existing engine).
3. **Observability:**
   - Per-cell HLC uncertainty metric.
   - Per-cell TrueTime metrics (Tier-4 cells only).
   - Per-µservice idempotency-key cache hit rate.
   - Per-µservice deadline-exceeded counter.
4. **Tooling:**
   - chronyd config templated per node (Helm chart).
   - GPS antenna + atomic clock procurement runbook (ops-compliance
     owns).
   - HLC integration starter template (per µservice scaffolding).
5. **Runbooks:**
   - `docs/runbooks/hlc-uncertainty-investigation.md` (chronyd
     diagnostic procedure)
   - `docs/runbooks/leap-second-event-readiness.md` (pre-leap-event
     checklist)
   - `docs/runbooks/truetime-cell-commissioning.md` (Tier-4 cell
     hardware setup)
   - `docs/runbooks/saga-stuck-recovery.md` (workflow engine
     replay procedure)
   - `docs/runbooks/distributed-lock-discovery-response.md`
     (procedure when static analysis discovers a new lock pattern)

### Sustainability

- **Atomic clock power draw.** Cesium + Rubidium pairs draw ~30-70W
  continuously per Tier-4 cell. Limited to Tier-4 cells (≤ 5% of
  fleet). Sustainability tag (ADR-0174) records the carbon impact;
  tenant FinOps attribution charges Tier-4 tenants.
- **GPS receivers and antennas.** ~10W continuously per Tier-4 cell.
  Carbon impact ~negligible at fleet scale.
- **PTP boundary clocks.** ~5W per rack in Tier-4 cells. Adds < 1%
  to cell power.
- **Non-Tier-4 cells unaffected.** ≥ 95% of the fleet pays no
  additional power for HLC (it's a software-only primitive).
- **chronyd discipline.** No additional power; chronyd runs alongside
  systemd-timesyncd or as a replacement, minimal CPU overhead.

### Compliance

- **GDPR Article 22 + EU AI Act Article 14.** HLC-ordered audit-
  chain provides individually-auditable decision trail.
- **HIPAA §164.312(b) audit controls.** HLC timestamps on PHI-
  touching audit rows.
- **SOC 2 CC7.2 (system monitoring).** HLC uncertainty metrics +
  brown-out signal integration.
- **KR-FSS financial trading.** Tier-4 cells with TrueTime satisfy
  ordering requirements.
- **CSAP (Korean Cloud Security Assurance Program) v3.1.** HLC
  audit ordering + idempotency-key dedup satisfy
  data-integrity-and-non-repudiation criteria.
- **NIST SP 800-160 timekeeping requirements.** Defines the
  timekeeping discipline for high-assurance systems; chronyd +
  HLC + TrueTime composition satisfies.
- **NIST SP 800-92 (audit log management).** HLC-ordered audit
  satisfies ordering + integrity requirements.
- **ISO 22301 business continuity.** Replay safety (D-12) supports
  T1/T2 RTO targets.
- **FRCP 37(e) preservation.** Idempotency-key retention durations
  align with legal-hold policy.

## Implementation surface

| Artifact | Status |
|---|---|
| `/specs/time-coordination-model.json` | NEW — canonical model spec |
| `/specs/idempotency-key-format.json` | NEW — key format spec |
| `/specs/hlc-uncertainty-budget.json` | NEW — per-cell uncertainty bound config |
| `crates/oya-shared-time-kernel/` | NEW — HLC + TrueTime primitives |
| `crates/oya-shared-idempotency-key/` | NEW — key generator + validator |
| `crates/oya-shared-time-kernel/src/hlc.rs` | NEW — HLC algorithm |
| `crates/oya-shared-time-kernel/src/truetime.rs` | NEW — TrueTime API |
| `crates/oya-shared-time-kernel/src/mock.rs` | NEW — MockClock for tests |
| `crates/oya-shared-time-kernel/src/uncertainty.rs` | NEW — uncertainty bound computation |
| `microservices/workflow-engine/src/cron.rs` | NEW — per-cell cron primitive |
| `microservices/workflow-engine/src/idempotency.rs` | NEW — saga-level idempotency wrapper |
| `microservices/policy-engine/fragments/baseline/action-deadline-coherence.cedar` | NEW — deadline enforcement fragment |
| `microservices/policy-engine/fragments/baseline/serialisable-isolation-gate.cedar` | NEW — SERIALIZABLE opt-in fragment |
| `microservices/policy-engine/fragments/baseline/time-based-activation.cedar` | NEW — time-based feature flag fragment template |
| `microservices/audit-chain/src/hlc_ordering.rs` | NEW — HLC-ordered audit accumulator |
| `microservices/audit-chain/src/cross_cell_gossip.rs` | NEW — gossip-based cross-cell merge |
| `infra/helm/chronyd-daemonset/` | NEW — chronyd Helm chart with leap-smear config |
| `infra/helm/truetime-provider/` | NEW — Tier-4 cell TrueTime provider Helm chart |
| `tools/oya-check-hlc-integration-coverage/` | NEW |
| `tools/oya-check-no-wall-clock-for-ordering/` | NEW |
| `tools/oya-check-idempotency-key-coverage/` | NEW |
| `tools/oya-check-no-distributed-lock/` | NEW |
| `tools/oya-check-leap-smear-configured/` | NEW |
| `tools/oya-check-per-cell-cron-jitter/` | NEW |
| `tools/oya-check-timeout-deadline-propagation/` | NEW |
| `tools/oya-check-idempotency-handler-determinism/` | NEW |
| `sdk/typescript/oya-idempotency-key/` | NEW — TS client SDK |
| `sdk/python/oya-idempotency-key/` | NEW — Python client SDK |
| `sdk/go/oya-idempotency-key/` | NEW — Go client SDK |
| `sdk/java/oya-idempotency-key/` | NEW — Java client SDK |
| `docs/standards/time-coordination-canonical.md` | NEW — full standards doc with examples |
| `docs/standards/idempotency-key-canonical.md` | NEW — key authoring standards |
| `docs/runbooks/hlc-uncertainty-investigation.md` | NEW |
| `docs/runbooks/leap-second-event-readiness.md` | NEW |
| `docs/runbooks/truetime-cell-commissioning.md` | NEW |
| `docs/runbooks/saga-stuck-recovery.md` | NEW |
| `docs/runbooks/distributed-lock-discovery-response.md` | NEW |
| Tier-4 hardware procurement: GPS + Cesium/Rubidium pairs for first Tier-4 cell | NEW (procurement) |
| Migration sweep: replace `SystemTime::now()` ordering in business logic across portfolio | SWEEP — bounded |
| Migration sweep: replace ad-hoc idempotency formats with `idem_<32-base32>` | SWEEP — bounded |
| Migration sweep: remove distributed locks from any µservice that uses them | SWEEP — expected to find few |

## Verification

- [ ] `crates/oya-shared-time-kernel/` builds; HLC unit tests pass (Demirbas + Kulkarni 2014 reference test vectors).
- [ ] `crates/oya-shared-time-kernel/src/truetime.rs` builds with feature flag `truetime`; mock TrueTime tests pass.
- [ ] `crates/oya-shared-idempotency-key/` builds; key format tests pass:
  - `IdempotencyKey::generate()` produces key matching `^idem_[0-9a-z]{32}$`
  - `IdempotencyKey::from_uuid_v7(...)` produces key matching same regex
  - `IdempotencyKey::from_str_validated("idem_foo")` returns InvalidFormat error
  - Generated keys are unique across 1M samples
- [ ] `oya gate validate hlc-integration-coverage` reports ≥ 1 µservice integrated as reference (bootstrap target; goal 100% by post-keystone +90 days).
- [ ] `oya gate validate no-wall-clock-for-ordering` reports zero violations in pilot µservice.
- [ ] `oya gate validate idempotency-key-coverage` reports ≥ 95% coverage of state-changing actions in pilot µservice.
- [ ] `oya gate validate no-distributed-lock` reports zero violations outside the §D-5 enumerated exceptions.
- [ ] `oya gate validate leap-smear-configured` reports all K8s nodes in pilot cell have chronyd with `leapsectz slew`.
- [ ] `oya gate validate per-cell-cron-jitter` reports all registered cron schedules in pilot cell have `jitter_pct > 0` OR are exempt (single-tenant schedules).
- [ ] `oya gate validate timeout-deadline-propagation` reports all gRPC calls in pilot µservice propagate `grpc-timeout` header.
- [ ] HLC uncertainty metric `oya_hlc_uncertainty_ms` < 500ms p99 across pilot cell over 1 hour.
- [ ] Tier-4 cell commissioning: at least one cell in commissioning state with GPS + Cesium/Rubidium pair installed; TrueTime uncertainty < 10ms p99 measured.
- [ ] Audit chain emits HLC-ordered entries; cross-cell gossip merge produces deterministic ordering across 2+ cells.
- [ ] Saga replay test: kill workflow engine mid-saga; on restart, saga resumes from last persisted HLC-tagged state; no idempotency-key duplication; no side-effect re-execution.
- [ ] Leap-second readiness drill: simulate leap insertion in test environment; chronyd smears; HLC ordering preserved; no service restart; no failed timeouts.
- [ ] Cedar fragment `action-deadline-coherence.cedar` evaluates correctly: expired deadlines forbidden; > 1h synchronous deadlines forbidden; brown-out outage forbidden.
- [ ] Postgres `SELECT current_setting('default_transaction_isolation')` returns `repeatable read` on platform Postgres clusters; SERIALIZABLE opt-in path via Cedar permit verified.
- [ ] `chronyd` config audit on every K8s node in every cell: `leapsectz slew` set; offset < 100ms; sources include ≥ 2 stratum-2 NTP servers.
- [ ] Multispectrum review v2.4.0 verdict on this ADR: F1 (correctness), F2 (hyperscaler-fitness), F5 (security), F6 (performance), F7 (supply chain), A1-A7 (own-policy adherence) all green.

## References

### Academic + foundational sources

- **Demirbas, M.; Kulkarni, S. "Logical Physical Clocks and Consistent Snapshot Isolation." OPODIS 2014.** DOI 10.1007/978-3-319-14472-6_2. The canonical HLC paper.
- **Demirbas, M.; Kulkarni, S. "Logical Physical Clocks." arXiv:1407.5208, 2014.** Extended technical report.
- **Corbett, J. C.; Dean, J.; Epstein, M.; Fikes, A.; Frost, C.; Furman, J. J.; Ghemawat, S.; Gubarev, A.; Heiser, C.; Hochschild, P.; Hsieh, W.; Kanthak, S.; Kogan, E.; Li, H.; Lloyd, A.; Melnik, S.; Mwaura, D.; Nagle, D.; Quinlan, S.; Rao, R.; Rolig, L.; Saito, Y.; Szymaniak, M.; Taylor, C.; Wang, R.; Woodford, D. "Spanner: Google's Globally-Distributed Database." OSDI 2012.** The canonical TrueTime paper. Also ACM TOCS 31(3), 2013.
- **Lamport, L. "Time, Clocks, and the Ordering of Events in a Distributed System." Communications of the ACM 21(7), 1978.** The canonical logical-clocks paper. HLC's logical component descends from Lamport clocks.
- **Fidge, C. J. "Timestamps in message-passing systems that preserve the partial ordering." Proceedings of the 11th Australian Computer Science Conference, 1988.** Vector clocks foundation.
- **Mattern, F. "Virtual time and global states of distributed systems." Parallel and Distributed Algorithms, 1989.** Vector clocks formalisation.
- **García-Molina, H.; Salem, K. "Sagas." SIGMOD 1987.** The canonical saga paper.
- **Pat Helland. "Life beyond Distributed Transactions: an Apostate's Opinion." CIDR 2007.** Foundational rejection of distributed transactions in favour of idempotent operations.
- **Cahill, M. J.; Röhm, U.; Fekete, A. D. "Serializable Isolation for Snapshot Databases." SIGMOD 2008.** SSI foundation; what Postgres SERIALIZABLE implements.
- **Brewer, E. "Towards Robust Distributed Systems." PODC 2000.** CAP theorem context.
- **Gilbert, S.; Lynch, N. "Brewer's conjecture and the feasibility of consistent, available, partition-tolerant web services." ACM SIGACT News 33(2), 2002.** CAP proof.

### Industry + practitioner sources

- **AWS S3 Service Disruption postmortem, 2017-02-28.** AWS official postmortem. https://aws.amazon.com/message/41926/
- **Cloudflare leap second outage postmortem, 2012-07-01.** Cloudflare engineering blog. Detailed account of the kernel livelock pattern.
- **Reddit leap second outage discussion, 2012-07.** Reddit engineering retrospective.
- **LinkedIn leap second outage discussion, 2012-07.** LinkedIn engineering blog.
- **AWS "Look Before You Leap" announcement, 2015 + 2016.** AWS official announcements regarding leap second handling.
- **Meta (Facebook) "It's Time to Leave the Leap Second in the Past." 2022-07-25.** Industry case to eliminate leap seconds; affirms smear pattern is current best practice while standards bodies catch up.
- **Google "Time, technology and leaping seconds" blog, 2008-09 + 2011 update.** Originating Google Smear announcement.
- **Beyer, B.; Murphy, N. R.; Rensin, D. K.; Kawahara, K.; Thorne, S. (Eds.). *The Site Reliability Workbook: Practical Ways to Implement SRE.* O'Reilly, 2018.** Chapter 24 "Distributed Periodic Scheduling with Cron" + lock anti-pattern guidance.
- **Beyer, B.; Jones, C.; Petoff, J.; Murphy, N. R. (Eds.). *Site Reliability Engineering: How Google Runs Production Systems.* O'Reilly, 2016.** Foundational SRE book.
- **Leach, B. "Implementing Stripe-like Idempotency Keys in Postgres." Brandur Leach engineering blog, 2014.** https://brandur.org/idempotency-keys
- **Leach, B. "Designing Robust and Predictable APIs with Idempotency." Stripe engineering blog, 2017.** https://stripe.com/blog/idempotency
- **McCaffrey, C. "Distributed Sagas: A Protocol for Coordinating Microservices." re:Invent 2017 keynote.** AWS Builders Library follow-up.
- **AWS DistributedSagas pattern documentation, AWS Builders Library, 2020-2025.** https://aws.amazon.com/builders-library/
- **Kleppmann, M. "How to do distributed locking." 2016 blog post.** Fencing token concept; argument against locks.
- **CockroachDB Design Document, 2015 (updated 2018+).** Cockroach Labs design doc describing HLC integration.
- **YugabyteDB Consistency Model docs, 2018+.** YugabyteDB documentation of HLC + raft.
- **MongoDB "Causal Consistency" documentation, 3.6+ (2017).** MongoDB Atlas causal consistency docs.
- **Microsoft Azure Cosmos DB consistency levels documentation.** Strong / BoundedStaleness / Session / ConsistentPrefix / Eventual; comparative reference.
- **AWS Aurora DSQL announcement, re:Invent 2024.** Aurora DSQL's use of microsecond-accurate PTP via Amazon Time Sync Service.
- **Amazon Time Sync Service documentation.** AWS PTP-disciplined time service for Aurora DSQL.
- **Stripe API Reference, "Idempotent Requests" section.** Canonical idempotency-key documentation.
- **IETF draft `draft-ietf-httpapi-idempotency-key-header-09`.** Idempotency-Key HTTP header standardisation.
- **Temporal.io documentation.** Workflow engine + saga + idempotency primitives.
- **Microsoft Durable Functions documentation.** Saga pattern in Azure Functions.
- **Netflix Conductor documentation.** Workflow engine + saga.
- **Atlassian 2022 outage postmortem.** Lessons on idempotency-less automated cleanup.
- **GitHub 2023 Redis-lock-zombie incident discussion.** Lessons on distributed-lock fragility.
- **Klein, B. "Cron at scale." Google SRE blog, 2018.** Origin of per-cell cron with jitter pattern.
- **chronyd documentation (chrony.conf manual page).** `leapsectz`, `smoothtime`, `maxslewrate` directives.
- **PTPv2 IEEE 1588-2019 standard.** Precision Time Protocol.

### Regulatory + standards sources

- **NIST SP 800-160 — Systems Security Engineering.** Timekeeping requirements for high-assurance systems.
- **NIST SP 800-92 — Guide to Computer Security Log Management.** Audit log ordering + integrity.
- **NIST SP 800-207 — Zero Trust Architecture.** Per-call deadline + policy evaluation.
- **NIST SP 800-162 — Attribute Based Access Control.** ABAC + context attributes (deadlines).
- **GDPR Article 22 — Automated individual decision-making.** Individual decision auditability.
- **GDPR Article 12 — DSAR response SLA.** Time-bounded compliance.
- **HIPAA Security Rule §164.312(b) — Audit Controls.** Audit log requirements.
- **SOC 2 Type II — CC7.2 (System Monitoring).** Monitoring requirements.
- **ISO 27001 Annex A.12.4 — Logging and monitoring.** Audit log integrity.
- **ISO 22301 — Business continuity management systems.** RTO / RPO requirements.
- **KR-PIPA Article 22 — Consent.** Consent record temporal validity.
- **KR-FSS financial trading regulations.** Order matching ordering requirements (drives Tier-4 TrueTime requirement).
- **CSAP (Cloud Security Assurance Program) v3.1.** Korean cloud regulator framework.
- **EU AI Act 2024/1689 Article 14 — Transparency.** AI-mediated decision audit trail.
- **FRCP 37(e) — Preservation of Electronically Stored Information.** Legal hold + retention.

### Internal portfolio ADRs

- **ADR-0005 — Eventing backbone outbox pattern.** Outbox events carry HLC.
- **ADR-0009 — Cell architecture per-tenant per-region.** Per-cell time discipline.
- **ADR-0010 — Regional pack architecture.** Per-pack overlay applies.
- **ADR-0028 — Cloud microservice architecture.** Audit-chain HLC integration.
- **ADR-0035 — Workflow engine state machine + DAG hybrid.** Saga coordinator.
- **ADR-0040 — Metric-gated rollback.** HLC-tagged metric windows.
- **ADR-0049 — Cross-region replication + residency.** Replication ordering via HLC.
- **ADR-0099 — Data class registry.** Data classes influence idempotency-key TTL.
- **ADR-0105 — Thirteen-layer canonical enum.** Time kernel is a shared kernel-layer concern.
- **ADR-0128 — Hyperscaler architecture invariants.** INV-IDEMPOTENCY formalised here.
- **ADR-0145 — Inter-microservice communication reform.** 2PC ban + deadline propagation.
- **ADR-0150 — Cedar policy engine.** Cedar fragments for deadline + isolation.
- **ADR-0174 — FinOps + sustainability tagging.** Atomic clock power draw attribution.
- **ADR-0176 — Brown-out + degradation signal.** Brown-out triggers on uncertainty breach.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Cedar policy at app tier.
- **ADR-0211 — In-house Rust-primary tech stack preference.** HLC + TrueTime implemented in-house.
- **ADR-0212 — Buildability doctrine.** This ADR is itself a deliverable artifact.
- **ADR-0222 — Saga + compensating-transaction portfolio policy.** Saga + idempotency-key composition.
- **ADR-0240 — Sovereign cloud per regional pack.** Cross-pack ordering via HLC.
- **ADR-0241 — DR + business-continuity portfolio policy.** RTO targets achievable via replay safety.
- **ADR-0242 — `oyatie`-is-a-tenant doctrine.** `oyatie.*` principals use the same time primitives.
- **ADR-0243 — Cedar as universal gate.** Time bounds + isolation opt-in via Cedar.
- **ADR-0244 — Tenant as universal scoping primitive.** Tenant timezone stored in tenant directory.
- **ADR-0247 — Self-hosting / self-modification doctrine.** Workflows under HLC + saga.
- **ADR-0248 — Amazon-shape cellular architecture.** Per-cell cron + per-cell idempotency.
- **ADR-0251 — Compliance pack + cell certification levels.** Tier-4 cells get TrueTime.

### Auto-memory feedback

- `feedback_time_coordination_distributed_consistency` — NEW.
- `feedback_quality_performance_scalability_bar` — reinforced.
- `feedback_no_silent_regression` — reinforced; HLC integration is versioned.
- `feedback_autonomous_implementation_artifacts` — reinforced; replay safety enables autonomous execution.
- `feedback_bominal_inheritance_precedence` — applies; this ADR overrides any Bominal-inherited wall-clock doctrine.
- `feedback_automate_everything` — reinforced.
- `feedback_clean_architecture_requirements` — reinforced; HLC is a kernel-layer concern, integrated uniformly.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in the keystone bundle, every
architectural decision in this ADR is attributed to a named
hyperscaler pattern + source + anti-pattern avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (HLC default clock primitive) | "Hybrid Logical Clock" | Demirbas + Kulkarni OPODIS 2014; CockroachDB design doc 2015; MongoDB Atlas Causal Consistency docs; YugabyteDB consistency docs; TiDB Percolator+HLC docs | "Wall-Clock Ordering" — every leap-second + NTP-skew outage in the §Context table |
| D-2 (TrueTime tier for Tier-4 cells) | "Atomic-Clock-Backed External Consistency" | Spanner OSDI 2012; AWS Aurora DSQL re:Invent 2024 | "TrueTime Everywhere" — cost prohibitive at fleet scale |
| D-3 (causal default; strict total order opt-in) | "Tiered Consistency Model" | MongoDB Atlas tiers; Azure Cosmos DB 5-level model; CockroachDB causality propagation | "One Size Fits All Consistency" — over-coordination cost or under-consistency bugs |
| D-4 (caller-supplied idempotency keys) | "Stripe Idempotency Key" | Brandur Leach 2014; Stripe API docs; AWS SDK retryable APIs 2018+; IETF draft `idempotency-key-header-09` | "Retry-Without-Dedup" — every AWS Lambda duplication / Atlassian cleanup class incident |
| D-5 (no distributed locks; saga + idempotency replacement) | "Saga + Compensation, Not Lock" | Google SRE Workbook Ch.24; Kleppmann fencing token essay; AWS Distributed Sagas; Temporal docs; ADR-0222 | "Distributed Lock" — GitHub Redis-lock-zombie 2023; every "lock-held-too-long" class incident |
| D-6 (per-cell cron with jitter) | "Per-Cell Periodic Scheduling" | SRE Workbook Ch.24; Klein "Cron at scale" 2018; AWS EventBridge per-region scheduler model | "Global Cron Service" — single-point-of-failure scheduler cascading across cells |
| D-7 (Google Smear leap second handling) | "Linear Time Smear" | Google blog 2008+2011; AWS announcement 2015+2016; Meta blog 2022; chronyd `leapsectz slew` | "Step-At-Leap-Boundary" — Linux kernel livelock 2012-06-30 |
| D-8 (idempotency key format spec) | "Opaque Self-Describing Key" | Stripe key shape; Square Idempotency-Key; Twilio Idempotency-Key | "Server-Generated Idempotency Key" — defeats the retry-safety guarantee |
| D-9 (cross-cell idempotency replication) | "Per-Cell Idempotency Store" | AWS Step Functions per-region state; Temporal per-cluster idempotency | "Global Idempotency Store" — single-point-of-failure for retry safety |
| D-10 (audit-chain HLC ordering + cross-cell gossip) | "HLC-Ordered Audit Chain" | CockroachDB CDC ordering; Cassandra cluster timestamp resolution post-HLC adoption | "Wall-Clock Audit Ordering" — forensic ambiguity during cross-region investigations |
| D-11 (time bound in Cedar context) | "Policy-Enforced Deadline" | NIST SP 800-207 ZTA per-call evaluation; AWS Builders Library "request budgets" | "Implicit Infinite Timeout" — unbounded latency from missing deadline |
| D-12 (replay safety via idempotency + saga + HLC) | "Deterministic Workflow Replay" | Temporal replay model; AWS Step Functions checkpoint replay; Cadence replay | "Non-Replayable Workflow" — manual recovery from arbitrary failure |
| D-13 (clock skew tolerance bounds + alerts) | "Uncertainty-Bounded Time" | Spanner TT uncertainty bound; CockroachDB max_offset config | "Silent Clock Drift" — incorrect timestamps without warning |
| D-14 (time-based feature flags in Cedar) | "Policy-as-Feature-Flag" | AWS Verified Permissions feature gates; ADR-0243 unification | "Separate Feature-Flag SDK" — LaunchDarkly-class parallel policy |
| D-15 (per-µservice HLC integration contract) | "Uniform Clock Abstraction" | CockroachDB hlc.Clock interface; MongoDB Cluster Time integration | "Per-µservice Clock Reimplementation" — drift between services |
| D-16 (Postgres REPEATABLE READ default; SERIALIZABLE Cedar gate) | "Tiered Isolation with Policy Opt-In" | Postgres SSI (Cahill 2008); CockroachDB SERIALIZABLE; Azure Cosmos consistency tiers | "SERIALIZABLE-Everywhere" — performance tax for ops that don't need it |

---

## Appendix B: Worked example — idempotent payment retry across cell failover

To illustrate that idempotency keys + saga compensation + HLC compose
correctly during a real failure mode, here is a worked example.

**Scenario.** A tenant on the `tenant-acme-corp` tenant initiates a
payment of $1000 USD to a vendor via the platform's Payments
µservice. The tenant's home cell is `data-plane-cell-us-west-2-a`
(`C1`); the vendor's settlement bank's webhook endpoint is in
`data-plane-cell-us-east-1-b` (`C2`). The payment flow is a saga
with three steps:

1. `payments.charge` (cell C1): debit the tenant's funding source.
2. `payments.transfer` (cell C2): credit the vendor's settlement
   account.
3. `payments.confirm` (cell C1): mark the payment as settled in the
   tenant's ledger.

Each step declares a compensation: `payments.refund` (compensates
charge); `payments.reverse-transfer` (compensates transfer);
`payments.unconfirm` (compensates confirm).

**Failure timeline.**

| HLC (pt, l) | Cell | Event |
|---|---|---|
| (1716220000000, 0) | C1 | Tenant client generates idempotency key `idem_jx9k2m4p7q3r8s5t1v6w0y2z4b8c6e9f` and sends `POST /payments` with `Idempotency-Key: idem_jx9k2m4p7q3r8s5t1v6w0y2z4b8c6e9f`, body `{vendor: "acme-vendor", amount_usd_cents: 100000}`. |
| (1716220000050, 0) | C1 | Payments µservice in C1 receives request. Computes `request_signature_hash = SHA256(canonicalised)`. Queries `idempotency_keys` table for `(tenant-acme-corp, idem_jx9k...)`. No row. |
| (1716220000051, 0) | C1 | Inserts `idempotency_keys` row with HLC `(1716220000051, 0)`, `request_signature_hash`, `cached_response_blob: null`, `expires_at: now + 7 days` (payments TTL). |
| (1716220000060, 0) | C1 | Calls Workflow Engine saga coordinator to begin saga `saga_payment_<uuid>` with three steps + compensations + per-step idempotency keys derived from `(saga_id, step_id, attempt_number)`. |
| (1716220000070, 0) | C1 | Saga begins. Step 1 (charge) executes. Tenant funding source debited $1000. Postgres transaction commits at HLC `(1716220000150, 0)`. |
| (1716220000150, 0) | C1 | Saga step 1 emits audit row class `SagaForward` with HLC. Saga advances to step 2. |
| (1716220000160, 0) | C1→C2 | Step 2: C1 calls C2's payments.transfer with the saga-step idempotency key. gRPC envelope carries HLC + saga coordinator token (per ADR-0145). |
| (1716220000300, 0) | C2 | C2's payments.transfer receives request. Inserts `idempotency_keys` row in C2 (cross-cell replication per D-9). Begins transfer. |
| (1716220000500, 0) | C2 | **Cell C2 experiences a regional network partition.** Workflow Engine in C1 doesn't receive the response from C2 within deadline. Saga step 2 status: unknown. |
| (1716220030000, 0) | C1 | Workflow Engine in C1 retries step 2 after exponential backoff. **Same saga-step idempotency key** is reused (per D-12 determinism requirement). |
| (1716220030200, 0) | C2 | **Cell C2 partition heals.** Workflow Engine retry call lands. C2's payments.transfer sees the idempotency key. Queries `idempotency_keys` table. Row exists with HLC `(1716220000300, 0)`. |
| (1716220030201, 0) | C2 | C2 checks `cached_response_blob`. Per the partial-state recovery flow (D-4 edge case), the row exists but `cached_response_blob` may be null (if the C2 transfer was mid-flight when the partition hit) or populated (if the transfer completed before the partition isolated C2). |
| | C2 | **Branch A: transfer completed.** `cached_response_blob` populated. C2 returns the cached response. No double-transfer. |
| | C2 | **Branch B: transfer mid-flight.** `cached_response_blob` null. C2 waits up to `Idempotency-Key-Timeout` (30s default) for the original transfer to complete. If completes, returns cached response. If still null after timeout, re-executes the transfer — but the transfer handler is **deterministic given the idempotency key**, so re-execution credits the vendor at most once (the handler's first action is to atomically claim the key via Postgres `INSERT ... ON CONFLICT DO NOTHING` against a `payment_transfer_executions` table; only the winning insert proceeds to actually credit). |
| (1716220030450, 0) | C2→C1 | C2 returns response (cached or re-executed). HLC advanced per HLC.observe(). |
| (1716220030500, 0) | C1 | Saga step 2 audit row emitted with HLC. Saga advances to step 3. |
| (1716220030600, 0) | C1 | Step 3 (confirm) executes. Tenant ledger updated. |
| (1716220030700, 0) | C1 | Saga completes. Workflow Engine emits `SagaCompleted` audit row. Updates `idempotency_keys` row with `cached_response_blob` (final response). |
| (1716220030750, 0) | C1 | Caller receives HTTP 200 with payment confirmation. |

**Why this is safe.**

1. **No double-charge.** The top-level `Idempotency-Key:
   idem_jx9k2m4p7q3r8s5t1v6w0y2z4b8c6e9f` ensures that even if the
   tenant client retries the entire payment request (e.g., HTTP
   timeout from C1), the saga is not re-initiated. The cached
   response is returned.
2. **No double-transfer.** Each saga step has its own derived
   idempotency key based on `(saga_id, step_id, attempt_number)`.
   But note: when Workflow Engine retries step 2, it reuses the
   **same** key (attempt_number is part of the key, but the engine
   reuses the original attempt_number for replay). The handler's
   determinism requirement ensures at-most-once execution.
3. **No partial-charge-without-transfer.** If step 2 had failed
   non-recoverably (e.g., vendor account closed), the saga
   coordinator would execute step 1's compensation
   (`payments.refund`) to undo the charge. The compensation also
   carries an idempotency key + is deterministic.
4. **HLC-ordered audit.** Forensics on this saga can reconstruct
   the exact order of events across both cells from the HLC-ordered
   audit chain. Cross-cell gossip merge produces a unified view.
5. **Replay safe across engine restart.** If the Workflow Engine in
   C1 had crashed during the partition, its restart would replay
   the saga log (per ADR-0035) from the last persisted HLC-tagged
   state. Step 2's retry would behave identically.
6. **No distributed lock involved.** At no point does the flow
   acquire a cross-µservice or cross-cell lock. Coordination is
   entirely via idempotency keys + saga compensation. The
   `oya-check-no-distributed-lock` lane passes.
7. **Deadline enforcement.** Each saga step carries a deadline
   propagated via gRPC `grpc-timeout` header (per ADR-0145). The
   Cedar gate (`action-deadline-coherence.cedar`) refuses actions
   whose deadline has passed. The 30s `Idempotency-Key-Timeout`
   wait is bounded; it does not block indefinitely.

**Without this ADR's primitives.** The same scenario under wall-
clock-only + no-idempotency-key + ad-hoc-locks would:

- Risk double-charge if the client retries (no top-level dedup).
- Risk double-transfer if the engine retries (no step-level dedup).
- Require a cross-cell lock during the transfer (Redlock-style;
  fragile across partitions; canonical anti-pattern).
- Produce ambiguous audit ordering across cells (wall clock skew
  between C1 and C2 creates forensic gaps).
- Cannot replay the saga deterministically after engine restart
  (no HLC-tagged state).
- Cannot enforce deadlines (no deadline propagation primitive).

The keystone closes these risks by construction. The compensation +
idempotency + HLC composition is the same composition Spanner +
Stripe + Temporal + AWS Step Functions converged on independently.

---

## Naming justification

Every name introduced or ratified by this ADR is validated against BNF v4.1
(`oya-<microservice>[-<bc-tokens>]-<layer>`) and the ADR-0105 13-value canonical
layer enum.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|------|-----------------|-------------------|---------------|
| `oya-shared-time-kernel` | `kernel` | `oya` · `shared` · `time` · `kernel` | Pure domain primitive: HLC algorithm + TrueTime API; no I/O, no side-effects; `kernel` layer per ADR-0105 (innermost pure logic) |
| `oya-shared-idempotency-key` | `kernel` | `oya` · `shared` · `idempotency-key` · (implicit `kernel`) | Key generator + validator; pure deterministic logic with no external I/O; `kernel` layer; BC token is two-word hyphenated identifier which is BNF-valid |
| `oya-check-hlc-integration-coverage` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `hlc-integration-coverage` | Fitness-check; verifies every µservice integrates `oya-shared-time-kernel` per §D-15; `oya-check-*` flat namespace |
| `oya-check-no-wall-clock-for-ordering` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `no-wall-clock-for-ordering` | Fitness-check; static analysis lane refusing business-logic use of `std::time::SystemTime` for ordering; `oya-check-*` flat namespace |
| `oya-check-idempotency-key-coverage` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `idempotency-key-coverage` | Fitness-check; verifies 100% of mutable RPC handlers carry idempotency key per §D-4; `oya-check-*` flat namespace |
| `oya-check-no-distributed-lock` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `no-distributed-lock` | Fitness-check; static analysis lane banning distributed lock patterns outside enumerated exceptions per §D-5; `oya-check-*` flat namespace |
| `oya-check-leap-smear-configured` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `leap-smear-configured` | Fitness-check; chronyd config audit verifying Google 24-hour linear leap smear per §D-7; `oya-check-*` flat namespace |
| `oya-check-per-cell-cron-jitter` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `per-cell-cron-jitter` | Fitness-check; workflow registration audit verifying jitter applied to all cron schedules per §D-6; `oya-check-*` flat namespace |
| `oya-check-timeout-deadline-propagation` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `timeout-deadline-propagation` | Fitness-check; gRPC header audit verifying deadline propagation per §D-11; `oya-check-*` flat namespace |
| `oya-check-idempotency-handler-determinism` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `idempotency-handler-determinism` | Fitness-check; handler unit-test scaffold verifying deterministic replay per §D-12; `oya-check-*` flat namespace |

---

*End of ADR-0252.*
