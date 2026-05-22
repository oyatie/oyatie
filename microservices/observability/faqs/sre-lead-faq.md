---
doc_class: FAQ
microservice: observability
persona: sre-lead
related_adrs: [ADR-0130, ADR-0131, ADR-0329, ADR-0330, ADR-0331]
date: 2026-05-20
doc_status: published
---

# observability — SRE Lead FAQ

## Q1: Why does ADR-0130 force every µservice to have SLOs before staging promotion? Why not let teams decide?

Because the staging→prod promotion gate is automated. Without an SLO, the gate has nothing to evaluate. The historical pattern (Datadog/New Relic-style "monitor your service however you want") produces fleet-wide observability inconsistency that makes cross-service incident-correlation impossible. Per ADR-0130 the answer is "every µservice authors at least 3 OpenSLO manifests; the engine evaluates them uniformly". You can argue the specific objectives (target percentages, windows) — you cannot argue the requirement that they exist.

## Q2: What's the difference between demo_trial tail-sampling at 1 % and paid tenant_class tail-sampling at 10 %? Do I lose 90 % of my traces at demo_trial?

You lose 90 % of *non-error, non-latency-tail* traces. The tail-sampler always retains 100 % of error spans, 100 % of latency-tail spans (above the 95th percentile of your service's recent latency), and 100 % of cross-tenant-correlation spans (any span carrying a tag indicating it crosses tenant boundaries). What gets sampled out for demo_trial is the long tail of normal-path requests that are individually uninteresting. For debugging a specific tenant's specific failed request, demo_trial is sufficient because the failed request lands in the error-spans bucket and is retained. For statistical analysis of normal-path latency distribution, demo_trial under-samples and paid tenant_class is the correct posture.

## Q3: Our µservice emits 200 M active metric series. Mimir for demo_trial caps at 5 M. What do I do?

Stop emitting 200 M series. That cardinality is almost always a label-explosion bug — typically `user_id`, `request_id`, or some other high-cardinality field added as a Prometheus label. Audit your metric-emission code: any label that has > 1000 distinct values should not be a label. Move it to a span attribute (low-cost in traces) or a log field (low-cost in logs). If after audit you legitimately need > 5 M series, move the tenant to paid baseline (25 M cap) or paid high-scale (100 M cap), but most cases are bugs not capacity.

## Q4: I want to author an SLO against a metric that doesn't exist yet. The metric is hard to add. Can I skip it?

No. The promotion gate evaluates SLOs against real metrics. An SLO without a metric is unevaluable and the gate will refuse to lift. File an IP for the missing metric instrumentation, ship it, wait one week for baseline data, then author the SLO. The temptation to "author the SLO now and instrument later" is the path to fleet-wide aspirational SLOs that nobody believes — and once nobody believes them, the substrate loses its meaning.

## Q5: How does the multi-window-multi-burn alert know to fire on a 1-h window vs a 2-min window?

It fires on both simultaneously. The kernel evaluates the burn rate over the short window (2 min) and the long window (1 h or 6 h depending on the SLO's `window` field). The alert fires only if BOTH windows exceed their respective thresholds — the short window catches rapid degradation, the long window prevents false positives from a single bad minute. This is the Google SRE Workbook Ch. 5 multi-window burn-rate algorithm; we implemented it as the SLO-engine kernel (IP-003).

## Q6: ClickHouse vs Tempo vs Mimir vs Loki — why four data stores instead of one?

Each is optimised for a different signal class and each has different retention + query economics. ClickHouse holds the rollup tables (cross-signal correlation, executive dashboards, regulator-evidence exports); Tempo holds raw trace blocks (cheap parquet on S3); Mimir holds TSDB metric blocks (cheap chunks on S3); Loki holds log chunks (cheap chunks on S3). The alternative — "put everything in ClickHouse" — was prototyped at the substrate-design stage and was rejected because (a) ClickHouse logs ingest is 3× more expensive than Loki at our log volumes, (b) Tempo's parquet block format with the Apache Parquet predicate-pushdown query path is 2× faster than ClickHouse for trace-id lookup at 7-d retention, (c) Mimir's chunk-store + TSDB is the upstream-canonical Prometheus query path and replacing it with ClickHouse would mean reimplementing PromQL.

## Q7: What happens if ClickHouse goes down? Do I lose all observability?

Tempo + Mimir + Loki are independently resilient. ClickHouse is the rollup + correlation layer. If ClickHouse is down, you lose cross-signal correlation views and the executive dashboards for ~ the duration of the outage, but Tempo (traces), Mimir (metrics), and Loki (logs) continue ingesting + serving queries normally. ClickHouse data is reconstructible from those three primaries via the rollup pipeline (`oya-observability-rollup-worker`). The longest realistic ClickHouse outage we drilled was 4 h (single-AZ Patroni leader failover on a corrupted disk), during which paid tenants saw degraded "correlation" views but no degradation in primary signal queries.

## Q8: My CFO wants to know why we're not on Datadog. What's the answer?

Three reasons: (1) cost — Datadog at 100 k events/sec sustained + 1 PiB cold-retention envelope runs ~ 2.3 M USD/yr at list price; oyatie paid baseline runs ~ 510 k USD/yr. Even at a 50 % Datadog discount we save 700 k USD/yr per cell. (2) sovereignty — Datadog cannot host KR-PIPA or EU-AI-Act-resident telemetry without crossing pack boundaries; oyatie paid sovereign-pack posture is air-gap-resident per pack. (3) substrate-fit — the promotion-gate evidence that ADR-0130 requires has to be queryable by other oyatie µservices (`oya-vcs` for merge admission, `audit-chain` for cryptographic anchoring); Datadog's API is rate-limited (~ 300 qps) and lacks the cross-signal correlation our substrate depends on.

## Q9: How do I know if my SLO objectives are too tight or too loose?

Run `cargo run -p oya-dev-cli -- observability slo-realism-report --ms <your-ms>`. This compares each SLO's `objective.target` against the actual SLI value over the last 30 d. If the SLI is consistently > 1 percentage point above the objective, the objective is too loose (you're not gaining alerting value). If the SLI is consistently below or within 0.1 percentage points of the objective, it's too tight (you'll alert constantly). The sweet spot is "objective slightly below where you actually operate, with realistic error budget consumption of 30-70 % per month".

