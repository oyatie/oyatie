---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j128-personal-tax-workflow
journey_id: j128-auditor-personal-side-uses-workflow-studio-for-family-taxes
microservice: workflow-studio
role: personal-tax-workflow
status: draft
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0245-substrate-vs-product-layering
  - ADR-0263-observability-emission-contract
date: 2026-05-20
owner_team: axis-workflow-studio + axis-personal-tenant
parallel_work_compatibility: |
  workflow-studio is tenant-scoped substrate. j128 IP extends with
  tax-domain connector library. Independent of j126 (work tenant)
  + j130 (Community surface).
---

# IP-journey-j128-personal-tax-workflow — Workflow Studio: tax-domain connector library + canvas surface for personal tenant

## Goal

Implement Workflow Studio surfaces for personal-tenant tax-domain
workflows:

1. **Tax-domain connector library** — first-class entries for IRS MeF,
   VA DOR, CA FTB, Vanguard, Fidelity, Stripe Consumer,
   Schwab, Robinhood. Each connector ships with metadata for the
   personal-tenant context-picker.
2. **Tax-workflow templates** — pre-built `family-tax-202X` template
   structure that users can clone.
3. **Per-step pause-for-review** — UI for the review checkpoint at
   step 10 of the tax-workflow DAG.

## Data model

```sql
CREATE TABLE workflow_studio_connectors (
  id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  category TEXT NOT NULL CHECK (category IN ('financial','tax','employer','marketplace','utility')),
  authentication_method TEXT NOT NULL CHECK (authentication_method IN ('oauth2','api-key','cross-tenant-permit','sso')),
  applicable_tenants TEXT NOT NULL CHECK (applicable_tenants IN ('personal-only','work-only','all')),
  required_packs TEXT[],
  per_call_cost_estimate_usd NUMERIC(10,4)
);

CREATE TABLE workflow_studio_user_workflows (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id TEXT NOT NULL,
  owner_principal_id TEXT NOT NULL,
  name TEXT NOT NULL,
  dag_definition JSONB NOT NULL,
  template_origin TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_run_at TIMESTAMPTZ,
  schedule_cron TEXT,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  UNIQUE (tenant_id, owner_principal_id, name)
);
```

## API surface

```protobuf
service WorkflowStudioPersonal {
  rpc ListUserWorkflows (ListUserWorkflowsRequest)
      returns (ListUserWorkflowsResponse);
  rpc CreateWorkflowFromTemplate (CreateWorkflowFromTemplateRequest)
      returns (CreateWorkflowFromTemplateResponse);
  rpc UpdateWorkflowDag (UpdateWorkflowDagRequest)
      returns (UpdateWorkflowDagResponse);
  rpc ListConnectors (ListConnectorsRequest)
      returns (ListConnectorsResponse);
}
```

## Files to author

| File | Purpose | Lines |
|---|---|---:|
| `microservices/workflow-studio/src/personal/connector_library.rs` | Connector library impl | ~280 |
| `microservices/workflow-studio/src/personal/tax_template_factory.rs` | Tax-workflow template factory | ~240 |
| `microservices/workflow-studio/src/ui/dag_canvas_personal.tsx` | Canvas React component | ~340 |
| `microservices/workflow-studio/src/ui/pause_for_review_modal.tsx` | Review checkpoint UI | ~200 |
| `microservices/workflow-studio/policy/personal-workflow-list.cedar` | Cedar permit | ~30 |
| `microservices/workflow-studio/policy/personal-workflow-update.cedar` | Cedar permit | ~30 |
| `microservices/workflow-studio/contracts/proto/personal.proto` | gRPC defs | ~120 |
| `microservices/workflow-studio/db/migrations/2026-05-20-001-workflow-studio-personal.sql` | DDL | ~50 |
| `microservices/workflow-studio/templates/family-tax-202X.json` | Tax template | ~280 |
| `microservices/workflow-studio/runbooks/tax-workflow-failure-recovery.md` | Runbook | ~150 |
| `microservices/workflow-studio/tests/integration/personal_tax_test.rs` | Tests | ~360 |
| `microservices/workflow-studio/dashboards/personal-tax-workflow-usage.json` | Grafana | ~100 |
| `microservices/workflow-studio/slos/personal-workflow-canvas-load-latency.openslo.yaml` | SLO | ~40 |

Total approximate: ~2,220 lines.

## Cedar fragments

