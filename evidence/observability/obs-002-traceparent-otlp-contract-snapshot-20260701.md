---
doc_class: Evidence
status: draft-contract-snapshot
source_task: t_c1f8aca4
substrate_task: t_62ecd32e
generated_at_utc: 2026-07-01T09:04:43Z
claim_ceiling: read-only/spec/fixture contract snapshot only; no runtime collector, live telemetry, measured SLO, production-readiness, or hyperscaler-maturity claim
---

# OBS-002 traceparent/OTLP contract snapshot for spec descendants

This snapshot exists so spec-, plan-, and fixture-only descendants can proceed from a bounded set of OBS-002 traceparent/OTLP assumptions while runtime collector hardening, emitted-evidence validation, rollout, and production readiness remain behind `t_62ecd32e` and its review/fix parent.

It does not promote OBS-002. It is not evidence that any service currently emits live traces, exports OTLP to a collector, satisfies SLO windows, or is production-ready.

## Source inspection basis

Inspected sources:

- `docs/standards/observability.md:44-61` mandates OpenTelemetry SDK emission and OTLP as wire format, with collector/agent+gateway deployment as the target pattern.
- `docs/standards/observability.md:69-78` mandates W3C Trace Context (`traceparent`, `tracestate`) on HTTP, gRPC, and message-queue boundaries.
- `docs/standards/observability.md:103-125` names structured logging fields, including `trace_id`, `span_id`, tenant fields, and redaction boundaries.
- `docs/standards/observability.md:157-169` requires exemplar-style trace-to-metric correlation without putting `trace_id` into metric labels.
- `docs/standards/logging-tracing.md:14-33` records OTel semantic convention/resource fields and mandatory span fields such as `trace_id`, `span_id`, `service.name`, tenant/cell fields, and data-class/autonomy fields when applicable.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md:42-47` is Accepted authority for the tracing invariant: every inter-microservice call propagates OpenTelemetry trace context and is validated by the `oya-check-otel-trace-propagation` lane.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md:80-89` permits direct sibling-service egress only under mTLS/Cedar plus audit and tracing invariants.
- `docs/decisions/ADR-0263-observability-emission-contract.md:147-164` describes the emission contract boundary and W3C propagation across gRPC, HTTP, workflow activities, audit-chain emission, and async messaging. ADR-0263 is Proposed in its frontmatter, so use it as context unless a card/root pointer elevates it for a lane.
- `docs/decisions/ADR-0263-observability-emission-contract.md:255-268` describes three signal streams and OTLP/gRPC export through Alloy/collector/gateway. Treat this as planned/contextual until promoted.
- `docs/decisions/ADR-0263-observability-emission-contract.md:282-294` names the Rust OpenTelemetry crate family expected by the target contract. Treat this as contextual; no dependency change is authorized by this snapshot.
- `docs/decisions/ADR-0263-observability-emission-contract.md:308-345` gives concrete `traceparent`/`tracestate` propagation carriers for HTTP, gRPC metadata, workflow envelopes, outbox headers, and async message headers. Treat this as contextual unless elevated.
- `specs/hyperscaler-architecture-invariants.json:328-333` records `INV-OBSERVABILITY-TRACING`: propagate W3C trace context on inbound/outbound service boundaries and emit service/operation/status/duration spans.
- `specs/cloud-observability-slo-target.json:21-40` is a Proposed-target observability/SLO target and explicitly says no collector, SLO engine, dashboard, measured SLO, or production evidence is implemented there.
- `specs/cloud-observability-slo-evidence-contract.json:10-20` explicitly blocks production-readiness, runtime-observability-engine, tenant-workload, measured-SLO, and hyperscaler-maturity claims for its metadata-only evidence contract.
- `libs/oya-check-otel-trace-propagation/src/lib.rs:84-107` defines current advisory/strict token sets for propagation and OTLP exporter-path detection.
- `libs/oya-check-otel-trace-propagation/src/lib.rs:142-201` defines the current strict-mode report shape and fail-closed checks, but current review says it still accepts static source evidence rather than concrete runtime emissions.
- `libs/oya-check-otel-trace-propagation/src/lib.rs:219-253` shows the current fixture-level valid-`traceparent` parser: version `00`, 32 lowercase-hex nonzero trace id, 16 lowercase-hex nonzero parent/span id, and 2 lowercase-hex flags.
- `libs/oya-http-telemetry-middleware-infrastructure/src/lib.rs:161-171` names HTTP telemetry constants including the `traceparent` header.
- `libs/oya-http-telemetry-middleware-infrastructure/src/lib.rs:234-239` currently uses `traceparent` only as a correlation-id fallback.
- `libs/oya-http-telemetry-middleware-infrastructure/src/lib.rs:245-277` records metrics/wide events in memory but does not prove OTLP export or live collector ingestion.

