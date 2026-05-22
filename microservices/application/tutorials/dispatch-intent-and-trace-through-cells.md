# Tutorial — Dispatch an intent and trace it through cells

Goal: a concrete walk-through that gets you from "I have a Rust binary, a CLI, or a `curl`" to "I see a successful dispatch event in
`audit-chain` and a matching span tree in `observability`, end-to-end across 2 cells".

Pre-reqs:
- Loopback dev cell up (`make dev-cell.up CELL=loopback-1`) and a second loopback (`make dev-cell.up CELL=loopback-2`).
- Tenant `oyatie.community.dev-sample` provisioned (`make dev-tenant.create T=oyatie.community.dev-sample tenant_class=demo_trial`).
- `oya` CLI on PATH (`./bin/oya --version` returns ≥ `0.42.0`).

## Step 1 — verify the cell topology

```bash
./bin/oya cell list --pool dev
```
Expected:
```
loopback-1    region=loopback   tier=tenant_class demo_trial   status=Healthy
loopback-2    region=loopback   tier=tenant_class demo_trial   status=Healthy
```
If either is `Degraded`, run `./bin/oya cell heal --cell <name>` and re-list.

## Step 2 — confirm the tenant is admitted in both cells

```bash
./bin/oya tenant show oyatie.community.dev-sample
```
Look for:
```
admitted_cells: [loopback-1, loopback-2]
tier: tenant_class demo_trial
active_packs: []
provider_credential_mode: platform_default
```

## Step 3 — fire a dispatch via curl

```bash
TRACE_ID=$(uuidgen | tr -d '-' | head -c 32)
curl --http3 -k \
  -H "x-oyatie-tenant: oyatie.community.dev-sample" \
  -H "traceparent: 00-${TRACE_ID}-$(uuidgen | tr -d '-' | head -c 16)-01" \
  -H "content-type: application/json" \
  -d '{
        "kind": "application::Intent::CreateWorkspace",
        "payload": {"name": "tutorial-ws-1", "owner_email": "alice@example.com"}
      }' \
  https://loopback.application.oyatie.local/v1/dispatch
```

Expected HTTP 202 with body:
```json
{ "outcome": "Accepted",
  "intent_hash": "blake3-256:9f3c…",
  "dispatch_target": "workflow-engine",
  "cell_id": "loopback-1" }
```

If you see `403`, your tenant lacks the Cedar permit. Inspect:
```bash
./bin/oya policy explain --principal oyatie.community.dev-sample \
  --action 'application::Action::Dispatch' \
  --resource 'application::Intent::CreateWorkspace'
```

## Step 4 — fire a second dispatch and confirm it landed in cell 2

Because cells route by HRW hash on `(tenant_id, intent_kind)`, **the same intent kind from the same tenant should be sticky**.
To force the second cell, change the intent variant:
```bash
curl --http3 -k \
  -H "x-oyatie-tenant: oyatie.community.dev-sample" \
  -H "traceparent: 00-${TRACE_ID}-$(uuidgen | tr -d '-' | head -c 16)-01" \
  -H "content-type: application/json" \
  -d '{ "kind": "application::Intent::ListWorkspaces", "payload": {} }' \
  https://loopback.application.oyatie.local/v1/dispatch | jq .cell_id
```

Expect `"loopback-2"`. If both intents land in the same cell, the HRW seed is stale; reseed with `./bin/oya cell rehash --pool dev`.

## Step 5 — view the trace tree

```bash
./bin/oya obs query --trace $TRACE_ID --window 5m --service application
```

You should see spans:
```
[api-gateway]            12 ms  → status=200
  ↳ [application:dispatch] 9 ms → outcome=Accepted target=workflow-engine cell=loopback-1
    ↳ [workflow-engine:exec] 4 ms → workflow_id=…
```

## Step 6 — confirm the audit-chain entry

```bash
./bin/oya audit query --tenant oyatie.community.dev-sample --intent-hash "blake3-256:9f3c…"
```
Expected JSON:
```json
{ "ts": "2026-05-20T08:14:22.913Z",
  "tenant_id": "oyatie.community.dev-sample",
  "intent_kind": "application::Intent::CreateWorkspace",
  "intent_hash": "blake3-256:9f3c…",
  "cell_id": "loopback-1",
  "outcome": "Accepted",
  "permit_decision": "Allow" }
```

## Step 7 — induce a failure and verify the audit trail still records it

```bash
curl --http3 -k \
  -H "x-oyatie-tenant: oyatie.community.dev-sample" \
  -H "content-type: application/json" \
  -d '{ "kind": "application::Intent::CreateWorkspace",
        "payload": {"name": "", "owner_email": "not-an-email"} }' \
  https://loopback.application.oyatie.local/v1/dispatch
```

Expected `400 Bad Request` and an audit-chain entry with `outcome: Rejected.ValidationFailed` and `validation_errors`.

## Step 8 — clean up

```bash
make dev-cell.down CELL=loopback-1
make dev-cell.down CELL=loopback-2
make dev-tenant.delete T=oyatie.community.dev-sample
```

## What you just proved

- Multi-cell HRW routing is working in the loopback environment.
- Cedar permits gate every dispatch, even in dev.
- Trace context propagates from edge through `application` into `workflow-engine`.
- `audit-chain` records both successful and rejected dispatches with full intent hash + tenant + cell.
