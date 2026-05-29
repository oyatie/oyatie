<!-- WAVE 15J-BATCH-2 SCRUB REPORT
  µservice: workflow-studio
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 2
  tier_references_scrubbed: 39
  ADR_0316_citations_replaced: 2
  cellular_criticality_preserved: 1
-->

## Wave 15-IP-substance scrub (2026-05-21)

- Assignment bucket: IP-BUCKET-K.
- Scope: `microservices/workflow-studio/IP-*.md`.
- Inventoried IPs: 42.
- Detected stamped IPs: 0.
- Rewritten in place: 0.
- Deleted as duplicative: none.
- Preserved as already-substantive: 42.
- Counterpart anchors added: 39 preserved workflow-studio IPs now carry a local `Counterpart Anchors` section naming n8n, Zapier, Make, and Workato as the benchmark envelope; files that already had counterpart references were left alone.
- Verification smoke: no 30-79-line IP shell remains; counterpart grep returns no missing workflow-studio IPs; placeholder grep returns no hits.
- Follow-up: none for Wave 15-IP-substance.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/workflow-studio/ARCHITECTURE.md`
- `microservices/workflow-studio/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/workflow-studio/IP-001-layer-a-cdn-waf-postgres-valkey-ws-gateway-iac.md`
- `microservices/workflow-studio/IP-005-collab-crdt-kernel-domain-adapter.md`
- `microservices/workflow-studio/IP-006-collab-crdt-worker-sdk.md`
- `microservices/workflow-studio/IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md`
- `microservices/workflow-studio/PHASE-01-VISUAL-AUTHORING-SUBSTRATE.md`
- `microservices/workflow-studio/PRD.md`
- `microservices/workflow-studio/capacity-model.md`
- `microservices/workflow-studio/catalog/oya-workflow-studio-collab-crdt-adapter-valkey.yaml`
- `microservices/workflow-studio/catalog/oya-workflow-studio-collab-crdt-worker.yaml`
- `microservices/workflow-studio/compliance.md`
- `microservices/workflow-studio/dashboards/collab-health.json`
- `microservices/workflow-studio/decisions/ADR-WS-0001-crdt-library-selection.md`
- `microservices/workflow-studio/feature-parity-matrix-2026-05-20.md`
- `microservices/workflow-studio/iac/helm/visual-canvas-rest/templates/deployment.yaml`
- `microservices/workflow-studio/iac/helm/visual-canvas-rest/values.yaml`
- `microservices/workflow-studio/iac/kustomize/base/kustomization.yaml`
- `microservices/workflow-studio/iac/kustomize/base/openbao-secret-references.yaml`
- `microservices/workflow-studio/manifest.json`
- `microservices/workflow-studio/migration-playbooks/from-n8n.md`
- `microservices/workflow-studio/multi-region.md`
- `microservices/workflow-studio/onboarding/no-code-builder-first-week.md`
- `microservices/workflow-studio/policy/data-residency.md`

Counterpart-fact preservations:
- `microservices/workflow-studio/feature-parity-matrix-2026-05-20.md` preserves n8n queue-mode Redis references as counterpart-fact.
- `microservices/workflow-studio/migration-playbooks/from-n8n.md` preserves Redis backing as external-redis customer/source-estate inventory.

Files renamed (git mv):
- `microservices/workflow-studio/IP-001-layer-a-cdn-waf-postgres-redis-ws-gateway-iac.md` -> `microservices/workflow-studio/IP-001-layer-a-cdn-waf-postgres-valkey-ws-gateway-iac.md`
- `microservices/workflow-studio/catalog/oya-workflow-studio-collab-crdt-adapter-redis.yaml` -> `microservices/workflow-studio/catalog/oya-workflow-studio-collab-crdt-adapter-valkey.yaml`

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: kept the existing 30s RTO / 1s RPO because ADR-0343 compliance floors (HIPAA/KR protected data at 3600s/300s and SOC2/ISO looser) are weaker than the editor-state claim. Alternative considered: relax to compliance floors; rejected because collaborative authoring would expose tenant-visible draft loss. Cost: active-active editor ingress, CRDT ownership discipline, and failover runbooks remain mandatory.
- Capacity model: set baseline to 0.25 vCPU/512MiB REST, 128MiB CRDT memory per active definition, 20MiB draft plus 50MiB snapshot storage, and two WebSockets per editor under ADR-0340. Alternative considered: size by generic tenant tier; rejected because session count and spec size drive real load. Cost: admission control must throttle workshops before the 100,000-session regional budget is exhausted.
- Sustainability + cost attribution: added ADR-0344 per-call `cost_usd_minor_units`, `co2_grams`, and `watt_hours` for editor and LLM-assist audit rows. Alternative considered: aggregate CDN/WASM and LLM spend after the fact; rejected because CSRD/SB-253/SEC exports need the tenant operation, cell, and provider dimensions. Cost: audit payloads and FinOps rollups grow on every authoring event.
- API versioning posture: adopted ADR-0342 date carrier triplet plus SDK semver and tenant pinning for Studio clients. Alternative considered: SDK semver only; rejected because long-lived workflow authoring integrations need a server-side contract date. Cost: three supported date versions for at least 180 days.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

- bucket: `D4-BUCKET-4`
- selection: trigger-matched `IP-*.md` only; unmatched IPs unchanged.
- scanned_ips: `42`; changed_ips: `42`; unmatched_ips: `0`.
- doctrine_sections: ADR-0342 API Versioning, ADR-0343 DR posture, ADR-0344 Sustainability emission, ADR-0338 Pod runtime tier.

| IP | Trigger matches | Sections added |
|---|---|---|
| `IP-001-layer-a-cdn-waf-postgres-valkey-ws-gateway-iac.md` | B HA-critical, C metered, D tenant-customer code | DR posture, Sustainability emission, Pod runtime tier |
| `IP-002-visual-canvas-kernel-domain.md` | D tenant-customer code | Pod runtime tier |
| `IP-003-dsl-emitter-loader-kernel-domain.md` | D tenant-customer code | Pod runtime tier |
| `IP-004-dsl-emitter-loader-usecase-api-adapter-sdk.md` | D tenant-customer code | Pod runtime tier |
| `IP-005-collab-crdt-kernel-domain-adapter.md` | D tenant-customer code | Pod runtime tier |
| `IP-006-collab-crdt-worker-sdk.md` | A contracts, D tenant-customer code | API Versioning, Pod runtime tier |
| `IP-007-node-library-registry-full.md` | D tenant-customer code | Pod runtime tier |
| `IP-008-llm-assist-adapter.md` | D tenant-customer code | Pod runtime tier |
| `IP-009-license-gate-cedar-full.md` | C metered, D tenant-customer code | Sustainability emission, Pod runtime tier |
| `IP-010-jurisdiction-overlay-renderer-full.md` | D tenant-customer code | Pod runtime tier |
| `IP-011-replay-debugger-frontend-full.md` | A contracts, D tenant-customer code | API Versioning, Pod runtime tier |
| `IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-013-observability-slo-manifests.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-014-branch-protection-and-hyperscaler-gates.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-015-hg-workflow-studio-registration-final.md` | B HA-critical, C metered, D tenant-customer code | DR posture, Sustainability emission, Pod runtime tier |
| `IP-016-svelte-flow-canvas-integration.md` | A contracts, B HA-critical, D tenant-customer code | API Versioning, DR posture, Pod runtime tier |
| `IP-017-leptos-canvas-scaffold.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-018-swiftui-canvas-impl.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-019-compose-canvas-impl.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-020-gtk-drawingarea-impl.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-021-winui-canvas-impl.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-022-loro-crdt-sync-binding.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-023-presence-awareness-protocol.md` | C metered, D tenant-customer code | Sustainability emission, Pod runtime tier |
| `IP-024-1000-node-perf-bench.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-025-codemirror-6-integration.md` | D tenant-customer code | Pod runtime tier |
| `IP-026-lsp-bridge.md` | D tenant-customer code | Pod runtime tier |
| `IP-027-cedar-grammar-impl.md` | D tenant-customer code | Pod runtime tier |
| `IP-journey-j100-pack-rollout-first-action.md` | C metered, D tenant-customer code | Sustainability emission, Pod runtime tier |
| `IP-journey-j128-personal-tax-workflow.md` | A contracts, B HA-critical, C metered, D tenant-customer code | API Versioning, DR posture, Sustainability emission, Pod runtime tier |
| `IP-journey-j144-job-search-template-and-canvas.md` | B HA-critical, C metered, D tenant-customer code | DR posture, Sustainability emission, Pod runtime tier |
| `IP-journey-j29-personal-builder-ui.md` | C metered, D tenant-customer code | Sustainability emission, Pod runtime tier |
| `IP-journey-j36-manager-review-console.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-journey-j46-rx-renewal-template.md` | B HA-critical, D tenant-customer code | DR posture, Pod runtime tier |
| `IP-journey-j91-us-msb-mtl-overlay.md` | B HA-critical, C metered, D tenant-customer code | DR posture, Sustainability emission, Pod runtime tier |
| `IP-journey-j92-br-lgpd-us-parent-dsar.md` | C metered, D tenant-customer code | Sustainability emission, Pod runtime tier |
| `IP-journey-j93-in-dpdpa-rbi-overlay.md` | B HA-critical, C metered, D tenant-customer code | DR posture, Sustainability emission, Pod runtime tier |
| `IP-journey-j94-sox404-public-company-controls.md` | B HA-critical, C metered, D tenant-customer code | DR posture, Sustainability emission, Pod runtime tier |
| `IP-journey-j95-iso27001-soc2-annual-audit.md` | C metered, D tenant-customer code | Sustainability emission, Pod runtime tier |
| `IP-journey-j96-ksa-uae-mena-onboarding.md` | C metered, D tenant-customer code | Sustainability emission, Pod runtime tier |
| `IP-journey-j97-sg-pdpa-mas-tenant.md` | C metered, D tenant-customer code | Sustainability emission, Pod runtime tier |
| `IP-journey-j98-au-privacy-apra-cps234.md` | C metered, D tenant-customer code | Sustainability emission, Pod runtime tier |
| `IP-journey-j99-multi-pack-conflict-resolution.md` | C metered, D tenant-customer code | Sustainability emission, Pod runtime tier |


## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- baseline_cpu_per_tenant: 0.18 vCPU; baseline_ram_per_tenant: 384 MiB; storage_per_tenant: 4 GB.
- connections_per_tenant: valkey=3, postgres=3, outbound_http=8.
- scaling_dimension: per_user; cell_placement_class: Tier-3.
- ADR: ADR-0340 capacity-model doctrine plus ADR-0248 cell criticality numbering.
- Why: 0.18 vCPU / 384 MiB / 4 GB reflects collaborative editor state, template metadata, and draft artifacts rather than durable execution.
- Rejected: per_workflow_run sizing was rejected because Studio emits definitions and does not execute workflow steps.
- Cost: Tier-3 cell placement keeps editor capacity cheaper than runtime cells while still preserving tenant-pinned public contracts.

### Block 2: dr
- rto_p99_seconds: 2100; rpo_p99_seconds: 5; multi_region_active_active: false.
- backup_substrate: postgres_wal_g, valkey, object_storage_versioned; failover_runbook: runbooks/run-history-replay-corruption.md; replication_shape: active-passive-cross-region-continuous.
- ADR: ADR-0343 recoverability doctrine and compliance-pack floors.
- Why: RTO 2100s and RPO 5s preserve draft/collaboration state so tenants can resume editing without replay divergence in the engine handoff.
- Rejected: active-active collaboration writes were rejected because the current CRDT and history repair runbooks are documented around active-passive recovery.
- Cost: Recovery SLOs now require drill evidence that proves the declared substrate set, not only service process restart.

### Block 3: pod_runtime_tier
- pod_runtime_tier: 2; evidence: microservices/workflow-studio/PRD.md, microservices/workflow-studio/ARCHITECTURE.md, microservices/workflow-studio/IP-004-dsl-emitter-loader-usecase-api-adapter-sdk.md, microservices/workflow-studio/contracts/openapi/workflow-studio.yaml.
- ADR: ADR-0338 pod runtime tier doctrine and ADR-0340 D-6 cell/runtime co-variance.
- Why: Workflow Studio is a first-party editor and contract emitter; execution is delegated to Workflow Engine, so ADR-0338 keeps the editor surface at Tier 2.
- Rejected: Tier 0 was rejected because tenant workflow code is not executed in Studio.
- Cost: Admission, scheduling, and isolation tests must preserve this tier when runtime surfaces move.

### Block 4: tenant_version_pinning
- declared_versions: 2025-11-21, 2026-02-21, 2026-05-21; default_version: 2026-05-21.
- supported_window_size: 3; supported_window_minimum_days: 180; supports_per_tenant_pinning: true.
- ADR: ADR-0342 tenant version pinning doctrine.
- Why: Public contracts are tenant-visible and must remain selectable across the minimum support window.
- Rejected: a single moving API version was rejected because tenant-generated workflow definitions outlive editor releases.
- Cost: Release work must carry compatibility tests and deprecation-calendar updates before any breaking contract change.

### Block 5: consumes_upstream_oss
- consumes_upstream_oss: postgresql, valkey, kafka, cedar, openbao, opentofu.
- oss_stewardship_class_overrides: none; registry defaults in specs/oss-stewardship-registry.json remain authoritative.
- ADR: ADR-0345 OSS stewardship doctrine.
- Why: Postgres, Valkey, Kafka, Cedar, OpenBao, and OpenTofu cover draft storage, collaboration caches/events, policy, secret references, and shared IaC.
- Rejected: service-local stewardship classes without registry backing.
- Cost: CVE response ownership must follow the registry/default ownership for every declared upstream.

### Block 6: iac_module_invocations
- iac_module_invocations: oci-guest/k8s-namespace-bootstrap@v1, oci-guest/secrets-bootstrap@v1, oci-guest/dns@v1.
- ADR: ADR-0339 shared IaC module doctrine.
- Why: Namespace, secret, and DNS module invocations match the public editor/API surface and its tenant-scoped secret references.
- Rejected: copying Workflow Engine runtime modules was rejected because Studio is editor/control-plane only.
- Cost: Cloud primitive changes now flow through shared module pins instead of service-local drift.
