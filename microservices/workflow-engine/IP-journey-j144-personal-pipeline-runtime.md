---
doc_class: Implementation-Plan-Journey-Slice
journey_id: j144
microservice: workflow-engine
status: draft
date: 2026-05-20
authority_tier: 3
intern_buildable: true
adr_anchors: [ADR-0244, ADR-0245, ADR-0247, ADR-0292, ADR-0311]
---

# workflow-engine — IP slice for j144 (personal pipeline runtime)

## Scope

1. Personal-tenant pipeline runtime — same engine substrate as enterprise, restricted Cedar.
2. **Adapter polling scheduler** with per-source cadence + rate-limit backoff.
3. **Pipeline aggregator** for the weekly digest.
4. **Cross-tenant application-submit router** routing `apply`-marked rows to the correct adapter or Community gRPC.

## API surface

```proto
service Pipeline {
  rpc Deploy(DeployRequest) returns (DeployResponse);
  rpc Observe(ObserveRequest) returns (stream ObserveEvent);
  rpc AddBlock(AddBlockRequest) returns (AddBlockResponse);
  rpc Pause(PauseRequest) returns (PauseResponse);
  rpc Resume(ResumeRequest) returns (ResumeResponse);
}

service Aggregator {
  rpc WeeklyDigest(WeeklyDigestRequest) returns (WeeklyDigestResponse);
}
```

## Implementation tasks

### T1 — Pipeline runtime

- Accept compiled template from Workflow Studio.
- Schedule per-block triggers (cron-like; some are event-driven).
- Maintain pipeline state in tenant-scoped store.

### T2 — Adapter polling scheduler

- Per-source cadence honored; rate-limit backoff with jitter.
- Failures retry; persistent failures surface to UX with yellow indicator.

### T3 — Submission router

- `apply`-marked Notes row → look up source-of-truth posting → call appropriate adapter (for external) or Community `JobApplication.Submit` (for internal cross-tenant).
- Submit attaches: cover letter, résumé, portfolio.
- Audit: `ApplicationSubmitted{employer_tenant, posting_id}`.

### T4 — Weekly digest aggregator

- Cron: Sun 18:00 ET (per Chris's timezone preference).
- Aggregates last 7d from Notes + + Intelligence telemetry.
- Generates structured digest text via Intelligence (with the same transparency floor).

## Cedar permits

| Permit | Granted to | Purpose |
|---|---|---|
| `b2c.workflow_engine.pipeline.deploy` | self | Deploy compiled pipeline |
| `b2c.workflow_engine.pipeline.observe` | self | Real-time observation |
| `b2c.workflow_engine.action.submit_application` | self | Submit on `apply` mark |
| `b2c.workflow_engine.aggregator.weekly_digest` | self | Generate digest |

## Audit emissions

- `JobSearchPipelineActivated`, `PipelineDashboardViewed`
- `AdapterPolled`, `AdapterFailureLogged`
- `ApplicationSubmitted`
- `WeeklyDigestEmitted`

## Performance

- Pipeline deploy p99 ≤ 1s.
- Steady-state per-poll cycle p99 ≤ 30s (poll → filter → drafts → notes write).
- Weekly digest p99 ≤ 30s.

## Acceptance criteria

- [ ] B.1 happy path runs 7 days simulated.
- [ ] B.6 OAuth revocation gracefully degrades.
- [ ] B.7 adapter failure isolated to that source.
- [ ] B.9 weekly digest fires on schedule.

## Out of scope

- Block-specific logic (per-µservice IPs).
- Workflow Studio canvas.

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j144-personal-pipeline-runtime.md` matched `p99`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j144-personal-pipeline-runtime.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