## Q10: Can I emit telemetry directly to Grafana Cloud / Datadog as a parallel destination?

Technically yes (OTel collector can fan out to multiple exporters). Practically no — the substrate is the source of truth for promotion-gate evidence, and fanning out to an external SaaS creates a divergent observability surface where teams start trusting the external view and stop trusting the substrate. Per ADR-0130 § "no parallel observability", a µservice MAY emit a sampled mirror to an external system for cross-org debugging IF (a) it's via the OTel collector's official exporter, (b) the substrate remains the primary destination with no fidelity loss, (c) it's documented in the µservice's ARCHITECTURE.md, and (d) the cost is on the µservice's own budget not the substrate's.

## Q11: My breach alert keeps firing for the same root cause and I want to silence it for 7 days while we fix it. How?

Use the SLO-engine's `silence` action: `cargo run -p oya-dev-cli -- observability silence-slo --ms <your-ms> --slo availability --duration 7d --reason "fixing root cause IP-1234, will release 2026-06-01" --approver <human-name>`. This requires an approver (Cedar permit `observability::slo::silence` is restricted to `sre-lead` + `engineering-manager`), and the silence is logged to `audit-chain` so it's discoverable in post-incident review. Silences NEVER hide the underlying SLI; the breach evidence is still recorded and counts against the error budget for promotion-gate purposes.

## Q12: How do I tune the breach-detection latency? Can I make it faster than 30 s p99 (paid baseline)?

Not without a paid tenant_class capacity exception. The latency is dominated by the Pulsar consumer lag + ClickHouse write commit + multi-window burn-rate evaluation cycle. paid baseline runs the evaluation every 30 s; paid high-scale runs every 10 s; paid sovereign-pack posture runs every 5 s. You can ask for a per-µservice exception on paid baseline (e.g. run a specific SLO at 10 s cadence) but it consumes ClickHouse query slots from other tenants and we ration this carefully. File a request with `iam::role::sre-lead` + a justification; the substrate team reviews weekly.
