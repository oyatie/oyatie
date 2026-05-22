# Ops Dashboard Control Center remediation notes

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/ops-dashboard-control-center/IP-journey-j137-corporate-internal-audit-sox-controls-test-audit-pane.md
- microservices/ops-dashboard-control-center/capacity-model.md

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture, ADR-0343: PRD now states manifest-matching DR of RTO 1800s/RPO 300s and the hot operator `rpo_rto` of RTO 300s/RPO 60s; HIPAA/SOC2/ISO floors; active-active incident, rollback, evidence-export, tenant-posture, and recovery commands; and runbooks for multi-region, deployment rollback, incident command, admin rollback, and KR escalation. Rejected a generic admin-console target because this surface is used during the outage being remediated. Cost: hot standby operator command capacity is required even when normal traffic is low.
- Capacity model, ADR-0340: PRD now states manifest-matching 0.18 vCPU, 384 MiB RAM, 2 GiB storage, 6 Postgres, 6 Valkey, 40 outbound HTTP sockets, `per_request` scaling, Tier-1 command placement, and 2 to 12 replicas per tenant slice. Rejected a single dashboard polling model because rollback and recovery commands require separate burst budgets. Cost: evidence-export workers are capped to protect audit-chain throughput.
- Sustainability and cost attribution, ADR-0344: PRD now requires cost, CO2, and watt-hour fields on incident declaration, deployment approval, rollback, health observation, tenant posture, policy review, evidence export, and recovery workflow rows; carbon routing is disabled for live control commands and enabled for export/report jobs. Rejected carbon-preferred placement for rollback because operational recovery is the hard constraint. Cost: operator actions now carry FinOps metadata in every audit row.
- API versioning, ADR-0342: PRD now states the YYYY-MM-DD header/URL/proto triplet, SDK semver, N=3 versions for at least 180 days, tenant pinning, and internal-mesh exemption. Rejected internal-only version posture because external GRC and operator workstation integrations are contract consumers. Cost: the control surface must maintain compatibility across regulated operator clients.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

- bucket: `D4-BUCKET-4`
- selection: trigger-matched `IP-*.md` only; unmatched IPs unchanged.
- scanned_ips: `40`; changed_ips: `40`; unmatched_ips: `0`.
- doctrine_sections: ADR-0342 API Versioning, ADR-0343 DR posture, ADR-0344 Sustainability emission, ADR-0338 Pod runtime tier.

| IP | Trigger matches | Sections added |
|---|---|---|
| `IP-001-control-plane-manifest-and-contracts.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-002-incident-command-workflows.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-003-deployment-approval-and-rollback.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-004-cluster-health-and-recovery.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-005-tenant-isolation-policy-audit.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-006-evidence-pack-export.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-007-localization-escalation-runbooks.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-008-step-up-auth-flow.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-009-audit-emission-integration.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-010-cedar-admin-console-surface.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-011-tenant-admin-panel.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-012-cell-operator-panel.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-013-adr-promotion-triage-panel.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-014-finops-portal-integration.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-015-observability-pivot.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-016-on-call-handoff-bc.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-journey-j100-pack-rollout-first-action.md` | C metered | Sustainability emission |
| `IP-journey-j126-3pao-docket-dashboard.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-journey-j137-corporate-internal-audit-sox-controls-test-audit-pane.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-journey-j139-internal-audit-cedar-permit-misuse-policy-pane.md` | B HA-critical, C metered | DR posture, Sustainability emission |
| `IP-journey-j143-export-tracking-surface.md` | B HA-critical, C metered | DR posture, Sustainability emission |
| `IP-journey-j19-ombudsman-operator-console.md` | A contracts, C metered | API Versioning, Sustainability emission |
| `IP-journey-j68-auditor-console.md` | A contracts, B HA-critical, C metered | API Versioning, DR posture, Sustainability emission |
| `IP-journey-j77-operator-evidence-console.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-journey-j78-operator-evidence-console.md` | A contracts | API Versioning |
| `IP-journey-j79-operator-evidence-console.md` | A contracts | API Versioning |
| `IP-journey-j81-operator-evidence-console.md` | A contracts | API Versioning |
| `IP-journey-j82-operator-evidence-console.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-journey-j86-operator-evidence-console.md` | A contracts, B HA-critical | API Versioning, DR posture |
| `IP-journey-j87-operator-evidence-console.md` | A contracts | API Versioning |
| `IP-journey-j88-operator-evidence-console.md` | A contracts | API Versioning |
| `IP-journey-j91-us-msb-mtl-overlay.md` | B HA-critical, C metered | DR posture, Sustainability emission |
| `IP-journey-j92-br-lgpd-us-parent-dsar.md` | C metered | Sustainability emission |
| `IP-journey-j93-in-dpdpa-rbi-overlay.md` | B HA-critical, C metered | DR posture, Sustainability emission |
| `IP-journey-j94-sox404-public-company-controls.md` | B HA-critical, C metered | DR posture, Sustainability emission |
| `IP-journey-j95-iso27001-soc2-annual-audit.md` | C metered | Sustainability emission |
| `IP-journey-j96-ksa-uae-mena-onboarding.md` | C metered | Sustainability emission |
| `IP-journey-j97-sg-pdpa-mas-tenant.md` | C metered | Sustainability emission |
| `IP-journey-j98-au-privacy-apra-cps234.md` | C metered | Sustainability emission |
| `IP-journey-j99-multi-pack-conflict-resolution.md` | C metered | Sustainability emission |

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.18 vCPU, 384 MiB RAM, 2 GB storage, and 6/6/40 connections per tenant; operator fan-out and control-plane integrations raise CPU/RAM and outbound connections.
- ADR: ADR-0340 requires per-service capacity manifest data; ADR-0248 and ADR-0338 shape the cell/runtime covariance.
- Rejected: copying another product service's baseline, because this service's load axis and data weight differ.
- Cost: capacity planning now carries explicit per-tenant CPU, RAM, storage, and connection reservations for cell admission.

