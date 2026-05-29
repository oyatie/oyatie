---
doc_class: Tutorial
microservice: observability
related_adrs: [ADR-0130, ADR-0131, ADR-0329, ADR-0330, ADR-0331]
date: 2026-05-20
doc_status: published
---

# Tutorial — Wire a µservice's SLOs into the ADR-0130 promotion gate

Goal: take a freshly scaffolded µservice (assume `oya-cloud-finops-portal` for this walkthrough) and wire it into the observability substrate end-to-end so its dev→staging→prod promotion is gate-enforced by real SLO evidence.

Prereqs:

- `oya` CLI ≥ 1.4.0 (`oya --version` to confirm).
- Cedar role `sre-lead` bound to your principal.
- The µservice already emits OTel spans + Prometheus metrics from its `oya-cloud-finops-portal-app` crate.
- `kubectl` configured against the dev cell (`kubeconfig.dev.yaml`).
- `gh` CLI authenticated (`gh auth status`).

## Step 1 — confirm telemetry is reaching ClickHouse

Open a terminal and verify the OTel collector is receiving spans from your service:

```sh
kubectl -n observability logs -f deploy/otelcol-contrib --tail=200 \
    | grep -i "service.name=oya-cloud-finops-portal"
```

You should see span batches arriving every 1-5 s. If you see nothing, your service is not exporting to the collector — check the `OTEL_EXPORTER_OTLP_ENDPOINT` env var on your pods (`kubectl -n cloud-finops-portal describe pod <pod> | grep OTEL`).

Now query ClickHouse directly:

```sh
kubectl -n observability exec -it clickhouse-keeper-0 -- \
    clickhouse-client --query "
        SELECT count(*), max(timestamp)
        FROM traces.spans
        WHERE service_name = 'oya-cloud-finops-portal'
          AND timestamp > now() - INTERVAL 5 MINUTE
    "
```

Expected output: a non-zero count and a recent timestamp. If count = 0, your collector pipeline is dropping the spans — most commonly a sample-recipe configuration error. Run `cargo run -p oya-dev-cli -- observability validate-recipe microservices/cloud-finops-portal/observability/sample-recipe.yaml` to lint.

## Step 2 — scaffold the OpenSLO manifests

Generate the three starter SLOs:

```sh
cargo run -p oya-dev-cli -- observability scaffold-slos \
    --ms cloud-finops-portal \
    --kind http-api \
    --output microservices/cloud-finops-portal/slos/
```

This writes:

- `microservices/cloud-finops-portal/slos/availability.openslo.yaml`
- `microservices/cloud-finops-portal/slos/p99-latency.openslo.yaml`
- `microservices/cloud-finops-portal/slos/error-rate.openslo.yaml`

Open the availability SLO and edit:

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: cloud-finops-portal-availability
  displayName: cloud-finops-portal availability
spec:
  service: oya-cloud-finops-portal
  description: |
    Tenants must be able to view their cost dashboards 99.9 % of the time during business hours.
    A breach means the FinOps team cannot answer "what did we spend this hour" — a P2 customer-facing incident.
  indicator:
    metadata:
      name: http-2xx-ratio
    spec:
      ratioMetric:
        counter: true
        good:
          metricSource:
            metricSourceRef: prometheus
            spec:
              query: |
                sum(rate(http_requests_total{service="oya-cloud-finops-portal",status=~"2.."}[5m]))
        total:
          metricSource:
            metricSourceRef: prometheus
            spec:
              query: |
                sum(rate(http_requests_total{service="oya-cloud-finops-portal"}[5m]))
  objectives:
    - target: 0.999
      window: 30d
  alertPolicies:
    - alertPolicyRef: multi-window-burn-rate-fast
    - alertPolicyRef: multi-window-burn-rate-slow
  annotations:
    runbook: https://github.com/oyatie/oyatie/blob/dev/microservices/cloud-finops-portal/runbooks/availability-breach.md
    dashboards: https://grafana.dev.oyatie.io/d/cloud-finops-portal-overview
```

Repeat for `p99-latency.openslo.yaml` (target: `p99 ≤ 500 ms`) and `error-rate.openslo.yaml` (target: `error_rate ≤ 0.1 %`).

## Step 3 — dryrun the SLOs against the last 7 d

```sh
cargo run -p oya-dev-cli -- observability dryrun-slo \
    --ms cloud-finops-portal \
    --slo availability.openslo.yaml \
    --window 7d
```

Sample output:

```
SLO: cloud-finops-portal-availability
Window: 7 d (2026-05-13 → 2026-05-20)
Objective: 0.999
Actual SLI: 0.9994
Status: GREEN (objective met)
Error budget consumed: 16 % (4.0 h of 30-d 0.1 % budget; 25.2 h budget remaining)
```

If the dryrun shows RED (objective not met), either tighten your µservice or loosen the target. Do not ship a known-RED SLO — the promotion gate will refuse to lift.

## Step 4 — author the breach runbook

Create `microservices/cloud-finops-portal/runbooks/availability-breach.md`:

```markdown
# cloud-finops-portal availability breach — runbook

