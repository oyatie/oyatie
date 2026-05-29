---
doc_class: SdkPlan
template_id: TPL-SDK-PLAN
microservice: sites
status: Accepted
date: 2026-05-17
owner_team: axis-sites
related_adrs: [ADR-0131, ADR-0133, ADR-SITES-0001, ADR-SITES-0002, ADR-SITES-0004]
doc_status: published
---

# SDK Plan — sites µservice

## Purpose

Plan SDK surfaces by language + release cadence + scope. Aligned with
ADR-0133 (industry-best-practice citation) and the per-µservice
launch sequence.

## SDK targets

| Language | Status | Surface | Scope |
|---|---|---|---|
| Rust | M03 GA (primary SDK) | full surface | typed clients for all 11 BCs |
| TypeScript (browser + Node) | M03 GA | full surface | typed wrappers via OpenAPI codegen; Loro CRDT browser binding |
| Python | M04 | partial — admin + CMS-collection + publish | typed clients via OpenAPI codegen |
| Go | M04 | partial — admin + publish | typed clients via OpenAPI codegen |
| Java | M05 | partial — admin | enterprise tenants |
| .NET | M05 | partial — admin | enterprise tenants |
| Swift / Kotlin | subsequent-to-M05-completion | mobile-admin-only | mobile editor companion |

## Per-language surface

### Rust SDK (M03 GA)

- `oya-sites-{bc}-sdk` per the layer-mapping table (3 BCs ship SDKs at M03: `site`, `page`, `cdn-delivery`).
- Future BCs ship SDKs as the µservice matures.
- Typed errors per kernel port-trait.
- Async via `tokio`.
- OpenTelemetry tracing on every call.
- Audit-chain trace correlation.

### TypeScript SDK (M03 GA)

- `@oyatie/sites-sdk` npm package.
- Generated from OpenAPI 3.1 contract (`contracts/openapi/sites.yaml`).
- Loro CRDT browser binding (`loro-crdt` npm).
- Browser editor SDK companion: `@oyatie/sites-editor-sdk` (depends on `@oyatie/blocks-renderer`).
- Type-safe via TypeScript 5.x.

### Python SDK (M04)

- `oya-sites-sdk` PyPI package.
- Generated from OpenAPI 3.1.
- Async via `asyncio`.
- Type-safe via `mypy`.

### Go SDK (M04)

- `github.com/oyatie/sites-sdk-go` Go module.
- Generated from OpenAPI 3.1 + Proto contracts.

## Release cadence

| Cadence | Action | Owner |
|---|---|---|
| Per-release | regenerate SDKs on contract change | axis-sites + foundry-providers |
| Per-LTS | pin SDK major version; sunset N-2 majors | axis-sites |
| Per-incident | SDK CVE patch within 7 days | ops-security + axis-sites |
| Annual | SDK release-train review (drop unused languages; add requested) | council-product |

## Versioning

- SDK semver tracks the µservice's contract version.
- Major bump on contract breaking change; minor bump on additive
  change; patch on internal-only / docs.
- Sunset policy: N-2 major versions supported (e.g., 1.x + 2.x
  supported when 3.x ships; 1.x EOL on 3.x GA + 12mo).

## Authentication

All SDKs support:
- OIDC bearer token.
- Per-tenant API key.
- SPIFFE-bound mTLS (internal cross-µservice only).
- Per-tenant DEK envelope for AI-page-build private inference.

## Examples (per language)

### Rust

```rust
use oya_sites_site_sdk::{SitesClient, CreateSiteRequest};

let client = SitesClient::new("https://sites.kr.oyatie.com/v1")
    .with_oidc_token(token)
    .with_tenant_api_key(api_key);

let site = client.sites()
    .create(CreateSiteRequest { name: "Acme Public Site".into(), visibility: SiteVisibility::Public })
    .await?;
```

### TypeScript

```ts
import { SitesClient } from '@oyatie/sites-sdk';

const client = new SitesClient({
  baseUrl: 'https://sites.kr.oyatie.com/v1',
  oidcToken: token,
  tenantApiKey: apiKey,
});

const site = await client.sites.create({ name: 'Acme Public Site', visibility: 'public' });
```

### Python (M04)

```python
from oya_sites_sdk import SitesClient

client = SitesClient(
    base_url='https://sites.kr.oyatie.com/v1',
    oidc_token=token,
    tenant_api_key=api_key,
)

site = await client.sites.create(name='Acme Public Site', visibility='public')
```

## SDK acceptance criteria

| Criterion | Verification |
|---|---|
| Rust SDK passes contract tests | `cargo nextest run -p oya-sites-site-sdk` |
| TS SDK passes contract tests | `npm test --workspace @oyatie/sites-sdk` |
| OpenAPI codegen reproducible | `oya gate validate sdk-contract-conformance --microservice sites` |
| Loro CRDT browser binding interop | `npm run test:loro-interop --workspace @oyatie/sites-editor-sdk` |
| SDK matrix coverage | each shipped BC has at least Rust + TS SDK |
| Sunset policy honoured | LEAN `oya-check-sdk-sunset-schedule` |

## References

- ADR-0131 (per-microservice layout).
- ADR-0133 (industry-best-practice + SDK conformance).
- ADR-SITES-0001 (Loro CRDT).
- ADR-SITES-0002 (rendering).
- ADR-SITES-0004 (ACME + custom domain).
- OpenAPI 3.1 specification.
- Loro CRDT documentation.
- TypeScript 5.x type-system reference.
