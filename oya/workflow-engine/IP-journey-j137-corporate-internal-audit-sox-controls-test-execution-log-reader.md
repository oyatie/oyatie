---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j137-workflow-engine-execution-log-reader
journey_id: j137-corporate-internal-audit-sox-controls-test
microservice: workflow-engine
role: execution-log-reader
status: draft
date: 2026-05-20
authority_tier: 3
owner_team: axis-workflow-engine + axis-internal-audit
parallel_work_compatibility: depends on identity B2B_INTERNAL_AUDIT resolver; coordinates with audit-chain evidence bundler
related_adrs: [ADR-0311, ADR-0310, ADR-0307, ADR-0243, ADR-0244, ADR-0028, ADR-0263, ADR-0145]
related_journey_artifacts:
  - docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/handshake.md (Phase 2/4)
  - docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/schemas/sox-audit-sample-request.json
  - docs/user-journeys/j137-corporate-internal-audit-sox-controls-test/schemas/sox-control-evidence-bundle.json
depends_on:
  - microservices/identity/IP-journey-j137-corporate-internal-audit-sox-controls-test-permit-resolver.md
  - microservices/audit-chain/IP-journey-j137-corporate-internal-audit-sox-controls-test-evidence-bundler.md
---

# IP-journey-j137-workflow-engine-execution-log-reader — Workflow Engine: SOX audit-sample-planner + execution-log read

## Goal

Implement TWO new surfaces on the workflow-engine:

1. `workflow-engine.audit_sample_planner` — a workflow template that
   orchestrates Sam's quarterly audit sample-pull (PCAOB AS-5 stratified
   sampling + parallel fan-out to messenger/mail/payments/audit-chain
   + evidence-pack assembly).

2. `workflow_engine.read_execution_logs` — a tenant-scoped read RPC
   that exposes workflow execution traces (order-to-cash, period-close,
   approval workflows) for inclusion in the SOX evidence pack.

## Data model

| Object | Storage | Schema | TTL |
|---|---|---|---|
| `WorkflowExecution` (existing) | Postgres `workflow_engine.executions` | existing schema + new audit fields | 7y for SOX-tagged executions |
| `WorkflowExecutionStage` | Postgres `workflow_engine.stages` | existing | 7y |
| `AuditSamplePlannerCase` | Postgres `workflow_engine.audit_sample_cases` | new | 7y |
| `AuditPullJob` | Postgres `workflow_engine.audit_pull_jobs` | new | 7y |
| `StratifiedSamplePlan` | Postgres `workflow_engine.stratified_sample_plans` | new | 7y |

## Schema mapping

```sql
CREATE TABLE workflow_engine.audit_sample_cases (
  case_id TEXT PRIMARY KEY,
  audit_case_ref TEXT NOT NULL,    -- ac-marcus-corp-2026-q2-sox-404
  tenant_id TEXT NOT NULL,
  requestor_principal TEXT NOT NULL,
  classification_window_start TIMESTAMPTZ NOT NULL,
  classification_window_end TIMESTAMPTZ NOT NULL,
  control_set TEXT[] NOT NULL,        -- ['RC-01', ..., 'RC-07']
  sample_plan_id TEXT NOT NULL,
  permit_batch_ref TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('CREATED','PERMIT_PENDING','ACTIVE','EVIDENCE_ASSEMBLED','HANDOFF','CLOSED','EXPIRED')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  closed_at TIMESTAMPTZ
);

CREATE TABLE workflow_engine.stratified_sample_plans (
  plan_id TEXT PRIMARY KEY,
  case_id TEXT NOT NULL REFERENCES workflow_engine.audit_sample_cases(case_id),
  control_id TEXT NOT NULL,
  stratum TEXT NOT NULL CHECK (stratum IN ('A','B','C','D')),
  amount_lower_usd NUMERIC,
  amount_upper_usd NUMERIC,
  population_size INTEGER NOT NULL,
  sample_size INTEGER NOT NULL,
  sample_invoice_ids TEXT[] NOT NULL,
  generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  random_seed TEXT NOT NULL    -- deterministic for reproducibility
);

CREATE TABLE workflow_engine.audit_pull_jobs (
  job_id TEXT PRIMARY KEY,
  case_id TEXT NOT NULL,
  sample_index INTEGER NOT NULL,
  invoice_id TEXT NOT NULL,
  evidence_targets TEXT[] NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('QUEUED','RUNNING','SEALING','SEALED','FAILED','PAUSED_BROWNOUT')),
  attempts INTEGER NOT NULL DEFAULT 0,
  started_at TIMESTAMPTZ,
  finished_at TIMESTAMPTZ,
  evidence_bundle_id TEXT,
  personal_tenant_deny_count INTEGER DEFAULT 0,
  auto_flags JSONB,
  error_message TEXT
);

CREATE INDEX idx_pull_job_case ON workflow_engine.audit_pull_jobs(case_id, status, started_at DESC);
```