## Symptom
HTTP 5xx ratio elevated; tenants see cost-dashboard load errors.

## First-glance dashboard panels
- https://grafana.dev.oyatie.io/d/cloud-finops-portal-overview → panel "HTTP 2xx ratio (5m)"
- https://grafana.dev.oyatie.io/d/cloud-finops-portal-overview → panel "Upstream errors by dependency"

## Top-3 likely causes + diagnostics
1. ClickHouse query failures (cost-rollup queries the FinOps portal depends on):
   ```sh
   kubectl -n observability logs -l app=clickhouse-keeper --tail=200 | grep -i error
   ```
2. cloud-billing-tax-app upstream errors (the FinOps portal calls billing for usage data):
   ```sh
   kubectl -n cloud-billing logs deploy/cloud-billing-tax-app --tail=200 | grep -E "5xx|ERROR"
   ```
3. Per-tenant rate limiter saturation:
   ```sh
   kubectl -n cloud-finops-portal exec deploy/cloud-finops-portal-app -- curl -s localhost:9090/metrics | grep oya_rate_limited_total
   ```

## Mitigation
- If cause 1: page the observability oncall (`@observability-substrate`) — restore ClickHouse availability.
- If cause 2: page the billing oncall.
- If cause 3: raise per-tenant rate-limit envelope via `oya iam set-quota --tenant <id> --limit-cost-portal-requests-per-second 200` (max 500).

## Escalation
@cloud-finops-portal-oncall → @cloud-finops-em → @vp-engineering
```

## Step 5 — provision the dashboard

Copy a sibling dashboard:

```sh
cp microservices/cloud-finops-api/dashboards/cloud-finops-api-overview.json \
   microservices/cloud-finops-portal/dashboards/cloud-finops-portal-overview.json
```

Edit the JSON: swap `service="oya-cloud-finops-api"` → `service="oya-cloud-finops-portal"` everywhere, rename the dashboard `title` field.

Provision into Grafana:

```sh
kubectl -n observability rollout restart deploy/grafana
kubectl -n observability rollout status deploy/grafana
```

Open `https://grafana.dev.oyatie.io/d/cloud-finops-portal-overview` and confirm panels render live data.

## Step 6 — simulate a breach + verify alert delivery

```sh
cargo run -p oya-dev-cli -- observability simulate-breach \
    --ms cloud-finops-portal \
    --slo cloud-finops-portal-availability \
    --duration 5m
```

Within 30 s you should receive:
- A Slack/Discord/Telegram alert (per your team's `notifications` µservice config) with the runbook URL embedded.
- An event on the `slo.breach.cloud-finops-portal.availability` Pulsar topic.
- A row in ClickHouse: `SELECT * FROM observability.slo_breaches WHERE service_name = 'oya-cloud-finops-portal' ORDER BY timestamp DESC LIMIT 5`.

Acknowledge the synthetic breach:

```sh
cargo run -p oya-dev-cli -- observability acknowledge-breach \
    --ms cloud-finops-portal \
    --slo cloud-finops-portal-availability \
    --reason "synthetic test, not a real incident"
```

## Step 7 — check the promotion gate

```sh
cargo run -p oya-dev-cli -- observability check-promotion-eligibility \
    --ms cloud-finops-portal \
    --from dev \
    --to staging
```

Expected output:

```
cloud-finops-portal: dev → staging
  ✔ sample-recipe present
  ✔ ≥ 3 SLOs authored
  ✔ all SLOs dryrun-green over last 7 d
  ✔ all SLOs have runbook annotations
  ✔ dashboard provisioned
  ✔ breach simulation green within last 24 h
  Status: ELIGIBLE for staging promotion
```

If any check is ✘, the gate explains what's missing. Resolve and re-run.

## Step 8 — open the PR + merge

```sh
git checkout -b ip-cloud-finops-portal-observability
git add microservices/cloud-finops-portal/{slos,dashboards,runbooks,observability}
git commit -m "Wire cloud-finops-portal into ADR-0130 promotion gate"
git push -u origin ip-cloud-finops-portal-observability
gh pr create --base dev --title "Wire cloud-finops-portal into observability substrate" \
    --body "Per ADR-0130 + IP-002/IP-003/IP-030. Promotion gate now eligible."
```

After merge, the staging promotion lifts automatically within 5 min via the event-driven workflow (IP-013).

## What you've accomplished

Your µservice now has:
- 3+ OpenSLO manifests, dryrun-green.
- Tail-sampling configured per IP-030.
- Dashboards provisioned + tenant-visible.
- Runbooks committed + linked from SLO annotations.
- Synthetic breach validation green.
- ADR-0130 promotion-gate evidence chain complete.

The substrate now has authoritative observability for your µservice and the promotion pipeline trusts it.
