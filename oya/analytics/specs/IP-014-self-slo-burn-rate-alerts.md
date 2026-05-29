# IP-014 — Self-SLO Authoring + 4-Window Burn-Rate Alerts

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** sre (council-analytics + ops-sre-reliability)
**Authority ADRs:** ADR-0139 4-window alerting, ADR-0186 Stage 5 SLO authoring, ADR-0180-slo-composition-inheritance-arithmetic
**Depends on:** IP-001
**Status:** Planned

## Scope

Author the analytics µservice's OpenSLO v1.0 sources. Each source declares an SLO indicator + objective + 4-window burn-rate alert policies. Sloth (per ADR-0186 Stage 5) compiles each source to a PrometheusRule CR; AlertManager routes to PagerDuty + Opsgenie per ADR-0186 Stage 4.

The 4-window burn-rate model is canonical per ADR-0139:

- **Fast burn:** 14.4× threshold over 1h window → page in 2 minutes (catches sudden outages).
- **Medium burn:** 6× over 6h → page in 15 minutes (catches sustained degradation).
- **Slow burn:** 3× over 1d → ticket in 1 hour (catches budget consumption).
- **Trickle burn:** 1× over 3d → notice in 6 hours (catches chronic drift).

## Deliverables

1. Nine (9) OpenSLO sources at `microservices/analytics/slos/` (all already authored as part of the SLO sweep):
   - `clickhouse-query-latency.openslo.yaml` (engine-level query latency).
   - `clickhouse-ingest-lag.openslo.yaml` (MV freshness).
   - `dashboard-api-latency.openslo.yaml` (API latency on dashboard routes).
   - `audit-log-query-latency.openslo.yaml` (hot tier).
   - `audit-log-query-cold-latency.openslo.yaml` (cold tier; relaxed budget).
   - `tenant-bootstrap-latency.openslo.yaml` (controller).
   - `keeper-quorum-availability.openslo.yaml` (DDL availability).
   - `cluster-availability.openslo.yaml` (per-shard availability).
   - `regulator-export-success.openslo.yaml` (compliance surface).
   - `billing-reconciliation.openslo.yaml` (accuracy).
2. Sloth CI lane verifies each authored source compiles to a valid PrometheusRule.
3. PagerDuty + Opsgenie webhook routes configured per ADR-0186 Stage 4.
4. Synthetic burn test in CI verifies the alert path end-to-end.
5. Runbook hooks on each alert reference the per-runbook file in `microservices/analytics/runbooks/`.

## Acceptance criteria

- `cargo run -p oya-governance-slo-coverage -- microservices/analytics/` reports the full set of authored SLOs.
- Each compiled PrometheusRule covers all 4 windows per ADR-0139.
- Burn-rate breach → AlertManager → PagerDuty + Opsgenie per ADR-0186 Stage 4 (verified in dev cell via synthetic burn).
- Every alert carries a `runbook` annotation referencing the proper runbook file.
- Each SLO source carries labels `microservice=analytics` and a relevant ADR anchor.

## Implementation tasks

### T1 — SLO authoring (already done)

The nine OpenSLO files at `microservices/analytics/slos/` are authored. They are uniform in shape:

- `apiVersion: openslo/v1`.
- `kind: SLO`.
- 4 `AlertPolicy` blocks (fastburn, mediumburn, slowburn, trickleburn).
- `notificationTargets` for at minimum the fastburn alert.
- `timeWindow: 30d rolling`.
- `budgetingMethod: Occurrences`.

### T2 — Sloth compile lane

CI lane: `.github/workflows/slo-compile.yml` (or per-µservice extension of an existing lane).

```yaml
name: slo-compile
on:
  pull_request:
    paths:
      - 'microservices/analytics/slos/**'
jobs:
  compile:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: slok/sloth-action@v3
      - run: |
          for src in microservices/analytics/slos/*.openslo.yaml; do
            sloth generate -i "$src" -o "/tmp/$(basename "$src" .openslo.yaml).rules.yaml" || exit 1
          done
```

### T3 — PrometheusRule rendering

Sloth compiles each OpenSLO source to a PrometheusRule CR. The compiled output lives at `microservices/analytics/slos/_generated/<name>.rules.yaml` (not committed; produced by sloth at install time).

Flux's `Kustomization` includes the sloth-controller subscription that consumes the OpenSLO sources from this directory.

### T4 — AlertManager routing

Per ADR-0186 Stage 4, AlertManager is configured at `microservices/observability/iac/helm/alertmanager/values.yaml`. The analytics-namespace routes:

```yaml
route:
  routes:
    - match:
        microservice: analytics
      receiver: analytics-team
      group_by: [slo, severity, microservice]
      group_wait: 30s
      group_interval: 5m
      repeat_interval: 4h
receivers:
  - name: analytics-team
    pagerduty_configs:
      - service_key_file: /etc/secrets/pagerduty-analytics-key
    webhook_configs:
      - url: https://api.opsgenie.com/v1/json/prometheus
        send_resolved: true
```