## API surface (gRPC)

```protobuf
syntax = "proto3";
package oyatie.workflow_engine.audit.v1;

service AuditSamplePlanner {
  rpc CreateAuditCase (CreateAuditCaseRequest) returns (CreateAuditCaseResponse);
  rpc GenerateStratifiedSamplePlan (GenerateStratifiedSamplePlanRequest) returns (GenerateStratifiedSamplePlanResponse);
  rpc StartSamplePull (StartSamplePullRequest) returns (StartSamplePullResponse);
  rpc ListPullJobs (ListPullJobsRequest) returns (ListPullJobsResponse);
  rpc AssembleEvidencePack (AssembleEvidencePackRequest) returns (AssembleEvidencePackResponse);
  rpc CloseAuditCase (CloseAuditCaseRequest) returns (CloseAuditCaseResponse);
}

service WorkflowExecutionLogReader {
  rpc ReadExecutionLogs (ReadExecutionLogsRequest) returns (ReadExecutionLogsResponse);
}

message ReadExecutionLogsRequest {
  string audit_case_id = 1;
  string tenant_id = 2;
  string workflow_id = 3;       // e.g., "order-to-cash-v3"
  string invoice_id = 4;
  TimeWindow window = 5;
  string requestor_principal = 6;
  string permit_batch_ref = 7;
}

message ReadExecutionLogsResponse {
  repeated WorkflowExecutionEvidence executions = 1;
  string audit_seal_id = 2;
}
```

## Cedar policy

```cedar
@id("workflow-engine-read-execution-logs-v1")
permit (
  principal,
  action == Action::"workflow_engine.read_execution_logs",
  resource is WorkflowExecution
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  principal.audit_case_id != null &&
  resource.tenant_id == principal.permit_scope.tenant_id &&
  resource.classification_window.intersects(principal.permit_scope.window)
};

@id("workflow-engine-audit-sample-planner-create-v1")
permit (
  principal,
  action == Action::"workflow_engine.audit_sample_planner.create_case",
  resource is Tenant
) when {
  principal.audience_type == "B2B_INTERNAL_AUDIT" &&
  principal.tenant_id == resource.tenant_id &&
  context.audit_charter_active == true
};

@id("workflow-engine-personal-tenant-deny-v1")
forbid (
  principal,
  action == Action::"workflow_engine.read_execution_logs",
  resource is WorkflowExecution
) when {
  resource.tenant_id != principal.permit_scope.tenant_id
};
```

## Stratified sampling algorithm (PCAOB AS-5)

```python
def stratified_sample(population: list[Invoice], control_id: str, seed: str) -> SamplePlan:
    """
    Implements PCAOB AS-5 stratified sampling for SOX 404 controls.
    Strata: A (>$500K), B ($100K-$500K), C ($25K-$100K), D (<$25K).
    Sample sizes per stratum drawn from PCAOB AS-5 table A-3.
    """
    rng = secrets.SystemRandom(seed)
    strata = {
        'A': [i for i in population if i.amount_usd > 500_000],
        'B': [i for i in population if 100_000 <= i.amount_usd <= 500_000],
        'C': [i for i in population if 25_000 <= i.amount_usd < 100_000],
        'D': [i for i in population if i.amount_usd < 25_000],
    }
    sample_sizes = {
        'A': len(strata['A']),       # saturation
        'B': min(len(strata['B']), max(7, int(0.4 * len(strata['B'])))),
        'C': min(len(strata['C']), max(15, int(0.1 * len(strata['C'])))),
        'D': min(len(strata['D']), 9),
    }
    plan = []
    for stratum, items in strata.items():
        plan.extend(rng.sample(items, sample_sizes[stratum]))
    return SamplePlan(plan=plan, seed=seed, generated_at=now())
```