```cedar
// personal-workflow-list.cedar
permit (
  principal is User,
  action == Action::"workflow_studio.list_user_workflows",
  resource is Tenant
) when {
  principal.tenant == resource.id &&
  principal.id == context.requested_owner_id  // self only by default
};

// personal-workflow-update.cedar
permit (
  principal is User,
  action == Action::"workflow_studio.update_workflow_dag",
  resource is Workflow
) when {
  principal.tenant == resource.tenant &&
  principal.id == resource.owner_principal_id
};
```

## Integration contracts

| Contract | Direction | Notes |
|---|---|---|
| `workflow-engine.StartWorkflow` | workflow-studio → workflow-engine | When user clicks Run |
| `connect.ListAdaptersForTenant` | workflow-studio → connect | For connector library |
| `audit-chain.EmitSealed` | workflow-studio → audit-chain | Per workflow open/edit |

## Latency budget

- `ListUserWorkflows`: ≤120ms p99
- Canvas load: ≤500ms p99 first paint

## Test plan

- Test A.1, A.2 — workflow runs and saves draft
- Test B.4 — GAO principal cannot read personal workflows

## Observability emissions

- `oya_workflow_studio_workflow_opened_total{tenant_id, workflow_id}`
- `oya_workflow_studio_workflow_runs_started_total{tenant_id}`

## Acceptance criteria

- Template renders in canvas.
- Connectors list properly per tenant_class.
- Cedar permits parse.

## Cross-references

- `docs/user-journeys/j128-*/handshake.md`
- ADR-0311, ADR-0245

## Completion expansion — j128 workflow-studio IP rigor pass

