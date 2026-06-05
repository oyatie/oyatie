---
doc_class: PolicySpec
title: Tenant Isolation Specification
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-observability
deciders: council-architecture, ops-security, axis-observability, council-privacy
related_adrs: [ADR-0028, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/observability/threat-model.md (Trust Boundary 2, T-S-01, T-I-01, T-E-01)
  - microservices/observability/dpia.md (R-02, R-15)
  - microservices/observability/policy/tenant-scope.cedar
  - microservices/observability/policy/ci-scope.cedar
  - microservices/observability/policy/auditor-scope.cedar
  - microservices/observability/policy/public-read.cedar
review_cadence: quarterly + on every Mimir/Loki/Tempo version upgrade
doc_status: published
---

# Tenant Isolation Specification (observability µservice)

## Purpose

Define the load-bearing tenant-isolation invariants of the observability substrate. This document is the authoritative reference for SOC 2 examiners (CC6.1 / CC6.2 / CC6.6), ISO 27001 auditors (A.5.15 / A.8.2 / A.8.3 / A.8.12), GDPR Art. 32 reviewers, KR PIPA Art. 23 / Art. 29 reviewers, and HIPAA §164.312(a)(1) reviewers asking *"how does observability prevent tenant-A from seeing tenant-B's data?"*

## Tenant Identity Model

### Tenant ID derivation

```text
canonical_tenant_id   = <opaque-string-issued-at-onboarding>   (NOT stored in observability)
hashed_tenant_id      = sha256(canonical_tenant_id ++ deployment_salt)[..16]
X-Scope-OrgID header  = "tenant:" + hashed_tenant_id           (sent on every Mimir/Loki/Tempo call)
```

Properties:
- `canonical_tenant_id` is the OpenBao-bound subject; observability never receives the raw value (the OpenBao tenant-resolver returns only the hashed form on request, mediated by Cedar policy).
- `deployment_salt` is a per-Mimir-cluster secret rotated every 12 months. Rotation event captured in audit-chain.
- `hashed_tenant_id` truncation to 16 hex chars (64 bits) provides ~10¹⁹ collision-free namespace; collision risk negligible for foreseeable scale (10⁶ tenants).
- The mapping `canonical_tenant_id → hashed_tenant_id` is recoverable only by OpenBao's tenant-resolver service; observability operators with Grafana access never see canonical IDs.

### Reserved tenant IDs

The following `X-Scope-OrgID` values are RESERVED and never issued as customer tenant IDs. Mimir distributor enforces this at ingress.

| Reserved tenant | Purpose | Write authority | Read authority |
|---|---|---|---|
| `tenant:oya-ci` | Promotion-readiness CI lane writes / reads | `slo-engine-worker` ServiceAccount only (via SPIFFE identity verification) | CI runners via short-lived OpenBao-issued read keys |
| `tenant:oya-self` | observability µservice self-observability (its own SLOs, capacity metrics, gate-internal SLI) | observability platform components (Mimir-internal, Loki-internal, etc.) emitting their own telemetry | observability platform operators + the CI lane for self-gating |
| `tenant:oya-aggregate` | Anonymised cross-tenant aggregates (for product metrics; never tenant-attributable) | `slo-engine-worker` aggregator job (DP-noise injected per `policy/dp-analysis.md`) | Public dashboards (no auth) for non-sensitive aggregates |

Any inbound write with `X-Scope-OrgID = tenant:oya-*` from a source other than the authorised SPIFFE identity is **rejected at Mimir distributor** with HTTP 403 and emits `oya_tenant_reserved_id_violation_total` metric (alert on > 0 over 5m).

### Tenant scope enumeration

Each tenant carries a `tenant_scope` enum recorded in OpenBao at onboarding:

```yaml
tenant_scope:
  enum: [trial, production, sandbox, internal]
  description: |
    - trial: pre-paid evaluation; 30-day TTL; no cross-region; capped capacity
    - production: paying customer; SLA-bound; primary-region pinned per pack
    - sandbox: customer-owned non-prod; isolated from prod-tier alerting
    - internal: oyatie's own services dogfooding observability
```

This drives capacity-allocator + retention-policy decisions but does NOT relax isolation boundaries — every tenant scope shares identical isolation invariants.

## Mimir Multi-Tenancy Invariants

### Invariant TI-01: `multitenancy_enabled: true` always

Mimir's `multitenancy_enabled` config flag MUST be `true` for every Mimir instance in every pack. There is no exception. The CI lane `oya-governance-mimir-tenancy-enforced` validates this against the Helm values at `microservices/observability/iac/helm/mimir/values.yaml`; PRs that disable multi-tenancy fail to merge.

### Invariant TI-02: Per-tenant API keys with bound-tenant claim

Every API key issued by OpenBao for Mimir / Loki / Tempo / Pyroscope writes carries a non-modifiable `bound_tenant` claim. Mimir distributor validates: `header.X-Scope-OrgID == key.bound_tenant`. Mismatch returns 401 + emits `oya_tenant_spoofing_attempt_total` (alert on > 0 over 1m).

Implementation: key-issuance is via OpenBao's `kv-v2` mount with claim binding at issuance. Mimir distributor's auth middleware verifies the JWT claim. Reference implementation in `microservices/observability/src/crates/oya-observability-slo-engine-adapter-mimir/src/auth.rs`.

### Invariant TI-03: Cross-tenant query refusal at distributor (server-side)

Cross-tenant queries are refused **server-side at the Mimir query-frontend** before any data is read. The query-frontend extracts `X-Scope-OrgID`, scopes the underlying TSDB read to that tenant's blocks, and rejects any PromQL that attempts to reference series outside that tenant. Wildcard tenant queries (`tenant=*`) are forbidden by config; even admin-bound tokens cannot wildcard-query without 2-person-rule JIT elevation through OpenBao.

CI lane: `oya-governance-mimir-no-wildcard-query` greps the Mimir config + the recording-rules YAML for any `tenant=*` pattern; presence fails the lane.

### Invariant TI-04: No client-side tenant filtering as the only check

PromQL `tenant{tenant="<bound>"}` selectors are advisory; the server enforces the actual scope. Even if a client somehow constructs a query that omits the tenant selector, the distributor's `X-Scope-OrgID` enforcement applies.

This invariant exists to prevent a class of bugs where a client-side library is the only line of defence; in observability's design, the server is the line of defence.

### Invariant TI-05: Per-tenant cardinality + rate limits

| Limit | Default | Configurable per tenant_scope | Enforcement |
|---|---|---|---|
| Max active series | 1M / tenant | trial: 100k; production: 1M (scaling per Mimir-runtime); sandbox: 50k; internal: 10M | Mimir distributor `max_global_series_per_user` |
| Max samples / sec | 100k / tenant | trial: 10k; production: 100k+; sandbox: 5k; internal: 1M | Mimir distributor `ingestion_rate` |
| Max bytes / sec (log ingest) | 100MB / tenant | analogous | Loki distributor |
| Max trace span / sec | 50k / tenant | analogous | Tempo distributor |

Excess returns HTTP 429. Per-tenant overage metric (`oya_tenant_rate_limit_exceeded_total`) feeds the tenant's own dashboard so they self-detect.

### Invariant TI-06: Reserved-namespace metric write authority

Metric names matching the prefixes below have **write authority restricted** to specific identities:

| Metric prefix | Authorised writer (SPIFFE identity) | Rejection action |
|---|---|---|
| `oya_promotion_*` | `spiffe://oyatie/observability/slo-engine-worker` only | 403 + `oya_promotion_metric_unauthorized_writer_total` |
| `oya_governance_*` | `spiffe://oyatie/governance/<lane>` only | 403 + analogous |
| `oya_observability_internal_*` | `spiffe://oyatie/observability/*` (any observability platform component) | 403 |
| `oya_audit_chain_*` | `spiffe://oyatie/audit-chain/*` | 403 |

Non-reserved metric names (everything else) follow normal per-tenant scope; tenant-owned metrics live in tenant-scoped namespaces.

### Invariant TI-07: Per-tenant read authorization via Cedar

Every read request (Grafana UI, REST API, SDK) passes through a Cedar policy evaluator (per ADR-0140) that enforces:

```cedar
permit (
  principal,
  action in [Action::"read_metrics", Action::"read_logs", Action::"read_traces", Action::"read_profiles"],
  resource is TenantData
) when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};
```

(Full fragment at `policy/tenant-scope.cedar`.) The evaluator runs in `slo-engine-rest`; non-matching reads return 403 + emit `oya_tenant_unauthorized_read_attempt_total`.

## Failure Modes

### FM-01: Mimir distributor crash → ingest write-availability degraded

**Behaviour:** Replication factor ≥ 3 across distributor pods; ingest queues to remaining replicas. Per-tenant rate limits prevent any one tenant from monopolising the surviving replicas.

**Tenant impact:** Latency degraded; no data loss.

**Detection:** SLI `mimir_distributor_request_duration_seconds` quantile alerts.

**Recovery:** Auto-restart pod; HPA scales replicas up; if persistent, ops-sre-reliability paged.

### FM-02: Mimir multi-tenancy config drift (someone disables it locally)

**Behaviour:** CI lane refuses merge; deployed-state drift alarms via continuous Helm-state-validator.

**Tenant impact:** Caught pre-merge; zero tenant impact.

**Detection:** `oya-governance-mimir-tenancy-enforced` lane + Helm-state validator (CronJob comparing live cluster to git).

**Recovery:** Auto-rollback to last green state; ops-security incident if cause is intentional.

### FM-03: Tenant API key compromised (leaked into a public repo)

**Behaviour:** Secret-scanner CI lane detects on commit; OpenBao rotates the key on detection signal.

**Tenant impact:** Brief window between leak and rotation; mitigation is rotation speed + per-key audit log.

**Detection:** `oya-governance-evidence-secret-scan` lane + GitHub secret-scanning push protection.

**Recovery:** Rotate API key (< 60s via OpenBao); revoke old key; forensic trace of leak path; tenant notification per breach-notification SLA.

### FM-04: SPIFFE identity spoofing (attacker stands up a workload with same SA name)

**Behaviour:** SPIFFE identity validation includes pod-identity binding (workload UUID + namespace + service-account); spoofed workload in same namespace would need cluster-admin to deploy, which is blocked by RBAC.

**Tenant impact:** Defence-in-depth required: combined with cluster-admin RBAC, Cedar policy, network policy.

**Detection:** SPIFFE attestation log; anomalous SVID issuance alarms.

**Recovery:** Revoke SVID; pod-identity audit; incident response.

### FM-05: Reserved-tenant write from non-authorised source

**Behaviour:** Mimir distributor's TI-02 + TI-06 checks reject; metric emitted; alert fires.

**Tenant impact:** None (write rejected).

**Detection:** `oya_tenant_reserved_id_violation_total > 0` over 5m fires page.

**Recovery:** Trace source; revoke offending credentials; root-cause (config bug vs intentional).

### FM-06: Cross-tenant data exposure via Grafana misconfiguration

**Behaviour:** Grafana roles managed via OpenTofu; UI-based role editing forbidden by Grafana's `[security]` config; LEAN check asserts Terraform-declared roles match live state.

**Tenant impact:** Caught pre-deploy.

**Detection:** Terraform-drift detector + `oya-governance-grafana-rbac-conformance` lane.

**Recovery:** OpenTofu apply restores declared state; root-cause investigation; if exposed, breach-notification chain.

### FM-07: Tenant attempts to query Mimir directly bypassing slo-engine-rest

**Behaviour:** Mimir's HTTP API exposed only to in-cluster traffic; ingress gateway routes tenant traffic only to `slo-engine-rest` (which performs Cedar policy check). Direct Mimir access requires VPC-internal network.

**Tenant impact:** Tenant cannot bypass; if a misconfigured route exposed Mimir, network policy denies tenant traffic.

**Detection:** Network policy violations logged; periodic external pen-test.

**Recovery:** Close route; CI lane prevents recurrence.

## Audit Trail

Every cross-tenant boundary event is audit-chain-emitted per Bominal ADR-0028:

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| Tenant spoofing attempt | Mimir distributor | `attempted_tenant_id, source_ip, source_spiffe_id, timestamp, request_id` | ≥ 1y (or longer per pack legal requirements; HIPAA 6y) |
| Reserved-namespace write attempt | Mimir distributor | `metric_name, source_spiffe_id, attempted_tenant_id, timestamp` | ≥ 1y |
| Unauthorised read attempt | slo-engine-rest (Cedar evaluator) | `principal_id, requested_tenant_id, action, resource, timestamp` | ≥ 1y |
| API key issuance | OpenBao tenant-resolver | `key_id, bound_tenant, issued_by, requestor, ttl, timestamp` | ≥ 1y |
| API key revocation | OpenBao | `key_id, revoked_by, reason, timestamp` | ≥ 1y |
| Deployment salt rotation | OpenBao + observability admin | `prev_salt_hash, new_salt_hash, rotated_by, timestamp` | indefinite (rotation-history record) |
| Grafana folder permission change | OpenTofu apply log | `folder_id, prev_perms, new_perms, applied_by, terraform_plan_id` | ≥ 2y |
| Cross-tenant aggregate query | slo-engine-worker | `aggregator_job_id, dp_epsilon, contributing_tenants_count, query_id` | ≥ 1y |

The audit log is itself stored in Mimir under `tenant:oya-self` and replicated to the `audit-chain` µservice for Merkle-tree sealing. Audit-of-audits: every audit-log read is itself audited.

## Per-Pack Overlay

### pack-kr (KR PIPA + ISMS-P)

- KR PIPA Art. 29 (technical safeguards) maps to TI-01..TI-07 + FM-01..FM-07.
- Audit log retention: ≥ 1 year per PIPA Enforcement Decree Art. 30; extended to 3 years for `tenant_scope: production` aligning with sectoral KR-FSS guidance.
- KR PIPA Art. 23 (sensitive personal information) — hashed tenant ID with auxiliary data is sensitive; salt rotation per Art. 29.

### pack-us-healthcare (HIPAA)

- §164.312(a)(1) (access control) mapping: TI-01..TI-07.
- §164.312(b) (audit controls) mapping: Audit Trail table.
- §164.312(e)(1) (transmission security): mTLS + TLS 1.3 for all inter-service traffic.
- §164.502(b) (minimum necessary): tenant-scope reads enforce least-data; cross-tenant queries impossible.
- Audit log retention extended to ≥ 6 years per §164.316(b)(2).
- BAA with Covered Entity tenants documented in `legal/baa-template.md` (Slice D).

### pack-eu (GDPR + EDPB)

- Art. 32(1)(a) pseudonymisation: `X-Scope-OrgID` is pseudonymous; canonical-tenant-id never exposed to observability operators.
- Art. 32(1)(b) confidentiality + integrity: enforced via TI-01..TI-07 + audit-chain.
- Art. 32(1)(c) availability: enforced via Mimir HA + per-tenant rate limits.
- Art. 32(1)(d) regular testing: annual pen-test + quarterly chaos drill.
- Art. 25 by design and default: pseudonymisation + multi-tenancy default-enabled.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Each pack's overlay at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/tenant-isolation-overlay.md` maps the local PII law's confidentiality + integrity requirements to TI-01..TI-07.

## Verification

- `cargo run -p oya-dev-cli -- gate validate mimir-tenancy-enforced` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate mimir-no-wildcard-query` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate grafana-rbac-conformance` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice observability` — exit 0.
- Annual pen-test against tenant boundary: scheduled Q4 of each calendar year (October 1 cycle) coinciding with ISO 27001 surveillance audit; documented in `runbooks/tenant-boundary-pentest.md` (Slice B).
- Quarterly chaos drill: induce reserved-tenant spoofing attempt + cross-tenant query attempt; verify rejection + alerting.

## References

- ADR-0028 (Bominal): audit-chain.
- ADR-0117: cloud-native infrastructure + residency.
- ADR-0139: agentic SLO-gated promotion.
- ADR-0131: per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- `microservices/observability/threat-model.md` §"Trust Boundaries" + T-S-01, T-I-01, T-E-01.
- `microservices/observability/dpia.md` R-02, R-15.
- `microservices/observability/policy/{tenant-scope, ci-scope, auditor-scope, public-read}.cedar`.
- `microservices/observability/policy/data-residency.md`.
- Grafana Mimir multi-tenancy docs — `grafana.com/docs/mimir/latest/manage/secure/`.
- SPIFFE / SPIRE — `spiffe.io`.
- OpenBao docs — `openbao.org`.