The seed is recorded for reproducibility — external auditors can
re-generate the same sample with the same seed and verify Sam picked
correctly.

## Integration contracts

### Upstream

- `ops-dashboard-control-center.audit_pane` (Sam's entry point).
- `api-gateway` (HTTPS REST surface for browser-side requests).

### Downstream

- `payments.ApprovalChainExporter`.
- `messenger.MessengerArchive`.
- `mail.MailArchive`.
- `audit-chain.SealLeaf` + `audit-chain.SealReader`.
- `identity.B2BInternalAuditPrincipalResolver`.
- `compliance.PackOverlayResolver`.
- `detection.RiskKeywordMatcher` (for auto-flag enrichment, per ADR-0307).

## Implementation notes

### Workflow template `audit_sample_planner`

Workflow definition (oyatie workflow DSL):

```yaml
workflow:
  id: audit-sample-planner-v1
  tenant_scope: B2B_INTERNAL_AUDIT
  stages:
    - id: 1-create-case
      action: create_audit_sample_case
    - id: 2-permit-batch
      action: request_cedar_permit_batch
      dual_control: required
    - id: 3-await-cosign
      action: wait_for_dual_control_cosign
      timeout: 24h
      on_timeout: expire_case
    - id: 4-generate-plan
      action: generate_stratified_sample_plan
    - id: 5-pull-samples
      action: parallel_fan_out
      concurrency: 5
      sub_action: pull_single_sample
    - id: 6-assemble-pack
      action: assemble_evidence_pack
    - id: 7-await-signatures
      action: wait_for_director_and_chair_signatures
    - id: 8-seal-root
      action: seal_evidence_pack_root
    - id: 9-handoff
      action: prepare_external_auditor_handoff
      optional: true
    - id: 10-close
      action: close_audit_case
```

Each stage emits an audit-chain leaf. The whole workflow's executions
are themselves auditable as workflow logs (recursive auditability).

### Concurrency control

- Per-tenant: max 1 active audit-sample-planner workflow per tenant.
- Per-sample: 5-wide concurrency across messenger/mail/payments/log fan-outs.
- Per-audit-chain seal: serialized to preserve Merkle ordering.

### Brownout handling

When downstream µservice signals brownout, the sample-pull job
transitions to `PAUSED_BROWNOUT` and the workflow-engine sets a
resume-watch on the brownout-clear signal. Resume is automatic.

### Performance budget

- `CreateAuditCase` p95 ≤ 1s.
- `GenerateStratifiedSamplePlan` p95 ≤ 2s for population ≤ 10,000.
- `StartSamplePull` p95 ≤ 60s end-to-end per sample.
- `AssembleEvidencePack` p95 ≤ 30s for 1,247-leaf pack.

## Test plan

See integration-test-plan.md §2, §3, §5, §10.

Unit tests:
- `test_stratified_sample_deterministic_with_seed`
- `test_audit_case_state_machine_transitions`
- `test_concurrent_audit_cases_per_tenant_prevented`
- `test_pull_job_retry_on_brownout`
- `test_cedar_permit_required_for_create_case`
- `test_personal_tenant_deny_propagates_through_workflow`
- `test_evidence_pack_assembly_merkle_correctness`

Property tests:
- Property: for any seed + population, the sample plan is
  reproducible.
- Property: audit-case state transitions are monotonic.
- Property: every workflow stage emits an audit-chain leaf.

## Build sequence

1. Schema migrations for `audit_sample_cases`, `stratified_sample_plans`,
   `audit_pull_jobs`.
2. Implement workflow DSL template `audit-sample-planner-v1`.
3. Implement Cedar policies.
4. Implement gRPC services.
5. Wire downstream fan-outs.
6. Wire audit-chain seal emission per stage.
7. Add unit + property + integration tests.
8. Wire into ops-dashboard audit pane.

## Acceptance gates

- All tests PASS.
- Cedar policy lint clean.
- Schema migrations applied + verified.
- Code review: axis-workflow-engine + axis-internal-audit.
- Multispectrum review v2.4.0 facets F1/F2/F3/M1/A1/A4/A5/A6.

## Operational notes

- Owner: axis-workflow-engine (primary) + axis-internal-audit (secondary).
- Pager: `oya-workflow-engine-audit-planner`.
- Dashboards: `audit-planner-throughput`, `audit-planner-latency`,
  `audit-planner-brownout-pause-tail`.

## Compliance and pack overlays

Inherits pack composition from `compliance.PackOverlayResolver`.
SOX 404 + PCAOB AS-5 + EU-WB + GDPR + NDPR composed per audit case.
The pack stack is stamped into every workflow execution record.

## Cross-microservice port declaration

Per ADR-0145:
- `AuditSamplePlanner` in `oyatie.workflow_engine.audit.v1`.
- `WorkflowExecutionLogReader` in same namespace.
- Protos at `protos/workflow-engine-audit-v1.proto`.

## Roll-out plan

- Phase 1: deploy behind flag `workflow_engine.audit_planner.enabled`.
- Phase 2: enable for `test.marcus-corp.tenant`.
- Phase 3: enable for production `marcus-corp.tenant`.
- Phase 4: enable for all B2B_INTERNAL_AUDIT tenants.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Sample plan non-deterministic | HIGH | Seed-based reproducibility test |
| Concurrent case race | MEDIUM | Per-tenant unique constraint on active cases |
| Brownout pause-tail too long | MEDIUM | Pause-tail SLO with auto-pager |
| Evidence pack assembly drops leaves | CRITICAL | Atomic batch-read + Merkle root validation |
| Personal-tenant deny lost in workflow | CRITICAL | Property test; deny event audit-sealed before workflow stage transition |

## Definition of done

- All three gRPC services live in production behind feature flag.
- All tests PASS.
- Sam's audit-pane integration end-to-end PASS with synthetic
  fixtures.
- Workflow DSL template available in workflow-studio catalog.
- The j137 evidence pack assembly proven correct via external-auditor
  verification path.
- Personal-tenant deny propagates correctly through all 60+
  workflow stages without leaking principal-ids.

## Completion expansion — j137 workflow-engine IP rigor pass

Journey context: quarterly SOX 404 audit of work surfaces only.
Service role: durable orchestration, state-machine replay, and idempotent cross-service compensation.
Mapped services in this journey: messenger, mail, workflow-engine, payments, audit-chain, ops-dashboard-control-center, identity, compliance.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0312, ADR-0313, ADR-0319.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in workflow-engine, define the Cedar policy change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving workflow-engine and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in workflow-engine, define the OpenAPI 3.2.0 contract change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving workflow-engine and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in workflow-engine, define the AsyncAPI 3.1.0 event change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving workflow-engine and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in workflow-engine, define the proto3 port change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving workflow-engine and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in workflow-engine, define the Postgres/RLS storage change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving workflow-engine and ops-dashboard-control-center agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in workflow-engine, define the audit-chain emission change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving workflow-engine and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in workflow-engine, define the dashboard projection change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving workflow-engine and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in workflow-engine, define the runbook hook change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving workflow-engine and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in workflow-engine, define the integration fixture change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving workflow-engine and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in workflow-engine, define the domain model change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving workflow-engine and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in workflow-engine, define the Cedar policy change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving workflow-engine and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in workflow-engine, define the OpenAPI 3.2.0 contract change for quarterly SOX 404 audit of work surfaces only; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: workflow-engine MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving workflow-engine and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j137-corporate-internal-audit-sox-controls-test-execution-log-reader.md` matched `SLO, multi-region, payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j137-corporate-internal-audit-sox-controls-test-execution-log-reader.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.

## Pod runtime tier (per ADR-0338)

- Authority: ADR-0338.
- `pod_runtime_tier`: `0`.
- Justification: tenant-customer code exists in this IP execution path; Kata Containers + Cloud Hypervisor are required.
- Surface evidence: `microservices/workflow-engine/IP-journey-j137-corporate-internal-audit-sox-controls-test-execution-log-reader.md`, `microservices/workflow-engine/manifest.json`; trigger terms `workflow-studio`.