## Draft OBS-002 contract assumptions for spec/fixture work

A1. The canonical propagation carrier is W3C Trace Context: `traceparent` is mandatory where a traced operation crosses HTTP, gRPC, workflow, outbox, or async-message boundaries; `tracestate` is optional and reserved for vendor/extensions.

A2. For current fixture/validator work, a valid sample `traceparent` must follow the current strict parser shape: `00-<32 lowercase hex nonzero trace id>-<16 lowercase hex nonzero parent/span id>-<2 lowercase hex flags>`. This is a fixture validity rule, not proof that runtime propagation is wired.

A3. Runtime work must not rely on dead constants or arbitrary source substrings as proof. `t_62ecd32e` remains blocked because review found the current strict validation can pass on static source text/canary values and the messenger plan is not wired into a request handler or emitted path.

A4. OTLP is the intended wire format. Spec/fixture descendants may refer to `OTEL_EXPORTER_OTLP_ENDPOINT`, `OYA_OTEL_ENDPOINT`, `otel-collector`, `alloy.observability`, `:4317`, and `/v1/traces` as expected fixture tokens, but those tokens alone do not prove live export.

A5. The target collector shape is agent or sidecar collection plus gateway fan-in. A pure centralized gateway without per-host/per-pod fan-in is not the desired target pattern in the accepted standards.

A6. Emitted spans should carry at least service identity, operation name/route or equivalent operation, status, duration, `trace_id`, and `span_id`; tenant/cell/data-class fields apply when scoped by the surface.

A7. Logs and wide events should preserve correlation fields (`trace_id`, `span_id`, request/correlation id, audit id where applicable) and must not place secrets or raw PII into log messages, span attributes, or metric labels.

A8. Metrics must remain low-cardinality. `trace_id` and high-cardinality tenant/user identifiers belong in traces/logs/audit evidence or controlled exemplars, not as unbounded metric labels.

A9. Gateway/mesh and route-contract fixtures may require `traceparent` preservation and audit-chain correlation on state-changing routes, but actual mTLS/SPIFFE/Cedar runtime behavior is outside this snapshot.

A10. SLO/burn-rate/status/rollback fixtures may use trace/OTLP fields as expected evidence inputs. Measured SLO windows, burn-rate alert receipts, collector availability, and rollback behavior remain unclaimed until live evidence lands.

A11. Proposed ADRs and Proposed-target specs are planning/contextual inputs here. Descendants that mutate product/cloud/runtime code still need accepted ADR/root-pointer authority or an explicit analysis-only/non-mutating disposition before implementation.

A12. Generated JSON remains out of scope. This snapshot authorizes no `*.generated.json` hand edits and no `.github/`, release/governance, production collector, or unrelated telemetry-library changes.

## Consumer map

