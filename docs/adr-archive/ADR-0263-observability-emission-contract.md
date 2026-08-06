---
id: ADR-0263
status: Superseded
superseded_by: [ADR-706]
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-security
  - council-privacy
  - ops-sre-reliability
  - ops-compliance
  - axis-observability
  - axis-audit-chain
  - axis-tenancy
  - axis-identity
  - axis-finops
supersedes: []
amends:
  - ADR-0042-observability-stack-otel-and-in-house-ui.md
  - ADR-0153-observability-backplane-high-level-reference.md
  - ADR-0186-observability-backplane-layering.md
related:
  - ADR-0003-audit-chain-and-evidence-emission.md
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0042-observability-stack-otel-and-in-house-ui.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0139-burn-rate-slo-alerting.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0153-observability-backplane-high-level-reference.md
  - ADR-0174-finops-cost-attribution.md
  - ADR-0186-observability-backplane-layering.md
  - ADR-0210-otel-tail-sampling.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md
  - ADR-0319-front-middle-back-office-information-barrier.md
related_adrs:
  - ADR-0297
  - ADR-0311
  - ADR-0313
  - ADR-0316
  - ADR-0319
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/observability.json
  - /specs/microservices/manifest-schema.json
  - /specs/hyperscaler-architecture-invariants.json
related_memory:
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_autonomous_implementation_artifacts
  - feedback_no_silent_regression
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_doc_coverage_enforced
doc_class: Architecture-Decision-Record
purpose: >
  Lock the Tier-1 observability emission contract for every oyatie µservice.
  Every workload — substrate, product, internal `oyatie.*` principal, customer
  tenant call — emits structured JSON logs, OpenTelemetry traces with W3C Trace
  Context propagation, Prometheus metrics with mandatory tenant_id label, and
  exemplars linking metrics to representative traces. Mimir+Loki+Tempo+
  ClickHouse stack (per existing observability µservice) is the canonical
  storage substrate. The Three Pillars of Observability (Charity Majors,
  Honeycomb 2017) — metrics + logs + traces — are unified via correlation IDs.
  Schema is versioned and evolves additive-only; field deprecation requires
  a contract handshake. PII scrubbing happens at the emission boundary, never
  at the storage boundary. Audit-chain integration ensures every state-changing
  emission carries an `audit_id` linking back to ADR-0003 sealed evidence.
enforcement_status: advisory-until-emission-client-lands
enforced_by:
  - oya gate validate observability-emission-contract
  - oya gate validate tenant-label-presence
  - oya gate validate trace-context-propagation
  - oya gate validate metric-naming-convention
  - oya gate validate log-schema-conformance
  - oya gate validate pii-scrubbing-at-emission
  - oya gate validate audit-id-on-state-change
keystone_position: tier-1-lockdown
---

# ADR-0263 — Observability Emission Contract

## Status

Proposed — 2026-05-20.

This ADR establishes the canonical emission contract that every oyatie
µservice must honour to participate in the observability substrate. It is
the Tier-1 lockdown that closes the gaps left by ADR-0042 (stack selection)
and ADR-0186 (backplane layering): those ADRs specified *what we run*; this
ADR specifies *what every µservice must emit, on what schedule, and with
what schema fields* to make the substrate operationally useful.

Enforcement is `advisory-until-emission-client-lands`: validators are
authored now and run REPORT-ONLY mode; they promote to BLOCKER once the
canonical Rust client crate `oya-shared-observability-client-{kernel,
domain,application,api,adapter,sdk,app}` is published and at least the
foundational µservices (`tenancy`, `identity`, `policy-engine`,
`audit-chain`, `observability` itself) have migrated. The promotion
schedule and CI lane gating are codified in §Verification.

## Date

2026-05-20.

## Context

### Three Pillars of Observability

The canonical formulation of "observability" in production systems — as
distinct from "monitoring" — was articulated by Charity Majors (CTO of
Honeycomb) in 2017 ("Observability — A 3-Year Retrospective," Honeycomb
blog) and earlier in the *Distributed Systems Observability* O'Reilly book
(Cindy Sridharan, 2018). The doctrine is:

1. **Metrics.** Numerical aggregates over time (counters, gauges,
   histograms, summaries). Cheap to store at scale; lossy by design;
   high-cardinality labels cost expensive. Used for SLOs, alerting,
   capacity planning, and dashboards.

2. **Logs.** Discrete events with structured payloads. Higher fidelity
   than metrics; more expensive per byte; full-text searchable. Used for
   forensics, debugging, audit, and compliance evidence.

3. **Traces.** Causal records of a single request flowing through the
   distributed system, composed of a tree of spans (each span = one
   operation in one service). Used for latency analysis, dependency
   mapping, and root-cause-on-latency.

Crucially, the Three Pillars are operationally useful **only when unified
via correlation IDs**: a single `trace_id` + `span_id` pair must appear
in the trace, in the structured log emitted during that span, and as an
*exemplar* attached to the metric data point sampled in that span.

Without this unification, the observability stack becomes three siloed
products: a slow log search, a vague metric dashboard, and a useless
trace browser. The whole pyramid collapses to "monitoring." This is the
failure mode ADR-0263 prevents.

### What "emission contract" means

