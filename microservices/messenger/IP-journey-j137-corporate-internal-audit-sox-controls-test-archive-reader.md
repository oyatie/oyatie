---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j137-messenger-archive-reader
journey_id: j137-corporate-internal-audit-sox-controls-test
microservice: messenger
role: archive-reader
status: draft
date: 2026-05-20
authority_tier: 3
owner_team: axis-messenger + axis-internal-audit
parallel_work_compatibility: independent of j138/j139/j140/j141 messenger-IPs; depends on identity B2B_INTERNAL_AUDIT resolver landing first
related_adrs: [ADR-0311, ADR-0313, ADR-0243, ADR-0244, ADR-0028, ADR-0263, ADR-0145, ADR-0310]
related_journey_artifacts:
  - docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/handshake.md (Phase 2)
  - docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/schemas/sox-audit-sample-request.json
  - docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/schemas/sox-control-evidence-bundle.json
  - docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/schemas/cedar-internal-audit-permit-decision.json
depends_on:
  - microservices/identity/IP-journey-j137-corporate-internal-audit-sox-controls-test-permit-resolver.md
  - microservices/audit-chain/IP-journey-j137-corporate-internal-audit-sox-controls-test-evidence-bundler.md
---

# IP-journey-j137-messenger-archive-reader — Messenger: tenant-scoped archive read for SOX internal audit

## Goal

Implement the `messenger.read_tenant_archive` surface that a
B2B_INTERNAL_AUDIT principal can use to pull message threads from
the work-tenant archive within the permit's classification window —
WITHOUT returning any personal-tenant correlated thread content.
The read must be Cedar-gated, audit-chain-sealed, and Merkle-anchored
per ADR-0028, with the personal-tenant deny default holding 100% of
the time per ADR-0311.

## Data model

| Object | Storage | Schema | TTL |
|---|---|---|---|
| `MessengerThread` (tenant-owned) | ClickHouse cold archive partitioned by `tenant_id, year_month` | per-tenant Avro v3 | 7y SOX retention |
| `MessengerMessage` | inline in thread; encrypted-at-rest per ADR-0251 | per-tenant Avro v3 | 7y |
| `ThreadCorrelationGraph` | Postgres `thread_correlation_view` (materialized from identity) | view-only | refreshed on principal-change |
| `PersonalTenantDenyEvent` | Kafka `messenger.personal_tenant.deny.events` + audit-chain leaf | `schemas/cedar-internal-audit-permit-decision.json` | 7y |
| `AuditReadAttempt` | Postgres `messenger_audit_read_attempts` | per-read row | 7y |

## Schema mapping

ClickHouse archive table:

```sql
CREATE TABLE messenger.tenant_archive
(
  thread_id          UUID,
  tenant_id          LowCardinality(String),
  cell_tier          UInt8,
  participant_set    Array(String),       -- principal ids
  participant_classes Array(String),      -- 'work_tenant_owned' | 'personal_tenant_owned' | ...
  classification_window_start DateTime64,
  classification_window_end   DateTime64,
  message_count      UInt32,
  body_encrypted     String CODEC(ZSTD(7)),
  body_key_envelope  String,
  pack_set           Array(String),
  sealed_at          DateTime64,
  trace_id           String
)
ENGINE = MergeTree
PARTITION BY (tenant_id, toYYYYMM(classification_window_start))
ORDER BY (tenant_id, classification_window_start, thread_id);
```

Postgres audit-read-attempts table:

```sql
CREATE TABLE messenger_audit_read_attempts (
  id UUID PRIMARY KEY,
  audit_case_id TEXT NOT NULL,
  requestor_principal TEXT NOT NULL,
  thread_id TEXT,
  resource_principal_class TEXT NOT NULL,
  cedar_decision TEXT NOT NULL CHECK (cedar_decision IN ('PERMIT', 'DENY', 'FORBID')),
  cedar_policy_id TEXT,
  audit_seal_id TEXT NOT NULL,
  emitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  trace_id TEXT NOT NULL,
  span_id TEXT NOT NULL,
  tenant_id TEXT NOT NULL,
  cell_tier INTEGER NOT NULL,
  pack_set TEXT[] NOT NULL
);

CREATE INDEX idx_msgr_audit_case ON messenger_audit_read_attempts(audit_case_id, emitted_at DESC);
CREATE INDEX idx_msgr_audit_decision ON messenger_audit_read_attempts(cedar_decision, emitted_at DESC);
```