### Block 2: dr
- Value: RTO 1800s, RPO 300s, multi-region active-active true, backup substrate postgres_wal_g, valkey_cluster, object_storage_versioned, audit_chain_merkle_seal, openbao_seal_unseal, failover runbook runbooks/deployment-rollback.md.
- ADR: ADR-0343 requires RTO/RPO by service and compliance floor; selected values follow the strictest relevant tenant-data and evidence obligations.
- Rejected: padding to generic 24h recovery, because this service's tenant workflow/evidence tolerance is tighter.
- Cost: DR drills must prove the declared manifest replication_shape and runbook-specific restore steps instead of relying on ad hoc restore claims.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=1; evidence microservices/ops-dashboard-control-center/PRD.md, microservices/ops-dashboard-control-center/ARCHITECTURE.md, microservices/ops-dashboard-control-center/IP-005-tenant-isolation-policy-audit.md.
- ADR: ADR-0338 requires runtime placement by execution surface; this classification follows whether the service executes tenant code, touches substrate tenant data, or remains a first-party app.
- Rejected: Tier 0, because no evidence shows tenant-customer code execution for this service.
- Cost: scheduling and nodepool admission must respect the declared runtime tier and its security overhead.

### Block 4: tenant_version_pinning
- Value: declared version 2026-05-21, default 2026-05-21, supported window policy of 3 versions and 180 days, per-tenant pinning enabled.
- ADR: ADR-0342 requires date-versioned public contracts with per-tenant pinning where tenant contracts exist.
- Rejected: semver-only or no per-tenant pinning, because tenant migration control is part of the public contract doctrine.
- Cost: every public contract change needs a migration doc/calendar entry before older versions sunset.

### Block 5: consumes_upstream_oss and oss_stewardship_class_overrides
- Value: consumes cedar, postgresql, valkey, opentelemetry, opentofu, openbao, cosign; no local stewardship override declared. OpenBao and Cosign remain explicit because operator actions depend on secret recovery and signed evidence provenance.
- ADR: ADR-0345 requires OSS dependency stewardship and CVE ownership to stay aligned with the registry.
- Rejected: per-service stewardship-class drift, because registry defaults are sufficient for this service's use of these dependencies.
- Cost: CVE response routing now follows the registry owner teams for every declared upstream.

### Block 6: iac_module_invocations
- Value: aws-guest/tenant-namespace@v1, aws-guest/postgres-wal-g@v1, aws-guest/valkey-cluster@v1, colo/audit-chain-merkle-seal@v1, oyatie-as-cloud-provider/openbao-seal-unseal@v1.
- ADR: ADR-0339 requires service IaC to consume shared module primitives instead of bespoke snowflake modules.
- Rejected: unpinned local IaC semantics, because the shared-module contract is the doctrine surface for admission and review.
- Cost: module upgrades must be version-pinned and reviewed per context before rollout.
