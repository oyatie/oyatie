<!-- WAVE 15J-BATCH-2 SCRUB REPORT
  µservice: api-gateway
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 2
  tier_references_scrubbed: 48
  ADR_0316_citations_replaced: 4
  cellular_criticality_preserved: 1
-->

## Wave 15-IP-substance scrub (2026-05-21)

- IPs inventoried: 32.
- IPs detected as stamped: 18 foundation IPs were thin or retained the 30-80 line signature during integration.
- IPs rewritten in place: 18 foundation IPs expanded with API-gateway A-G substance, validation matrices, and counterpart anchors.
- IPs deleted as duplicative: 0.
- IPs preserved as already-substantive: 14 journey IPs; preserved and given narrow GitHub/GitLab API-ingress counterpart anchors for verification.
- Counterpart references added: 32.
- Follow-ups: none.

## Wave 15J-final-cleanup

- Bucket: F-BUCKET-3.
- Action: deleted stale 2026-05-20 coherence audit and feature parity artifacts; scrubbed remaining non-allowed fixture wording in IP/architecture prose.
- Verification: tier-name grep and `capability_tier|max_tier|tier_threshold` grep both return 0 outside remediation notes.
- Follow-ups: none.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/api-gateway/ARCHITECTURE.md
- microservices/api-gateway/AUDIT-FINDINGS-2026-05-18.json
- microservices/api-gateway/IP-002-routing-domain-crate.md
- microservices/api-gateway/IP-009-rate-limit-domain-crate.md
- microservices/api-gateway/IP-010-rate-limit-adapter-valkey.md
- microservices/api-gateway/IP-012-abuse-defence-domain.md
- microservices/api-gateway/IP-013-abuse-defence-adapter-wasm.md
- microservices/api-gateway/IP-016-app-supervisor.md
- microservices/api-gateway/IP-journey-j01-emergency-911-dispatch-psap-attestation.md
- microservices/api-gateway/PRD.md
- microservices/api-gateway/benchmarks/api-gateway-vs-envoy-vs-kong-vs-tyk-vs-apigee-vs-aws-api-gateway.md
- microservices/api-gateway/capacity-model.md
- microservices/api-gateway/catalog/oya-api-gateway-rate-limit-adapter-valkey.yaml
- microservices/api-gateway/compliance.md
- microservices/api-gateway/cost-budget.md
- microservices/api-gateway/dashboards/rate-limit-hits.json
- microservices/api-gateway/dashboards/rate-limit-hits.md
- microservices/api-gateway/failure-modes.md
- microservices/api-gateway/iac/network-policy.yaml
- microservices/api-gateway/manifest.json
- microservices/api-gateway/onboarding/edge-engineer-first-week.md
- microservices/api-gateway/performance-benchmark-numbers-2026-05-20.md
- microservices/api-gateway/runbooks/ddos-mitigation.md
- microservices/api-gateway/runbooks/rate-limit-saturation.md

Counterpart-fact preservations:
- microservices/api-gateway/benchmarks/api-gateway-vs-envoy-vs-kong-vs-tyk-vs-apigee-vs-aws-api-gateway.md — Envoy global rate-limit row preserves Redis as an external product benchmark fact.
- microservices/api-gateway/benchmarks/api-gateway-vs-envoy-vs-kong-vs-tyk-vs-apigee-vs-aws-api-gateway.md — Kong Gateway row preserves Redis-backed as an external product benchmark fact.

Files renamed (git mv):
- microservices/api-gateway/IP-010-rate-limit-adapter-redis.md -> microservices/api-gateway/IP-010-rate-limit-adapter-valkey.md
- microservices/api-gateway/catalog/oya-api-gateway-rate-limit-adapter-redis.yaml -> microservices/api-gateway/catalog/oya-api-gateway-rate-limit-adapter-valkey.yaml

## Wave 15-doctrine-propagation-IPs (2026-05-21)

Bucket: D4-BUCKET-3.
Trigger command scope: `microservices/<service>/IP-*.md`.
IPs scanned: 32.
Trigger A matches: 20.
Trigger B matches: 15.
Trigger C matches: 19.
Trigger D matches: 5.

Manifest DR note: when `manifest.json#dr` was absent or unavailable in this checkout, DR posture sections use `specs/compliance-pack-floors.json` floors and mark manifest reconciliation as a follow-up.