## API surface (gRPC)

```protobuf
syntax = "proto3";
package oyatie.messenger.audit.v1;

import "google/protobuf/timestamp.proto";

service MessengerArchive {
  // Tenant-scoped archive read. Returns work-tenant content only.
  // Personal-tenant correlated principals appear only in the
  // personal_tenant_deny_summary field (count + class).
  rpc ReadTenantArchive (ReadTenantArchiveRequest) returns (ReadTenantArchiveResponse);

  // Correlated-principals read. Behaves as ReadTenantArchive but
  // additionally surfaces deny-count summaries for personal-tenant
  // principals that are correlated to the work-tenant threads.
  rpc ReadCorrelatedPrincipals (ReadCorrelatedPrincipalsRequest) returns (ReadCorrelatedPrincipalsResponse);
}

message ReadTenantArchiveRequest {
  string audit_case_id = 1;
  string tenant_id = 2;
  ParticipantFilter participants = 3;
  KeywordFilter keywords = 4;
  TimeWindow window = 5;
  string requestor_principal = 6;
  string permit_batch_ref = 7;
  string trace_id = 8;
}

message ReadTenantArchiveResponse {
  repeated MessengerThreadEvidence threads = 1;
  PersonalTenantDenySummary personal_tenant_deny_summary = 2;
  string audit_seal_id = 3;
  google.protobuf.Timestamp evaluated_at = 4;
}

message PersonalTenantDenySummary {
  uint32 count = 1;
  repeated string principal_classes = 2; // 'personal_tenant_owned' etc; NO ids
}

message MessengerThreadEvidence {
  string thread_id = 1;
  string tenant_id = 2;
  repeated string participants = 3; // work-tenant principals only
  repeated MessengerMessage messages = 4;
  string merkle_leaf_hash = 5;
}
```

## Cedar policy

```cedar
@id("messenger-read-tenant-archive-v1")
permit (
  principal,
  action == Action::"messenger.read_tenant_archive",
  resource is MessengerThread
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  principal.audit_case_id != null &&
  resource.tenant_id == principal.permit_scope.tenant_id &&
  resource.classification_window.intersects(principal.permit_scope.window) &&
  context.dual_control_approval_at != null &&
  context.audit_charter_active == true
};

@id("messenger-personal-tenant-deny-v1")
forbid (
  principal,
  action == Action::"messenger.read_tenant_archive",
  resource is MessengerThread
) when {
  resource.tenant_id != principal.permit_scope.tenant_id ||
  resource.principal_class == "personal_tenant_owned"
};
```

## Integration contracts

### Upstream (callers)

- `workflow-engine.audit_sample_planner` — primary caller during sample-pull.
- `ops-dashboard-control-center.audit_pane` — secondary caller for
  ad-hoc explorations within an active audit case.
- All callers must carry a valid `permit_batch_ref` and SPIFFE
  attestation tagged `b2b-internal-audit-client`.

### Downstream (callees)

- `identity.B2BInternalAuditPrincipalResolver` — to resolve the
  requestor's principal context (audience_type, permit scope).
- `audit-chain.SealLeaf` — for every read attempt's audit event.
- `compliance.PackOverlayResolver` — to compose the pack set
  applied to this read.
- `observability` — OTLP metrics + traces.

### Cross-tenant invariant

The archive reader MUST refuse cross-tenant reads at the database
layer (ClickHouse partition pruning by `tenant_id`) AND at the Cedar
gate. Defense in depth: a single failure cannot expose cross-tenant
content.

## Implementation notes

### Personal-tenant detection

The archive reader's challenge is detecting when a participant in
a work-tenant thread ALSO has a personal-tenant principal that
might correlate. The detection is done via the `participant_classes`
column on the archive row. The column is populated at thread-seal
time by the identity µservice's principal-class-resolver and is
NEVER modified post-seal.

When the Cedar gate denies a personal-tenant principal, the archive
reader:

1. Increments the `personal_tenant_deny_summary.count` field.
2. Adds the principal-class label to the `principal_classes` array
   (e.g., `"personal_tenant_owned"`).
3. DOES NOT include the principal-id (email) in the response.
4. Emits a `MessengerPersonalTenantReadDenied` event with the
   principal-id sealed (the seal hash is the only externally visible
   reference; the cleartext id is never logged).

### Body encryption

Per ADR-0251, message bodies are stored encrypted-at-rest with
tenant-scoped envelope keys. The archive reader decrypts in-memory
only for principals whose Cedar evaluation returned PERMIT. The
decryption path is in-process; no plaintext crosses µservice
boundaries except within the response itself (which is then
re-encrypted in transit).

### Performance budget

Per `docs/standards/cross-microservice-latency-budget.md` §3.1:

- `ReadTenantArchive` p95 ≤ 200ms for thread sets ≤ 50 messages.
- `ReadCorrelatedPrincipals` p95 ≤ 350ms for correlated sets ≤ 10.
- Single-sample audit pull (Phase 2 in handshake) p95 ≤ 60s end-to-end.

### Brownout behaviour

If ClickHouse archive is in brownout (per ADR-0286), the archive
reader returns 503 with `Retry-After` header; the workflow-engine
pause-resumes the sample pull. No partial reads.

### Failure modes

- ClickHouse unavailable → 503; retry with exponential backoff.
- Cedar gate timeout → fail-closed deny; emit `CedarGateTimeoutDenied`.
- audit-chain partial seal → async retry; emit `AuditSealRetryQueued`.
- Decryption key unavailable → 500; emit `DecryptionKeyUnavailable`.

## Test plan

See `docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/integration-test-plan.md`
sections 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 4.4, 4.5, 13.4.

Unit tests:

- `test_cedar_permit_required_for_archive_read`
- `test_personal_tenant_principal_class_triggers_deny`
- `test_cross_tenant_request_denied`
- `test_decryption_only_after_permit`
- `test_audit_seal_emitted_for_every_read_attempt`
- `test_personal_tenant_principal_id_never_in_response_body`
- `test_classification_window_filter_correct`
- `test_keyword_filter_does_not_leak_across_tenants`

Property tests:

- Property: for any randomly-generated request, the response never
  contains any personal-tenant principal id.
- Property: every response has exactly one audit-seal-id.
- Property: every Cedar evaluation has a matching audit event.

## Build sequence

1. Author Cedar policies (`messenger-read-tenant-archive-v1`,
   `messenger-personal-tenant-deny-v1`).
2. Add `participant_classes` column to ClickHouse archive schema
   (migration `messenger-2026-q2-q3-add-participant-classes`).
3. Backfill `participant_classes` for the audit window (Q2-2026)
   using identity µservice's principal-class resolver.
4. Implement gRPC service skeleton (server + client stubs).
5. Wire Cedar gate to api-gateway sidecar.
6. Wire audit-chain seal emission.
7. Wire OTLP metrics + traces.
8. Add unit + property tests.
9. Add integration test against test fixtures.
10. Wire into `workflow-engine.audit_sample_planner` workflow template.

## Acceptance gates

- All unit tests PASS.
- All integration tests for §3, §4 PASS.
- Cedar policy lint clean (`oya-lint-cedar`).
- ClickHouse migration applied in test environment with backfill verified.
- Schema validation clean (`oya-schema-validate`).
- Observability dashboard renders the `oya_messenger_archive_read_total`
  + `oya_personal_tenant_deny_total` metrics in pre-prod.
- Code review by axis-messenger + axis-internal-audit councils.
- Pre-merge multispectrum review v2.4.0 facets F1/F2/F3/M1/A1/A4/A5.

## Operational notes

- Owner team: axis-messenger (primary) + axis-internal-audit (secondary).
- Pager: PagerDuty service `oya-messenger-audit-reader`.
- Dashboards:
  - Grafana `messenger-audit-reader` (read rate, deny rate, latency).
  - Audit-trail viewer `audit-chain.kibana/messenger-audit`.
- Runbook: `microservices/messenger/runbooks/audit-archive-reader.md`.

## Compliance and pack overlays

The archive reader composes its active pack set from `compliance` at
read time. Sample composition for a Q2 2026 SOX audit:

