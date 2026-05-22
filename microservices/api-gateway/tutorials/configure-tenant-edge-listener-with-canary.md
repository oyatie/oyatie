---
doc_class: Tutorial
microservice: api-gateway
persona: edge-engineer + tenant-platform-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Configure a tenant edge listener with HTTP/3, WAF, rate-limit, and canary rollout

You will: provision an edge listener for tenant `acme`, configure HTTP/3+QUIC, attach the OWASP CRS 4.3 WAF, set rate-limit under the paid tenant_class policy (50 k sustained / 500 k burst), then deploy via canary-cohort-shifter (5 % → 15 % → 50 % → 100 %). Total time ≤ 90 minutes.

## Pre-requisites

- A paid tenant_class-eligible api-gateway cell.
- Tenant `drill-acme` provisioned (per tutorial `microservices/tenancy/tutorials/provision-tenant.md`).
- An ACME issuer (Let's Encrypt prod or a private internal CA).
- A target upstream µservice (we'll use `billing-api.drill-syd-1.oyatie.local:8443`).

## Step 1 — Define the listener config (≤ 10 min)

```sh
oya api-gateway listener init \
    --tenant drill-acme \
    --listener tenant-edge-acme-v1 \
    --output ./configs/acme-edge-v1.yaml
```

This emits a starter yaml. Open it + edit:

```yaml
listener:
  name: tenant-edge-acme-v1
  tenant: drill-acme
  bind:
    - host: edge.acme.oyatie.io
      port: 443
      protocols: [http2, http3]   # h3 default per ADR-0253
  tls:
    cert_source:
      type: acme
      issuer: https://acme-v02.api.letsencrypt.org/directory
      contact: ops@acme.example
    cipher_suites:
      - TLS_AES_256_GCM_SHA384
      - TLS_CHACHA20_POLY1305_SHA256
      - TLS_AES_128_GCM_SHA256
    min_version: TLSv1.3
    hsm:
      backend: cloudhsm
      cluster_id: hsm-cluster-syd-1
      key_id: drill-acme-edge-2026
  routes:
    - match:
        path_prefix: /api/v1/billing/
      route:
        upstream: billing-api.drill-syd-1.oyatie.local:8443
        retry:
          max_attempts: 3
          per_try_timeout_ms: 1500
        circuit_breaker:
          max_connections: 1024
          max_pending_requests: 256
          consecutive_5xx: 5
          interval_seconds: 30
    - match:
        path_prefix: /api/v1/billing/streams
      route:
        upstream: billing-streams.drill-syd-1.oyatie.local:8443
        grpc: true
        stream_idle_timeout_seconds: 300
  rate_limit:
    tenant_class: paid
    per_tenant:
      sustained_rps: 50000
      burst_rps: 500000
      burst_window_seconds: 60
    per_route:
      "/api/v1/billing/usage/export":
        sustained_rps: 100      # this is a heavy endpoint; cap per-route
        burst_rps: 500
  waf:
    engine: modsecurity
    ruleset: owasp-crs-4.3
    rule_groups:
      - bot
      - sqli
      - xss
      - rfi
      - rce
      - lfi
    exceptions: []
    cedar_overlay:
      policy_set: pack-us-financial
  cors:
    allowed_origins:
      - https://app.acme.example
      - https://admin.acme.example
    max_age_seconds: 3600
  observability:
    access_log: structured-json
    sample_rate_pct: 100
    trace:
      sampler: tail-based
      sample_pct_baseline: 1
      sample_pct_error: 100
```

## Step 2 — Validate the config (≤ 5 min)

```sh
oya api-gateway listener validate \
    --config ./configs/acme-edge-v1.yaml
```

The validator runs:

1. Schema validation (YAML against the listener JSON schema).
2. Upstream resolution (DNS + port reachability).
3. TLS issuer reachability (ACME directory or private CA).
4. HSM key existence + permissions (the key must exist + the gateway service account must have `kms::sign` on it).
5. Cedar policy-set existence.

Expected output:

```
[OK] schema valid
[OK] upstream billing-api.drill-syd-1.oyatie.local:8443 reachable
[OK] upstream billing-streams.drill-syd-1.oyatie.local:8443 reachable
[OK] acme issuer reachable; account registered for ops@acme.example
[OK] hsm key drill-acme-edge-2026 exists in cluster hsm-cluster-syd-1
[OK] cedar policy-set pack-us-financial loaded; 18 actions registered
[OK] waf ruleset owasp-crs-4.3 loaded; 943 rules
```

## Step 3 — Issue the cert (≤ 5 min)

```sh
oya api-gateway listener cert issue \
    --listener tenant-edge-acme-v1 \
    --wait
```

Watch:

```
[acme] order placed; challenge type=DNS-01
[acme] DNS challenge token published to _acme-challenge.edge.acme.oyatie.io
[acme] CA verified challenge; cert issued
[hsm] cert + private key stored; key handle 0x7f3a9b2c
```

## Step 4 — Initial canary deploy (5 %) (≤ 15 min)

```sh
oya api-gateway listener canary-deploy \
    --tenant drill-acme \
    --listener tenant-edge-acme-v1 \
    --config ./configs/acme-edge-v1.yaml \
    --cohort-pct 5 \
    --duration 10m \
    --watch-slos edge-latency,waf-block-rate,rate-limit-drop
```

The canary-cohort-shifter (per IP-015) routes 5 % of `edge.acme.oyatie.io` traffic via the new listener; the other 95 % continues on the old (or, for a fresh tenant, returns 503 to the 95 % cohort — which is fine because we have no real traffic at provision time).

Watch the panels at 10 min:

- `api-gateway-edge-latency` for `tenant=drill-acme,listener=tenant-edge-acme-v1`: p99 should be ≤ 35 ms.
- `api-gateway-tls-handshake` for the new listener: p99 ≤ 30 ms (post first handshake).
- `api-gateway-waf-block-rate`: should be 0 (we're freshly deployed; no attacks).
- `api-gateway-rate-limit-drop`: should be 0 (we're far below envelope).

## Step 5 — Ramp to 15 %, 50 %, 100 % (≤ 45 min)

```sh
oya api-gateway listener canary-step \
    --listener tenant-edge-acme-v1 \
    --cohort-pct 15 \
    --duration 10m
```

If SLOs remain green:

```sh
oya api-gateway listener canary-step \
    --listener tenant-edge-acme-v1 \
    --cohort-pct 50 \
    --duration 15m
```

If SLOs remain green:

```sh
oya api-gateway listener canary-step \
    --listener tenant-edge-acme-v1 \
    --cohort-pct 100 \
    --duration 10m
```

At each step, the shifter checks:

- p99 edge latency within 10 % of baseline.
- WAF block-rate within ±5 % of baseline.
- Upstream error-rate within 0.5 % of baseline.

If any check fails, the shifter halts + alerts. You can manually rollback:

```sh
oya api-gateway listener canary-abort --listener tenant-edge-acme-v1
```

## Step 6 — Verify under synthetic load (≤ 10 min)

```sh
oya api-gateway drill load \
    --listener tenant-edge-acme-v1 \
    --target-rps 25000 \
    --duration 5m \
    --shape sustained
```

Expected:

- Edge p99 stable at ≤ 35 ms.
- WAF block-rate 0.
- Rate-limit drops 0 (we're well below 50 k sustained envelope).

Now test the burst envelope:

```sh
oya api-gateway drill load \
    --listener tenant-edge-acme-v1 \
    --target-rps 400000 \
    --duration 30s \
    --shape sudden-burst
```

Expected: burst rides for 60 s then rate-limit kicks in; drop-rate ≈ (400 k - 500 k) / 400 k = -25 % (no drops; we're below burst envelope).

## Step 7 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant drill-acme --since 90m --service api-gateway
```

Expected events:

- `listener_validated` × 1
- `cert_issued` × 1
- `listener_canary_step` × 4 (5 %, 15 %, 50 %, 100 %)
- `listener_promoted_full` × 1
- `load_drill_completed` × 2

## What you've learned

- The listener config schema (TLS + HTTP/3 + WAF + rate-limit + observability).
- The validate → issue-cert → canary-deploy → ramp flow.
- The canary-cohort-shifter step gates.
- The load drill + envelope verification.
- The audit-chain shape for api-gateway operations.

Next tutorial: `tutorials/configure-cross-region-failover.md` — provision the same listener across 3 regions with anycast + Cedar-evaluated cross-region routing.