This routing config lives in the observability µservice's Helm values, but the analytics µservice contributes the SLO labels that drive it.

### T5 — Runbook hooks

Each PrometheusRule (generated from OpenSLO) carries a `runbook` annotation. Sloth supports custom labels/annotations in the OpenSLO source. For the analytics SLOs, each `AlertPolicy`'s `notificationTargets` includes:

```yaml
notificationTargets:
  - target: pagerduty-analytics
    additionalLabels:
      runbook: microservices/analytics/runbooks/<runbook>.md
```

Mapping:

| SLO | Runbook |
|---|---|
| clickhouse-query-latency | runbooks/clickhouse.md |
| clickhouse-ingest-lag | runbooks/ingest-lag-burn.md |
| dashboard-api-latency | runbooks/clickhouse.md (cluster-level) |
| audit-log-query-latency (both) | runbooks/cold-tier-latency.md (cold path) / runbooks/clickhouse.md (hot) |
| tenant-bootstrap-latency | runbooks/tenant-onboard-failure.md |
| keeper-quorum-availability | runbooks/keeper-quorum-recovery.md |
| cluster-availability | runbooks/clickhouse.md |
| regulator-export-success | runbooks/clickhouse.md + cosign-key rotation runbook (deferred) |
| billing-reconciliation | runbooks/mv-lag-triage.md |

### T6 — Synthetic burn test

File: `.github/workflows/slo-synthetic-burn.yml`

A weekly job in dev cell injects synthetic burn against each SLO:

1. Inject 100 fake-slow query observations.
2. Wait for AlertManager to fire.
3. Verify PagerDuty webhook received the alert.
4. Resolve.

This catches the case where an OpenSLO source compiles but the alert wire is broken.

### T7 — SLO composition arithmetic (per ADR-0180-slo-composition)

Composite SLO: the analytics µservice availability = MIN(clickhouse-cluster-availability, dashboard-api-availability, audit-log-availability). Per ADR-0180-slo-composition, composite SLOs are computed downstream by the observability µservice from the leaf SLOs authored here. This IP does not author the composite directly.

### T8 — Per-tenant SLO opt-in (deferred)

Tenants on paid tenant_class contract overlays may have a contracted SLO different from the fleet default (e.g., 99.99% instead of 99.95%). This is deferred — phase 2 — and tracked at `evidence/per-tenant-slo-roadmap.md` (deferred).

## Out of scope

- Composite SLOs (downstream of leaf SLOs; observability µservice owns).
- Per-µservice burn-rate budget reallocation (per ADR-0180-slo-composition).
- ML-based anomaly detection (per ADR-0180-anomaly).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| OpenSLO source compiles but alert wire broken | synthetic burn test fails | fix the route; re-run |
| AlertManager misroutes | synthetic burn lands in wrong receiver | fix the route match |
| PagerDuty integration key invalid | webhook 401 | rotate via OpenBao |
| Burn-rate threshold tuned wrong | recurring false-positive | adjust threshold via ADR amendment |

## SLO commitment (downstream — self)

This IP authors the SLOs. The SLO targets themselves are recorded in each source. Summary:

| SLO | Target |
|---|---|
| ClickHouse query latency | p99 < 500ms |
| Ingest lag | p99 < 5s |
| Dashboard API | p99 < 500ms |
| Audit-log hot | p99 < 800ms |
| Audit-log cold | p95 < 2s |
| Tenant bootstrap | p99 < 30s |
| Keeper quorum | 99.99% leader present |
| Cluster availability | 99.95% |
| Regulator export success | 99.9% |
| Billing reconciliation | 99.99% within 0.01% drift |

## Rollback

- Each SLO source is independently revertable.
- Disabling an SLO does not affect data plane; only alerting stops.

## Evidence emission

- Per SLO compilation: sloth emits `_generated/*.rules.yaml`.
- Per alert fired: AlertManager logs to the observability µservice.
- Per synthetic burn run: result emitted to `evidence/slo-synthetic-burns/analytics/<date>.json`.

## References

- ADR-0139 4-window alerting.
- ADR-0186 Stage 5 OpenSLO authoring.
- ADR-0180-slo-composition-inheritance-arithmetic.
- `microservices/analytics/slos/*.openslo.yaml`.
- Sloth project: https://sloth.dev
- Google SRE Workbook ch. 5 (alerting on SLOs): https://sre.google/workbook/alerting-on-slos/

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/analytics/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/analytics/specs/IP-014-self-slo-burn-rate-alerts.md:1` - # IP-014 — Self-SLO Authoring + 4-Window Burn-Rate Alerts; `microservices/analytics/specs/IP-014-self-slo-burn-rate-alerts.md:5` - **Authority ADRs:** ADR-0139 4-window alerting, ADR-0186 Stage 5 SLO authoring, ADR-0180-slo-composition-inheritance-arithmetic.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/analytics/specs/IP-014-self-slo-burn-rate-alerts.md:195` - ## Evidence emission.