IP changes:
- `microservices/api-gateway/IP-001-api-gateway-design-readiness.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-002-routing-domain-crate.md`: Trigger A -> API Versioning; Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-003-routing-kernel-crate.md`: Trigger A -> API Versioning; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/api-gateway/IP-004-routing-usecase-crate.md`: Trigger A -> API Versioning; Trigger D -> Pod runtime tier.
- `microservices/api-gateway/IP-005-routing-adapter-crate.md`: Trigger A -> API Versioning; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/api-gateway/IP-006-routing-rest-crate.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `microservices/api-gateway/IP-007-routing-grpc-crate.md`: Trigger A -> API Versioning.
- `microservices/api-gateway/IP-008-routing-worker-crate.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-009-rate-limit-domain-crate.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger D -> Pod runtime tier.
- `microservices/api-gateway/IP-010-rate-limit-adapter-valkey.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `microservices/api-gateway/IP-011-auth-handoff-usecase.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `microservices/api-gateway/IP-012-abuse-defence-domain.md`: Trigger A -> API Versioning.
- `microservices/api-gateway/IP-013-abuse-defence-adapter-wasm.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `microservices/api-gateway/IP-014-tls-cert-rotation-worker.md`: Trigger B -> DR posture.
- `microservices/api-gateway/IP-015-canary-cohort-shifter.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-016-app-supervisor.md`: Trigger B -> DR posture.
- `microservices/api-gateway/IP-017-sov-cell-routing.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `microservices/api-gateway/IP-018-honeypot-route-mgr.md`: Trigger A -> API Versioning.
- `microservices/api-gateway/IP-journey-j01-emergency-911-dispatch-psap-attestation.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission; Trigger D -> Pod runtime tier.
- `microservices/api-gateway/IP-journey-j03-crisis-line-bypass.md`: Trigger A -> API Versioning; Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j100-pack-rollout-first-action.md`: Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j12-emergency-services-elevated-rate-limit.md`: Trigger A -> API Versioning; Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j78-edge-contract-gate.md`: Trigger A -> API Versioning.
- `microservices/api-gateway/IP-journey-j91-us-msb-mtl-overlay.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j92-br-lgpd-us-parent-dsar.md`: Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j93-in-dpdpa-rbi-overlay.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j94-sox404-public-company-controls.md`: Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j95-iso27001-soc2-annual-audit.md`: Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j96-ksa-uae-mena-onboarding.md`: Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j97-sg-pdpa-mas-tenant.md`: Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j98-au-privacy-apra-cps234.md`: Trigger C -> Sustainability emission.
- `microservices/api-gateway/IP-journey-j99-multi-pack-conflict-resolution.md`: Trigger C -> Sustainability emission.

Unmatched IPs:
- none.

Follow-ups:
- Reconcile `manifest.json#dr` numeric service targets when the D-2 manifest DR fields land for this service.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now records manifest RTO 300 s / RPO 0 s, cites HIPAA/EU-AI/SOC2 floors, names `runbooks/cell-evac.md`, blue-green rollback, and edge-admission regression runbooks, and preserves manifest Tier-0 cell eligibility per ADR-0343. Alternative rejected: workload-tier failover only, because tenants hit the edge before any workload. Cost: active-active edge cells and hot route/JWKS caches.
- Capacity model: PRD now binds manifest values 0.10 vCPU, 128 MiB RAM, 0 GB durable storage, connections `{valkey:6, postgres:0, outbound_http:16}`, per-request scaling, Tier-4 placement in Tier-0 cells, 50K TLS handshakes/sec, 5M connections, 250K HTTP req/sec, and ADR-0338 Tier-3 runtime to ADR-0340. Alternative rejected: average RPS sizing, because DDoS and tenant hot keys define the edge envelope. Cost: four-node per-cell floor and shuffle-sharded Valkey capacity.
- Sustainability + cost attribution: PRD now requires ADR-0344 FinOps fields on admitted/denied/WAF/rate-limit/TLS/cell/DDoS rows, with carbon routing excluded for realtime admission, emergency, HIPAA, PCI fraud, and DDoS paths. Alternative rejected: carbon-aware realtime routing, because safety and latency dominate at the edge. Cost: per-request audit dimensions and FinOps portal aggregation.
- API versioning posture: PRD now adopts ADR-0342 date carriers, SDK semver, N=3 / 180-day support, tenant route-management pinning, and ADR-0145 mesh exemption. Alternative rejected: gateway-wide unversioned route APIs, because partner integrations need stable contract carriers. Cost: three active route/admission versions and emergency-deny bypass handling.
## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: baseline_cpu_per_tenant 0.1 vCPU; baseline_ram_per_tenant 128 MiB; storage_per_tenant 0 GB; connections valkey=6, postgres=0, outbound_http=16; scaling_dimension per_request; cell_placement_class Tier-4.
- ADR: ADR-0340 capacity_model; ADR-0248 cellular criticality numbering.
- Why: The gateway is an edge data-plane service; load scales with inbound requests, route admission, WAF, TLS, and rate-limit cache pressure, not tenant storage.
- Rejected: cell_placement_class=Tier-3 because ADR-0340 assigns edge data-plane surfaces to Tier-4.
- Cost: Keeps high outbound and cache connection budget while avoiding unnecessary database allocation.

