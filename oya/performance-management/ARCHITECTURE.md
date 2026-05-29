# Performance Management — Architecture

| Field | Value |
|---|---|
| Microservice | `performance-management` |
| Doc class | ARCHITECTURE |
| Date | 2026-05-21 |
| Status | wave-4-rolling-remediation |
| Binding ADRs | 0105, 0131, 0132, 0244, 0245, 0248, 0314, 0315, 0316, 0321, 0328, 0329, 0330, 0331 |

## 1. System context

`performance-management` is an HR/Payroll Big-8 family µservice that owns the operational
concern of employee performance, talent, engagement, and recognition. It is **not** a system
of record for employment status (that is `people-records`), nor for pay (that is `compensation`
or `payroll`), nor for course enrollment (that is `learning-management`). It is the
system of action for everything that happens *between* employees and managers about how work
is being done and how it should evolve.

```
+---------------------+        +----------------------+
| Employees, Managers |        |  External Auditors   |
| HRBPs, Executives   |        |  (regulator pack)    |
+---------+-----------+        +-----------+----------+
          |                                |
          v                                v
+---------+--------------------------------+----------+
|               edge (HTTP/3 + QUIC, ECH, PQC)        |
|                  iac/edge-waf.yaml                  |
+-----------------------+-----------------------------+
                        |
                        v
+-----------------------+-----------------------------+
|        performance-management µservice              |
|  +-------------------------------------------------+|
|  | api  -> rest -> application -> usecase ->       ||
|  |        domain -> kernel -> adapter -> worker -> ||
|  |        governance -> interface -> shared ->     ||
|  |        contract -> infrastructure (ADR-0105)    ||
|  +-------------------------------------------------+|
+--+------+------+--------+--------+--------+---------+
   |      |      |        |        |        |
   v      v      v        v        v        v
people  comp.  learning  time-   workforce  recruit-
record         mgmt      track   planning   ing
```

## 2. Layer architecture (ADR-0105 13-layer enum)

Every concrete file in `src/` resides in exactly one of the thirteen layer slots. The inward
dependency rule is enforced by a CI lane (`oya-check-dependency-seam`):

| Layer | Role | Source path |
|---|---|---|
| `api` | request shape, transport adapter | `src/api/` |
| `rest` | REST endpoint dispatch | `src/rest/` |
| `application` | composition of use cases | `src/application/` |
| `usecase` | one use case = one transactional intent | `src/usecase/` |
| `domain` | entities, aggregates, invariants | `src/domain/` |
| `kernel` | tenant scope, time, identity primitives | `src/kernel/` |
| `adapter` | ports to external systems | `src/adapter/` |
| `worker` | background jobs, schedulers | `src/worker/` |
| `governance` | policy and audit hooks | `src/governance/` |
| `interface` | trait definitions for ports | `src/interface/` |
| `shared` | crate-local utilities | `src/shared/` |
| `contract` | OpenAPI/AsyncAPI/proto3 generated bindings | `src/contract/` |
| `infrastructure` | concrete repo, queue, cache impls | `src/infrastructure/` |

The kernel layer was missing as of the 2026-05-21 audit (Finding 1.5.A); the remediation
adds `src/kernel/mod.rs` with `TenantScope`, `PrincipalId`, `Clock`, `AuditEventId` types.

## 3. Data model

### 3.1 Aggregate map

```
Goal ─┬─ GoalAlignmentEdge (cascade)
      └─ GoalCheckIn (quarterly progress)

ReviewCycle ─┬─ ReviewForm (per employee per cycle)
             ├─ ReviewEvidenceSeal (immutable proof)
             └─ ReviewRating (finalized; triggers comp handoff)

FeedbackEntry ─── FeedbackRequest (request-response chain)
                  └─ Feedback360Collection (peer aggregation)

EngagementPulse ─── EngagementResponse (anonymized)
                    └─ EngagementRelease (anonymity-checked)

CalibrationSession ─┬─ CalibrationBucket (rating distribution)
                    ├─ NineBoxCell (performance × potential)
                    └─ CalibrationLedger (audit chain)

OneOnOne ─── OneOnOneAgenda
             └─ OneOnOneActionItem

WeeklyCheckIn ─── WeeklyCheckInResponse
                  └─ WeeklyCheckInRollup (manager view)

SuccessionTalentCard ─── SuccessionReadiness
                         └─ SuccessionPlan (per role)

RecognitionPost ─── RecognitionReaction
```

