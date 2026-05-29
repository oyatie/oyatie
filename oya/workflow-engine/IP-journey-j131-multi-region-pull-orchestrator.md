---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j131-multi-region-pull-orchestrator
journey_id: j131-cross-jurisdiction-audit-eu-vs-kr-discrepancy
microservice: workflow-engine
role: multi-region-pull-orchestrator
status: draft
related_adrs:
  - ADR-0304-cross-jurisdiction-conflict-resolution
date: 2026-05-20
owner_team: axis-workflow-engine
parallel_work_compatibility: Independent
---

# IP-journey-j131-multi-region-pull-orchestrator — Workflow-engine µservice: parallel per-region pull + reconciliation

## Goal

Implement workflow-engine for multi-jurisdiction audit pulls:

1. **`StartMultiRegionPull`** — spawn parallel workflow-engine
   instances in each subsidiary's cell.
2. **Per-region pull** — each instance pulls evidence locally,
   seals per-region, returns PI-free summary.
3. **Reconciliation** — coordinator assembles the final manifest +
   composes reconciliation root.

## Data model

```sql
CREATE TABLE multi_region_pull_workflows (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  docket_id TEXT NOT NULL,
  coordinating_cell_id TEXT NOT NULL,
  per_region_status JSONB NOT NULL,
  status TEXT NOT NULL CHECK (status IN (
    'SPAWNING','PER_REGION_PULLING','RECONCILING','COMPLETED','FAILED'
  )),
  started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  completed_at TIMESTAMPTZ
);
```

## API surface

```protobuf
service WorkflowEngineMultiRegion {
  rpc StartMultiRegionPull (StartMultiRegionPullRequest)
      returns (StartMultiRegionPullResponse);
  rpc SpawnLocalPull (SpawnLocalPullRequest)
      returns (SpawnLocalPullResponse);
  rpc ReturnMetadataSummary (ReturnMetadataSummaryRequest)
      returns (ReturnMetadataSummaryResponse);
}
```

## Files to author

| File | Purpose | Lines |
|---|---|---:|
| `microservices/workflow-engine/src/multi_region/coordinator.rs` | Coordinator | ~340 |
| `microservices/workflow-engine/src/multi_region/local_executor.rs` | Per-cell executor | ~280 |
| `microservices/workflow-engine/src/multi_region/metadata_validator.rs` | PI-free schema validator | ~240 |
| `microservices/workflow-engine/policy/multi-region-pull.cedar` | Cedar permit | ~30 |
| `microservices/workflow-engine/contracts/proto/multi_region.proto` | gRPC defs | ~140 |
| `microservices/workflow-engine/db/migrations/2026-05-20-001-multi-region-pulls.sql` | DDL | ~40 |
| `microservices/workflow-engine/workflow-templates/multi-region-pull-v1.yaml` | YAML | ~180 |
| `microservices/workflow-engine/runbooks/multi-region-pull-partial-failure.md` | Runbook | ~160 |
| `microservices/workflow-engine/tests/integration/multi_region_test.rs` | Tests | ~440 |
| `microservices/workflow-engine/dashboards/multi-region-pull-health.json` | Grafana | ~100 |
| `microservices/workflow-engine/slos/multi-region-pull-completion.openslo.yaml` | SLO | ~40 |

Total: ~1,990 lines.

## Cedar fragments

```cedar
// multi-region-pull.cedar
permit (
  principal is User,
  action == Action::"workflow_engine.start_multi_region_pull",
  resource is Docket
) when {
  principal.audience_type == "INTERNAL_AUDITOR_3PAO" &&
  resource.spans_multiple_jurisdictions == true
};
```

## Integration contracts

| Contract | Direction | Notes |
|---|---|---|
| Cross-region gRPC | coordinator → per-region | mTLS + SPIFFE per ADR-0295 |
| `audit-chain.SealRegionLocalBundle` | per-region executor → audit-chain | Per-region |
| `audit-chain.ComposeReconciliationRoot` | coordinator → audit-chain | Final |

## Latency budget

- Spawn + complete: ≤5 minutes p99 for full audit pull

## Test plan

- Test A.1 — multi-region pull completes
- Test B.4 — metadata cross-region is PI-free
- Test E.1 — reconciliation root verifies

## Observability emissions

- `oya_workflow_engine_multi_region_pull_total{outcome}`
- `oya_workflow_engine_multi_region_per_region_latency_ms{cell}`

## Acceptance criteria

- Per-region parallel execution works.
- PI-free schema enforced.

## Cross-references

- `docs/user-journeys/j131-*/handshake.md`

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j131-multi-region-pull-orchestrator.md` matched `SLO, multi-region, p99`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j131-multi-region-pull-orchestrator.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
