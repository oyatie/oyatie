# IP-015: self-observability SLO wiring + HG-CONSENT registration

- Bounded context: self-observability
- Layers: app + wiring
- Crates:
  - `oya-consent-graph-self-observability` (slice; wraps all 6 BC apps for export)
- Acceptance status: ga
- Authority: ADR-0130 (agentic SLO-gated promotion — every µservice must self-emit SLO data BEFORE
  promotion past dev), ADR-0214 §7 verification.
- Depends on: `oya-shared-otel-exporter`, `oya-shared-prometheus-registry`,
  `microservices/observability` (substrate).

## 1. Goal

Wire all 9 SLO manifests authored in `slos/*.openslo.yaml` to:
1. Prometheus metric emission (every metric named in an SLI exists + is incremented).
2. OTEL trace exporter (consent-graph traces flow into observability µservice).
3. HG-CONSENT (Hyperscaler Group "consent") registration so observability µservice routes alerts
   correctly.

Without IP-015, the SLOs are paper-only; with it, they're live in dev and produce burn-rate alerts.

## 2. Metric emission inventory

| SLO | Metric(s) emitted by |
|-----|----------------------|
| `consent-grant-latency` | `oya_consent_graph_grant_duration_seconds_bucket` (agreement-usecase) |
| `cross-tenant-projection-freshness` | `oya_consent_graph_projection_lag_seconds_bucket` (projection-gateway-worker) |
| `revocation-propagation-latency` | `oya_consent_graph_revocation_propagation_seconds_bucket` (revocation-worker + subscribers) |
| `cedar-evaluation-latency` | `oya_consent_graph_cedar_eval_duration_seconds_bucket` (enforcement-usecase) |
| `audit-chain-coverage-completeness` | `oya_consent_graph_audit_emit_total{outcome=...}` (audit-bridge-worker) |
| `agreement-state-divergence-zero` | `oya_consent_graph_state_divergence_total`, `oya_consent_graph_state_check_total` (agreement-worker reconciliation task) |
| `sovereignty-violation-zero` | `oya_consent_graph_sovereignty_violation_total`, `oya_consent_graph_projection_emit_total` (projection-gateway-kernel guard + emitter) |
| `bilateral-chain-link-integrity` | `oya_consent_graph_bilateral_link_total{outcome=...}` (audit-bridge cross-pointer reconciler) |
| `partner-handshake-latency` | `oya_consent_graph_handshake_duration_seconds_bucket` (partner-directory-usecase) |

## 3. Cardinality discipline

Per ADR-0119 (cardinality-bounded observability), every metric label is bounded:

| Label | Source | Bound |
|-------|--------|-------|
| `region` | k8s topology | ≤16 |
| `mode` | SharingMode enum | 3 |
| `outcome` | enum per metric | ≤8 |
| `action_taken` | PropagationAction enum | 5 |
| `tier` | TenantClass enum | 4 |
| `pack` | regulatory pack | 11 |

No high-cardinality labels (`agreement_id`, `tenant_id`, `entity_id`, `principal_id`) appear in any
metric — those go to OTEL traces only.

Total cardinality budget for consent-graph metrics: ≤200K time-series across all instances.

## 4. OTEL trace span tree

```
consent_graph.api_request                          (root span, per-API-call)
├── consent_graph.enforcement.evaluate             (hot-path)
│   ├── consent_graph.cache_lookup
│   ├── consent_graph.policy_compile               (cache miss only)
│   │   └── consent_graph.agreement_load
│   ├── consent_graph.cedar_evaluate
│   └── consent_graph.audit_emit_async
└── consent_graph.projection.emit                  (worker-driven span tree)
    ├── consent_graph.scope_narrow
    ├── consent_graph.pii_classifier_check
    ├── consent_graph.sovereignty_assert
    └── consent_graph.pulsar_publish
```

Trace exporter sends to observability µservice's OTEL collector via OTLP/gRPC. Tail sampling per
ADR-0210 (1% baseline, 100% on Deny outcomes, 100% on errors).

## 5. HG-CONSENT registration

The observability µservice maintains a `hyperscaler_group` taxonomy (HG-OBS, HG-AUDIT, HG-IDENT, ...).
A new µservice registers via:

```yaml
# microservices/consent-graph/iac/helm/consent-graph/templates/hg-consent-registration.yaml
apiVersion: observability.oya.dev/v1
kind: HyperscalerGroupRegistration
metadata:
  name: hg-consent
spec:
  microservice: consent-graph
  slos:
    - oya-consent-graph-consent-grant-latency
    - oya-consent-graph-cross-tenant-projection-freshness
    - oya-consent-graph-revocation-propagation-latency
    - oya-consent-graph-cedar-evaluation-latency
    - oya-consent-graph-audit-chain-coverage-completeness
    - oya-consent-graph-agreement-state-divergence-zero
    - oya-consent-graph-sovereignty-violation-zero
    - oya-consent-graph-bilateral-chain-link-integrity
    - oya-consent-graph-partner-handshake-latency
  alert_routing:
    p0: pagerduty:axis-consent-graph-p0
    p1: pagerduty:axis-consent-graph-p1
    p2: slack:#axis-consent-graph
    p3: email:axis-consent-graph@oya
  dashboards:
    - consent-grant-funnel
    - projection-freshness
    - revocation-fan-out
    - bilateral-chain-integrity
```

Observability µservice's admission controller validates the registration on apply.

## 6. Self-burn-rate alerts

Per ADR-0130, every SLO gets a 1h/6h fast-burn page rule (≥14.4× budget burn) and a 24h slow-burn
page rule (≥3× budget burn). Generated by observability µservice's PrometheusRule template; consent-graph
just declares the SLO manifests + HG registration.

Examples:
- `OyaConsentGraphRevocationPropagationFastBurn1h14x` → page P0
- `OyaConsentGraphSovereigntyViolationAny` → page P0 (target=1.0, any violation = instant page)
- `OyaConsentGraphCedarEvalLatencySlow24h3x` → ticket P2

## 7. Dashboards

3 dashboards delivered (`dashboards/*.json`, Grafana):

1. **consent-grant-funnel** — funnel from Drafted → Offered → Accepted → Active per region/pack/mode.
   Tracks drop-off rate; surfaces partner-onboarding friction.
2. **projection-freshness** — per-(grantor-region, grantee-region, mode) projection lag histogram +
   SLO error budget gauge.
3. **revocation-fan-out** — per-revocation propagation map; subscriber-by-subscriber latency; deadline
   reconciliation rate.

A fourth dashboard `bilateral-chain-integrity` is added in IP-013 (it's natively cross-pointer-reconciliation
focused).

## 8. Tests

- `every_slo_metric_present_at_runtime` — start consent-graph; scrape Prometheus; assert every metric
  named in any SLO file exists.
- `cardinality_under_budget` — synthetic 1h workload; total time-series ≤200K.
- `trace_export_smoke` — single API call → trace appears in collector with expected span tree shape.
- `hg_registration_admission_accepted` — observability admission controller accepts the registration.
- `burn_rate_alert_fires_on_synthetic_breach` — inject 100% failure for 1min; PrometheusRule fires.

## 9. Verification

- `cargo build` + `cargo test` clean.
- `helm template microservices/consent-graph/iac/helm/consent-graph/` renders 0-error.
- `oya-dev-cli observability validate-hg-registration hg-consent` passes.
- Synthetic SLO burn test: spike revocation latency to 5s for 5min → fast-burn alert fires within
  90s.

## 10. Risk

- **R**: A new metric added in a follow-up IP but not registered → silently missing from dashboards.
  **M**: CI lane `oya-check-slo-metric-coverage` lints that every metric referenced in an OpenSLO
  SLI is emitted by at least one crate.
- **R**: HG-CONSENT registration drift if observability µservice schema changes.
  **M**: HG schema versioned; consent-graph CI lane validates against current schema; bump on schema
  change.
- **R**: Cardinality explosion via accidental high-cardinality label.
  **M**: `oya-check-cardinality-budget` enforces label allow-list per metric.

## 11. Out of scope (deferred)

- Custom anomaly-detection ML on consent-graph metrics (observability µservice supplies a generic
  baseline; bespoke models in PHASE-02+).
- Cross-µservice trace correlation (consent-graph → audit-chain → ontology spans linked via shared
  trace-id; already free via OTEL).
- Real-time dashboard streaming (Grafana refresh is 30s baseline; sub-30s is out of scope).

## 12. Output

After this IP lands:
- 9 SLOs live in dev with PrometheusRule alerts.
- 4 dashboards published in observability µservice's Grafana.
- HG-CONSENT registered.
- Trace export flowing.

This satisfies ADR-0130 promotion gate: consent-graph may now proceed from dev → stage on the
SLO-gated promotion path.

## Wave 15-IP-substance counterpart evidence

Preserved as substantive. Counterpart anchors: OneTrust publishes selected API SLOs and rate limits; Snowflake/Databricks/AWS provide operational monitoring for their data-share products; TrustArc/Cookiebot emphasize reporting rather than service-owned SLO promotion. This IP makes consent-graph's counterpart posture machine-checkable through nine OpenSLO manifests, HG-CONSENT registration, dashboards, and promotion gates.
