<!-- WAVE 15J-BATCH-2 SCRUB REPORT
  µservice: workflow-engine
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 5
  tier_references_scrubbed: 78
  ADR_0316_citations_replaced: 2
  cellular_criticality_preserved: 8
-->

## Wave 15-IP-substance scrub (2026-05-21)

Bucket: IP-BUCKET-C.
µservice: workflow-engine.
Agent: wave15-ip-substance-codex.

Scope decision:
- Inventoried 111 `microservices/workflow-engine/IP-*.md` files and no `microservices/workflow-engine/ips/` directory.
- Treated the short foundational stamp cluster as the assigned conversion target because Wave 15-IP-substance called out stamped 55-line IP slices and the workflow-engine inventory had a tight 51-70 line cluster with duplicated heading shapes and no counterpart rows.
- Preserved the much larger journey IPs for a separate journey-doc scrub because they are not the stamped 55-line bucket, even though many still show generated structure and should be handled by a later journey-specific pass.

Rewritten in place:
- `microservices/workflow-engine/IP-003-state-machine-kernel-domain.md` — replaced scaffold with transition-kernel/state-checkpoint plan tied to OpenAPI, proto, AsyncAPI, spec-integrity, tenant-scope Cedar, and Temporal/Cadence/Camunda/Step Functions/n8n counterparts.
- `microservices/workflow-engine/IP-007-event-bus-rest-worker-sdk-app.md` — replaced scaffold with tenant-stamped event publish/subscribe/replay, outbox relay, SDK idempotency, AsyncAPI channel, Cedar, and Temporal/AWS Step Functions/n8n/GitHub Actions counterpart closure.
- `microservices/workflow-engine/IP-008-spec-store-usecase-api-adapter-rest-sdk-app.md` — replaced scaffold with append-only spec-store usecase, lifecycle ledger, Ed25519 verification, OpenBao key evidence, OpenAPI/proto routes, and Temporal/Step Functions/Camunda/n8n counterpart closure.
- `microservices/workflow-engine/IP-009-execution-engine-rest-worker-sdk-app.md` — replaced scaffold with run-control REST, worker lease/resume semantics, two-person cancel, SDK idempotency, SLO evidence, and Temporal/Cadence/Step Functions/Argo/n8n counterpart closure.
- `microservices/workflow-engine/IP-010-replay-debugger-backend-kernel-domain.md` — replaced scaffold with pure replay domain, step snapshots, cursor/range validation, diff reporting, replay SLO evidence, and Temporal/Cadence/Camunda/AWS Step Functions/Airflow counterpart closure.
- `microservices/workflow-engine/IP-011-replay-debugger-backend-usecase-adapter.md` — replaced scaffold with replay-session orchestration, Postgres authoritative reads, ClickHouse analytics-only boundary, tenant predicates, audit events, and Temporal/Camunda/AWS Step Functions/Datadog/OpenTelemetry counterpart closure.
- `microservices/workflow-engine/IP-012-replay-debugger-backend-rest-sdk-app.md` — replaced scaffold with replay REST/SDK/streaming surface, auditor read-only scope, cursor resume, payload redaction, app config, and Temporal Web/Camunda Operate/AWS Step Functions/n8n counterpart closure.

Deleted as duplicative:
- none. The targeted foundational IPs each own a distinct bounded-context layer or surface. Deleting any would collapse separate state-machine, event-bus, spec-store, execution, or replay-debugger responsibilities.

Preserved as already more substantive or out of this bucket:
- `IP-001`, `IP-002`, `IP-004`, `IP-005`, `IP-006`, `IP-013`, `IP-014`, and `IP-015` were not rewritten in this pass because they already contain more concrete implementation targets than the short stamp cluster or sit outside the 55-line signature.
- Journey IPs remain for a future journey-template scrub; they were inventoried but not counted as rewritten here.

Verification notes:
- `oya vcs claim` could not run in this shell because `oya` was not on PATH; no claim id was created.
- Rewritten IPs now include real workflow-engine artifacts, ordered implementation steps, acceptance tests/gates, evidence anchors, and explicit counterpart comparison rows.

## Wave 15J-final-cleanup

- Bucket: F-BUCKET-3.
- Action: deleted stale 2026-05-20 coherence audit and feature parity artifacts; scrubbed IP-015 deterministic replay lane plus remaining non-allowed fixture/observability wording.
- Verification: tier-name grep and `capability_tier|max_tier|tier_threshold` grep both return 0 outside remediation notes.
- Follow-ups: none.

## Wave 15-journey-IP substance pass

