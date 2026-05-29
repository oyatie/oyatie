# IP-015 — TypeScript SDK

**microservice**: feature-flags
**bc**: flag
**layer**: adapter
**qualifier**: typescript-sdk
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0245, ADR-0248, ADR-0253, ADR-0258
**companion_ips**: IP-013, IP-014, IP-016
**references**: contracts/openfeature-sdk-contract.md; sdk-plan.md

## Scope

TypeScript/JavaScript SDK implementing the OpenFeature `Provider` interface. Browser + Node.js targets. HTTP/3 via `fetch` (browser native); EventSource SSE; LKG cache in `localStorage` (browser) or `fs` (Node). Used by Workflow Studio and all TS µservices.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `OyatieProvider` class | Implements `@openfeature/core` `Provider` interface; `resolveBooleanEvaluation`, `resolveStringEvaluation`, `resolveNumberEvaluation`, `resolveObjectEvaluation` |
| 2 | HTTP/3 transport | `fetch()` with `{ signal }` for abort; `Alt-Svc: h3` negotiation; TLS 1.3; fallback to HTTP/2 |
| 3 | `Map<string, CachedFlag>` cache | TTL 30s; LKG: `localStorage.setItem('oya-ff-lkg', JSON.stringify(cache))` (browser); JSON file (Node) |
| 4 | SSE invalidation | `EventSource('/api/v1/flags/stream?tenant_id=...')` per ADR-0253; reconnect with jittered backoff |
| 5 | `OyatieEvaluationContext` type | `tenantId`, `audienceType`, `sessionId`, `deviceFingerprintHash`, `packId` |
| 6 | Error handling | Returns `ResolutionDetails` with `errorCode` and `reason`; never throws on evaluation |
| 7 | Bundle size | ≤12 KB gzipped (ESM); tree-shakeable; no runtime dependencies beyond `@openfeature/core` |
| 8 | Tests | Jest unit tests; Playwright browser integration test; bundle size assertion in CI |

## Usage

```typescript
import { OpenFeature } from '@openfeature/web-sdk';
import { OyatieProvider } from '@oyatie/feature-flags-sdk';

await OpenFeature.setProviderAndWait(
  new OyatieProvider({
    endpoint: 'https://feature-flags.internal',
    tenantId: 'tenant_abc',
    audienceType: 'B2B',
  })
);

const client = OpenFeature.getClient();
const enabled = await client.getBooleanValue('my-flag', false);
```

## Definition of Done

- `npm test` green
- Bundle size ≤12 KB gzipped verified in CI
- SSE reconnect: simulated disconnect → reconnects within 3s
- OpenFeature conformance set passes (TS target via `@openfeature/test-harness`)
