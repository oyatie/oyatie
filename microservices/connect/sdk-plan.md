---
microservice: connect
doc_class: SdkPlan
date: 2026-05-20
owner_team: axis-integration + dx
status: Accepted
related_adrs: [ADR-0258]
doc_status: published
---

# SDK Plan — connect

## Client SDK languages (priority order)

1. **Rust** — internal substrate consumers (workflow-engine, foundry, ops-dashboard). Library-first dispatch per ADR-0246. **GA at M01.**
2. **TypeScript / Node.js** — workflow-studio (browser + server). **GA at M01.**
3. **Python** — Foundry job authors; intelligence µservice. **GA at M01.**
4. **Go** — third-party integrations + CLI tooling. **GA at M02.**
5. **Java / Kotlin** — enterprise tenants. **GA at M02.**
6. **C# / .NET** — enterprise tenants. **GA at M03.**
7. **Ruby** — community + legacy. **Best-effort.**

## SDK shape (per language)

```typescript
// TypeScript SDK example
import { ConnectClient } from '@oyatie/connect-sdk';

const client = new ConnectClient({
  tenantId: 'tnt_01H...',
  oauth: { mode: 'byok', clientId: '...', clientSecret: 'openbao://...' },
});

// List catalog
const catalog = await client.catalog.search({ q: 'salesforce' });

// Initiate OAuth grant
const grantUrl = await client.oauth.initiate({ connector: 'salesforce', scopes: ['read_contacts'] });

// Invoke action
const result = await client.actions.invoke({
  connector: 'slack',
  action: 'chat.postMessage',
  args: { channel: '#general', text: 'hi' },
});
```

## Versioning per ADR-0258

- SemVer; minor versions backward-compatible.
- Deprecation cadence: 90-day deprecation notice + 180-day sunset.
- SDK version pinning via package.json/Cargo.toml; SDK auto-upgrade prompts for security patches.

## Documentation

- API reference auto-generated from OpenAPI 3.2.0 contract.
- Cookbook: `docs/cookbook/connect-<language>/` with ≥20 worked examples per SDK.
- Migration guides between major versions.