Agent: wave15-journey-ip-substance-codex.
Scope: workflow-engine long journey IPs with generated row-loop structure.

Inventory:
- Long journey IPs (>200 lines) inventoried before edits: 86.
- Template-loop IPs detected and edited: 42.
- Rows rewritten into bespoke grounded rows: 158.
- Rows deleted as un-grounded/generated loop residue: 2033.
- Counterpart references added: 158.

Grounding used:
- Shared REST surface: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`.
- Shared event surface: `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`.
- Shared proto surface: `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Cedar fragments: `policy/tenant-scope.cedar`, `policy/auditor-scope.cedar`.
- Evidence anchors: workflow-engine OpenSLO files, audit seal references, replay debugger snapshots, and ADR-0263 lifecycle events.

Substance decisions:
- Cross-tenant orchestration IPs j101/j102/j103/j104/j105/j107/j109/j111/j112/j114/j115 now have eight journey-specific rows instead of 68 repeated path-number rows.
- Cadence orchestrator IPs j76-j80 and j82-j90 now have five regulator-specific rows instead of 24 repeated implementation rows.
- Generated completion-expansion loops were removed where the existing top-half plan already contained concrete schema/API/Cedar/workflow detail; rows without backing artifacts were not replaced with new speculation.

Verification:
- Re-ran long-IP inventory and row-label duplicate checks after edits.
- Re-ran targeted greps for `### Deliverable`, `### IP row`, and high-count `Implementation task` loops.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/workflow-engine/ARCHITECTURE.md`
- `microservices/workflow-engine/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/workflow-engine/IP-001-layer-a-postgres-citus-valkey-clickhouse-iac.md`
- `microservices/workflow-engine/IP-005-execution-engine-usecase-durable-execution.md`
- `microservices/workflow-engine/IP-006-event-bus-kernel-domain-adapter.md`
- `microservices/workflow-engine/IP-007-event-bus-rest-worker-sdk-app.md`
- `microservices/workflow-engine/PHASE-01-DURABLE-EXECUTION-SUBSTRATE.md`
- `microservices/workflow-engine/PRD.md`
- `microservices/workflow-engine/benchmarks/temporal-camunda-airflow-step-functions-vs-oyatie.md`
- `microservices/workflow-engine/capacity-model.md`
- `microservices/workflow-engine/catalog/oya-workflow-engine-event-bus-adapter-valkey.yaml`
- `microservices/workflow-engine/catalog/oya-workflow-engine-execution-engine-adapter-valkey.yaml`
- `microservices/workflow-engine/compliance.md`
- `microservices/workflow-engine/failure-modes.md`
- `microservices/workflow-engine/faqs/workflow-engineer-faq.md`
- `microservices/workflow-engine/iac/helm/valkey/Chart.yaml`
- `microservices/workflow-engine/iac/helm/valkey/values.yaml`
- `microservices/workflow-engine/iac/helm/workflow-runtime/values.yaml`
- `microservices/workflow-engine/iac/kustomize/base/kustomization.yaml`
- `microservices/workflow-engine/iac/kustomize/overlays/pack-kr/kustomization.yaml`
- `microservices/workflow-engine/manifest.json`
- `microservices/workflow-engine/multi-region.md`
- `microservices/workflow-engine/onboarding/workflow-engineer-first-week.md`
- `microservices/workflow-engine/performance-benchmark-numbers-2026-05-20.md`
- `microservices/workflow-engine/policy/data-residency.md`
- `microservices/workflow-engine/runbooks/valkey-failover.md`

Counterpart-fact preservations:
- none

Files renamed (git mv):
- `microservices/workflow-engine/IP-001-layer-a-postgres-citus-redis-clickhouse-iac.md` -> `microservices/workflow-engine/IP-001-layer-a-postgres-citus-valkey-clickhouse-iac.md`
- `microservices/workflow-engine/catalog/oya-workflow-engine-event-bus-adapter-redis.yaml` -> `microservices/workflow-engine/catalog/oya-workflow-engine-event-bus-adapter-valkey.yaml`
- `microservices/workflow-engine/catalog/oya-workflow-engine-execution-engine-adapter-redis.yaml` -> `microservices/workflow-engine/catalog/oya-workflow-engine-execution-engine-adapter-valkey.yaml`
- `microservices/workflow-engine/iac/helm/redis/` -> `microservices/workflow-engine/iac/helm/valkey/`
- `microservices/workflow-engine/runbooks/redis-failover.md` -> `microservices/workflow-engine/runbooks/valkey-failover.md`

## Wave 15-doctrine-propagation-IPs (2026-05-21)

D4-BUCKET-1 trigger-based IP doctrine propagation.

- Root IPs scanned: 111
- Trigger A additions: 76
- Trigger B additions: 78
- Trigger C additions: 69
- Trigger D additions: 11
- Root IPs unmatched: 6
- Doctrine sources: ADR-0338, ADR-0342, ADR-0343, ADR-0344, ADR-0345; `specs/compliance-pack-floors.json`.
- Idempotence: skipped any IP section that already existed; no unmatched root IPs were edited.

IP-by-IP changes:
- `microservices/workflow-engine/IP-001-layer-a-postgres-citus-valkey-clickhouse-iac.md`: added DR posture.
- `microservices/workflow-engine/IP-003-state-machine-kernel-domain.md`: added API Versioning.
- `microservices/workflow-engine/IP-005-execution-engine-usecase-durable-execution.md`: added Sustainability emission, Pod runtime tier.
- `microservices/workflow-engine/IP-006-event-bus-kernel-domain-adapter.md`: added API Versioning.
- `microservices/workflow-engine/IP-007-event-bus-rest-worker-sdk-app.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-008-spec-store-usecase-api-adapter-rest-sdk-app.md`: added API Versioning.
- `microservices/workflow-engine/IP-009-execution-engine-rest-worker-sdk-app.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-010-replay-debugger-backend-kernel-domain.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-011-replay-debugger-backend-usecase-adapter.md`: added API Versioning.
- `microservices/workflow-engine/IP-012-replay-debugger-backend-rest-sdk-app.md`: added API Versioning.
- `microservices/workflow-engine/IP-013-observability-slo-manifests.md`: added DR posture.
- `microservices/workflow-engine/IP-014-branch-protection-and-hyperscaler-gates.md`: added DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j01-emergency-911-dispatch-er-intake.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-journey-j02-healthcare-code-blue-ehr-break-glass-code-blue-workflow.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j08-cooloff-state-machine.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j100-pack-rollout-first-action.md`: added Sustainability emission.
- `microservices/workflow-engine/IP-journey-j101-cross-tenant-orchestration.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j102-cross-tenant-orchestration.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j103-cross-tenant-orchestration.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j104-cross-tenant-orchestration.md`: added API Versioning, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j105-cross-tenant-orchestration.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j107-cross-tenant-orchestration.md`: added API Versioning, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j109-cross-tenant-orchestration.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j111-cross-tenant-orchestration.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j112-cross-tenant-orchestration.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j114-cross-tenant-orchestration.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j115-cross-tenant-orchestration.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/workflow-engine/IP-journey-j117-incident-response-orchestrator.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j120-hedge-approval-state-machine.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j121-loan-underwriting-dag.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j122-approval-and-release-state-machine.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j123-cross-tenant-launch-dag.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-journey-j124-four-tenant-emergency-dag.md`: added API Versioning.
- `microservices/workflow-engine/IP-journey-j125-close-day-state-machine.md`: added API Versioning, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j127-offboarding-orchestrator.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j128-personal-tax-orchestrator.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/workflow-engine/IP-journey-j129-warrant-orchestrator.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j131-multi-region-pull-orchestrator.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j132-mass-hiring-cascade.md`: added DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j133-rif-cascade.md`: added DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j134-engagement-workflow-and-replacement-guarantee.md`: added DR posture.
- `microservices/workflow-engine/IP-journey-j135-investigation-orchestration.md`: added DR posture.
- `microservices/workflow-engine/IP-journey-j136-open-enrollment-orchestration.md`: added DR posture.
- `microservices/workflow-engine/IP-journey-j137-corporate-internal-audit-sox-controls-test-execution-log-reader.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/workflow-engine/IP-journey-j138-corporate-audit-investigation-case-orchestrator.md`: added DR posture.
- `microservices/workflow-engine/IP-journey-j14-delegated-agent-runner.md`: added API Versioning, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j142-offboarding-state-machine.md`: added DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j144-personal-pipeline-runtime.md`: added DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j145-hiring-decision-template.md`: added DR posture.
- `microservices/workflow-engine/IP-journey-j147-cohort-governance-and-cross-tenant-referrals.md`: added DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j148-recycling-route-dag.md`: added API Versioning, DR posture, Pod runtime tier.
- `microservices/workflow-engine/IP-journey-j149-tax-and-availability-automation.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j18-mandatory-report-routing.md`: added API Versioning, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j29-label-filing-runner.md`: added Sustainability emission, Pod runtime tier.
- `microservices/workflow-engine/IP-journey-j36-approval-cascade-runtime.md`: added DR posture, Pod runtime tier.
- `microservices/workflow-engine/IP-journey-j41-deployment-workflow.md`: added DR posture, Pod runtime tier.
- `microservices/workflow-engine/IP-journey-j46-prescriber-routing.md`: added DR posture, Pod runtime tier.
- `microservices/workflow-engine/IP-journey-j50-hiring-onboarding-flow.md`: added DR posture.
- `microservices/workflow-engine/IP-journey-j51-approval-cascade.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j52-shipping-orchestration.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j53-dunning-cascade.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j54-contract-generation.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j55-dispute-routing.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j56-interview-loop.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j57-onboarding-checklist.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j58-review-routing.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j59-offboarding-checklist.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j60-promotion-routing.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j61-specialist-routing.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j62-pharmacy-routing.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j63-screening-flow.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j64-referral-tracking.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j65-dsar-orchestration.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j66-jurisdiction-workflows.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j67-legal-review.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j68-audit-routing.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j69-task-execution.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j70-legal-review.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j71-fraud-review.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j72-locale-routing.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j74-plugin-workflow-actions.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/workflow-engine/IP-journey-j75-tenant-notification.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/workflow-engine/IP-journey-j76-cadence-orchestrator.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-journey-j77-cadence-orchestrator.md`: added API Versioning.
- `microservices/workflow-engine/IP-journey-j78-cadence-orchestrator.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-journey-j79-cadence-orchestrator.md`: added API Versioning.
- `microservices/workflow-engine/IP-journey-j80-cadence-orchestrator.md`: added API Versioning.
- `microservices/workflow-engine/IP-journey-j82-cadence-orchestrator.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-journey-j83-cadence-orchestrator.md`: added API Versioning.
- `microservices/workflow-engine/IP-journey-j84-cadence-orchestrator.md`: added API Versioning.
- `microservices/workflow-engine/IP-journey-j85-cadence-orchestrator.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-journey-j86-cadence-orchestrator.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-journey-j87-cadence-orchestrator.md`: added API Versioning.
- `microservices/workflow-engine/IP-journey-j88-cadence-orchestrator.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-journey-j89-cadence-orchestrator.md`: added API Versioning.
- `microservices/workflow-engine/IP-journey-j90-cadence-orchestrator.md`: added API Versioning, DR posture.
- `microservices/workflow-engine/IP-journey-j91-us-msb-mtl-overlay.md`: added DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j92-br-lgpd-us-parent-dsar.md`: added Sustainability emission.
- `microservices/workflow-engine/IP-journey-j93-in-dpdpa-rbi-overlay.md`: added DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j94-sox404-public-company-controls.md`: added DR posture, Sustainability emission.
- `microservices/workflow-engine/IP-journey-j95-iso27001-soc2-annual-audit.md`: added Sustainability emission.
- `microservices/workflow-engine/IP-journey-j96-ksa-uae-mena-onboarding.md`: added Sustainability emission.
- `microservices/workflow-engine/IP-journey-j97-sg-pdpa-mas-tenant.md`: added Sustainability emission.
- `microservices/workflow-engine/IP-journey-j98-au-privacy-apra-cps234.md`: added Sustainability emission.
- `microservices/workflow-engine/IP-journey-j99-multi-pack-conflict-resolution.md`: added Sustainability emission.


## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- baseline_cpu_per_tenant: 0.6 vCPU; baseline_ram_per_tenant: 768 MiB; storage_per_tenant: 12 GB.
- connections_per_tenant: valkey=4, postgres=6, outbound_http=12.
- scaling_dimension: per_workflow_run; cell_placement_class: Tier-0.
- ADR: ADR-0340 capacity-model doctrine plus ADR-0248 cell criticality numbering.
- Why: 0.6 vCPU / 768 MiB / 12 GB is reserved because replay, step dispatch, Citus writes, Valkey leases, and ClickHouse trace fan-out are per active run.
- Rejected: per_request sizing was rejected because retries and deterministic replay would hide the workflow-run state that actually drives capacity.
- Cost: Tier-0 cellular placement commits this service to sandbox/nodepool overhead and tighter admission controls for all cells that host tenant workflow execution.

### Block 2: dr
- rto_p99_seconds: 2100; rpo_p99_seconds: 5; multi_region_active_active: false.
- backup_substrate: postgres_wal_g, valkey, clickhouse_iceberg_layered, object_storage_versioned; failover_runbook: runbooks/durable-execution-restart.md; replication_shape: active-passive-cross-region-continuous.
- ADR: ADR-0343 recoverability doctrine and compliance-pack floors.
- Why: RTO 2100s follows the documented 35 minute recovery target; RPO 5s follows workflow state durability because losing completed steps would corrupt replay.
- Rejected: active-active writers were rejected because deterministic workflow replay needs a single-writer recovery shape.
- Cost: Recovery SLOs now require drill evidence that proves the declared substrate set, not only service process restart.