Journey context: Diana uses personal Workflow Studio for family taxes outside agency visibility.
Service role: personal or work workflow authoring canvas, template packaging, and UX state projection.
Mapped services in this journey: workflow-studio, workflow-engine, connect, payments, notes, identity.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0314, ADR-0317.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in workflow-studio, define the Cedar policy change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in workflow-studio, define the OpenAPI 3.2.0 contract change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in workflow-studio, define the AsyncAPI 3.1.0 event change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in workflow-studio, define the proto3 port change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving workflow-studio and notes agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in workflow-studio, define the Postgres/RLS storage change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving workflow-studio and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in workflow-studio, define the audit-chain emission change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving workflow-studio and workflow-studio agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in workflow-studio, define the dashboard projection change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in workflow-studio, define the runbook hook change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in workflow-studio, define the integration fixture change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in workflow-studio, define the domain model change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving workflow-studio and notes agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in workflow-studio, define the Cedar policy change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving workflow-studio and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in workflow-studio, define the OpenAPI 3.2.0 contract change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving workflow-studio and workflow-studio agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in workflow-studio, define the AsyncAPI 3.1.0 event change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in workflow-studio, define the proto3 port change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in workflow-studio, define the Postgres/RLS storage change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in workflow-studio, define the audit-chain emission change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving workflow-studio and notes agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in workflow-studio, define the dashboard projection change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving workflow-studio and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in workflow-studio, define the runbook hook change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving workflow-studio and workflow-studio agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 018: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 019: in workflow-studio, define the integration fixture change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 019: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 019: add pack-overlay coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 019: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 020: in workflow-studio, define the domain model change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 020: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 020: add unit coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 020: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 02: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 021: in workflow-studio, define the Cedar policy change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 021: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 021: add property coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 021: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 022: in workflow-studio, define the OpenAPI 3.2.0 contract change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 022: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 022: add contract coverage proving workflow-studio and notes agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 022: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 023: in workflow-studio, define the AsyncAPI 3.1.0 event change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 023: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 023: add integration coverage proving workflow-studio and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 023: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 024: in workflow-studio, define the proto3 port change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 024: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 024: add replay coverage proving workflow-studio and workflow-studio agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 024: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 025: in workflow-studio, define the Postgres/RLS storage change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 025: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 025: add load coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 025: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 026: in workflow-studio, define the audit-chain emission change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 026: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 026: add chaos coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 026: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 027: in workflow-studio, define the dashboard projection change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 027: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 027: add negative authorization coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 027: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 028: in workflow-studio, define the runbook hook change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 028: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 028: add multi-region coverage proving workflow-studio and notes agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 028: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 029: in workflow-studio, define the integration fixture change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 029: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 029: add pack-overlay coverage proving workflow-studio and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 029: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 030: in workflow-studio, define the domain model change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 030: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 030: add unit coverage proving workflow-studio and workflow-studio agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 030: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 03: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 031: in workflow-studio, define the Cedar policy change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 031: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 031: add property coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 031: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 032: in workflow-studio, define the OpenAPI 3.2.0 contract change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 032: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 032: add contract coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 032: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 033: in workflow-studio, define the AsyncAPI 3.1.0 event change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 033: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 033: add integration coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 033: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 034: in workflow-studio, define the proto3 port change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 034: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 034: add replay coverage proving workflow-studio and notes agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 034: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 035: in workflow-studio, define the Postgres/RLS storage change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 035: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 035: add load coverage proving workflow-studio and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 035: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 036: in workflow-studio, define the audit-chain emission change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 036: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 036: add chaos coverage proving workflow-studio and workflow-studio agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 036: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 037: in workflow-studio, define the dashboard projection change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 037: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 037: add negative authorization coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 037: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 038: in workflow-studio, define the runbook hook change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 038: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 038: add multi-region coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 038: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 039: in workflow-studio, define the integration fixture change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 039: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 039: add pack-overlay coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 039: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 040: in workflow-studio, define the domain model change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 040: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 040: add unit coverage proving workflow-studio and notes agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 040: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 04: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 041: in workflow-studio, define the Cedar policy change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 041: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 041: add property coverage proving workflow-studio and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 041: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 042: in workflow-studio, define the OpenAPI 3.2.0 contract change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 042: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 042: add contract coverage proving workflow-studio and workflow-studio agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 042: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 043: in workflow-studio, define the AsyncAPI 3.1.0 event change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 043: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 043: add integration coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 043: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 044: in workflow-studio, define the proto3 port change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 044: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 044: add replay coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 044: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 045: in workflow-studio, define the Postgres/RLS storage change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 045: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 045: add load coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 045: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 046: in workflow-studio, define the audit-chain emission change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 046: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 046: add chaos coverage proving workflow-studio and notes agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 046: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 047: in workflow-studio, define the dashboard projection change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 047: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 047: add negative authorization coverage proving workflow-studio and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 047: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 048: in workflow-studio, define the runbook hook change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 048: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 048: add multi-region coverage proving workflow-studio and workflow-studio agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 048: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 049: in workflow-studio, define the integration fixture change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 049: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 049: add pack-overlay coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 049: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 050: in workflow-studio, define the domain model change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 050: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 050: add unit coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 050: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 05: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 051: in workflow-studio, define the Cedar policy change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 051: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 051: add property coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 051: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 052: in workflow-studio, define the OpenAPI 3.2.0 contract change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 052: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 052: add contract coverage proving workflow-studio and notes agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 052: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 053: in workflow-studio, define the AsyncAPI 3.1.0 event change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 053: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 053: add integration coverage proving workflow-studio and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 053: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 054: in workflow-studio, define the proto3 port change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 054: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 054: add replay coverage proving workflow-studio and workflow-studio agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 054: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 055: in workflow-studio, define the Postgres/RLS storage change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 055: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 055: add load coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 055: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 056: in workflow-studio, define the audit-chain emission change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 056: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 056: add chaos coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 056: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 057: in workflow-studio, define the dashboard projection change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 057: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 057: add negative authorization coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 057: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 058: in workflow-studio, define the runbook hook change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 058: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 058: add multi-region coverage proving workflow-studio and notes agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 058: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 059: in workflow-studio, define the integration fixture change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 059: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 059: add pack-overlay coverage proving workflow-studio and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 059: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 060: in workflow-studio, define the domain model change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 060: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 060: add unit coverage proving workflow-studio and workflow-studio agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 060: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 06: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 061: in workflow-studio, define the Cedar policy change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 061: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 061: add property coverage proving workflow-studio and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 061: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 062: in workflow-studio, define the OpenAPI 3.2.0 contract change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 062: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 062: add contract coverage proving workflow-studio and connect agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 062: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 063: in workflow-studio, define the AsyncAPI 3.1.0 event change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 063: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0314 scope grants it; refusals are success states, not exceptions.
Verification 063: add integration coverage proving workflow-studio and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 063: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 064: in workflow-studio, define the proto3 port change for Diana uses personal Workflow Studio for family taxes outside agency visibility; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 064: workflow-studio MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`.proto`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/workflow-studio/IP-journey-j128-personal-tax-workflow.md` matched [`financial`, `SLO`, `p99`, `payment`, `multi-region`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOX-404`, `SOC2-T2`, `ISO27001-2022`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/workflow-studio/IP-journey-j128-personal-tax-workflow.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/ARCHITECTURE.md`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/multi-region.md`, `microservices/workflow-studio/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/workflow-studio/IP-journey-j128-personal-tax-workflow.md` matched [`emission`, `cost`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/workflow-studio/IP-journey-j128-personal-tax-workflow.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/capacity-model.md`, `microservices/workflow-studio/compliance.md`, `microservices/workflow-studio/ARCHITECTURE.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-journey-j128-personal-tax-workflow.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
