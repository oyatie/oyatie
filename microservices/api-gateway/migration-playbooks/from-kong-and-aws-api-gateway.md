---
doc_class: MigrationPlaybook
microservice: api-gateway
vendor: Kong Gateway + AWS API Gateway (parallel migration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Kong Gateway / AWS API Gateway → oyatie api-gateway

Audience: an oyatie tenant migrating their edge ingress from Kong Gateway (OSS or Konnect) or AWS API Gateway (REST or HTTP) to oyatie's `api-gateway` µservice. Apigee, Tyk, and Envoy/Istio sidecar are covered as variants below.

## Why this migration is non-trivial

- **Kong** is plugin-based; tenant policies live in Kong-specific plugins (rate-limiting, ACL, JWT, OAuth2-introspection). The plugin chain order matters.
- **AWS API Gateway** is region-bound + integration-pattern-bound; REST APIs ↔ Lambda + DynamoDB + Cognito differ from HTTP APIs ↔ ALB + Lambda. Mapping templates (Velocity) require manual port.
- **oyatie** is xDS-driven (Envoy-based); the config model is YAML; per-tenant Cedar policies replace Kong plugin chain + AWS WAF rules.

The 80/20: route definitions + upstream targets port cleanly via auto-converter; the 20 % needing manual review is in plugin chains (Kong) + integration patterns (AWS).

## Step 1 — Inventory the source (≤ 1-3 days per 100 services)

For Kong:

```sh
oya api-gateway migrate inventory \
    --source kong \
    --kong-admin-url https://kong-admin.acme.example \
    --kong-admin-token "$KONG_ADMIN_TOKEN" \
    --window 2020-01-01..2026-05-20 \
    --out inventory/kong-services.yaml
```

For each Kong Service: routes, plugins (with config), consumers, ACL groups, JWT configurations, upstream targets, healthcheck config.

For AWS API Gateway:

```sh
oya api-gateway migrate inventory \
    --source aws-api-gateway \
    --aws-region us-east-1 \
    --aws-credentials-profile prod \
    --out inventory/aws-apis.yaml
```

For each API: stages, resources, methods, integration types (Lambda / HTTP / VPC link / Mock), mapping templates, usage plans, API keys, request validators, model schemas.

## Step 2 — Audit policy portability (≤ 1 week)

```sh
oya api-gateway migrate policy-portability-audit \
    --inventory inventory/kong-services.yaml \
    --source-platform kong \
    --out audit/kong-policy-portability.yaml
```

The audit classifies each Kong plugin:

| Kong plugin | Portability | oyatie destination |
|---|---|---|
| `rate-limiting` | Auto-port | `rate_limit:` field |
| `rate-limiting-advanced` | Auto-port | `rate_limit:` field + per-route override |
| `request-transformer` | Manual port | Cedar policy + Envoy lua filter (per IP-013 Wasm) |
| `response-transformer` | Manual port | Envoy lua/wasm filter |
| `ip-restriction` | Auto-port | Cedar policy `ip_allow_deny` |
| `bot-detection` | Manual port (replace) | Use oyatie's CRS bot-rules + WASM bot-score (IP-013) |
| `cors` | Auto-port | `cors:` field |
| `acl` | Auto-port | Cedar policy `consumer_acl` |
| `key-auth` | Auto-port | tenant JWT issuance via tenancy µservice |
| `jwt` | Auto-port | tenant JWT via tenancy µservice |
| `oauth2` | Auto-port | tenancy µservice OAuth2 substrate |
| `oauth2-introspection` | Auto-port | identity µservice introspection endpoint |
| `mtls-auth` | Auto-port (config differs) | listener-level mTLS in oyatie |
| `request-size-limiting` | Auto-port | `route.max_body_bytes` |
| `request-validator` | Manual port | OpenAPI contract validator at gateway level |
| `prometheus` | Auto-port | always-on observability |
| `zipkin` / `opentelemetry` | Auto-port | always-on tracing |
| `correlation-id` | Auto-port | `X-Oya-Trace-Id` propagation |
| `acme` | Auto-port | listener `tls.cert_source.type: acme` |
| custom Lua plugins | Manual port | Re-author as Cedar policy + Envoy Wasm |

For AWS:

| AWS feature | Portability | oyatie destination |
|---|---|---|
| Lambda integration | Manual port | Author oyatie µservice OR keep Lambda + use HTTP-integration |
| Velocity mapping templates | Manual port (heavy) | Author Envoy lua/wasm filter |
| Cognito User Pool authorizer | Manual port | Migrate users to oyatie identity µservice OR JWT-bridge |
| API Key + Usage Plan | Auto-port | tenant JWT + rate_limit fields |
| Request validator (model schema) | Auto-port | OpenAPI contract validator |
| CORS configuration | Auto-port | `cors:` field |
| Custom domain + base-path mapping | Auto-port | listener `bind` + route `path_prefix` |
| WAF (AWS WAF v2) | Manual port | ModSecurity CRS + Cedar policy overlay |
| AWS X-Ray tracing | Auto-port | always-on OTel tracing |

## Step 3 — Convert + stage configs (≤ 2-4 weeks)

For Kong:

```sh
oya api-gateway migrate convert-kong \
    --inventory inventory/kong-services.yaml \
    --output-dir ./migration-staging/kong/ \
    --target-tenant drill-acme \
    --tenant-class paid
```

For each Kong Service → oyatie listener + routes:

- Service.url → upstream target.
- Routes (path, host, method) → route match rules.
- Plugins → field mappings + Cedar policies + lua filters (per the audit).
- Healthcheck → upstream healthcheck config.

For AWS:

```sh
oya api-gateway migrate convert-aws \
    --inventory inventory/aws-apis.yaml \
    --output-dir ./migration-staging/aws/ \
    --target-tenant drill-acme
```

The converter generates one yaml per AWS Stage. Mapping templates emit a manual-port TODO file per resource.

## Step 4 — Re-author custom plugins / mapping templates (≤ 2-6 weeks)

For each custom Lua plugin or Velocity mapping template flagged manual-port:

```sh
oya api-gateway migrate scaffold-wasm-filter \
    --source-lang lua \
    --source-file ./kong-plugins/my-auth.lua \
    --output ./wasm-filters/my-auth/
```

This scaffolds a Rust WASM filter (per IP-013) with the equivalent logic skeleton. Engineer time per filter: 0.5-3 days depending on complexity.

For Velocity templates, the converter emits a YAML mapping doc + a TODO checklist; complex templates need 1-2 days each.

## Step 5 — Parallel deploy + cutover (≤ 4-12 weeks)

Per the canary-cohort-shifter (IP-015):

```sh
oya api-gateway listener canary-deploy \
    --tenant drill-acme \
    --listener tenant-edge-acme-v1 \
    --config ./migration-staging/kong/converted-edge.yaml \
    --cohort-pct 5 \
    --duration 1h
```

Run 5 % cohort for 1 hour; observe SLOs; ramp 15 % → 50 % → 100 % per the tutorial. Keep Kong / AWS API Gateway live for the rollback window.

## Step 6 — Validate behaviour parity (≤ 1-2 weeks)

```sh
oya api-gateway migrate parity-check \
    --tenant drill-acme \
    --baseline-source kong \
    --target-listener tenant-edge-acme-v1 \
    --duration 24h \
    --output evidence/migration-parity-drill-acme.json
```

The parity check:

1. Replays Kong-recorded request traffic (the previous 24 h) against both Kong + oyatie.
2. Compares response status, body shape, header set, latency.
3. Reports any divergence.

Expected: > 99.5 % parity. Below 99.5 % = a plugin or mapping template wasn't ported correctly.

## Step 7 — Decommission source (≤ 1 month)

```sh
oya api-gateway migrate decommission \
    --tenant drill-acme \
    --source kong \
    --evidence-out evidence/migrations/kong-to-oyatie-drill-acme.json
```

Or for AWS:

```sh
oya api-gateway migrate decommission \
    --tenant drill-acme \
    --source aws-api-gateway \
    --evidence-out evidence/migrations/aws-api-gateway-to-oyatie-drill-acme.json
```

For AWS: separately, cancel the per-month API Gateway charges + Cognito charges (if migrated). Plan to keep AWS resources for 30 d as rollback buffer.

## Variant — Apigee X

Apigee is policy-XML based. The converter parses ProxyEndpoint + TargetEndpoint XML + flows + per-flow policies (KeyValueMap, ServiceCallout, JavaScript, etc.). Apigee mediations require manual port; budget 2-5 days per non-trivial proxy.

## Variant — Tyk

Tyk uses per-API JSON definitions. Auto-port handles routes, rate-limit (per-second / per-min), JWT, basic-auth, key-auth, mTLS. Tyk middleware-chain (Python/Go) requires manual port to WASM filters.

## Variant — Envoy/Istio sidecar

If the tenant already runs Envoy as a sidecar with Istio, the listener config maps almost 1:1. The chief difference: oyatie's xDS control-plane vs Istio's xDS control-plane. Migration is config-translation; no behaviour gaps expected. Budget 1-2 weeks.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Kong custom Lua plugins do not port | High | Audit per Step 2; budget engineer time per custom plugin |
| AWS Velocity mapping templates do not port | High | Same; budget engineer time per non-trivial template |
| AWS Cognito users do not migrate cleanly | High | Tenant Outcome 2 of identity µservice; budget 4-8 wk for user-base migration |
| OAuth2 introspection endpoint URL changes | Medium | Tenant updates client config; 1 wk lead time + dual-issue period |
| WAF false-positives on first cutover | Medium | Run 5 % cohort for 4 h before ramping; budget on-call engineer |
| Tenant uses Kong service-mesh features (ingress + mesh) | Medium | Document mesh as out-of-scope; migrate ingress first; mesh later |
| Tenant uses AWS API Gateway custom authorizer Lambda | Medium | Port to oyatie Cedar policy OR keep Lambda + use HTTP-integration |
| Per-request pricing model lock-in | Low | The migration ITSELF is the un-lock |
