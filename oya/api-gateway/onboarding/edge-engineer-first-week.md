---
doc_class: Onboarding
microservice: api-gateway
persona: edge-engineer + sre
related_adrs: [ADR-0253, ADR-0329, ADR-0330, ADR-0331, ADR-0131, ADR-0248]
date: 2026-05-20
doc_status: published
---

# Edge Engineer onboarding — first 5 working days

Audience: a new edge engineer or SRE joining the `api-gateway` rotation. By Day-5 they will have: deployed a canary listener config, walked an mTLS rotation drill, triaged a WAF false-positive, exercised the rate-limit envelope under load, and shadowed an XDS rollback.

## Day 1 — Tour the substrate

1. Read `PRD.md` § Tenant Outcomes 1-3 (∼ 45 min) + `decisions/ADR-0253-http3-quic-default-protocol.md` + `decisions/ADR-0329-tier-system-retired-replaced-by-tenant-class.md` + `decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md` + `decisions/ADR-0331-cross-microservice-tenant-class-adoption-template.md` (∼ 60 min). Optional `ARCHITECTURE.md` for the listener-chain figure.
2. Open the Grafana folder `api-gateway`. Identify boards: `api-gateway-edge-latency`, `api-gateway-tls-handshake`, `api-gateway-rate-limit`, `api-gateway-waf-block-rate`, `api-gateway-xds-push-lag`, `api-gateway-h3-fallback-rate`.
3. Walk the runbook index. On-call runbooks: `tls-cert-expiring.md`, `waf-false-positive.md`, `rate-limit-burst-storm.md`, `xds-push-failure.md`, `h3-fallback-spike.md`, `honeypot-hit.md`, `mtls-handshake-stall.md`.
4. Sit in on Tuesday's edge handoff.

Acceptance: you can sketch the request path: client → anycast → Envoy edge → JWT validate → Cedar admit → WAF inspect → rate-limit decrement → upstream µservice.

## Day 2 — Deploy a canary listener config

```sh
oya api-gateway listener canary-deploy \
    --tenant drill-acme \
    --listener tenant-edge-acme-v2 \
    --config ./configs/acme-edge-v2.yaml \
    --cohort-pct 5 \
    --shifter linear-15m
```

The `canary-cohort-shifter` (per IP-015) ramps traffic from 5 % to 100 % over 15 minutes, watching the SLO panels at each step. If any panel goes RED, the shifter halts.

Watch the per-step Grafana panel `api-gateway-canary-step-health`:

- Step 1 (5 %): edge p99 ≤ 35 ms; WAF block-rate within ±10 % of baseline; rate-limit drops < 0.5 %.
- Step 2 (15 %): same gates.
- Step 3 (50 %): same gates + Cedar admission p99 ≤ 5 ms.
- Step 4 (100 %): all of above.

Acceptance: canary completes; you can read the per-step health from Grafana.

## Day 3 — mTLS rotation drill

Read `runbooks/tls-cert-expiring.md` + skim `IP-014-tls-cert-rotation-worker.md`.

Force a rotation drill:

```sh
oya api-gateway drill tls-rotation \
    --tenant drill-acme \
    --cert-source acme \
    --age-target now \
    --mode chaos
```

The drill marks the current cert as "about to expire in 30 minutes" and watches the rotation worker pick up + renew + reload.

Expected sequence (visible in `oya api-gateway events tail --tenant drill-acme`):

1. `cert_expiring_warning` (t=0).
2. `cert_renewal_started` (t=30 s; the worker noticed).
3. `cert_acquired` (t=2 min; ACME challenge completed).
4. `cert_staged` (t=2.5 min; new cert on disk).
5. `listener_hot_reload` (t=3 min; Envoy SIGHUP + zero-downtime reload).
6. `cert_rotation_completed` (t=3.5 min; verified handshake on new cert).

No connection drops (verify via `tls-handshake-success` panel).

Acceptance: rotation completed in ≤ 5 min; zero handshake failures during reload.

## Day 4 — WAF false-positive triage

A tenant reports: "all our requests to `/api/billing/usage/export` are getting 403 from the WAF."

```sh
oya api-gateway waf incidents --tenant drill-acme --since 1h
```

Identify the rule that fired:

```sh
oya api-gateway waf inspect-request \
    --tenant drill-acme \
    --request-id req-7f3a9b2c
```

The inspector shows: "OWASP CRS rule 942100 (SQL Injection: SQL Injection Attack Detected via libinjection) fired on body field `query`. Body sample: `SELECT id, amount FROM invoices WHERE tenant='acme' AND created_at > '2026-01-01'`. Score: 5/5."

This is a legitimate read-API request. The fix path:

1. Verify with the tenant that the body shape is expected.
2. Author a per-tenant rule exception (Cedar-evaluated):

```sh
oya api-gateway waf exception add \
    --tenant drill-acme \
    --rule-id 942100 \
    --path '/api/billing/usage/export' \
    --method POST \
    --justification "billing-export-uses-SQL-DSL-in-body" \
    --signoff edge-on-call
```

3. The exception ships via XDS push; takes ~ 30 s to propagate.
4. Verify the request now succeeds.

Acceptance: incident triaged + exception authored + tenant unblocked in ≤ 30 min.

## Day 5 — Rate-limit envelope under load + XDS rollback shadow

Provision a load drill:

```sh
oya api-gateway drill load \
    --tenant drill-acme \
    --target-rps 75000 \
    --ramp-window 5m \
    --shape sustained-with-bursts \
    --duration 30m
```

Watch the rate-limit dashboard (`api-gateway-rate-limit`):

- Sustained 75 k RPS: rate-limit drops should be ~ 0 if tenant is on paid tenant_class policy (50 k sustained envelope + burst); drops will start above the burst envelope.
- Burst: 500 k RPS for 60 s; expect drops ≈ (burst_rps - 500 k) / burst_rps.

The token-bucket implementation (per IP-010-rate-limit-adapter-valkey):

- Per-tenant bucket sized to tenant_class policy (50 k for paid tenant_class policy).
- Refill rate = sustained envelope.
- Bucket size = burst envelope × 60 s.
- Per-route sub-buckets for fine-grained quotas.

Now shadow an XDS rollback. A previous Envoy config push had a subtle bug (wrong upstream for `/api/v2/billing`).

```sh
oya api-gateway xds rollback \
    --tenant drill-acme \
    --listener tenant-edge-acme \
    --to-version v2026-05-20-1430 \
    --justification "v2026-05-20-1500 broke /api/v2/billing routing"
```

The rollback pushes the previous version via XDS; Envoy hot-reloads; rollback completes in ~ 60 s.

Watch the panels:

- `api-gateway-xds-push-lag` shows the push spike.
- `api-gateway-upstream-error-rate` for `/api/v2/billing` drops from the error spike back to baseline.

Acceptance: load drill completed; rate-limit envelope confirmed by tenant_class policy; XDS rollback walked.

## What you've learned

- The edge listener chain + Envoy XDS substrate.
- The mTLS rotation flow (ACME + HSM + hot-reload).
- The WAF inspection + exception flow.
- The rate-limit token-bucket envelope.
- The canary-cohort-shifter + XDS rollback.

Next week: BGP/anycast routing shadow, HTTP/3 fallback-rate root-cause analysis, abuse-defence WASM module review.
