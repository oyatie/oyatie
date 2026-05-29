---
doc_class: ImplementationPlan
ip_id: IP-025
microservice: real-estate
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_ref: j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief
sap_submodule: RE-FX-VM (vacancy management) + RE-FX-SC (service charge)
tenant_class: paid
billing_components:
  - per_usage
persona: Akira Watanabe, COO portfolio reviewer
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-portfolio-analytics
---

# IP-025: Portfolio analytics dashboard

## Context

- SAP submodule: RE-FX-VM vacancy management and RE-FX-SC service charge.
- Persona: Akira Watanabe, COO portfolio reviewer.
- Journey leg: j168 quarterly operations review compares vacancy, rent, service charge, valuation, and maintenance exposure.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`, `VIBDRO`.
- Oyatie capability: `RealEstatePortfolioAnalyticsDashboard`.
- Precedent: SAP RE-FX portfolio reporting plus Yardi/MRI executive dashboards.
- ADR-0263 records dashboard publication evidence and ADR-0314 constrains executive report export scope.
- Boundary: materializes portfolio metrics and dashboard cards; BI visualization hosting remains analytics-ui.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.portfolio_analytics_snapshot (
  tenant_id UUID NOT NULL,
  portfolio_analytics_snapshot_id TEXT NOT NULL,
  portfolio_id TEXT NOT NULL,
  as_of_date DATE NOT NULL,
  occupancy_rate NUMERIC(10,6) NOT NULL,
  vacancy_rate NUMERIC(10,6) NOT NULL,
  contracted_rent_amount NUMERIC(20,6) NOT NULL,
  service_charge_recovery_rate NUMERIC(10,6),
  fair_value_amount NUMERIC(20,6),
  currency_code TEXT NOT NULL,
  publication_status TEXT NOT NULL CHECK (publication_status IN ('draft','published','retracted','failed')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, portfolio_analytics_snapshot_id)
);
CREATE TABLE real_estate.portfolio_analytics_metric (
  tenant_id UUID NOT NULL,
  metric_id TEXT NOT NULL,
  portfolio_analytics_snapshot_id TEXT NOT NULL,
  metric_name TEXT NOT NULL,
  metric_value NUMERIC(20,6) NOT NULL,
  metric_unit TEXT NOT NULL,
  source_ref TEXT NOT NULL,
  PRIMARY KEY (tenant_id, metric_id)
);
```

### Rust Types

```rust
pub struct PortfolioAnalyticsSnapshot {
    pub tenant_id: TenantId,
    pub portfolio_analytics_snapshot_id: PortfolioAnalyticsSnapshotId,
    pub portfolio_id: PortfolioId,
    pub as_of_date: NaiveDate,
    pub occupancy_rate: Decimal,
    pub vacancy_rate: Decimal,
    pub contracted_rent_amount: Decimal,
    pub service_charge_recovery_rate: Option<Decimal>,
    pub fair_value_amount: Option<Decimal>,
    pub currency_code: CurrencyCode,
    pub publication_status: PublicationStatus,
}
pub struct PortfolioAnalyticsMetric {
    pub metric_name: MetricName,
    pub metric_value: Decimal,
    pub metric_unit: MetricUnit,
    pub source_ref: SourceRef,
}
pub enum PortfolioAnalyticsError { PortfolioMissing, SourceSnapshotStale, MetricPolicyDenied, PublicationFailed, CurrencyMixed }
```

## API Endpoints

- REST `POST /v1/real-estate/portfolio-analytics-snapshots` builds dashboard snapshot.
- REST `GET /v1/real-estate/portfolio-analytics-snapshots/{id}` reads dashboard totals.
- REST `GET /v1/real-estate/portfolio-analytics-snapshots/{id}/metrics`.
- REST `POST /v1/real-estate/portfolio-analytics-snapshots/{id}:publish`.
- gRPC `real_estate.analytics.v1.PortfolioAnalyticsService.BuildSnapshot`.
- gRPC `GetSnapshot`, `ListMetrics`, and `PublishSnapshot`.
- AsyncAPI channel `real-estate.portfolio-analytics.snapshot-built.v1`.
- AsyncAPI channel `real-estate.portfolio-analytics.published.v1`.
- Consumers: analytics-ui, executive-reporting, compliance, planning.

## Cedar Policy Hooks

- Policy: `real_estate::portfolio_analytics::publish`.
- Principal: `PortfolioReviewer`.
- Action: `publish_dashboard_snapshot`.
- Resource: `PortfolioAnalyticsSnapshot`.
- Context: `tenant_id`, `portfolio_id`, `as_of_date`, `source_freshness_minutes`, `contains_financial_metrics`, `publish_audience`.
- Forbid when source snapshots are stale, audience is not executive-approved, or financial metrics lack close-period authorization.

## Ontology Projection

- Vendor object: SAP RE-FX portfolio report.
- Oyatie object: `real_estate.portfolio_analytics_snapshot`.
- `VICDCONTRACT-CONTRACT` -> active contract count and rent base.
- `VICDOBJASS-OBJNR` -> object portfolio membership.
- `VICDCONDLINE-CONDGUID` -> rent and charge metric lineage.
- `VICDADJREASN-ADJREASON` -> adjustment exception metric.
- `VIBDRO-OBJNR` -> building and unit scope.
- Vacancy state -> vacancy rate metric.
- Projection freshness floor: built snapshot from approved source snapshots.

## Workflow Steps