- `pack-us-sox-404` (read-window retention 7y)
- `pack-pcaob-as5` (sample-traceability)
- `pack-eu-whistleblower-2019-1937` (EU counterparties)
- `pack-eu-gdpr-cross-border` (Munich-based customer)
- `pack-ng-data-protection-2023` (Lagos employee residency)
- `pack-corporate-internal-audit-baseline` (tenant overlay)

The pack set is stamped into every emitted audit event so the
evidence pack carries the exact pack composition active at the read.

## Cross-microservice port declaration

This IP introduces the public gRPC service `MessengerArchive` in
namespace `oyatie.messenger.audit.v1` per ADR-0145. The proto file
lives at `protos/messenger-audit-v1.proto` and is consumed by
workflow-engine and ops-dashboard.

## Roll-out plan

- Phase 1 (Wave-3-F+1): Implement behind feature flag
  `messenger.archive_reader.enabled`. Default off.
- Phase 2 (Wave-3-F+2): Enable for `test.marcus-corp.tenant`
  synthetic; run integration suite.
- Phase 3 (Wave-3-F+3): Enable for `marcus-corp.tenant` production
  in dry-run mode (no human reads; only synthetic Cedar evals).
- Phase 4 (Wave-3-F+4): Enable for production human reads with the
  audit committee chair's co-signed approval.
- Phase 5 (Wave-3-F+5): Open to all B2B_INTERNAL_AUDIT tenants on
  the platform (per their own dual-control governance).

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Personal-tenant principal id leaks via correlation reconstruction | CRITICAL | Property test + Cedar deny + principal-class label only in response |
| ClickHouse partition pruning misconfiguration | HIGH | Lane test for cross-tenant query patterns; alerting on `tenant_id != principal.tenant` reads |
| Cedar policy evaluator regression | HIGH | Cedar policy CI lane; reference-output tests for the personal-tenant forbid block |
| Decryption-key cache staleness | MEDIUM | Per-read key freshness check; cache TTL ≤ 60s |
| Brownout during sample pull | MEDIUM | workflow-engine pause-resume; idempotent retry semantics |
| Observability metric cardinality explosion | LOW | Cardinality budget per metric enforced at scrape; alerting on overage |

## Definition of done

- gRPC service deployed in production behind feature flag.
- All tests in test plan PASS.
- Observability dashboard live.
- Runbook authored.
- Internal-audit council signed off on the Cedar policy text.
- The j137 audit-pane integration end-to-end-tested with synthetic
  Sam principal and synthetic fixtures.
- The personal-tenant deny invariant holds in all 80+ integration
  tests.

## Completion expansion — j137 messenger IP rigor pass

Journey context: quarterly SOX 404 audit of work surfaces only.
Service role: work/personal message-surface separation, archive read, and deny-by-default enforcement.
Mapped services in this journey: messenger, mail, workflow-engine, payments, audit-chain, ops-dashboard-control-center, identity, compliance.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0312, ADR-0313, ADR-0319.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in messenger, define the Cedar policy change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving messenger and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in messenger, define the OpenAPI 3.2.0 contract change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving messenger and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in messenger, define the AsyncAPI 3.1.0 event change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving messenger and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in messenger, define the proto3 port change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving messenger and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in messenger, define the Postgres/RLS storage change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving messenger and ops-dashboard-control-center agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in messenger, define the audit-chain emission change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving messenger and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in messenger, define the dashboard projection change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving messenger and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in messenger, define the runbook hook change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving messenger and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in messenger, define the integration fixture change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving messenger and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in messenger, define the domain model change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving messenger and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in messenger, define the Cedar policy change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: messenger MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving messenger and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in messenger, define the OpenAPI 3.2.0 contract change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/messenger/contracts/openapi/messenger.yaml`, `microservices/messenger/contracts/asyncapi/messenger-events.yaml`, `microservices/messenger/contracts/proto/messenger.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-journey-j137-corporate-internal-audit-sox-controls-test-archive-reader.md` matched `SLO, multi-region, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/messenger/IP-journey-j137-corporate-internal-audit-sox-controls-test-archive-reader.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/messenger/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