An emission contract is the set of schema invariants every µservice must
honour when it emits a metric, a log, or a span. It is **not** the
storage substrate (that's ADR-0042 + ADR-0186) and **not** the sampling
policy (that's ADR-0210). It is the *boundary at the µservice's outbound
edge* — the moment a log line, a metric sample, or a span finishes
serialisation and enters the OpenTelemetry SDK or Loki/Prometheus client.

Contract elements:

- **Mandatory fields.** Every emission carries `tenant_id`, sub-scope,
  `trace_id`/`span_id` (when applicable), schema version.
- **Naming convention.** Metric names, span names, log keys follow a
  predictable pattern that survives across µservices.
- **Propagation rules.** W3C Trace Context headers traverse every gRPC
  call (per ADR-0145 communication reform), every HTTP request, every
  workflow-engine durable activity, every audit-chain emission.
- **Schema versioning.** Every emission declares a schema version;
  consumers reject incompatible payloads; deprecation requires a
  multi-stage handshake.
- **PII scrubbing.** Every emission passes through a tenant-aware
  scrubber before serialisation; PII never enters the storage substrate.
- **Audit linkage.** Every state-changing emission carries an `audit_id`
  pointing to the sealed audit-chain entry (per ADR-0003).

### Why "Tier-1 lockdown" terminology

Per the `feedback_no_silent_regression` memory, public contracts are
protected from silent change by version bump + ADR + sunset requirements.
The observability emission contract is, in effect, a *public contract
between every µservice and the substrate*: a µservice that emits
non-conformant logs/metrics/traces will produce data the substrate
cannot ingest, cannot aggregate per tenant, and cannot link via
exemplars to traces. The result is silent observability rot — dashboards
go quietly stale, SLO evaluation silently misses signals, incident
response loses fidelity.

ADR-0263 promotes the contract to Tier-1 (the same protection tier as
authentication, audit-chain, and Cedar policy enforcement). Tier-1
means: contract is closed; non-conformant emissions are rejected at
the SDK boundary; schema evolution is additive-only; field deprecation
requires the handshake codified in §D-15.

### Why now (2026-05-20)

Three forcing functions:

1. **ADR-0242 (`oyatie`-is-a-tenant doctrine, 2026-05-20)** unified the
   tenant model. Every µservice now serves all tenants under Cedar
   policy. This makes the `tenant_id` label *universally applicable* —
   there are no longer "internal" emissions that lack a tenant. The
   observability emission contract is the operational consequence: every
   emission carries `tenant_id`, no exceptions.

2. **ADR-0210 (OTel tail-sampling, 2026-05-18)** locked the sampling
   policy assuming a per-µservice emission shape. Tail-sampling requires
   complete trace context (root span closes, evaluator inspects span
   attributes, makes the sample decision). The complete-trace invariant
   depends on every µservice in the call chain emitting spans with W3C
   Trace Context propagation. ADR-0263 codifies the SDK + propagation
   discipline that ADR-0210 presupposes.

3. **Autonomous-masterplan goal (`feedback_autonomous_implementation_artifacts`).**
   The agentic pipeline (Foundry-pipeline lane) requires deterministic
   per-tenant per-µservice observability evidence to answer "did the
   change I just made improve or degrade the system?" If emissions
   diverge in schema, the agentic pipeline cannot make that
   determination automatically. The contract is a prerequisite for the
   autonomous-implementation goal.

### Prior state

ADR-0042 (2025-Q4 era) selected the OTel + Grafana stack and an
in-house UI; ADR-0153 (early 2026) framed the backplane at a high
level; ADR-0186 (mid 2026) layered the backplane into five stages
(collection, trace storage, log storage, metric storage, SLO compute).
ADR-0210 (2026-05-18) tuned the sampling policy. ADR-0242 (2026-05-20)
established tenant universality.

What was *not* codified across those four ADRs: the per-µservice
emission shape. Each µservice was free to choose its own log key names,
metric naming conventions, trace span structure, and field set. PR #143
close-out audit (per the
`evidence/pr-143-close-out-plan-and-gap-audit-2026-05-18.json` ledger)
surfaced 14 instances of emission drift across the M01-foundation
µservices already in flight:

- 6 µservices used `tenant_id`; 4 used `tenantId`; 2 used `tenant`;
  2 omitted entirely.
- 3 µservices emitted nanosecond-resolution timestamps; 7 emitted
  millisecond; 4 emitted RFC 3339 strings.
- 2 µservices used Prometheus-naming `service_microservice_action_total`;
  9 used `<microservice>_<action>_total`; 3 used ad-hoc names.
- 5 µservices propagated W3C `traceparent` consistently; 9 had partial
  propagation (lost the header on the worker side of an outbox flush);
  1 used B3 headers (Zipkin legacy).

Without lockdown, these drifts compound into a substrate that cannot
correlate emissions across µservices for a single request — silent
observability rot.

## Decision

The fifteen decisions below collectively form the emission contract.
Every µservice MUST honour every decision; the CI lanes listed in
§Verification enforce.

### D-1. Three Pillars: metrics + logs + traces

Every oyatie µservice emits **three signal streams**, each governed by
the contract sections below:

1. **Metrics** — Prometheus exposition format (text 0.0.4 minimum;
   OpenMetrics 1.0.0 preferred). Scraped by Grafana Alloy (per ADR-0186
   Stage 1) into Mimir (Stage 4).
2. **Logs** — Structured JSON, one event per line, written to stdout/
   stderr (12-factor compatibility) and tailed by Alloy into Loki
   (Stage 3).
3. **Traces** — OpenTelemetry spans exported via OTLP/gRPC to the
   Alloy DaemonSet (Stage 1), then to the OTel gateway (Stage 2),
   then to Tempo for hot storage and ClickHouse for cold/columnar
   queries (per the existing observability µservice `IP-021-clickhouse-cluster-iac.md`
   and `IP-022-otel-to-clickhouse-bridge.md`).

A µservice that emits only one or two pillars is non-conformant. The
contract requires all three. Reason: correlation across the pillars
is the operational point of observability; partial pillar coverage
breaks correlation.

Profiles (continuous profiling via Pyroscope per ADR-0186 Stage 6) are
NOT part of the Three Pillars contract; they are a fourth optional
pillar declared per µservice in the manifest. Profile emission follows
its own ADR (forthcoming) and is out of scope for ADR-0263.

### D-2. OpenTelemetry SDK mandatory

Every µservice instruments emissions using the **OpenTelemetry SDK**
of its host language. For Rust (the canonical oyatie language per
ADR-0211), this is the `opentelemetry-rust` crate family:

| Crate | Purpose |
|---|---|
| `opentelemetry` | Core API |
| `opentelemetry_sdk` | SDK implementation |
| `opentelemetry-otlp` | OTLP exporter (gRPC) |
| `opentelemetry-semantic-conventions` | Standard attribute names |
| `tracing-opentelemetry` | Bridge from the `tracing` crate to OTel |
| `opentelemetry-prometheus` | Prometheus metrics exporter |
| `opentelemetry-stdout` | Stdout exporter for testing |

Version pin: `opentelemetry = "0.27"` minimum (current stable as of
2026-05-20); rotate per ADR-0098 LTS cadence. The pin is enforced by
the workspace `Cargo.toml` per ADR-0211 in-house tech stack
discipline.

For non-Rust languages used at the SDK / language-binding boundary
(TypeScript for frontends, Python for ML adapters), use the official
OpenTelemetry SDK for that language. No language-specific custom OTel
implementations are permitted.

### D-3. W3C Trace Context propagation (RFC 9384 traceparent header)

All trace context propagation uses the **W3C Trace Context** standard
(W3C Recommendation, 2020; RFC 9384 codifies HTTP header rules):

- HTTP requests carry `traceparent` (mandatory) and `tracestate`
  (optional vendor extensions).
- gRPC requests carry the equivalent via gRPC metadata (key
  `traceparent`).
- Workflow Engine durable activities propagate `traceparent` in the
  activity input envelope (per ADR-0145 inter-µservice
  communication reform).
- Outbox events (per ADR-0005 outbox pattern) propagate `traceparent`
  in the event header.
- Cross-µservice gRPC calls (per ADR-0145 §"Direct gRPC + 3
  invariants") MUST propagate `traceparent`; the `ts1-traceparent-
  propagation` invariant in ADR-0145 §Invariants is hereby
  cross-referenced.
- Async messaging (Kafka, NATS, internal pub/sub) propagates
  `traceparent` as a message header. Kafka headers use the literal
  byte key `traceparent`; NATS uses the Nats header equivalent.

**Legacy protocol prohibitions:**

- B3 headers (Zipkin legacy) are PROHIBITED for new emissions. The 1
  µservice still using B3 (per PR #143 audit) migrates before
  enforcement promotes to BLOCKER.
- Jaeger native propagation format is PROHIBITED.
- Vendor-specific headers (`x-datadog-*`, `x-newrelic-*`,
  `x-honeycomb-*`) are PROHIBITED.

**Propagation across cell boundaries:**

When a request crosses a cell boundary (per ADR-0009 cell architecture)
or a region boundary (per ADR-0049 cross-region replication), the
`traceparent` header is preserved end-to-end. The cross-cell
aggregation cell (§D-10) consumes the unified trace stream and
reconstructs the cross-cell view.

### D-4. Mandatory tenant_id label on every emission

Every metric, log, and trace carries a `tenant_id` label/field/attribute:

- **Metrics** — `tenant_id` is a Prometheus label on every metric
  series. Cardinality budget: ≤10,000 distinct tenants per µservice
  per cell (per Mimir-published cardinality guidance). Higher
  cardinality requires per-tenant rollup (§D-9).
- **Logs** — `tenant_id` is a top-level field in the JSON log
  schema (§D-6).
- **Traces** — `tenant_id` is a span attribute on every span (root
  and child). The attribute key follows OpenTelemetry semantic
  conventions: `oyatie.tenant.id`.

**Special tenant values:**

- For `oyatie.*` principals (per ADR-0242), `tenant_id` is `oyatie`
  (the root tenant) and a separate `oyatie.tenant.subscope` attribute
  carries the dotted sub-scope (e.g., `oyatie.foundry.ci-agent`).
- For requests not yet authenticated (unauthenticated entry point
  before identity resolves the caller), `tenant_id` is the literal
  string `unresolved`. The transition from `unresolved` to a real
  tenant ID happens inside the identity µservice's `ResolveCaller`
  span; the transition is itself a logged event.
- For platform-internal background sweeps that have no caller (e.g.,
  the SLO evaluator's cadence loop), `tenant_id` is `oyatie` with
  sub-scope `oyatie.observability.evaluator`.

**Why mandatory:** the substrate's multi-tenancy story (Mimir's
`X-Scope-OrgID` per-tenant isolation; Loki's similar tenant routing;
Tempo's tenant-aware indexing) collapses without this label. Per-tenant
cost attribution (§D-11) requires it. Per-tenant DSAR (per ADR-0242
§D-4) requires it. Per-tenant rollups (§D-9) require it.

### D-5. Sub-scope label propagation

Every emission also carries a sub-scope label/attribute identifying the
principal initiating the action, formatted per ADR-0242 §D-2:

```
oyatie.tenant.subscope = "oyatie.foundry.ci-agent"
oyatie.tenant.subscope = "tenant-acme-corp.user-7421"
oyatie.tenant.subscope = "tenant-acme-corp.service-account.api-bot"
oyatie.tenant.subscope = "oyatie.preview.123"
```

**Format rules:**

- Dotted-path notation only; no slashes, no hyphens as separators.
- Lowercase ASCII + digits + hyphens (within a path segment) + dots
  (between segments).
- Maximum 4 levels deep per ADR-0242 §"Negative consequences."
- Maximum 128 characters total (cardinality budget; Mimir-published
  guidance).

**Cardinality concerns:**

Per-engineer sandbox tenants (`oyatie.dev.<engineer-id>`) and per-PR
preview tenants (`oyatie.preview.<pr-number>`) can produce high
cardinality in metric labels. Mitigation:

- Per-µservice manifest declares `observability.metric_subscope_policy`:
  `full | rollup-to-parent | omit`. Default `rollup-to-parent` (roll
  ephemeral sub-scopes to their parent for metrics; preserve full
  sub-scope for logs and traces).
- Logs always carry full sub-scope (log storage is per-event, not
  cardinality-constrained).
- Traces carry full sub-scope as a span attribute (cardinality
  doesn't apply at the span level).

### D-6. Structured JSON log schema

**Authority note — layer enum.** The `layer` field in this schema is
governed by **ADR-0105** (13-value canonical layer enum, `Accepted`
status). This ADR does **not** extend or restrict the canonical layer
enum; any deviation from the ADR-0105 set — addition or removal of a
value — requires an ADR-0105 amendment first, per ADR-0056
§"12-Value Layer Enum (closed)": "Adding a layer value is a 1-ADR
action. No aliases or overlaps."

All logs are emitted as **single-line JSON** to stdout/stderr. The
schema is versioned; the current version is `oyatie/log/v1`:

**Mandatory fields** (every log event):

| Field | Type | Description |
|---|---|---|
| `schema` | string | Always `oyatie/log/v1` |
| `timestamp` | string | RFC 3339 with nanosecond precision in UTC (e.g., `2026-05-20T14:23:45.123456789Z`) |
| `level` | string | One of `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL` |
| `message` | string | Human-readable summary, UTF-8, ≤1024 chars |
| `microservice` | string | Owning µservice (e.g., `tenancy`, `identity`, `observability`) |
| `bc` | string | Bounded context (e.g., `slo-engine`, `otel-ingest`) |
| `layer` | string | One of the 13 canonical layers per ADR-0105: `kernel`, `domain`, `application`, `app`, `adapter`, `infrastructure`, `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, `api`. For test/benchmark/fixture code, use an optional companion field `emission_class` (see below) rather than inventing layer values. |
| `trace_id` | string | W3C trace ID (16-byte hex, lowercase) if within a traced operation; otherwise omitted |
| `span_id` | string | W3C span ID (8-byte hex, lowercase) if within a traced span; otherwise omitted |
| `tenant_id` | string | Per §D-4 |
| `subscope` | string | Per §D-5 |
| `principal` | string | OIDC subject claim of the caller (e.g., `service-account:oyatie.foundry.ci-agent`); for unauthenticated, the literal `unauthenticated` |
| `action` | string | Verb describing what was attempted (e.g., `register_tenant`, `evaluate_slo`, `enforce_cedar_policy`) |
| `audit_id` | string | Sealed audit-chain entry ID if this log emission corresponds to a state-changing action (per §D-13); otherwise omitted |
| `cell_id` | string | Cell identifier per ADR-0009 |
| `region` | string | Regional pack identifier per ADR-0010 |

**Optional structured fields** (per emission):

| Field | Type | Description |
|---|---|---|
| `error` | object | When `level` is `ERROR`/`FATAL`: `{type, message, stack, code, retryable}` |
| `duration_ms` | number | If timing was captured |
| `request_id` | string | Inbound request correlation ID (when not derivable from `trace_id`) |
| `request_method` | string | HTTP method or gRPC method name |
| `request_path` | string | HTTP path or gRPC method path |
| `response_status` | number | HTTP status code or gRPC code |
| `resource_id` | string | ID of the resource the action targets (e.g., tenant_id, user_id, slo_id) |
| `resource_type` | string | Type of the resource (e.g., `Tenant`, `User`, `SLOTarget`) |
| `outcome` | string | `success`, `failure`, `partial`, `denied`, `held` |
| `denied_reason` | string | If `outcome=denied`: the Cedar policy fragment ID that denied |
| `emission_class` | string | Optional. Classifies the emission origin context. One of `production`, `test`, `benchmark`, `fixture`. Absent means `production`. Use this field — NOT a non-canonical `layer` value — to identify test/bench/fixture code that emits logs. The `layer` field always carries a value from the ADR-0105 13-value canonical enum. |

**Reserved key prefix:**

- `oyatie.*` is reserved for platform-owned attributes.
- `tenant.*` is reserved for tenant-supplied custom dimensions
  (forthcoming ADR for tenant-supplied labels).
- `_internal.*` is reserved for substrate-internal debugging; SHOULD
  NOT cross the emission boundary in production builds.

**Prohibited fields:**

- Raw PII (email addresses, names, phone numbers, addresses) without
  PII scrubbing (§D-14).
- Authentication tokens, passwords, API keys, session cookies.
- Cryptographic material (private keys, signing keys).
- Raw payloads from end-user requests unless explicitly tagged with
  data class `LOG_SAFE` (see ADR-0034 data-class overrides).

**Encoding rules:**

- UTF-8 throughout.
- Maximum log event size: 64 KiB. Beyond this, the emission is split
  across `parent_log_id` + `chunk_index` fields.
- Newlines within a field value are escaped (`\n`).
- JSON serialiser is `serde_json` with `compact` formatter (no
  pretty-printing) per ADR-0211.

### D-7. Metric naming convention

Metrics follow a **deterministic naming convention** with the colon-
separated prefix structure:

```
oya:<microservice>:<bc>:<metric>_<unit>:<type>
```

Examples:

```
oya:tenancy:lifecycle:tenant_registrations_total:counter
oya:identity:oidc:token_exchange_duration_seconds:histogram
oya:observability:slo-engine:burn_rate_evaluations_total:counter
oya:observability:slo-engine:eligibility_verdicts_emitted_total:counter
oya:audit-chain:writer:event_seal_duration_seconds:histogram
oya:cedar-policy-engine:gate:policy_decisions_total:counter
oya:foundry:ci-agent:workflow_steps_completed_total:counter
oya:workflow-engine:execution:durable_step_duration_seconds:histogram
oya:cloud-secrets:openbao:unseal_status:gauge
oya:cloud-data:postgres:replication_lag_seconds:gauge
```

**Rules:**

- Prefix is always `oya:` (lowercase, colon-terminated).
- `<microservice>` is the µservice slug (matches the manifest
  `microservice` field).
- `<bc>` is the bounded-context name (matches the BC declared in the
  µservice's PRD).
- `<metric>` is `snake_case`, English verb-noun pairs preferred
  (e.g., `evaluations`, `decisions`, `tokens_exchanged`).
- `<unit>` is one of the Prometheus base units: `seconds`, `bytes`,
  `total` (for counters), `info`, `ratio`, or one of the OpenMetrics
  units. No millisecond/microsecond; convert to seconds.
- `<type>` is one of `counter`, `gauge`, `histogram`, `summary`.

**Mandatory labels on every metric:**

| Label | Source | Cardinality budget |
|---|---|---|
| `tenant_id` | §D-4 | ≤10,000 per µservice per cell |
| `subscope` | §D-5 (rollup-to-parent default) | ≤10,000 per µservice per cell |
| `cell_id` | ADR-0009 | bounded (≤256 per region) |
| `region` | ADR-0010 | bounded (≤32) |
| `microservice` | manifest | bounded (≤200) |
| `bc` | PRD declaration | bounded (≤500) |
| `version` | µservice semantic version | bounded (rolling) |

**Histogram bucketing:**

Histograms use **exponential buckets** by default with base `2` and
12 buckets (covering 4 orders of magnitude). Latency histograms use
the OpenTelemetry-recommended buckets: `0.005, 0.01, 0.025, 0.05,
0.1, 0.25, 0.5, 1, 2.5, 5, 10, +Inf` (seconds). Size histograms use
power-of-two buckets from 64 B to 64 MiB.

**Counter monotonicity:**

Counters MUST be monotonically increasing; reset only on process
restart. Use a separate gauge for "current value" semantics.

**Prohibited metric patterns:**

- Free-text label values (e.g., `error_message` as a label).
- Unbounded cardinality labels (e.g., `request_id`, `user_id`
  without rollup).
- Composite labels (e.g., `tenant_and_method=acme:GET`); use
  separate labels.

### D-8. Tail sampling per ADR-0210

Trace emission honours the ADR-0210 sampling policy:

- **Head sampling:** 1% baseline at the per-µservice agent collector
  (configurable to 0.1% / 0.01% per ADR-0210 high-traffic escape
  hatch).
- **Tail sampling:** 100% of error traces, 100% of p99-latency traces,
  100% of new-endpoint traces (30-day warm-up window), 100% of
  SLO-burn-touching traces, 100% of audit-event-emitting traces;
  1% baseline random sample for the rest.

**Implication for emission contract:**

Every µservice MUST emit traces with sufficient context for the
tail-sampling processor to evaluate the policy:

- Root span `status_code` reflects the operation's error/success
  state (so the `status_code=ERROR` policy can match).
- Spans carry `oyatie.slo.touched=<slo_id>` when the operation
  touches an active SLO window (so the `slo_burn` policy can match).
- Spans carry `oyatie.audit.event_id=<id>` when the operation emits
  an audit-chain event (so the `audit_event` policy can match).
- Spans carry `oyatie.endpoint.is_new=true` for endpoints in their
  30-day warm-up window (so the `new_endpoint_warmup` policy can match).
- Spans carry `oyatie.latency.bucket=<bucket>` for fast post-hoc
  filtering of p99 outliers.

Sampling is a property of trace emission, not log/metric emission;
logs and metrics are emitted at full fidelity (subject to cardinality
budgets and PII scrubbing).

### D-9. Per-tenant rollup via Mimir HA + Loki HA + Tempo distributed

The substrate enforces per-tenant data isolation and rollup at the
storage layer:

**Mimir (metrics):**

- Multi-tenancy via `X-Scope-OrgID` header; the OTel collector sets
  this from the emission's `tenant_id` label.
- HA mode: 3 replicas per cell; quorum write; query federation across
  replicas.
- Per-tenant ingestion limits enforced at the distributor (per Mimir
  documentation: `ingestion_rate`, `ingestion_burst_size`,
  `max_series_per_user`).
- Recording rules for per-tenant SLI aggregation; sub-scope rollup
  recording rules pre-compute `sum by (tenant_id) (...)` to keep
  query-time work bounded.

**Loki (logs):**

- Multi-tenancy via the same `X-Scope-OrgID` mechanism.
- HA mode: 3 ingester replicas; ring-based consistent hashing per
  Loki published design.
- Per-tenant log retention follows ADR-0242 §D-4 (per-tenant
  retention rules: hot 7 days, warm 30 days, cold 1 year unless
  legal hold or compliance retention overrides).
- Per-tenant query rate limiting via Loki query-frontend.

**Tempo (traces):**

- Distributed mode: ingester per-cell ring; querier reads via the
  query-frontend.
- Per-tenant trace retention: hot 7 days (Tempo), cold 90 days
  (ClickHouse columnar mirror per IP-022).
- Tempo's per-tenant `max_traces_per_user` enforced at the
  distributor; spills to dead-letter for review.

**ClickHouse (cold/columnar):**

- Per ADR-0153 + IP-021 + IP-022.
- Per-tenant tables (`oya_traces_<tenant_id>` partition pattern) for
  columnar analytics that span longer retention than Tempo's hot
  tier.
- Per-tenant data residency enforced at the ClickHouse cluster
  level (per-cell deployment; per-region replication per ADR-0049).

### D-10. Cross-cell aggregation via control-plane aggregator cell

Per ADR-0009 cell architecture, every cell runs its own observability
stack (Alloy + Mimir + Loki + Tempo + ClickHouse). For platform-wide
operational views (e.g., total `oyatie.foundry.ci-agent` workflow
throughput across all cells; aggregate SLO burn rate for the platform),
a **cross-cell aggregator cell** runs the read-side aggregation:

**Architecture:**

```
[cell-a-mimir] --(remote write, per-tenant scoped)--> [aggregator-mimir]
[cell-b-mimir] --(remote write, per-tenant scoped)--> [aggregator-mimir]
[cell-c-mimir] --(remote write, per-tenant scoped)--> [aggregator-mimir]

[cell-a-loki]  --(log shipper, scoped)--> [aggregator-loki]
[cell-b-loki]  --(log shipper, scoped)--> [aggregator-loki]

[cell-a-tempo] --(trace fanout)--> [aggregator-tempo]
```

**Per-tenant scoping:**

The aggregator cell remains tenant-scoped — the aggregator can read
across cells but only for tenants whose data class allows
cross-cell replication. Sovereign-cloud-restricted tenants (per
ADR-0240 sovereign-cloud-per-regional-pack) have their data class
flag `cross_cell_replication=false`, in which case the cell remains
self-contained and the aggregator simply records "data is restricted
to cell X."

**Replication semantics:**

- Eventual consistency; replication lag ≤60 s p99.
- Replication is per-tenant per-data-class; the OTel-equivalent of
  per-µservice `remote_write` configuration carries the data class
  filter.
- Cross-region replication follows ADR-0049's pack-overlay rules.

**Aggregator-only views:**

- Platform-wide SLO compliance dashboard (per ADR-0139).
- Cross-tenant cost attribution rollups (per ADR-0174 + §D-11 below).
- Fleet-wide capacity planning dashboards.
- Cross-cell incident heat maps (security operations).

**Per-cell vs aggregator query routing:**

- Single-tenant queries go to the home cell.
- Cross-tenant + cross-cell queries go to the aggregator.
- Routing happens in the Grafana datasource configuration; the
  Grafana proxy resolves `tenant_id` → home cell.

### D-11. Per-tenant cost attribution emission

Every observation has a cost (bytes ingested, samples stored, query
CPU consumed, trace storage occupied). Per ADR-0174 (FinOps cost
attribution), every µservice emits **cost-attribution metrics** that
the FinOps portal aggregates per-tenant per-sub-scope.

**Cost-attribution metric pattern:**

```
oya:<microservice>:<bc>:cost_units_total:counter
  labels:
    tenant_id, subscope, cell_id, region,
    resource_type ∈ {llm_tokens, compute_cpu_seconds,
                     compute_gpu_seconds, storage_bytes_seconds,
                     network_egress_bytes, api_call_count,
                     observability_samples_ingested,
                     observability_log_bytes,
                     observability_trace_bytes}
```

Examples:

```
oya:foundry:ci-agent:cost_units_total{
  tenant_id="oyatie", subscope="oyatie.foundry.ci-agent",
  resource_type="llm_tokens"} 1234567
oya:observability:otel-ingest:cost_units_total{
  tenant_id="tenant-acme-corp", subscope="tenant-acme-corp.user-7421",
  resource_type="observability_samples_ingested"} 891234
oya:cloud-data:postgres:cost_units_total{
  tenant_id="tenant-bravo-org", subscope="tenant-bravo-org.service-api",
  resource_type="storage_bytes_seconds"} 5678901
```

**Emission timing:**

- LLM tokens: emitted post-inference, in the same span as the LLM
  call.
- Compute: emitted by the Kubernetes Vertical Pod Autoscaler
  integration; per-pod CPU/memory rolled up by `tenant_id` label.
- Storage: emitted nightly by storage-substrate ratchet; per-
  tenant byte-seconds accrue.
- Network egress: emitted by the egress-proxy (Istio Ambient per
  ADR-0044) per-flow, labelled by source/dest tenant.
- API calls: emitted per call (subject to sampling for
  high-volume).
- Observability: emitted by the OTel collector itself, per-tenant
  ingestion counters.

**Recursive cost:**

Observability emission has a cost itself; that cost is emitted by
the observability stack, attributed back to the tenant whose
emissions caused it. The recursive cost is bounded because emission
of cost metrics does not emit further cost metrics for itself
(`_internal.*` reserved key prefix per §D-6 marks substrate-internal
flows that are not re-attributed).

**Cost-attribution audit:**

Per-tenant cost attribution is itself state-changing (FinOps spend
attributable to a tenant changes their invoice). Cost-attribution
emissions carry `audit_id` per §D-13 so the chain seals each per-
tenant attribution.

### D-12. Exemplars linking metrics to representative traces

The OpenMetrics exposition format supports **exemplars** — single
trace context references attached to a histogram bucket or counter
increment, allowing operators to jump from a dashboard metric to a
representative trace.

**Every histogram MUST emit exemplars:**

- The exemplar carries `trace_id` and `span_id` of a trace that
  produced a measurement in that bucket.
- Exemplar emission frequency: one per bucket per scrape interval
  (60 s default).
- Exemplar selection policy: prefer error traces (when the bucket
  contains them), else prefer slow traces, else random.

**Exemplar format (OpenMetrics):**

```
oya_observability_slo_engine_burn_rate_evaluation_duration_seconds_bucket{
  tenant_id="tenant-acme-corp", subscope="tenant-acme-corp.service-api",
  cell_id="cell-kr-seoul-1", le="0.1"
} 8421 # {trace_id="a3b2c1d4e5f6...", span_id="0a1b2c3d4e..."} 0.087 1716224625.123
```

The trailing `# {...}` syntax is the OpenMetrics exemplar.

**Counter exemplars:**

Counters MAY emit exemplars (OpenMetrics 1.0.0+); when emitted, one
exemplar per scrape interval per counter is sufficient.

**Why exemplars matter:**

Without exemplars, an operator sees a p99 latency dashboard with no
way to investigate the actual slow request. With exemplars, a click
on the dashboard cell opens a representative trace in Tempo. This is
the Three Pillars correlation in action.

### D-13. Audit chain integration: every state-changing emission carries audit_id

Per ADR-0003 (audit-chain + evidence emission), every state-changing
action emits a sealed audit-chain entry. ADR-0263 codifies the
linkage from observability emissions to audit-chain entries:

**Definition of state-changing:**

A state-changing emission is one where the µservice mutates a record
that is durably stored AND tenant-visible. Examples:

- `tenancy::register_tenant` — creates a tenant row.
- `identity::issue_token` — issues an OIDC token.
- `policy-engine::deploy_policy_fragment` — installs new Cedar policy.
- `audit-chain::seal_period` — seals a Merkle period.
- `observability::write_eligibility_verdict` — writes a verdict to
  the ledger.
- `cloud-data::insert_row` — for tenant-visible rows.
- `workflow-engine::complete_workflow_step` — durable step boundary.

Non-state-changing examples (no `audit_id` required):

- Read-only queries.
- Health checks.
- Cache reads.
- Span emissions for internal substrate plumbing.

**Audit ID linkage:**

When a state-changing action commits, the µservice:

1. Generates the audit entry per ADR-0003 (Merkle-sealed; Ed25519
   signed).
2. Receives the audit entry's ID (`audit_id`, format: `EVT-<tenant_id>-
   <period_id>-<sequence>`).
3. Includes the `audit_id` in the corresponding observability emissions:
   - The action's primary log line carries `audit_id` (§D-6).
   - The action's root span carries the attribute
     `oyatie.audit.event_id=<audit_id>`.
   - The corresponding counter increment carries the audit_id as a
     trace exemplar (§D-12).

**Dual-emission ordering:**

The audit-chain emission happens AFTER the state change commits but
BEFORE the corresponding observability log/metric/trace finalises.
The ordering is enforced by the audit-chain client SDK; the µservice
calls `audit_chain.seal_event(...).await?` which returns the
`audit_id`, then uses that ID in the subsequent log/metric/trace
emissions.

**Failure modes:**

- If the audit-chain emission fails, the state change is rolled back
  (per ADR-0003 atomicity invariant).
- If the observability emission fails after the audit-chain has
  sealed, the audit-chain entry remains; the observability emission
  is retried via the local emission buffer (Alloy WAL) and eventually
  surfaces. The `audit_id` is recoverable from the audit-chain query
  API.

**Backfill semantics:**

For backfill replays (per `microservices/observability/backfill-replay.md`),
historical observability emissions are reconstructed from the
audit-chain (which is canonical) plus any surviving raw log streams.
The `audit_id` field is the join key.

**Audit-event-class registry references:**

Audit-event-class taxonomy lives in
`microservices/audit-chain/policy/event-class-registry/` and is
codified by downstream ADRs that originate concrete classes. ADR-0263
is the canonical registry contract; each downstream ADR §D-x
"Observability — emitted audit-event classes" section defines the
schema, retention class, emission targets, and cardinality budget per
class. The following classes are registered by Wave-3-G doctrine ADRs
(reverse-cross-referenced here so the registry contract surfaces all
known classes from a single ADR-0263 anchor):

| Originating ADR | Class count | Registry anchor |
|---|---:|---|
| ADR-0297 (abuse-defence baseline) | 18 | `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md` §D-6.3 |
| ADR-0313 (conglomerate tenant hierarchy) | 6 | `docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md` §D-10 |
| ADR-0319 (front/middle/back office information barrier) | 22 | `docs/decisions/ADR-0319-front-middle-back-office-information-barrier.md` §E-4 |

Registered ADR-0297 classes:
`AbuseDefenceBotBlocked`, `AbuseDefenceSpoofDetected`,
`AbuseDefenceScrapeBlocked`, `AbuseDefenceChallengeIssued`,
`AbuseDefenceChallengeSolved`, `AbuseDefenceChallengeFailed`,
`AbuseDefenceRateLimitHit`, `AbuseDefenceRateLimitFallback`,
`AbuseDefenceHoneypotHit`, `AbuseDefenceCanaryRecovered`,
`AbuseDefenceQuotaExceeded`, `AbuseDefenceCredentialPwned`,
`AbuseDefenceCredentialStuffing`, `AbuseDefenceAttestationFailed`,
`AbuseDefenceVendorOutage`, `AbuseDefenceFragmentActivated`,
`AbuseDefenceWatermarkRecovered`, and `AbuseDefenceSPIREOutage`.

Registered ADR-0313 classes:
`ConglomerateGrantCreated`, `ConglomerateGrantRevoked`,
`ConglomerateParentReadAction`,
`ConglomerateCrossJurisdictionResidencyEnforced`,
`ConglomerateInformationBarrierCrossingRefused`, and
`ConglomeratePersonalTenantBoundaryRefused`.

Registered ADR-0319 classes:
`OfficeScopeAssignmentCreated`, `OfficeScopeAssignmentChanged`,
`OfficeScopeAssignmentRevoked`, `OfficeBoundaryClearanceRequested`,
`OfficeBoundaryClearanceApproved`, `OfficeBoundaryClearanceDenied`,
`OfficeBoundaryClearanceRevoked`, `OfficeBoundaryAttemptEvaluated`,
`OfficeBoundaryAttemptDenied`, `OfficeBoundaryAttemptAllowed`,
`InformationBarrierTaintAttached`, `InformationBarrierTaintDerived`,
`InformationBarrierTaintReleased`, `RestrictedDealCreated`,
`RestrictedDealParticipantAdded`, `RestrictedDealParticipantRemoved`,
`RestrictedDealReleased`, `AdvisorRelationshipBarrierCreated`,
`AdvisorRelationshipBarrierAttemptEvaluated`,
`OfficePackOverlayActivated`, `OfficePackOverlayChanged`, and
`OfficePackOverlayRetired`.

Every registered class inherits ADR-0263's mandatory envelope:
`tenant_id`, `sub_scope_path`, `event_id`, `trace_id`, `span_id`,
`audit_id`, `schema_version`, `source_microservice`, `cell_id`,
`jurisdiction_code`, and PII-scrubbed payload fields only. Classes
produced by a Cedar decision additionally require
`cedar_policy_version`, `policy_pack` or `policy_fragment_id`,
`evaluation_id`, `action`, `resource_ref`, and `decision`. ADR-0319
information-barrier classes additionally require
`jurisdiction_anchor_set` and redacted descriptors or stable
identifiers instead of restricted payload.

Each downstream ADR §D-x section is the canonical authority for the
event-class schema; ADR-0263 enforces the registry contract (schema
validation, retention compliance, cardinality budget adherence). When
a new doctrine ADR introduces additional classes, the ADR MUST (a)
declare them under its §D-x "Observability" subsection per the schema
above, (b) reverse-reference ADR-0263 in the ADR's `related:` or
`related_adrs:` list, and (c) update this table via the same PR. The
`oya gate validate audit-event-class-registered` lane (per the
`enforced_by` list above) verifies that every emitted class surfaces
in this registry.

### D-14. PII scrubbing at the emission boundary

PII (Personally Identifiable Information) is scrubbed **at the
emission boundary**, never at the storage boundary. Reason:
storage-layer scrubbing means PII is in transit and at rest in the
substrate before scrubbing — that's a data breach surface.

**Scrubbing happens in the emission client SDK:**

`oya-shared-observability-client-sdk` exports a `Scrubber` trait
implemented by:

- Tenant-aware field-name allowlist (per-tenant manifest declares
  which fields are PII; default deny).
- Pattern-based scrubbing for common PII shapes (email regex,
  US/EU/KR phone formats, SSN, NIN, RRN, credit card Luhn).
- ML-based scrubbing for unstructured fields (forthcoming ADR
  for the ML scrubber).

**Scrubbing actions:**

- **Redact:** Replace field value with `[REDACTED]`.
- **Hash:** Replace with `sha256(field_value || tenant_salt)`. The
  salt is per-tenant and held in `microservices/cloud-secrets/`.
- **Truncate:** Truncate to a non-identifying prefix (e.g., email
  to domain only).
- **Pseudonymise:** Replace with a per-tenant stable pseudonym
  (reversible only via legal hold + escalated access).

**Mandatory scrubbing in log emissions:**

- Email addresses → hash by default.
- Names → redact by default.
- Phone numbers → truncate to country code + carrier prefix.
- IDs that match SSN/NIN/RRN/passport patterns → redact.
- Credit card numbers (Luhn-passing 13-19 digit strings) → hash.

**Tenant-extensible scrubbing:**

Tenants extend the scrubber via their manifest:

```yaml
# microservices/<tenant>/observability/scrubbing-rules.yaml
scrubbing_rules:
  - field_path: "request.body.customer_email"
    action: hash
  - field_path: "request.body.customer_dob"
    action: redact
  - regex: "^[A-Z]{2}[0-9]{6,8}$"  # passport-shaped
    action: redact
```

**Scrubbing audit:**

Every scrub is itself an emission (`oya:observability:scrubber:
fields_scrubbed_total` counter, labelled by `action` and
`field_path_class`). This makes scrubbing operationally visible —
if scrubbing rates spike, the substrate notices.

**Test fixtures:**

Per the contract, every µservice's test set includes scrubbing
fixtures: known PII payloads enter the emission boundary; assertion
verifies they exit scrubbed. The lane
`oya-check-pii-scrubbing-at-emission` enforces fixture coverage.

### D-15. Schema evolution: additive-only; field deprecation handshake

Schemas evolve **additive-only**:

- New optional fields may be added at any time without bumping the
  schema version.
- New mandatory fields require bumping the schema version (`oyatie/log/v1`
  → `oyatie/log/v2`).
- Existing fields cannot change type or semantics in place.
- Field deprecation requires a multi-stage handshake (below).

**Multi-stage deprecation handshake:**

To deprecate a field (e.g., `tenantId` → `tenant_id` already
happened in early 2026):

1. **Stage 1 — Dual emit (mandatory).** The emitting µservice
   emits BOTH the old field and the new field for at least 90 days.
   The corresponding ADR documents the deprecation timeline.
2. **Stage 2 — Soft warn (consumers).** Substrate consumers
   (dashboards, alerting, ML pipelines) log a warning when they
   see the old field. The warning includes the deprecation ADR ID.
3. **Stage 3 — Hard refuse (substrate).** The OTel collector
   refuses ingestion of emissions that omit the new field. The
   transition is gated by a CI lane (`oya-check-deprecated-fields`).
4. **Stage 4 — Remove from emitters.** Emitters drop the old field.
   The deprecation ADR is marked Accepted.

**Schema registry:**

The schema registry lives at `microservices/observability/policy/
schema-registry/`:

```
schema-registry/
├── log/
│   ├── v1.json      # current
│   └── v2.json      # proposed
├── metric-naming/
│   └── v1.json
├── span-attributes/
│   └── v1.json
└── deprecation-handshakes/
    └── DEPR-001-tenantId-to-tenant_id.json
```

Each schema version is a JSON Schema document; the OTel collector
validates incoming emissions against the current schema.

**Breaking schema changes (rare):**

A truly breaking schema change (e.g., `oyatie/log/v1` → `oyatie/log/v2`
with new mandatory field) requires:

- ADR-class document (forthcoming).
- 180-day sunset window for v1.
- Version-aware substrate that ingests both v1 and v2 during the
  sunset.
- Per-µservice ratchet: each µservice migrates v1 → v2; CI lane
  tracks progress.

## Alternatives considered

### Alt-1. Free-form emission, schema-on-read (Splunk model)

Allow each µservice to emit logs/metrics/traces in any shape;
normalise at query time via Splunk-style schema-on-read (or
ClickHouse with materialised views).

**Pros:**

- Zero per-µservice migration cost.
- Maximum emitter velocity (no contract to violate).
- Familiar to operators coming from Splunk / Elasticsearch / Datadog.

**Cons:**

- **Drift compounds.** PR #143 audit already surfaced 14 instances
  of drift in 14 µservices; no contract means new µservices drift
  freely.
- **Correlation collapses.** Without canonical `tenant_id` and
  `trace_id` field positions, cross-pillar correlation requires
  schema-on-read mappings per µservice; mappings drift.
- **Cardinality unbounded.** Free-form labels produce metric
  cardinality explosions; per-tenant Mimir bills go non-linear.
- **PII surfaces.** Without scrubbing at the emission boundary,
  PII enters the substrate; storage-side scrubbing is a
  post-hoc cleanup.
- **No CI enforcement.** Without a closed contract, validators
  can't validate; emission rot goes undetected.

**Rejected** because every named hyperscaler reference enforces a
canonical emission shape (Google's structured logging spec;
AWS CloudWatch Embedded Metric Format; Azure Monitor structured
logging schema; Stripe's `stripe-logger` SDK). Schema-on-read is a
juvenile-substrate symptom.

### Alt-2. OpenTelemetry-only (no Prometheus, no Loki-direct)

Use OpenTelemetry as the single emission protocol; replace
Prometheus exposition + Loki direct ingest with OTel metrics +
OTel logs.

**Pros:**

- Single SDK; single protocol; single set of semantic conventions.
- Future-proof (OTel logs is now GA as of 2024).
- Reduces the surface area of "what to instrument with."

**Cons:**

- **Prometheus exposition is the de-facto Kubernetes ecosystem
  standard.** Kubernetes ServiceMonitor / PodMonitor / annotations
  presume Prometheus scrape; ripping that out drops K8s ecosystem
  fit.
- **Loki direct ingest via Promtail/Alloy is more efficient** than
  forcing logs through the OTel collector for our log volume.
- **Tooling maturity gap.** OTel logs SDK in Rust is less mature
  than `tracing` + `tracing-opentelemetry` for traces.
- **Cardinality model differs.** OTel metrics has a different
  cardinality model than Prometheus; migrating recording rules
  is non-trivial.

**Rejected** because the existing observability µservice (per
PRD-observability) already builds against Grafana Alloy + Mimir +
Loki + Tempo; an OTel-only re-architecture is a much larger
change for marginal benefit. The hybrid posture (OTel for traces;
Prometheus exposition for metrics; structured JSON to stdout for
logs picked up by Alloy) is the industry-standard Grafana stack
shape.

### Alt-3. Vendor SDK (Datadog, New Relic, Honeycomb)

Adopt a commercial APM vendor's SDK as the emission contract.

**Pros:**

- Turn-key; vendor handles SDK maintenance.
- Best-in-class UX for incident response (Honeycomb's BubbleUp,
  Datadog's APM).
- Less in-house substrate maintenance burden.

**Cons:**

- **Vendor lock-in.** ADR-0173 (in-house tech stack preference)
  + ADR-0211 (in-house Rust-primary) + ADR-0186 (Grafana stack
  self-hosted) all argue against vendor lock-in.
- **Sovereignty.** ADR-0164 (data sovereignty) + ADR-0240
  (sovereign-cloud-per-regional-pack) prohibit unsanctioned data
  egress. SaaS APM vendors are unsanctioned per the current pack
  matrix.
- **Cost.** Datadog/New Relic per-host pricing at fleet scale
  becomes uneconomic; the Grafana stack self-hosted is ~10× cheaper
  at our projected scale (per ADR-0210 cost model).
- **Differentiation.** ADR-0211 doctrine is to build the substrate
  ourselves where the substrate is differentiated; observability
  *is* differentiated for an agentic-development platform (per
  PRD-observability §"Competitive Benchmark").

**Rejected** because every other ADR in the in-house-substrate
family already rejects vendor lock-in for substrate components;
ADR-0263 inherits the doctrine.

### Alt-4. Per-µservice emission contract (each µservice authors its own)

Allow each µservice to author its own emission contract; require
only that the substrate ingests "something" per µservice.

**Pros:**

- Maximum per-µservice flexibility.
- No central coordination cost.

**Cons:**

- **Identical to Alt-1 schema-on-read; same drift problems.**
- **Per-µservice contracts cannot be validated cross-µservice;
  correlation requires platform-wide invariants.**
- **Doc burden 32× higher** (one contract per µservice instead of
  one).

**Rejected** for the same reasons as Alt-1.

### Alt-5. Tier-1 lockdown contract with deferred PII handling

Adopt this ADR but defer §D-14 PII scrubbing to a future ADR.

**Pros:**

- Smaller initial scope.
- Faster to land.

**Cons:**

- **PII enters the substrate before scrubbing.** Once data is in
  Loki/Tempo/ClickHouse, removing it requires backfill replay +
  storage-layer redaction — that's an expensive, error-prone
  process at scale.
- **DSAR cascade gap.** ADR-0242 §D-4 + §Appendix B presume PII
  is *not* in the observability substrate by default. Deferring
  scrubbing means DSAR cascade must include observability backfill
  — a significant scope creep.
- **GDPR Article 25 (Data Protection by Design)** is a regulatory
  requirement, not a feature. Scrubbing at emission boundary is the
  Privacy-by-Design pattern.

**Rejected** because deferring §D-14 creates a regulatory + DSAR
cascade liability that's harder to retrofit than to bake in. The
incremental scope of §D-14 is bounded; the deferred-retrofit cost
is unbounded.

### Alt-6. Full lockdown contract as specified (CHOSEN)

Adopt §Decision in full: all 15 decisions land as one ADR + one
client SDK + one validator suite + one schema registry. Promotion
from advisory to BLOCKER is per §Verification.

**Pros:**

- **Closes correlation gap.** Three Pillars unify on `trace_id` +
  `tenant_id` + `audit_id`.
- **Closes PII gap.** Scrubbing at emission boundary; DSAR cascade
  uniform.
- **Closes cost gap.** Per-tenant cost attribution mandatory; FinOps
  portal pencil sharpens.
- **Closes audit gap.** Audit-chain linkage mandatory; forensics
  uniform.
- **Closes drift gap.** Validator suite + CI lanes + schema registry
  prevent emission rot.
- **Closes vendor-lock gap.** Pure OSS stack + W3C standard
  propagation; portable.
- **Matches every named hyperscaler reference** (Google structured
  logging; AWS CloudWatch EMF; Azure Monitor schema; Stripe/Honeycomb
  emission patterns; Palantir Foundry observability).
- **Enables agentic pipeline determinism** — every emission carries
  the labels the agentic pipeline needs to evaluate per-tenant
  per-µservice change impact.

**Cons:**

- **One-time migration cost** for the 14 µservices already in
  flight + every greenfield µservice (~32 µservices total at M01).
  Bounded; sequenced per the promotion schedule in §Verification.
- **Client SDK is a single point of dependency.** Mitigation:
  ADR-0211 in-house Rust SDK with thorough test coverage; SDK
  version pinning per workspace `Cargo.toml`.
- **Cardinality discipline required.** Operators must understand
  cardinality budgets; training material at
  `docs/standards/observability-emission-contract.md` (Turn 3
  deliverable).

**Accepted** as the foundational Tier-1 lockdown. The cons are
bounded one-time costs; the pros include closing every named gap
in the prior observability ADRs.

## Consequences

### Positive

1. **Correlation across the Three Pillars.** Operators can pivot
   from a metric dashboard to a representative log line to a
   representative trace to the corresponding audit entry — all via
   the canonical correlation IDs (`trace_id`, `span_id`, `tenant_id`,
   `audit_id`).
2. **Per-tenant operational view.** Every emission carries
   `tenant_id`; every dashboard, query, and alert can filter by
   tenant; per-tenant cost, per-tenant SLO, per-tenant incident
   counts are pre-computed.
3. **`oyatie`-as-tenant operationalised.** Per ADR-0242, `oyatie`
   is a tenant; the emission contract makes this concrete —
   `oyatie.foundry.ci-agent` workflows appear in the same Mimir
   queries and Loki searches as customer-tenant principals.
4. **Agentic pipeline determinism.** Foundry pipeline runs can
   query the substrate for "did this PR change observability metrics
   for the affected µservices in a measurable way?" — the
   determinism unblocks the autonomous-masterplan goal.
5. **Privacy by Design.** PII scrubbing at emission boundary
   satisfies GDPR Article 25; DSAR cascade per ADR-0242 §D-4 is
   simplified because observability substrate is PII-free by
   construction.
6. **Hyperscaler-shape achieved.** Matches Google structured
   logging spec; AWS CloudWatch EMF; Azure Monitor schema; Stripe's
   `stripe-logger`. Closes the
   `feedback_quality_performance_scalability_bar` requirement for
   the observability axis.
7. **Schema evolution discipline.** Additive-only + multi-stage
   deprecation handshake prevent silent emission rot per
   `feedback_no_silent_regression`.
8. **CI-enforced.** Seven new lanes (per §Verification) move from
   advisory to BLOCKER at the promotion checkpoint; emission drift
   is detected at PR time, not at production-incident time.

### Negative

1. **One-time migration of 14 µservices** already in flight.
   Bounded; sequenced per §Verification promotion checklist.
2. **Client SDK as central dependency.** Mitigation: per-layer
   Rust crates (kernel/domain/application/api/adapter/sdk/app) with
   thorough test coverage; ADR-0211 in-house tech stack discipline;
   version pinning at the workspace root.
3. **Cardinality budget discipline.** Operators authoring new
   metrics must respect cardinality budgets; training cost.
   Mitigation: `docs/standards/observability-emission-contract.md`
   includes a "cardinality 101" section; cardinality lints in the
   client SDK (compile-time `const`-checked where possible).
4. **Schema registry overhead.** Maintaining JSON Schema documents
   + deprecation handshakes is non-zero work. Mitigation:
   `microservices/observability/policy/schema-registry/` is a
   thin discipline; registry changes are themselves audited via
   ADR or smaller-class governance docs.
5. **PII scrubbing latency.** Emission boundary scrubbing adds
   ~50-200 μs per emission. Mitigation: scrubbing is per-field
   bounded; budgets tracked under `oya:observability:scrubber:
   scrub_duration_seconds:histogram`; high-volume emitters can
   sample scrubbing (not the emission itself) at the audit-chain
   sub-scope's discretion.

### Operational

1. **New CI lanes (advisory; promote to BLOCKER per §Verification):**
   - `oya-check-observability-emission-contract` — top-level lane.
   - `oya-check-tenant-label-presence` — per metric/log/span.
   - `oya-check-trace-context-propagation` — W3C compliance.
   - `oya-check-metric-naming-convention` — §D-7 conformance.
   - `oya-check-log-schema-conformance` — §D-6 conformance.
   - `oya-check-pii-scrubbing-at-emission` — fixture coverage.
   - `oya-check-audit-id-on-state-change` — §D-13 linkage.
2. **Client SDK crate set** added to the workspace:
   `oya-shared-observability-client-{kernel,domain,application,api,adapter,sdk,app}`.
3. **Schema registry** at
   `microservices/observability/policy/schema-registry/`.
4. **Per-µservice manifest fields:**
   - `observability.emission_contract_version = "v1"`.
   - `observability.metric_subscope_policy ∈ {full, rollup-to-parent, omit}`.
   - `observability.pii_scrubbing_rules = "scrubbing-rules.yaml"` (path
     relative to µservice folder).
5. **Standards doc:**
   `docs/standards/observability-emission-contract.md` — the
   developer-facing tutorial covering all 15 decisions, with worked
   examples.
6. **Validator suite** at
   `microservices/observability/policy/contract-validators/` — Rust
   crate per validator, invoked by `oya gate validate
   observability-emission-contract`.
7. **Grafana dashboards** added for per-tenant per-sub-scope
   substrate health, cost attribution rollups, PII scrubbing rates,
   contract compliance scoreboard.

### Sustainability

- Per-tenant observability cost attribution (§D-11) surfaces the
  observability substrate's own carbon footprint per-tenant. The
  FinOps portal's sustainability tag (per ADR-0174) extends to
  observability cost; tenants who emit excessive logs/metrics/traces
  see the sustainability impact in their dashboard.
- Schema evolution discipline reduces re-emission backfill cost
  (and its carbon footprint) by preventing breaking changes that
  would force fleet-wide replay.

### Compliance

- **GDPR Article 25 (Data Protection by Design):** PII scrubbing at
  emission boundary is the canonical Privacy-by-Design pattern.
- **GDPR Article 17 (Right to Erasure):** Per ADR-0242 §D-4 + §Appendix
  B, the DSAR cascade reaches into observability substrate only via
  pseudonymised IDs; full-PII never enters. Erasure is faster and
  more reliable.
- **KR PIPA Article 36 (Information Subject's Rights):** Same as GDPR
  Article 17.
- **EU AI Act high-risk classification (Article 17):** Observability
  emissions for AI workloads include `oyatie.ai.tier=<tier>` attribute
  per the forthcoming AI tier ADR; per-tier sampling rules apply.
- **SOC 2 Type II — CC7.2 (System Monitoring):** Emission contract is
  the system-monitoring control; CI-enforced compliance is the
  auditable evidence.
- **ISO 27001 A.12.4 (Logging and Monitoring):** Structured JSON +
  audit-chain linkage is the canonical operating model.
- **PCI DSS 10 (Logging and Monitoring):** For tenants processing
  payment data, the emission contract's scrubbing rules redact PAN
  (Primary Account Number) automatically.

## Implementation surface

The following artifacts are required for this keystone to be
considered implemented:

| Artifact | Status |
|---|---|
| `/specs/microservices/observability.json` § emission-contract section | NEW — derived from §D |
| `/specs/microservices/manifest-schema.json` § `observability.emission_contract_version` field | NEW |
| `crates/oya-shared-observability-client-kernel/` (Rust crate) | NEW |
| `crates/oya-shared-observability-client-domain/` | NEW |
| `crates/oya-shared-observability-client-application/` | NEW (per ADR-0105 13-layer canonical enum; replaces the legacy `usecase` suffix per ADR-0106 rename) |
| `crates/oya-shared-observability-client-api/` | NEW |
| `crates/oya-shared-observability-client-adapter/` | NEW |
| `crates/oya-shared-observability-client-adapter-otlp/` | NEW (backend-qualified adapter per ADR-0105 Amendment 3) |
| `crates/oya-shared-observability-client-adapter-prometheus/` | NEW (backend-qualified adapter) |
| `crates/oya-shared-observability-client-adapter-loki/` | NEW (backend-qualified adapter) |
| `crates/oya-shared-observability-client-sdk/` | NEW |
| `crates/oya-shared-observability-client-app/` | NEW |
| `microservices/observability/policy/schema-registry/log/v1.json` | NEW |
| `microservices/observability/policy/schema-registry/metric-naming/v1.json` | NEW |
| `microservices/observability/policy/schema-registry/span-attributes/v1.json` | NEW |
| `microservices/observability/policy/contract-validators/` (validator crates) | NEW |
| Seven new CI lanes (see §Operational) | NEW |
| `docs/standards/observability-emission-contract.md` (developer tutorial) | NEW |
| `microservices/observability/runbooks/emission-contract-migration.md` | NEW |
| Migration of 14 in-flight µservices to contract conformance | SWEEP (per §Verification schedule) |
| Removal of legacy B3 trace context (1 µservice) | SWEEP |
| `microservices/observability/dashboards/contract-compliance-scoreboard.json` | NEW |
| `microservices/observability/dashboards/per-tenant-cost-attribution.json` | NEW |
| `microservices/observability/dashboards/pii-scrubbing-rates.json` | NEW |

## Verification

### Functional verification

- [ ] `cargo build -p oya-shared-observability-client-sdk` succeeds.
- [ ] `cargo test --workspace -- observability_contract` exits 0.
- [ ] `oya gate validate observability-emission-contract --microservice <m>` exits 0 for every M01-foundation µservice post-migration.
- [ ] `oya gate validate tenant-label-presence --microservice <m>` exits 0.
- [ ] `oya gate validate trace-context-propagation --microservice <m>` exits 0.
- [ ] `oya gate validate metric-naming-convention --microservice <m>` exits 0.
- [ ] `oya gate validate log-schema-conformance --microservice <m>` exits 0.
- [ ] `oya gate validate pii-scrubbing-at-emission --microservice <m>` exits 0.
- [ ] `oya gate validate audit-id-on-state-change --microservice <m>` exits 0.

### Schema verification

- [ ] `microservices/observability/policy/schema-registry/log/v1.json` validates against JSON Schema 2020-12 meta-schema.
- [ ] A test log payload conforming to §D-6 validates against `log/v1.json`.
- [ ] A test log payload missing `tenant_id` is rejected by `log/v1.json` validation.
- [ ] A test log payload with extraneous PII fails the scrubber's regression test.

### Correlation verification

- [ ] A single test request flowing through 3 µservices (tenancy → identity → policy-engine) produces 3 spans with a shared `trace_id`, 3 log lines with the same `trace_id`+`span_id` linkage to each span, and at least 1 exemplar in the corresponding histograms.
- [ ] The 3-µservice test request emits an audit-chain event; the `audit_id` appears in the log line of the state-changing step.

### Cardinality verification

- [ ] Mimir cardinality report for the contract-compliance scoreboard dashboard's metrics shows ≤10,000 series per µservice per cell.
- [ ] No metric series carries unbounded labels (e.g., free-text error messages, raw request IDs).

### PII scrubbing verification

- [ ] PII fixture suite (email, name, phone, SSN, credit card) feeds into a test µservice; the resulting emissions show all PII redacted/hashed/pseudonymised.
- [ ] Scrubber's per-action metrics (`oya:observability:scrubber:fields_scrubbed_total`) increment on the fixture run.

### Promotion checklist (advisory → BLOCKER)

The contract promotes from advisory to BLOCKER once:

1. [ ] Client SDK crate set published to the workspace; semver-stable.
2. [ ] Foundation µservices (`tenancy`, `identity`, `policy-engine`, `audit-chain`, `observability`) migrated; all CI lanes green.
3. [ ] At least 14 of the 32 M01-foundation µservices migrated; CI lanes green for those.
4. [ ] Schema registry initialised; `log/v1.json`, `metric-naming/v1.json`, `span-attributes/v1.json` accepted.
5. [ ] Migration runbook landed; on-call runbook for emission-contract incident response landed.
6. [ ] Standards doc landed and reviewed.
7. [ ] Per-tenant cost attribution dashboard live; cost data flowing.
8. [ ] PII scrubbing fixture suite landed; lane green on every covered µservice.
9. [ ] Audit-chain linkage verified for state-changing operations on the 5 foundation µservices.
10. [ ] Cross-cell aggregator cell deployed in the dev region; replication validated.

Once items 1-10 are met, the CI lanes promote to BLOCKER via an
amendment ADR (or a manifest field flip if the change is purely
configuration).

### Post-promotion verification

- [ ] All M01-foundation µservices conform to the contract; no advisory exemptions remain.
- [ ] Quarterly schema-evolution review surfaces any drift; deprecations follow §D-15 handshake.
- [ ] Annual recompute of cardinality budgets; budget adjustments per growth.

## References

### Industry sources

- **OpenTelemetry Specification (2024).** https://opentelemetry.io/docs/specs/otel/ — Canonical OTel signal definitions, semantic conventions, propagation rules. CNCF Graduated as of 2023 (project status), 2024 (specification stability).
- **OpenTelemetry Semantic Conventions (2024).** https://opentelemetry.io/docs/specs/semconv/ — Attribute naming for span attributes, log records, resource detection.
- **W3C Trace Context (W3C Recommendation, 2020).** https://www.w3.org/TR/trace-context/ — `traceparent` and `tracestate` HTTP header format.
- **W3C Baggage (W3C Recommendation, 2022).** https://www.w3.org/TR/baggage/ — Companion to Trace Context for propagating user-defined key-value pairs.
- **RFC 9384 — Trace Context Propagation (2023).** Codifies the W3C Trace Context model as an IETF RFC.
- **OpenMetrics Specification 1.0.0 (2020).** https://openmetrics.io/ — Exposition format including exemplars.
- **Prometheus Exposition Format (2014, current).** https://prometheus.io/docs/instrumenting/exposition_formats/ — De-facto metrics standard.
- **Charity Majors, "Observability — A 3-Year Retrospective" (Honeycomb blog, 2017).** Articulates the Three Pillars formulation as distinct from "monitoring."
- **Cindy Sridharan, *Distributed Systems Observability* (O'Reilly, 2018).** Book-length treatment of the Three Pillars + correlation.
- **Charity Majors, Liz Fong-Jones, George Miranda, *Observability Engineering* (O'Reilly, 2022).** The canonical text.
- **Honeycomb 3 Pillars discussion (Honeycomb blog series, 2017-2024).** Multiple articles refining the Three Pillars + critique (Charity Majors later argues for "Observability 2.0" as a single high-cardinality event stream; ADR-0263 acknowledges this but retains the Three Pillars formulation for substrate-engineering tractability).
- **Google Dapper paper (Sigelman et al., Google Technical Report, 2010).** https://research.google/pubs/pub36356/ — The foundational distributed tracing paper.
- **Google "Site Reliability Engineering" book (Beyer et al., 2016).** Chapter 12 (Effective Troubleshooting), Chapter 5 (Eliminating Toil).
- **Google "Site Reliability Workbook" (Beyer et al., 2018).** Chapter 4 (SLO Engineering).
- **CNCF Observability Whitepaper (2024).** https://github.com/cncf/tag-observability/blob/main/whitepaper.md — Industry-wide alignment on Three Pillars + correlation.
- **Brendan Gregg, *Systems Performance: Enterprise and the Cloud* (2nd ed., 2020).** Authoritative on metrics + profiling instrumentation patterns.
- **Cloudflare Engineering, "Pingora architecture" (2022).** Demonstrates the in-house OTel approach at scale.
- **Stripe Engineering, "How Stripe instruments Stripe" (Brandur Leach blog series, 2014-2018).** Per-tenant instrumentation patterns at a hyperscaler.
- **Grafana Labs, "How to use exemplars" (2021).** Worked exemplar emission examples in Prometheus + Mimir.
- **AWS CloudWatch Embedded Metric Format (EMF) (2019).** AWS's structured log + metric unification standard; informs §D-6's structured-log shape.
- **Microsoft Azure Monitor schema documentation (2024).** Schema for Application Insights, Log Analytics.
- **Google Cloud Logging structured-log spec (2024).** Field names + reserved keys.
- **Salesforce Trailhead "Multi-tenant observability" (2024).** Per-tenant label propagation patterns at scale.

### Regulatory sources

- **GDPR Article 17 (Right to Erasure).** Observability substrate must be PII-free by construction to make erasure tractable.
- **GDPR Article 25 (Data Protection by Design and by Default).** Mandates Privacy-by-Design; PII scrubbing at emission boundary is the canonical pattern.
- **KR PIPA Article 36 (Information Subject's Rights).** Erasure right equivalent.
- **HIPAA 164.312(b) — Audit Controls.** Audit-chain linkage (§D-13) satisfies.
- **SOX Section 404 — Internal Controls.** Audit-chain linkage for state-changing operations.
- **SOC 2 Type II Trust Service Criteria CC7.2 — System Monitoring.** Emission contract is the system-monitoring control.
- **ISO 27001:2022 A.12.4 — Logging and Monitoring.** Structured JSON + correlation is the operating model.
- **PCI DSS v4.0 Requirement 10 — Logging and Monitoring.** Per-field scrubbing of PAN.
- **EU AI Act Article 17 — High-Risk AI Systems Logging.** AI workload sampling per the forthcoming AI tier ADR.

### Internal portfolio ADRs

- **ADR-0003 — Audit-chain and evidence emission.** Audit ID linkage per §D-13.
- **ADR-0005 — Eventing backbone outbox pattern.** Trace context propagation via outbox.
- **ADR-0009 — Cell architecture per-tenant per-region.** Cross-cell aggregation per §D-10.
- **ADR-0010 — Regional pack architecture.** Per-region observability stack.
- **ADR-0028 — Cloud microservice architecture.** Inter-µservice gRPC.
- **ADR-0034 — Per-microservice data class overrides.** Data-class-aware scrubbing per §D-14.
- **ADR-0042 — Observability stack OTel + in-house UI.** Stack selection antecedent.
- **ADR-0044 — Service mesh Istio Ambient + Envoy Gateway.** Egress proxy emits cost attribution per §D-11.
- **ADR-0049 — Cross-region replication and residency.** Cross-region observability replication.
- **ADR-0098 — LTS rotation cadence.** OpenTelemetry version pin rotation.
- **ADR-0105 — Thirteen-layer canonical enum.** `layer` field in log schema per §D-6.
- **ADR-0131 — Per-microservice flat layout.** Manifest field locations.
- **ADR-0132 — No-grouping forward policy.** Single observability µservice; no bundled suite.
- **ADR-0139 — Burn-rate SLO alerting.** Trace `oyatie.slo.touched` attribute per §D-8.
- **ADR-0145 — Inter-microservice communication reform.** W3C Trace Context propagation invariant; direct gRPC pattern.
- **ADR-0153 — Observability backplane high-level reference.** Antecedent.
- **ADR-0173 — In-house tech stack preference.** Argues against vendor APM.
- **ADR-0174 — FinOps cost attribution.** Per-tenant cost attribution per §D-11.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Cedar policy IDs in `denied_reason` field.
- **ADR-0186 — Observability backplane layering.** Stage-1-through-Stage-5 layering antecedent.
- **ADR-0210 — OTel tail-sampling: 100% error/p99/new-endpoint + 1% baseline.** Sampling policy per §D-8.
- **ADR-0211 — In-house Rust-primary tech stack.** Mandates `opentelemetry-rust` SDK choice.
- **ADR-0240 — Sovereign cloud per regional pack.** Cross-cell replication respects data class.
- **ADR-0242 — `oyatie`-is-a-tenant doctrine.** Unifies `tenant_id` for `oyatie.*` principals.
- **ADR-0243 — Cedar as universal gate.** Cedar decisions emit per the contract.
- **ADR-0244 — Tenant as universal scoping primitive.** `tenant_id` is universally applicable.

### Auto-memory feedback

- `feedback_quality_performance_scalability_bar` — reinforced; hyperscaler-grade observability.
- `feedback_clean_architecture_requirements` — applied; per-layer SDK crate set; cross-product refusal honoured (observability is shared substrate, not a product).
- `feedback_autonomous_implementation_artifacts` — reinforced; emission contract enables agentic-pipeline change-impact analysis.
- `feedback_no_silent_regression` — applied; schema evolution discipline (§D-15) prevents silent emission rot.
- `feedback_oyatie_is_a_tenant_doctrine` — applied; `tenant_id=oyatie` for internal `oyatie.*` principals.
- `feedback_doc_coverage_enforced` — applied; standards doc + runbook + dashboards required.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in the keystone ADRs (ADR-0242
Appendix A onwards), every architectural decision in this ADR is
attributed to a named hyperscaler pattern + source + anti-pattern
avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (Three Pillars) | "Three Pillars of Observability" | Charity Majors 2017; Cindy Sridharan 2018; CNCF Observability Whitepaper 2024 | "Monitoring-only stack" — metrics dashboards without traces or structured logs |
| D-2 (OpenTelemetry SDK mandatory) | "CNCF-Graduated Single SDK Substrate" | OpenTelemetry Specification 2024; Google's internal Census/OpenCensus → OTel convergence | "Vendor SDK lock-in" — Datadog/New Relic/Honeycomb proprietary SDK |
| D-3 (W3C Trace Context) | "W3C Standard Propagation" | W3C Trace Context Recommendation 2020; RFC 9384 (2023) | "Vendor-specific propagation" — B3, Jaeger native, x-datadog-* |
| D-4 (mandatory tenant_id label) | "Multi-Tenant Substrate Per-Tenant Isolation" | Mimir `X-Scope-OrgID` design; AWS CloudWatch per-account isolation; Salesforce multi-tenant schema | "Single-tenant blind substrate" — cardinality explosion + no per-tenant cost story |
| D-5 (sub-scope label) | "Hierarchical Principal Path Propagation" | AWS IAM principal ARN propagation; GCP IAM resource hierarchy in audit logs | "Flat principal namespace" — loss of inheritance + rollup |
| D-6 (structured JSON log schema) | "Structured Logging at Source" | Google Cloud Logging structured-log spec; AWS CloudWatch EMF; Stripe stripe-logger | "Free-form log strings" — schema-on-read drift; correlation failures |
| D-7 (metric naming convention) | "Predictable Metric Namespace" | Prometheus naming guidelines; OpenMetrics 1.0.0; Google's internal metric naming (referenced in SRE book Ch. 5) | "Ad-hoc metric naming" — duplicate metrics; dashboard rot |
| D-8 (tail sampling per ADR-0210) | "Dapper-Style Tail Sampling" | Google Dapper 2010; Honeycomb dynamic sampling 2018 | "Always-on 100%" (unaffordable); "Pure head 1%" (blind to errors) |
| D-9 (per-tenant rollup) | "Substrate-Layer Tenant Isolation" | Mimir HA + multi-tenancy; Loki multi-tenancy; Tempo per-tenant indexing | "Application-layer-only isolation" — substrate cardinality + cost cross-contaminate |
| D-10 (cross-cell aggregation) | "Cell-Replicated Read-Only Aggregator" | Amazon shape cellular architecture (per ADR-0248); Google Borg cross-cell aggregation | "Single global cluster" — blast radius cross-cell |
| D-11 (per-tenant cost attribution emission) | "Showback / Chargeback at Source" | Stripe billing instrumentation; AWS Cost Explorer per-resource tags; Google Cloud Billing labels | "Storage-layer cost reconstruction" — lossy; lagging; non-auditable |
| D-12 (exemplars) | "Metric-to-Trace Pivot" | Grafana exemplars 2021; OpenMetrics 1.0.0 spec; Google Dapper sampling-trace-via-metric pattern | "Siloed metrics" — no pivot to representative trace; long forensic loops |
| D-13 (audit chain integration) | "Audit-Linked Observability" | Stripe audit-event linkage; Google internal "Justification chains"; Palantir Foundry audit-chain integration | "Audit-isolated logs" — observability cannot prove what audit shows |
| D-14 (PII scrubbing at emission boundary) | "Privacy by Design at Source" | GDPR Article 25 mandate; Apple Privacy Differential Privacy patterns; Honeycomb's "Don't log PII" doctrine | "Storage-layer scrubbing" — PII at rest in substrate before scrub; breach surface |
| D-15 (schema evolution: additive-only; deprecation handshake) | "Schema Registry with Compatibility Modes" | Confluent Schema Registry compatibility levels (BACKWARD, FORWARD, FULL); Protocol Buffers wire-compatibility rules | "Breaking schema change" — fleet-wide replay; silent emission rot |

---

## Appendix B: Worked example — a single tenant request flowing through three µservices

To illustrate that the contract is genuinely operational (not just
documented), here is a worked example showing all 15 decisions
landing on a single request.

**Scenario:** A user of `tenant-acme-corp` (sub-scope
`tenant-acme-corp.user-7421`) issues an HTTP POST to register a new
ontology entity. The request flows through:

1. **Entry: `cloud-iam` (HTTP boundary).** Resolves the OIDC token,
   sets `tenant_id=tenant-acme-corp`, `subscope=tenant-acme-corp.user-7421`,
   `principal=oidc:user-7421@acme-corp`.
2. **Authorisation: `policy-engine`.** Cedar evaluates the action;
   permits.
3. **Action: `ontology`.** Creates the entity row; emits audit
   event.

### Emissions on the entry µservice (`cloud-iam`)

**HTTP request received** — `cloud-iam-rest` handler. The handler
generates a new `trace_id` (since no `traceparent` arrived from the
public internet; per ADR-0145 §"Public-internet boundary rule"). It
opens a root span:

```
span: cloud_iam.handle_create_entity_request
trace_id: 4f1c9e6b8d2a7f3e5c8b1a9d4e2c7f3b
span_id: a3b7f1c9e6b8d2a7
status: OK
attributes:
  oyatie.tenant.id: "tenant-acme-corp"
  oyatie.tenant.subscope: "tenant-acme-corp.user-7421"
  oyatie.principal: "oidc:user-7421@acme-corp"
  oyatie.action: "create_entity"
  oyatie.cell.id: "cell-kr-seoul-1"
  oyatie.region: "kr"
  http.method: "POST"
  http.url: "/api/v1/ontology/entities"
  http.status_code: 200
  oyatie.endpoint.is_new: false
  oyatie.latency.bucket: "p50"
```

**Structured log line** — emitted at INFO level on successful
handler entry:

```json
{
  "schema": "oyatie/log/v1",
  "timestamp": "2026-05-20T14:23:45.123456789Z",
  "level": "INFO",
  "message": "received create_entity request",
  "microservice": "cloud-iam",
  "bc": "oidc",
  "layer": "rest",
  "trace_id": "4f1c9e6b8d2a7f3e5c8b1a9d4e2c7f3b",
  "span_id": "a3b7f1c9e6b8d2a7",
  "tenant_id": "tenant-acme-corp",
  "subscope": "tenant-acme-corp.user-7421",
  "principal": "oidc:user-7421@acme-corp",
  "action": "create_entity",
  "cell_id": "cell-kr-seoul-1",
  "region": "kr",
  "request_method": "POST",
  "request_path": "/api/v1/ontology/entities",
  "outcome": "started"
}
```

**Metric increment** — counter:

```
oya:cloud-iam:oidc:requests_received_total{
  tenant_id="tenant-acme-corp",
  subscope="tenant-acme-corp.user-7421",
  cell_id="cell-kr-seoul-1",
  region="kr",
  microservice="cloud-iam",
  bc="oidc",
  version="0.32.4"
} 1
# {trace_id="4f1c9e6b8d2a7f3e5c8b1a9d4e2c7f3b",
#  span_id="a3b7f1c9e6b8d2a7"} 1 1716224625.123
```

### Emissions on the `policy-engine` µservice

`cloud-iam` makes a gRPC call to `policy-engine.evaluate(...)`,
propagating `traceparent` in gRPC metadata. The
`policy-engine-rest` handler opens a child span:

```
span: policy_engine.evaluate
trace_id: 4f1c9e6b8d2a7f3e5c8b1a9d4e2c7f3b
parent_span_id: a3b7f1c9e6b8d2a7
span_id: b4c8e2f0d5a6b8e1
status: OK
attributes:
  oyatie.tenant.id: "tenant-acme-corp"
  oyatie.tenant.subscope: "tenant-acme-corp.user-7421"
  oyatie.principal: "oidc:user-7421@acme-corp"
  oyatie.action: "evaluate"
  oyatie.cedar.policy_id: "tenant-acme-corp:ontology-rw-permit"
  oyatie.cedar.decision: "permit"
  oyatie.cell.id: "cell-kr-seoul-1"
  oyatie.region: "kr"
```

**Structured log line:**

```json
{
  "schema": "oyatie/log/v1",
  "timestamp": "2026-05-20T14:23:45.124567890Z",
  "level": "INFO",
  "message": "cedar policy evaluated",
  "microservice": "policy-engine",
  "bc": "gate",
  "layer": "application",
  "trace_id": "4f1c9e6b8d2a7f3e5c8b1a9d4e2c7f3b",
  "span_id": "b4c8e2f0d5a6b8e1",
  "tenant_id": "tenant-acme-corp",
  "subscope": "tenant-acme-corp.user-7421",
  "principal": "oidc:user-7421@acme-corp",
  "action": "evaluate",
  "cell_id": "cell-kr-seoul-1",
  "region": "kr",
  "resource_type": "OntologyEntity",
  "outcome": "permit"
}
```

**Metric:**

```
oya:policy-engine:gate:policy_decisions_total{
  tenant_id="tenant-acme-corp",
  subscope="tenant-acme-corp.user-7421",
  outcome="permit",
  cell_id="cell-kr-seoul-1",
  region="kr",
  microservice="policy-engine",
  bc="gate",
  version="0.41.2"
} 1
```

### Emissions on the `ontology` µservice (state-changing)

`cloud-iam` makes a gRPC call to `ontology.create_entity(...)`. The
ontology handler:

1. Validates input.
2. Inserts the row into the tenant's ontology Postgres database.
3. Calls `audit_chain.seal_event(...)` and receives `audit_id =
   "EVT-tenant-acme-corp-2026-05-20-period-23-seq-89421"`.
4. Emits the corresponding log/metric/trace with that `audit_id`.

**Span (state-changing — note `oyatie.audit.event_id` attribute):**

```
span: ontology.create_entity
trace_id: 4f1c9e6b8d2a7f3e5c8b1a9d4e2c7f3b
parent_span_id: a3b7f1c9e6b8d2a7
span_id: c5d9f3a1e6b7c9f2
status: OK
attributes:
  oyatie.tenant.id: "tenant-acme-corp"
  oyatie.tenant.subscope: "tenant-acme-corp.user-7421"
  oyatie.principal: "oidc:user-7421@acme-corp"
  oyatie.action: "create_entity"
  oyatie.resource.type: "OntologyEntity"
  oyatie.resource.id: "ent-9183afb2"
  oyatie.audit.event_id: "EVT-tenant-acme-corp-2026-05-20-period-23-seq-89421"
  oyatie.cell.id: "cell-kr-seoul-1"
  oyatie.region: "kr"
  db.system: "postgresql"
  db.statement.digest: "<sha256-of-prepared-statement>"
```

**Structured log line (carries audit_id):**

```json
{
  "schema": "oyatie/log/v1",
  "timestamp": "2026-05-20T14:23:45.131234567Z",
  "level": "INFO",
  "message": "ontology entity created",
  "microservice": "ontology",
  "bc": "entity-store",
  "layer": "application",
  "trace_id": "4f1c9e6b8d2a7f3e5c8b1a9d4e2c7f3b",
  "span_id": "c5d9f3a1e6b7c9f2",
  "tenant_id": "tenant-acme-corp",
  "subscope": "tenant-acme-corp.user-7421",
  "principal": "oidc:user-7421@acme-corp",
  "action": "create_entity",
  "audit_id": "EVT-tenant-acme-corp-2026-05-20-period-23-seq-89421",
  "cell_id": "cell-kr-seoul-1",
  "region": "kr",
  "resource_type": "OntologyEntity",
  "resource_id": "ent-9183afb2",
  "outcome": "success"
}
```

**Metric (with exemplar to the slow trace if applicable):**

```
oya:ontology:entity-store:entities_created_total{
  tenant_id="tenant-acme-corp",
  subscope="tenant-acme-corp.user-7421",
  resource_type="OntologyEntity",
  cell_id="cell-kr-seoul-1",
  region="kr",
  microservice="ontology",
  bc="entity-store",
  version="0.28.7"
} 1
# {trace_id="4f1c9e6b8d2a7f3e5c8b1a9d4e2c7f3b",
#  span_id="c5d9f3a1e6b7c9f2"} 1 1716224625.131
```

**Cost-attribution metric** — emitted post-handler:

```
oya:ontology:entity-store:cost_units_total{
  tenant_id="tenant-acme-corp",
  subscope="tenant-acme-corp.user-7421",
  cell_id="cell-kr-seoul-1",
  region="kr",
  resource_type="compute_cpu_seconds"
} 0.0023
oya:ontology:entity-store:cost_units_total{
  tenant_id="tenant-acme-corp",
  subscope="tenant-acme-corp.user-7421",
  cell_id="cell-kr-seoul-1",
  region="kr",
  resource_type="storage_bytes_seconds"
} 482.7
```

### Correlation pivot

An operator investigating "why did `tenant-acme-corp.user-7421`
report slow ontology writes at 14:23:45 KST?" pivots:

1. **Grafana dashboard** — opens the `oya:ontology:entity-store:
   write_duration_seconds:histogram` p99 chart for `tenant-acme-corp`.
   Sees a spike at 14:23:45.
2. **Exemplar click** — clicks the exemplar dot for the p99 bucket;
   Grafana opens the corresponding trace in Tempo: `trace_id =
   4f1c9e6b8d2a7f3e5c8b1a9d4e2c7f3b`.
3. **Trace view** — sees the 3-µservice span tree
   (cloud-iam → policy-engine → ontology); identifies the
   `ontology.create_entity` span as the bottleneck.
4. **Log pivot** — clicks the span ID; Grafana opens the corresponding
   Loki query: `{trace_id="4f1c9e6b8d2a7f3e5c8b1a9d4e2c7f3b"}`. Sees
   the structured log line above with `audit_id`.
5. **Audit pivot** — clicks the `audit_id`; Grafana opens the
   audit-chain entry. Confirms the event sealed correctly.
6. **Per-tenant cost** — switches dashboard to per-tenant cost
   attribution; confirms the spike is reflected in
   `tenant-acme-corp`'s cost units.

The full forensic loop completes in <2 minutes. Under the prior
free-form emission model, the same loop required 15+ minutes of
correlation guessing and frequently failed when `tenant_id` was
omitted from one of the three µservices.

### What the contract guarantees

The example demonstrates:

- **D-1** (Three Pillars present at every step).
- **D-2** (OpenTelemetry SDK emitted spans).
- **D-3** (W3C Trace Context propagated across gRPC calls).
- **D-4** (`tenant_id` on every emission).
- **D-5** (sub-scope on every emission).
- **D-6** (structured JSON log schema honoured).
- **D-7** (metric naming convention honoured: `oya:<ms>:<bc>:<metric>_<unit>:<type>`).
- **D-8** (the trace would be tail-sampled at 100% if it had errored or been a p99 outlier).
- **D-9** (per-tenant rollup via Mimir multi-tenancy).
- **D-10** (cross-cell aggregation if the request crossed cells; not in this single-cell example).
- **D-11** (cost-attribution metrics emitted).
- **D-12** (exemplars link the histogram bucket to the trace).
- **D-13** (`audit_id` on the state-changing emission).
- **D-14** (PII scrubbing applied to any name/email in the entity payload).
- **D-15** (schema version `oyatie/log/v1` declared; future migration via handshake).

Every one of the 15 decisions lands; the operator forensic loop is
fast and reliable.

## Naming justification

Per `feedback_naming_justification`: every new name introduced by this ADR carries a one-line BNF v4.1 + ADR-0105 13-layer conformance justification.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|---|---|---|---|
| `oya-shared-observability-client-kernel` | `kernel` | `shared`.`observability-client`.`kernel` | Cross-µservice shared client crate; slot2=`shared` per ADR-0056 §"Microservice registry" for cross-µservice utilities; BC=`observability-client`; layer=`kernel` (port traits, zero I/O). Conforms to BNF v4.1. |
| `oya-shared-observability-client-domain` | `domain` | `shared`.`observability-client`.`domain` | Pure domain logic for emission batching and PII scrubbing decisions; layer=`domain`. |
| `oya-shared-observability-client-application` | `application` | `shared`.`observability-client`.`application` | Use-case orchestrators wiring scrubber + exporter; layer=`application` per ADR-0105 13-layer canonical enum (was `usecase` before the ADR-0106 rename). |
| `oya-shared-observability-client-api` | `api` | `shared`.`observability-client`.`api` | Protocol-neutral typed emission contracts; layer=`api` per ADR-0105 Amendment 1 (protocol-neutral contract surface). |
| `oya-shared-observability-client-adapter` | `adapter` | `shared`.`observability-client`.`adapter` | OTel SDK + Loki + Prometheus exporter bindings; layer=`adapter` (framework/driver glue). |
| `oya-shared-observability-client-sdk` | `sdk` | `shared`.`observability-client`.`sdk` | High-level Rust SDK re-exported for downstream consumers; layer=`sdk` (external consumer library per ADR-0105). |
| `oya-shared-observability-client-app` | `app` | `shared`.`observability-client`.`app` | Composition-root binary for standalone emission agent mode; layer=`app` (composition-root binary). |
| `oya-check-pii-scrubbing-at-emission` | N/A (check-family) | `check`.`pii-scrubbing-at-emission` | CI fitness-check lane; `oya-check-*` flat namespace per ADR-0105 Amendment 2 (check-family self-layering); verifies PII scrubbing fixtures present per §D-14. |
| `oya-check-trace-context-propagation` | N/A (check-family) | `check`.`trace-context-propagation` | CI fitness-check; verifies W3C Trace Context propagation per §D-3. |
| `oya-check-metric-naming-convention` | N/A (check-family) | `check`.`metric-naming-convention` | CI fitness-check; verifies metric names match `oya:<ms>:<bc>:<metric>_<unit>:<type>` per §D-7. |
| `oya-check-log-schema-conformance` | N/A (check-family) | `check`.`log-schema-conformance` | CI fitness-check; validates structured log schema `oyatie/log/v1` per §D-6. |
| `oya-check-audit-id-on-state-change` | N/A (check-family) | `check`.`audit-id-on-state-change` | CI fitness-check; verifies `audit_id` present on state-changing emissions per §D-13. |
| `oya-check-tenant-label-presence` | N/A (check-family) | `check`.`tenant-label-presence` | CI fitness-check; verifies `tenant_id` label on every metric/log/trace per §D-4. |
| `oya-check-observability-emission-contract` | N/A (check-family) | `check`.`observability-emission-contract` | CI fitness-check; umbrella gate validating all 15 emission contract decisions. |
| `oyatie/log/v1` | N/A (schema version URI) | N/A | Schema-version literal following `<namespace>/<artifact>/v<n>` convention (JSON Schema + OpenAPI ecosystem idiom); `oyatie` is the registered platform namespace. Not a crate; not subject to BNF v4.1 3-slot rule. |
| `oya:` (metric prefix) | N/A (metric prefix) | N/A | Metric namespace prefix per §D-7; `oya` is the platform owner prefix (matches tenant reserved root per ADR-0242). Not a crate name. |

---

*End of ADR-0263.*