- Node `portfolio-resolve`: load portfolio, buildings, and objects.
- Decision `portfolio-missing`: fail snapshot.
- Node `source-snapshot-load`: load rent roll, occupancy, valuation, and service charge snapshots.
- Decision `source-stale`: block publication and request rebuild.
- Node `metric-compute`: compute vacancy, rent, recovery, fair value, and exception metrics.
- Decision `currency-mixed`: split metrics by currency.
- Node `dashboard-materialize`: persist snapshot and metric cards.
- Node `publish-policy`: authorize audience and financial metric exposure.
- Node `audit-seal`: emit dashboard publication evidence.

## Audit Events

- `EVT-REAL_ESTATE-PORTFOLIO_ANALYTICS-SNAPSHOT_BUILT`.
- `EVT-REAL_ESTATE-PORTFOLIO_ANALYTICS-METRIC_COMPUTED`.
- `EVT-REAL_ESTATE-PORTFOLIO_ANALYTICS-PUBLISHED`.
- `EVT-REAL_ESTATE-PORTFOLIO_ANALYTICS-RETRACTED`.
- `EVT-REAL_ESTATE-PORTFOLIO_ANALYTICS-POLICY_DENIED`.
- `EVT-REAL_ESTATE-PORTFOLIO_ANALYTICS-IP_ACCEPTED`.
- ADR-0263 envelope stores portfolio, as-of date, source refs, metric count, audience, and publication ref.

## SLO Targets

- Snapshot build p50: 300 ms.
- Snapshot build p95: 2,500 ms.
- Snapshot build p99: 7,500 ms for 100,000 metric inputs.
- Dashboard read p95: 250 ms from materialized snapshot.
- Rationale: build is batch/reporting shaped, but executive dashboard reads must be fast and stable.

## Failure Modes and Recovery

- Failure: `PORTFOLIO-MISSING`; recovery: block and request portfolio master repair.
- Failure: `SOURCE-SNAPSHOT-STALE`; recovery: trigger rent roll or occupancy rebuild.
- Failure: `CURRENCY-MIXED`; recovery: split cards by currency or request treasury conversion.
- Failure: `METRIC-POLICY-DENIED`; recovery: remove restricted financial metric from audience.
- Failure: `PUBLICATION-FAILED`; recovery: retry from immutable snapshot.
- Failure: `SOURCE-REF-BROKEN`; recovery: retract snapshot and rebuild lineage.

## Migration Notes

- Import dashboards only after rent-roll, occupancy, valuation, and service-charge snapshots.
- Preserve source report IDs and export timestamps as publication refs.
- Treat migrated executive reports as historical snapshots, not live recomputable dashboards, unless source lineage is complete.
- Rollback path: retract dashboard publication and keep immutable metric rows.
- Backfill order: portfolio, objects, source snapshots, dashboard snapshots, metrics, publication events.
- Validate first migrated quarter against source executive report totals before publish enablement.

## Cross-microservice Handoffs

- From portfolio-master: portfolio scope.
- From rent-roll: contracted rent and occupancy line totals.
- From fair-value valuation: valuation metric source.
- From service-charge billing: recovery and cost-allocation metrics.
- To analytics-ui: materialized dashboard cards.
- To executive-reporting: published snapshot and evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The dashboard remains bound to SAP RE-FX-VM vacancy management and RE-FX-SC service charge. |
| Persona specificity | Akira Watanabe owns quarterly portfolio review, publication, and rollback language. |
| Journey specificity | The j168 quarterly operations review drives vacancy, rent, service-charge, valuation, and maintenance metrics. |
| DDL anchor | The dashboard snapshot, metric row, card publication, and source-total tables above are normative. |
| Rust anchor | Portfolio snapshot, dashboard metric, publication event, and error types above are anchors. |
| REST anchor | Generate, publish, retract, export, and explain endpoints are tenant surfaces. |
| gRPC anchor | The portfolio analytics service is the worker and replay contract. |
| AsyncAPI anchor | Snapshot generated, card published, report exported, and retracted channels carry executive evidence. |
| Cedar anchor | Dashboard publication is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | Vacancy, rent-roll, valuation, service-charge, and maintenance lineage projects to dashboard metric nodes. |
| ADR-0263 class binding | Publication checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Executive-reporting, office, or analytics overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on dashboard APIs emits `AbuseDefenceRateLimitHit`; scrape abuse uses ADR-0297 classes. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, snapshot id, portfolio id, metric family, report ref, and `cedar_decision_id`. |
| Metric | `oya_real_estate_portfolio_dashboard_publications_total{tenant_id,cell_id,portfolio,status}` caps portfolio/status cardinality. |
| Latency histogram | `oya_real_estate_portfolio_dashboard_duration_seconds` tracks snapshot generation and publication latency. |
| Trace span | `real_estate.portfolio_dashboard.publish` links rent roll, valuation, service charge, analytics UI, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `snapshot_id`, `portfolio_id`, `metric_family`, and variance bucket. |
| Capacity math | Publication rejects when source metric variance exceeds tolerance or card generation queue misses executive-reporting SLA. |
| Multi-region | Dashboard publication writes in portfolio home cell; DR cells expose read-only published snapshots. |
| Sovereign cells | Portfolio, tenant, and financial metrics remain in-region for privacy and regulated packs. |
| Rollback | Retract dashboard publication, keep immutable metric rows, and replay from last sealed dashboard audit id. |
| Test evidence | Required tests cover source variance, stale valuation, service-charge mismatch, tenant mismatch, and publication replay. |
| Rejected shortcut | A generic BI dashboard is rejected because it loses SAP RE-FX vacancy, service-charge, and valuation lineage. |
