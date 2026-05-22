---
doc_class: MigrationPlaybook
microservice: observability
source_vendor: Datadog
related_adrs: [ADR-0130, ADR-0131, ADR-0329, ADR-0330, ADR-0331]
date: 2026-05-20
doc_status: published
---

# Migration Playbook — Datadog → oyatie observability

Audience: an SRE team currently on Datadog APM + Metrics + Logs + SLO who wants to move onto the oyatie observability substrate over 8-12 weeks without an observability gap.

Outcome: every µservice's telemetry flows to oyatie's Tempo/Mimir/Loki/ClickHouse fleet; SLOs are authored in OpenSLO and gated by ADR-0130; dashboards live in self-hosted Grafana; Datadog is decommissioned per pack policy.

## Phase 0 — discovery (1 week)

1. Inventory Datadog usage:
   - Active services on Datadog APM (`Settings → Service Catalog`).
   - Active dashboards (export count via `dogapi dashboard list`).
   - Active monitors (`dogapi monitor list --output json > datadog-monitors.json`).
   - Active SLOs (`dogapi service-level-objective list --output json > datadog-slos.json`).
   - Custom metrics + cardinality (`dogapi metric list-active`).
   - Log indexes + retention.
2. Inventory contractual exposure: Datadog contract end date, minimum commit, per-host pricing, span retention SLA.
3. Identify pack-bound services: any service that handles KR-PIPA / EU-AI-Act / HIPAA-Provider data MUST move first because Datadog cannot host their telemetry under sovereign-pack rules.
4. Estimate target tenant_class posture: demo_trial for constrained trial cells; paid for production scale; paid plus sovereign compliance pack for residency.

Deliverable: `migration-plan.md` enumerating every Datadog artefact, current cost, target oyatie tenant_class posture, and migration priority.

## Phase 1 — dual-emit prep (week 2)

1. Stand up the oyatie observability stack in your target cell. Use the IaC at `microservices/observability/iac/` — Helm charts for ClickHouse, Tempo, Mimir, Loki, Grafana, OTel collector. Smoke-test by emitting a synthetic span and confirming it lands in ClickHouse via Step 1 of `tutorials/wire-microservice-slos-into-promotion-gate.md`.
2. In the OTel collector pipeline, configure the `datadog` exporter as a parallel exporter alongside the oyatie ClickHouse exporter. Reference config:

```yaml
exporters:
  otlphttp/clickhouse:
    endpoint: http://clickhouse-keeper.observability.svc.cluster.local:8123
  datadog:
    api:
      key: ${DD_API_KEY}
    metrics:
      delta_ttl: 3600
    traces:
      span_name_remappings: {}
service:
  pipelines:
    traces:
      exporters: [otlphttp/clickhouse, datadog]
    metrics:
      exporters: [otlphttp/clickhouse, datadog]
```