### 3.2 Invariants

- **I1 (tenant integrity)**: every aggregate root has `tenant_id` enforced at the kernel
  layer; cross-tenant joins are impossible because primary keys are composite
  `(tenant_id, entity_id)`.
- **I2 (org-tree consistency)**: goal cascade and review-cycle org dependencies are read from
  `people-records` via the `EmployeeDirectoryProjection`; stale projections trigger
  `IP-016-backfill-replay-worker` reconciliation.
- **I3 (anonymity floor)**: engagement-pulse releases require ≥`minimum_cohort_size`
  responses (default 8); below floor, the gate denies aggregate release and emits
  `EVT-PERFORMANCE-ENGAGEMENT-RELEASE-HELD`.
- **I4 (calibration lock)**: a calibration session holds a single-writer lock for its
  duration; lock owner is recorded in `CalibrationLedger`; deadlock recovery via
  `runbooks/calibration-deadlock.md`.
- **I5 (rating immutability post-seal)**: once `ReviewEvidenceSeal` is sealed, ratings can
  only change via `rating.change.breakglass` action requiring a security-plus-service-owner
  approval chain.
- **I6 (handoff atomicity)**: outbound events to siblings are emitted only after local
  transaction commit; outbox pattern per IP-006.

## 4. Sequence diagrams

### 4.1 review-cycle close

```
HRBP            review-svc       calibration       compensation       people-records
 |                  |                 |                  |                  |
 |--kickoff-------->|                 |                  |                  |
 |                  |--load-cohort--->|                  |                  |
 |                  |                 |--read-org------->|                  |--projection
 |                  |                 |<-------org-tree--|                  |
 |                  |<--cohort--------|                  |                  |
 |<--cycle-open-----|                 |                  |                  |
 ...(managers draft, peers feedback)...                                     |
 |--lock-calibration->|               |                  |                  |
 |                  |--session-start->|                  |                  |
 |                  |                 |--bucket-write    |                  |
 |                  |                 |--lock            |                  |
 |                  |<-outcomes-------|                  |                  |
 |--seal----------->|                 |                  |                  |
 |                  |--RatingFinalizedEvent ─────────────>|                 |
 |                  |--CalibrationOutcomeRecord ─────────────────────────-->|
 |<--sealed---------|                 |                  |                  |
```

### 4.2 goal-cycle cascade

```
Employee        goal-svc          ontology         people-records
   |               |                 |                 |
   |--draft goal-->|                 |                 |
   |               |--read org tree--|---------------->|
   |               |<--manager + peers----------------|
   |               |--cascade-edge-->|                 |
   |<--draft saved-|                 |                 |
   |--align-up---->|                 |                 |
   |               |--AlignmentEvent->|                |
   |<--cascade ok--|                 |                 |
```

### 4.3 engagement-pulse anonymity-guarded release

```
HRBP        pulse-svc       anonymity-guard       audit-chain
 |              |                 |                    |
 |--release---->|                 |                    |
 |              |--cohort-size?-->|                    |
 |              |                 |-- count vs floor   |
 |              |<-- ok/deny -----|                    |
 |              |--seal release--->|                   |
 |              |                  |--audit event----->|
 |<--ok--------|                  |                    |
```

### 4.4 one-on-one prep packet

```
Manager      one-on-one-svc     manager-tool      feedback-svc
 |               |                  |                 |
 |--open-1on1--->|                  |                 |
 |               |--draft-packet--->|                 |
 |               |                  |--get feedback-->|
 |               |                  |<----entries-----|
 |               |<----packet-------|                 |
 |<--ready------|                  |                 |
```

### 4.5 succession talent-card publish

```
HRBP        succession-svc      workforce-planning      audit
 |              |                       |                  |
 |--author----->|                       |                  |
 |              |--talent-card ready    |                  |
 |--publish---->|                       |                  |
 |              |--SuccessionTalentCardEvent-->            |
 |              |                       |--ingest          |
 |              |--seal evidence---------------->          |
 |<--published--|                       |                  |
```

## 5. Component decomposition

The crate (`Cargo.toml`) is the single binary. Internally it is composed of seven modules
(one per ADR-0316 capability tier projection):