### Block 3: pod_runtime_tier
- pod_runtime_tier: 0; evidence: microservices/workflow-engine/PRD.md, microservices/workflow-engine/ARCHITECTURE.md, microservices/workflow-engine/IP-003-state-machine-kernel-domain.md, microservices/workflow-engine/IP-004-execution-engine-kernel-domain.md.
- ADR: ADR-0338 pod runtime tier doctrine and ADR-0340 D-6 cell/runtime co-variance.
- Why: Workflow Engine is the tenant workflow execution substrate and includes tenant-authored workflow/custom-node execution surfaces, so ADR-0338 requires Tier 0 isolation rather than the first-party app default.
- Rejected: Tier 2 first-party app placement was rejected because the engine, unlike Studio, owns tenant-authored execution.
- Cost: Admission, scheduling, and isolation tests must preserve this tier when runtime surfaces move.

### Block 4: tenant_version_pinning
- declared_versions: 2025-11-21, 2026-02-21, 2026-05-21; default_version: 2026-05-21.
- supported_window_size: 3; supported_window_minimum_days: 180; supports_per_tenant_pinning: true.
- ADR: ADR-0342 tenant version pinning doctrine.
- Why: Public contracts are tenant-visible and must remain selectable across the minimum support window.
- Rejected: unpinned workflow contract drift was rejected because tenants may have long-running workflows across release boundaries.
- Cost: Release work must carry compatibility tests and deprecation-calendar updates before any breaking contract change.