| Task | May use this snapshot now | Must still wait for runtime OBS-002 / other gates |
|---|---|---|
| `t_38500376` OPS-001 ops deploy/status panel slice | May encode expected trace/audit/OTLP fields for a typed deploy/status panel or API-adapter contract, and may cite the no-CLI/shell-scraping boundary. | Cannot claim live ops telemetry, collector ingestion, runtime propagation, production status, or CI/rollout readiness. Product/cloud mutation still needs current authority/ADR clarification noted on the card. |
| `t_553821bd` DOGFOOD-003 clean-cell bring-up smoke plan | May define a smoke-plan evidence matrix that expects `traceparent`, OTLP endpoint/collector, trace/log/metric/audit correlation fields, and explicit blockers for missing live evidence. | Cannot claim a clean-cell smoke passed, live collector exists, or runtime deployment is ready. ADR-0537 remains Proposed/context unless elevated or the card stays analysis-only. |
| `t_646222a9` FINOPS-001 cost attribution/chargeback/autoscaling economics slice | May specify cost/chargeback fixture labels and trace/correlation fields needed to join workload class, route/service, operation, and cost-center evidence without putting trace ids into metric labels. | Cannot claim measured cost telemetry, anomaly detection over live traces, OpenCost/FOCUS runtime integration, or autoscaling economics from production data. Proposed ADR-0314/0315 inputs still need disposition. |
| `t_9e4e1495` NETWORK-001 API gateway + mesh boundary contract slice | May encode one route's gateway/mesh contract requirements for `traceparent`/`tracestate` preservation, audit correlation, mTLS/SPIFFE/Cedar preconditions, and OTLP evidence fields. | Cannot claim actual gateway/mesh propagation, collector export, or one-route runtime compliance. Proposed ADR-0044 inputs still need disposition unless the lane is analysis-only or elevated. |

Not covered by this snapshot as a substitute for runtime OBS-002:

- `t_62ecd32e` OBS-002 itself: remains blocked until `t_bfcbdde5` or equivalent review/fix proves concrete emitted runtime traceparent + OTLP evidence and independent review passes.
- `t_8684a836` INTEL-RUNTIME-001: still depends on runtime/substrate gates because it is a runtime contract slice for provider/MCP/sandbox/eval behavior, not one of the reparented read-only/spec descendants.
- `t_c127bb35` RESILIENCE-001: still depends on runtime OBS-002 because chaos/status/brownout/SLO control-loop evidence requires real emitted telemetry and SLO/trace inputs.
- `t_cb12fdb1` OBS-004: still depends on runtime OBS-002 because rollback-trigger evidence requires actual OpenSLO-to-alert/rollback fixtures tied to telemetry inputs.

## Reconciliation needs before runtime promotion

R1. Replace source-substring validation with a data-bearing evidence seam. `validate_strict` or its successor should consume emitted/constructed trace/export evidence (for example, a typed record from request handling or exporter configuration) rather than arbitrary source text.

R2. Wire at least one service path to produce concrete request-path trace context and OTLP exporter binding evidence. The previous messenger slice exposed constants/plan data but review found it did not prove a handler/outbound/runtime emission path.

R3. Preserve fail-closed behavior: empty inputs, invalid/all-zero traceparents, missing `traceparent`, missing OTLP endpoint/collector binding, and missing emitted evidence must fail the runtime proof.

R4. Keep ADR authority honest. Accepted authority is currently strongest in `docs/standards/observability.md`, `docs/standards/logging-tracing.md`, ADR-0145, root-hub pointers, and `INV-OBSERVABILITY-TRACING`; ADR-0263, ADR-OBS-001, and cloud observability target specs are context/planning unless promoted or explicitly elevated.

R5. If runtime OBS-002 changes the fixture field names, carrier names, endpoint tokens, or evidence record shape, update this snapshot or supersede it before descendants claim compatibility.

R6. Any descendant that moves from spec/fixture into product/cloud/runtime mutation must carry its own accepted-authority citations, verification commands, review/fix card, and claim ceiling.

## Verification notes for this snapshot

This snapshot was produced after read-only inspection of the sources listed above and the Kanban state for `t_62ecd32e`, `t_38500376`, `t_553821bd`, `t_646222a9`, `t_9e4e1495`, `t_8684a836`, `t_c127bb35`, and `t_cb12fdb1`.

Expected verification for this file:

- Markdown/text diff only under `evidence/observability/`.
- No generated JSON edits.
- No runtime collector, deployment, `.github/`, release/governance, or unrelated telemetry-library edits.
- Board closeout should cite this artifact so the four reparented descendants can read it from the parent handoff.
