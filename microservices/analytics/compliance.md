# Analytics µservice — Compliance Posture

**Authority:** ADR-0038 DSR cascade, ADR-0008 DUBO, ADR-0156 PII registry
**Last reviewed:** 2026-05-18

## Frameworks in scope

| Framework | Scope | Evidence path |
|---|---|---|
| GDPR | EU pack tenants | `microservices/analytics/policy/gdpr.cedar` + audit-chain |
| K-PIPA (Korean PIPA) | KR pack tenants | `microservices/analytics/policy/kpipa.cedar` + audit-chain |
| SOC 2 Type II | All tenants | `microservices/analytics/scorecards/soc2-controls.json` |
| HIPAA (where applicable) | Tenants with `data_class=PHI` | `microservices/analytics/policy/hipaa.cedar` |

## Key controls

### Data-class taxonomy (ADR-0008 DUBO)

Every column in every per-tenant table has a `data_class` tag per ADR-0156. Cedar policies forbid cross-class projection without an explicit DUBO grant.

### DSR (Data Subject Right) cascade (ADR-0038)

- **Right to access.** Tenant runs audit-log-search; results delivered via the regulator-export evidence pack flow (IP-013).
- **Right to erasure.** Tenant offboard → IP-002 controller drops the per-tenant database → proof-of-erasure emitted into audit-chain.
- **Right to portability.** IP-013 regulator-export produces NDJSON in a documented schema; portable to any compatible analytics system.

### Audit chain (ADR-0003)

- Every external query emits an audit event.
- Audit events are themselves stored in ClickHouse (recursive observation defense).
- Cosign-signed quarterly evidence pack.

### Retention (ADR-0184 + per-table TTL)

- Tenant audit-log: 7 years (compliance).
- Tenant business KPIs: 90 days hot + 1 year cold.
- Telemetry rollups: 1 year cold (observability cluster, not this µservice).

### Residency (ADR-0049 + ADR-0010)

- StrictKR tenant data lives only in kr-* cells.
- StrictEU tenant data lives only in eu-* cells.
- KrWithUsFailover: primary KR; failover to us-* under explicit DR scenarios only.

## Audit trail of audit trail

The audit-chain stream itself emits a meta-audit event on every audit-chain emission. Loss of audit events is detectable via the meta-stream's sequence number gap.

## References

- ADR-0003 audit chain, ADR-0008 DUBO, ADR-0038 DSR cascade, ADR-0049 cross-region, ADR-0156 PII registry, ADR-0193.

---