### Block 5: consumes_upstream_oss
- consumes_upstream_oss: postgresql, valkey, clickhouse, kafka, cedar, openbao, wasmtime, opentofu.
- oss_stewardship_class_overrides: none; registry defaults in specs/oss-stewardship-registry.json remain authoritative.
- ADR: ADR-0345 OSS stewardship doctrine.
- Why: Wasmtime is declared alongside Postgres, Valkey, ClickHouse, Kafka, Cedar, OpenBao, and OpenTofu because the service combines sandboxed workflow code with state, event, policy, secret, and IaC substrates.
- Rejected: service-local stewardship classes without registry backing.
- Cost: CVE response ownership must follow the registry/default ownership for every declared upstream.

### Block 6: iac_module_invocations
- iac_module_invocations: oci-guest/k8s-namespace-bootstrap@v1, oci-guest/secrets-bootstrap@v1, oci-guest/dns@v1.
- ADR: ADR-0339 shared IaC module doctrine.
- Why: Namespace, secret, and DNS modules are declared because the runtime exposes externally addressed workflow APIs while keeping tenant execution secrets out of service-local Terraform.
- Rejected: service-local bespoke Terraform modules were rejected because ADR-0339 centralizes shared cloud primitives under cloud-iac.
- Cost: Cloud primitive changes now flow through shared module pins instead of service-local drift.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-6. PRD updated: `microservices/workflow-engine/PRD.md`. Related ADRs added: ADR-0337, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345.
- DR posture (ADR-0343): values: manifest RTO p99 2100s, RPO p99 5s, multi_region_active_active=false, active-passive-cross-region-continuous replication, `runbooks/durable-execution-restart.md`, with EU-AI high-risk 1800s floor recorded as admission/runbook follow-up. Alternative rejected: overriding D-2 manifest values from PRD prose. Cost: high-risk placement must refuse or tighten RTO before admission.
- Capacity model (ADR-0340): values: manifest 0.6 vCPU, 768 MiB RAM, 12 GB storage, valkey=4, postgres=6, outbound_http=12, `per_workflow_run` scaling, Tier-0 placement, 3-pod worker floor with capacity toward 500k active runs and 200k steps/s per cell. Alternative rejected: one global worker pool or Tier-2 substrate placement. Cost: tenant/cell admission, sandbox/nodepool overhead, and replay isolation.
- Sustainability and cost attribution (ADR-0344): values: per-call `cost_usd_minor_units`, `co2_grams`, `watt_hours` on run, step, event, replay, timer, adapter, and seal rows. Alternative rejected: treating workflow-engine as unallocated shared substrate overhead. Cost: every hot-path step audit must carry finops dimensions.
- API versioning posture (ADR-0342): values: public `YYYY-MM-DD` carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for specs/runs/adapters/SDKs, ADR-0145 internal mesh exemption. Alternative rejected: SDK-only workflow versioning. Cost: long-lived workflow compatibility and replay fixtures.