- `goal_module` (goal-cycle + cascade + check-in).
- `review_module` (review-cycle + form + seal + rating).
- `feedback_module` (entry + request + 360).
- `engagement_module` (pulse + anonymity-guard + release).
- `calibration_module` (session + bucket + nine-box + ledger).
- `talent_module` (succession + talent-card + readiness).
- `coaching_module` (one-on-one + weekly-check-in + manager-tooling).

Each module owns its domain aggregates and exposes use cases through the `usecase/` layer.
The shared `kernel/` layer enforces I1 (tenant integrity) and I5 (rating immutability) at
the type-system level.

## 6. Transport architecture

HTTP/3 + QUIC per ADR-0253-amendment. Three internal transport channels:

1. **Public REST + GraphQL** — over HTTP/3, exposed at the cell ingress.
2. **Internal gRPC** — over HTTP/3 with mTLS (OpenBao certs); used for sibling RPC.
3. **Async events** — over the workflow-engine substrate; AsyncAPI 3.1.0 envelopes.

The `iac/ech-config.yaml` enables Encrypted Client Hello; `iac/pqc-cert.yaml` enables
hybrid X25519+Kyber768 KEM cert chain for the QUIC handshake.

## 7. Storage architecture

Per-tenant logical isolation, per-cell physical sharding:

- **Primary store**: PostgreSQL 17 with row-level security keyed on `tenant_id`.
- **Audit chain**: append-only ledger via `audit-chain` µservice.
- **Cache**: Valkey cluster (cell-local) for projection cache; not authoritative.
- **Object storage**: S3-compatible (cell-local) for review-form attachments and exported
  redacted evidence packets.
- **Search**: opensearch index for feedback text + recognition wall.

Cross-cell replication: metadata only (per `cell_eligibility.cross_cell_replication`); a
tenant's home cell is authoritative for content.

## 8. Cross-microservice handoff edges

Per audit Finding 2.2.A and §3.4.B, the following sibling edges are first-class architecture:

| Edge ID | Direction | Event/Operation | Sibling | Contract file |
|---|---|---|---|---|
| B-1 | outbound | `RatingFinalizedEvent` | `compensation` | `contracts/hr-handoff-compensation.asyncapi.yaml` |
| B-2 | outbound | `CalibrationOutcomeRecord` | `people-records` | `contracts/hr-handoff-people-records.asyncapi.yaml` |
| B-3 | inbound | `EmployeeDirectoryProjection` | `people-records` | same |
| B-4 | inbound | `CompensationBandReference` | `compensation` | `contracts/hr-handoff-compensation.asyncapi.yaml` |
| B-5 | inbound | `LearningCompletionEvent` | `learning-management` | `contracts/hr-handoff-learning-management.asyncapi.yaml` |
| B-6 | inbound | `TimeOffPeriod` | `time-tracking` | `contracts/hr-handoff-time-tracking.asyncapi.yaml` |
| B-7 | outbound | `SuccessionTalentCardEvent` | `workforce-planning` | `contracts/hr-handoff-workforce-planning.asyncapi.yaml` |
| B-8 | outbound | `ReviewCycleStateEvent` | `analytics` | (via substrate) |
| B-9 | inbound | `RecruitingHiredEvent` | `recruiting` | `contracts/hr-handoff-recruiting.asyncapi.yaml` |

## 9. Cedar policy architecture

Seven policy files in `policies/`. The policy-engine substrate (per ADR-0243) evaluates every
authorization request server-side, with caller-side library cache for sub-millisecond hot
path. Default-deny; explicit `permit` rules per role + context.

Policy evaluation pipeline:

1. Request arrives at `rest/` layer with `tenant_id`, `principal_id`, `action`, `resource`.
2. `governance/` layer constructs `context` object including `tenant_class`, `pack_overlay`,
   `requested_data_class`.
3. Caller-side Cedar library evaluates against cached policies (TTL 5min).
4. If miss or stale, defers to substrate `policy-engine` µservice.
5. Decision logged to audit chain.

## 10. Workflow engine integration

Long-running stateful flows registered under `workflow-engine` substrate (per ADR-0263):

- `wf.perf.goal-cycle.cycle-life` — 12-month state machine.
- `wf.perf.review-cycle.annual` — 90-day state machine.
- `wf.perf.engagement-pulse.quarterly` — 4-week state machine.
- `wf.perf.calibration.session` — 1-day state machine with lock semantics.
- `wf.perf.succession.review-cadence` — annual cadence.
- `wf.perf.weekly-check-in.weekly` — 7-day recurring.