### Block 2: dr
- Values: rto_p99_seconds 300; rpo_p99_seconds 0; multi_region_active_active true; backup_substrate valkey_cluster, object_storage_versioned, openbao_seal_unseal; failover_runbook runbooks/cell-evac.md; replication_shape active-active-multi-az-cross-region-warm.
- ADR: ADR-0343 DR RTO/RPO matrix and compliance-pack floors.
- Why: The gateway is the tenant ingress boundary; active-active recovery prevents regional edge loss from becoming a platform-wide availability incident.
- Rejected: backup-restore recovery because edge failover must happen before tenant APIs become unreachable.
- Cost: Requires active multi-region edge routing, replicated rate-limit state, and cert/key recovery drills.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier 3; evidence microservices/api-gateway/PRD.md, microservices/api-gateway/ARCHITECTURE.md, microservices/api-gateway/IP-010-rate-limit-adapter-valkey.md, microservices/api-gateway/IP-014-tls-cert-rotation-worker.md, microservices/api-gateway/runbooks/cell-evac.md.
- ADR: ADR-0338 pod runtime tiering; ADR-0340 D-6 cell/runtime co-variance.
- Why: Api-gateway is a north-south edge and performance-critical data-plane service handling TLS, HTTP/3, WAF, route admission, rate limits, and circuit breaking. It does not run tenant code or own substrate tenant records, so ADR-0338 Tier 3 edge placement is correct.
- Rejected: pod_runtime_tier=1 because the gateway handles edge traffic and route policy but does not own tenant substrate records.
- Cost: Tier 3 favors edge performance and requires separate evidence that sensitive state remains upstream.

### Block 4: tenant_version_pinning
- Values: declared_versions 2026-05-21, 2026-02-21, 2025-11-21; default_version 2026-05-21; supported_window_size 3; supported_window_minimum_days 180; supports_per_tenant_pinning true.
- ADR: ADR-0342 hybrid date-versioned public API policy.
- Why: Gateway route, admission, and event contracts are tenant-facing because all public APIs traverse them.
- Rejected: internal-only gateway versioning because route admission semantics and events are externally visible through tenant traffic behavior.
- Cost: Maintains three edge contract windows while preserving route compatibility.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Values: consumes_upstream_oss valkey, cedar, openbao, opentelemetry, cilium, istio, kyverno; oss_stewardship_class_overrides empty because registry-default stewardship applies.
- ADR: ADR-0345 OSS stewardship class and CVE response policy.
- Why: Gateway consumes registry-governed cache/rate-limit, policy, secret, telemetry, mesh, and admission dependencies.
- Rejected: service-local stewardship overrides without a registry delta.
- Cost: No local stewardship override; gateway CVE response follows registry teams and edge patch SLAs.

### Block 6: iac_module_invocations
- Values: aws-guest/edge-gateway@v1, oci-guest/edge-gateway@v1, on-prem/edge-gateway@v1, colo/edge-gateway@v1, oyatie-as-cloud-provider/edge-gateway@v1.
- ADR: ADR-0339 shared IaC module library.
- Why: Gateway ingress must be provisioned by shared edge modules across every deployment context.
- Rejected: context-local edge IaC because ingress and evacuation behavior must be uniform.
- Cost: Edge rollout now depends on shared module pin promotion across five contexts.