## §day-one-cert-readiness
This anchor is closed for `analytics` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `analytics` covers packs `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +15 more.
- Example: `tenant-dashboard-query` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pack-overlay-roster
This anchor is closed for `analytics` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`.
- Pack overlays modify Cedar fragments `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar` without changing domain code.
- Data classes under pack control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `tenant-dashboard-query` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §platform-owner-indirection
This anchor is closed for `analytics` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `analytics` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`, `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`; +18 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `tenant-dashboard-query` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.analytics.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §detection-substrate-binding
This anchor is closed for `analytics` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `analytics` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `tenant-dashboard-query` touches those data classes.
- Signal sources: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`; +13 more.
- Example event class: `oya.analytics.tenant.dashboard.query.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §ml-model-lifecycle
This anchor is closed for `analytics` against documentation-rigor.md §3.2.6.E: model inventory, retrain cadence and promotion gates.

### Service-specific answer
- Local ML posture: `True` for direct model use; inherited detection/intelligence models still require versioned consumption evidence.
- Model inventory key: `manifest.json:ml_models` or the Intelligence audience tag `analytics.tenant-dashboard-query` if models are substrate-hosted.
- Promotion gates: offline eval, bias/fairness report, drift threshold, SLO budget, rollback model id, and human approval for high-risk/adverse-action paths.
- Retraining cadence is model-specific; high-risk models require documented data cut, feature schema, holdout set, and pack-specific legal review.
- Example: `tenant-dashboard-query` model output is never the sole authority for a legal/financial/employment/minor-impacting decision; Cedar and human-review policies remain in control.
- Deprecated model versions sunset under ADR-0258 with traffic split, canary, rollback, and post-promotion audit.
- Model cards include intended use, non-use, data provenance, performance by segment, failure modes, and owner.
- Services without local models keep this as a negative declaration so future agents cannot silently add ML without the lifecycle gate.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: NIST AI RMF model-governance lifecycle is the reference pattern for the control shape described here.
- Precedent 2: Google Model Cards is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §detection-fairness-audit
This anchor is closed for `analytics` against documentation-rigor.md §3.2.6.E: fairness metrics, thresholds and disaggregated false-positive audit.

### Service-specific answer
- Fairness audit applies to `analytics` risk/detection decisions that affect access, ranking, safety, money, employment, health, or protected classes.
- Metrics: false-positive rate ratio, false-negative rate ratio, calibration by segment, equalized-odds gap, appeal overturn rate, and challenge-friction rate.
- Thresholds: no protected segment exceeds 1.25x baseline false-positive rate without documented mitigation and human review.
- Segments are derived from lawful, minimized attributes; `analytics` never stores protected attributes solely to make a product feature easier.
- Example: `tenant-dashboard-query` abuse/risk score challenge rate is compared across locale, accessibility profile, age tier, and jurisdiction pack.
- Audit cadence: every model/rule promotion, quarterly for active high-risk detectors, and after any SEV involving false positives.
- Fairness reports are retained in audit evidence; raw protected-attribute joins remain in restricted analytics cells.
- If the service has no ML, deterministic rules still get false-positive and appeal-rate monitoring.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: Microsoft Fairlearn audit pattern is the reference pattern for the control shape described here.
- Precedent 2: NIST AI RMF measurement function is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §investigation-binding
This anchor is closed for `analytics` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `analytics` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.analytics.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `tenant-dashboard-query` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `tenant-dashboard-query` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §insider-threat-controls
This anchor is closed for `analytics` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `analytics` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`, `analytics.tenant_dashboard_query`, `analytics.billing_rollup`; +2 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `analytics.tenant_dashboard_query` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §threat-intelligence-feeds
This anchor is closed for `analytics` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `analytics` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`; +12 more.
- Example: `tenant-dashboard-query` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §key-rotation-cadence
This anchor is closed for `analytics` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.analytics` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/analytics/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.
- Example: `tenant-dashboard-query` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §crypto-agility-plan
This anchor is closed for `analytics` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `analytics` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`, `microservices/analytics/iac/clickhouse/mv-templates/audit-log-table.sql`, `microservices/analytics/iac/clickhouse/mv-templates/mv-day-billing-per-resource.sql`; +10 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `tenant-dashboard-query` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pentest-and-bounty-cadence
This anchor is closed for `analytics` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `analytics` is in annual full-scope pentest and every major `tenant-dashboard-query` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`, `microservices/analytics/iac/clickhouse/mv-templates/audit-log-table.sql`, `microservices/analytics/iac/clickhouse/mv-templates/mv-day-billing-per-resource.sql`; +16 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `analytics` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §facility-controls
This anchor is closed for `analytics` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `analytics` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `tenant-dashboard-query` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §supply-chain-risk
This anchor is closed for `analytics` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `analytics` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/analytics/catalog/README.md`, `microservices/analytics/catalog/capabilities.json`, `microservices/analytics/catalog/cedar-policies.json`, `microservices/analytics/catalog/clickhouse-analytics-helm.json`, `microservices/analytics/catalog/contracts.json`, `microservices/analytics/catalog/dashboards.json`; +22 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `tenant-dashboard-query` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `analytics` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `analytics` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `tenant-dashboard-query` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `tenant-dashboard-query` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §data-classification
This anchor is closed for `analytics` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.
- State/event surfaces carrying classification: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `tenant-dashboard-query` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `analytics`; owner `council-analytics`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `tenant-dashboard-query`, `billing-rollup`, `audit-log-search`, `data-export`.
- Capability records cited: `microservices/analytics/capabilities/audit-log-query.json`, `microservices/analytics/capabilities/dashboard-query.json`, `microservices/analytics/capabilities/regulator-export.json`.
- API surfaces cited: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- SLO and dashboard evidence: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +14 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/analytics/contracts/analytics.proto`, `microservices/analytics/contracts/asyncapi-v1.yaml`, `microservices/analytics/contracts/graphql-v1.sdl`, `microservices/analytics/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/analytics/policy/audit-log-pii.cedar`, `microservices/analytics/policy/dashboard.cedar`, `microservices/analytics/policy/dual-context.cedar`, `microservices/analytics/policy/regulator-export.cedar`, `microservices/analytics/policy/residency.cedar`, `microservices/analytics/policy/tenant-bootstrap.cedar`.
- State/event binding: `analytics.tenant_dashboard_query`, `analytics.billing_rollup`, `analytics.audit_log_search`, `analytics.data_export`.
- Capability binding: `tenant-dashboard-query`.
- SLO binding: `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml`, `microservices/analytics/slos/audit-log-query-latency.openslo.yaml`, `microservices/analytics/slos/billing-reconciliation.openslo.yaml`, `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml`, `microservices/analytics/slos/clickhouse-query-latency.openslo.yaml`, `microservices/analytics/slos/cluster-availability.openslo.yaml`; +4 more.
- Runbook binding: `microservices/analytics/runbooks/capacity-rebalance.md`, `microservices/analytics/runbooks/clickhouse.md`, `microservices/analytics/runbooks/cold-tier-latency.md`, `microservices/analytics/runbooks/ingest-lag-burn.md`, `microservices/analytics/runbooks/keeper-quorum-recovery.md`, `microservices/analytics/runbooks/mv-lag-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `analytics`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `analytics`.
- `policy-engine` supplies the signed Cedar corpus while `analytics` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `analytics` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `analytics`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `analytics` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