Each workflow has a template at `IP-004-workflow-template-library.md` with explicit step
list, branch conditions, timeout policy, and idempotency keys.

## 11. Tenant-class branching architecture (ADR-0331)

Per audit Finding 3.4.C all surfaces branch on `tenant_class ∈ {demo_trial, paid}`:

| Surface | demo_trial behavior | paid behavior |
|---|---|---|
| REST writes | accepted but synthetic-only | full |
| Exports | denied (Cedar gate) | redacted via pack |
| Marketplace settlement (ADR-0314) | suppressed | full DealSet flow |
| SLO targets | 5x loose | strict |
| Cost-budget allocation | OCI Always Free pool | billed cell |
| Dashboards | demo-noise pane | tenant pane |
| Compliance pack overlays | gdpr only | tenant-selected |

## 12. Multi-context deployment architecture

Six OpenTofu modules under `iac/<context>/`. Each module ships:

- `main.tf` — top-level provisioning.
- `versions.tf` — OpenTofu + provider pinning.
- `variables.tf` — `tenant_id`, `cell_tier`, `tenant_class` inputs.
- `outputs.tf` — endpoint URLs, secret refs.
- `helm.tf` — `helm_release` for the chart.
- `network.tf` — VPC/VCN/cluster network shape.
- `billing.tf` — billing-component binding to the marketplace settlement substrate.
- `README.md` — context-specific runbook.

The `iac/oci-guest/always-free/` sub-module specifically targets OCI Always Free
(2× Ampere A1 ARM, Autonomous DB, Vault, LB) for demo_trial workloads per
`feedback_oci_always_free_maximization_2026_05_20`.

## 13. OS support architecture

Per `supported_oses.json` thirteen OSes are first-class. Container images are the universal
distribution format (Talos/Flatcar/Photon run containers natively); per-OS packages exist
for sidecar binaries:

- RPM (RHEL, Oracle Linux, Rocky, AlmaLinux, Amazon Linux, CentOS Stream).
- DEB (Ubuntu LTS, Debian).
- Container image OCI (Talos, Flatcar, Photon).
- pkg (macOS Apple Silicon M5+ for the `oya` CLI; not the service itself).

Arch matrix: linux/amd64, linux/arm64, darwin/arm64 (Tier-1); ppc64le, s390x (Tier-2).

## 14. Observability architecture

Per ADR-0130 SLO objects gate promotion. Telemetry:

- **Logs**: structured JSON with `tenant_id`, `principal_id`, `trace_id`, `span_id`,
  `audit_event_id`. Levels: error, warn, info, debug; default info.
- **Metrics**: Prometheus-compatible via `iac/local-prometheus-rule.yaml`. Per-bounded-context
  RED metrics + per-operation histograms.
- **Traces**: OpenTelemetry via OTLP collector (`iac/local-otel-collector.yaml`).
- **Audit**: `audit-chain` substrate, append-only.

Dashboards under `dashboards/*.json` (10 JSON files, one per bounded context + ops).

## 15. Capacity model

`capacity-model.md` defines peak load profile. ADR-0248 cell shape:

- Cell tier T1: up to 100k employees per tenant per cell.
- Cell tier T2: up to 25k employees per tenant per cell.
- Shuffle sharding distributes review-cycle close load across 4 sub-cells.
- Hot spots: annual review close (Nov-Jan in N. hemisphere), engagement pulse send (Mon AM).

## 16. Failure modes and recovery

See `failure-modes.md` for the eleven scenario classes and `runbooks/*` for the playbook per
class. Architecture-level invariants:

- All writes go through the workflow-engine for replay safety.
- Outbox table for outbound events (no fire-and-forget).
- Cell-local cache TTL ≤5min so stale tenant-projection blast radius is bounded.
- Breakglass paths require security-plus-service-owner approval chain.

## 17. Migration architecture

Per ADR-0330 the deprecation policy is 90-day with explicit migration guides. Inbound data
migration from Lattice/15Five/Workday Performance covered in `IP-027` and the
`competitor-parity-matrix.md` migration table.

## 18. References

- README: `README.md`
- PRD: `PRD.md`
- Compliance: `compliance.md`
- Parity matrix: `competitor-parity-matrix.md`
- Implementation Plans: `IP-001` … `IP-037`
- Audit: `coherence-audit-2026-05-20.md`
- Remediation: `REMEDIATION-NOTES-2026-05-21.md`
