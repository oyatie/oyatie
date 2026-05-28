---
doc_id: finops-portal/sdk-reference
authored: 2026-05-18
status: ready
authority: ADR-0199 + OpenAPI 3.1 contract
classification: external (tenant-facing)
---

# SDK reference — finops-portal

This document is the **tenant-facing** SDK reference for accessing
`finops-portal` from a tenant's own systems. The contract is the
OpenAPI spec at `contracts/tenant-invoice-public.openapi.yaml`;
this doc translates that contract into idiomatic examples.

## Authentication

All endpoints require a Bearer JWT issued by the oyatie auth
service. The JWT carries:

- `sub` — principal id.
- `tenant_id` — the tenant the principal is scoped to.
- `tenant_scope` — one of `tenant`, `fleet`, `customer-success-managed`.
- `residency_region` — region label (e.g. `kr-1`, `eu-central-1`).
- `regulatory_pack` — pack label (e.g. `generic`, `eu`, `kr`).

Cedar policies (see `policy/cedar/`) authorize the request based
on these claims.

## Code samples — Rust

```rust
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct TenantInvoice {
    tenant_id: String,
    period: String,
    total_amount_cents: i64,
    presentation_currency: String,
}

async fn fetch_invoice(
    client: &Client,
    bearer_token: &str,
    tenant_id: &str,
    period: &str,
) -> anyhow::Result<TenantInvoice> {
    let url = format!(
        "https://api.oyatie.com/v1/tenants/{tenant_id}/invoices/{period}"
    );
    let resp = client
        .get(&url)
        .bearer_auth(bearer_token)
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.json().await?)
}
```

## Code samples — Python

```python
import httpx

def fetch_invoice(token: str, tenant_id: str, period: str) -> dict:
    url = f"https://api.oyatie.com/v1/tenants/{tenant_id}/invoices/{period}"
    with httpx.Client(headers={"Authorization": f"Bearer {token}"}) as c:
        r = c.get(url)
        r.raise_for_status()
        return r.json()
```

## Code samples — TypeScript

```ts
async function fetchInvoice(
  token: string, tenantId: string, period: string,
): Promise<TenantInvoice> {
  const r = await fetch(
    `https://api.oyatie.com/v1/tenants/${tenantId}/invoices/${period}`,
    { headers: { Authorization: `Bearer ${token}` } },
  );
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return r.json();
}
```

## Streaming FOCUS export

```rust
async fn trigger_focus_export(
    client: &Client,
    bearer: &str,
    tenant_id: &str,
    period: &str,
) -> anyhow::Result<()> {
    let url = format!(
        "https://api.oyatie.com/v1/tenants/{tenant_id}/focus-export"
    );
    let resp = client
        .post(&url)
        .bearer_auth(bearer)
        .json(&serde_json::json!({ "period": period, "format": "parquet" }))
        .send()
        .await?
        .error_for_status()?;
    let receipt: serde_json::Value = resp.json().await?;
    let download_url = receipt["download_url"].as_str().unwrap();
    // 1-hour TTL; download immediately.
    let bytes = client.get(download_url).send().await?.bytes().await?;
    std::fs::write("invoice.parquet", &bytes)?;
    Ok(())
}
```

## Embedded dashboard

```ts
// Request a signed embed URL.
const r = await fetch(
  `/v1/tenants/${tenantId}/dashboards/tenant-cost-drilldown/embed-url`,
  { headers: { Authorization: `Bearer ${token}` } },
);
const { url, expires_at } = await r.json();

// Embed in a sandboxed iframe.
const iframe = document.createElement("iframe");
iframe.src = url;
iframe.sandbox = "allow-scripts allow-same-origin";
iframe.style.width = "100%";
iframe.style.height = "800px";
document.getElementById("dashboard-host").appendChild(iframe);
```

## Error handling

The API returns structured errors with shape:

```json
{
  "kind": "TenantBoundaryViolation",
  "message": "principal cannot read tenant T's invoice",
  "correlation_id": "01HXY..."
}
```

`kind` enumeration:

- `Unauthorized` — JWT missing or invalid (401).
- `TenantBoundaryViolation` — Cedar deny (403).
- `InvoiceNotFound` (404).
- `AlreadyFinalized` — idempotent get works (409).
- `Upstream` — downstream upstream failed (503).
- `Internal` — unexpected (500); rate-limited in body.

## Rate limits

| Endpoint                              | Per-tenant per-minute | Per-tenant per-hour |
|---------------------------------------|-----------------------|---------------------|
| `GET /invoices`                       | 60                    | 1000                |
| `GET /invoices/{period}.pdf`          | 6                     | 100                 |
| `POST /focus-export`                  | 1                     | 12                  |
| `GET /dashboards/.../embed-url`       | 20                    | 500                 |
| `POST /credit-ledger/entries`         | 6                     | 60                  |

429 returns `Retry-After` header.

## Idempotency

POST endpoints accept an `Idempotency-Key` header (UUID). The
server stores the result for 24 h; subsequent requests with the
same key return the prior result.

## Locale + currency

Query params:

- `?locale=ko-KR` (defaults to en-US).
- `?currency=krw` (defaults to usd; conversion via daily ECB rate).

## Versioning

The API is versioned in the path (`/v1`). Breaking changes go in
a new prefix (`/v2`) with a parallel deprecation window per
docs/standards/api-versioning.md.

## References

- `contracts/tenant-invoice-public.openapi.yaml`.
- `contracts/focus-export-internal.asyncapi.yaml`.
- ADR-0199 FinOps canonical.