3. Confirm: same telemetry arrives at both Datadog (you'll see new traffic on the DD service catalog) AND oyatie (queryable in Grafana).

## Phase 2 — SLO translation (weeks 3-4)

Datadog SLOs are not OpenSLO-compatible by default. For each Datadog SLO:

1. Export the Datadog SLO definition: `dogapi service-level-objective get <slo-id>`.
2. Map fields:
   | Datadog field | OpenSLO field |
   |---|---|
   | `name` | `metadata.name` |
   | `description` | `spec.description` |
   | `thresholds[0].target_display` | `spec.objectives[0].target` |
   | `thresholds[0].timeframe` | `spec.objectives[0].window` |
   | `query.numerator` | `spec.indicator.spec.ratioMetric.good.metricSource.spec.query` |
   | `query.denominator` | `spec.indicator.spec.ratioMetric.total.metricSource.spec.query` |
3. Translate the Datadog query language to PromQL. Common patterns:
   - DD `sum:metric.name{tags}` → PromQL `sum(metric_name{tags})`.
   - DD `count_not_null` → PromQL `count`.
   - DD `as_rate()` → PromQL `rate(...[5m])`.
4. Run the dryrun: `cargo run -p oya-dev-cli -- observability dryrun-slo --ms <ms> --slo <name> --window 30d`.
5. Compare the dryrun SLI to the historical Datadog SLI for the same window. If they diverge by > 0.5 percentage points, the PromQL query is mis-translated — re-check the rate windows and label matchers.

Automate the bulk translation with:

```sh
cargo run -p oya-dev-cli -- observability migrate-from-datadog \
    --datadog-slos datadog-slos.json \
    --output microservices/<ms>/slos/
```

This produces OpenSLO YAML for every Datadog SLO; review each before committing.

## Phase 3 — dashboard translation (weeks 5-6)

1. Export Datadog dashboards: `dogapi dashboard list | jq -r '.dashboards[].id' | xargs -I {} dogapi dashboard get {} > dashboards/dd-{}.json`.
2. For each, run the converter: `cargo run -p oya-dev-cli -- observability dashboard-import --source datadog --input dashboards/dd-<id>.json --output microservices/<ms>/dashboards/<name>.json`.
3. The converter maps:
   - DD `timeseries` widget → Grafana `timeseries` panel.
   - DD `query_value` → Grafana `stat`.
   - DD `toplist` → Grafana `table` with sort by value desc.
   - DD `heatmap` → Grafana `heatmap`.
   - DD `service_map` → no Grafana equivalent; reference the oyatie service-map dashboard (`microservices/observability/dashboards/service-topology.json`) instead.
4. PromQL query translation: same rules as Phase 2.
5. Provision into Grafana: `kubectl -n observability rollout restart deploy/grafana`.
6. Side-by-side verify: open the original DD dashboard and the new Grafana dashboard in adjacent windows; confirm the same values render. Discrepancies usually mean a PromQL query window mismatch.

## Phase 4 — monitor → alert translation (week 7)

Datadog monitors map to oyatie SLO `alertPolicies`. For each monitor:

1. Determine which SLO it should attach to. If the monitor is standalone (not SLO-bound), it likely doesn't belong in the substrate — substrate alerts are SLO-bound by construction. Either fold it into an SLO or move it to a separate alerting path.
2. Attach the corresponding multi-window burn-rate policy:

```yaml
spec:
  alertPolicies:
    - alertPolicyRef: multi-window-burn-rate-fast
    - alertPolicyRef: multi-window-burn-rate-slow
```

3. Configure delivery via the `notifications` µservice (Slack/Discord/Telegram/PagerDuty/Opsgenie are all supported routes).
4. Acknowledge silence: any DD monitor with a configured silence schedule maps to an OpenSLO `silenceWindows[]` entry.

## Phase 5 — log routing (week 8)

1. Update your app's logging library to emit OTLP logs (most languages via `otel-log-exporter`).
2. Configure the OTel collector with a `loki` exporter alongside Datadog logs.
3. Cut over log queries from DD log explorer to LogQL via the Grafana Explore tab. Common patterns:
   - DD `service:foo @level:error` → LogQL `{service="foo"} | level="error"`.
   - DD log facet histograms → LogQL `sum by (level) (count_over_time({service="foo"}[5m]))`.

## Phase 6 — cutover + Datadog wind-down (weeks 9-12)

1. Disable the Datadog exporter in the OTel collector pipeline; oyatie becomes the sole telemetry destination.
2. Monitor for 2 weeks for any teams still pointing at Datadog dashboards or DD-API integrations. Audit via `dogapi audit-log list`.
3. After 2 weeks of clean operation: cancel the Datadog contract. Honour any minimum-commit period.
4. Update the µservice's `ARCHITECTURE.md` § "Observability" to reference oyatie substrate exclusively.
5. Open the ADR-0130 promotion-gate check: `cargo run -p oya-dev-cli -- observability check-promotion-eligibility --ms <ms> --from staging --to prod`. Confirm green.
6. Cross-emit a `migration_complete` event to `audit-chain` for retention evidence.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| Cardinality explosion when emitting DD tags as Prometheus labels | Audit your span attributes + metric labels; move high-cardinality fields (request_id, user_id) to trace attributes or log fields |
| Datadog tags with `:` characters confusing Prometheus | Run `cargo run -p oya-dev-cli -- observability label-sanitize` to convert `:` → `_` |
| PromQL `rate()` window mismatch with DD's `as_rate()` over different baseline windows | Standardise on 5 m rate windows; document any exceptions |
| Custom Datadog metric forwarders (DogStatsD aggregators) | Replace with the OTel SDK's metric exporter; statsd is bridgeable but not first-class |
| Datadog Synthetics tests | Migrate to k6 or Playwright-based synthetic tests; the `observability` µservice doesn't run synthetics natively (different substrate) |

## Cost validation checklist

After cutover, confirm cost reduction:

```sh
cargo run -p oya-dev-cli -- observability tco-report --since-date 2026-01-01
```

Compare against the last 12 months of Datadog invoices. Expected reduction under paid tenant_class high-scale envelope: ~ 60 % vs Datadog Enterprise at equivalent scale.
