# Analytics µservice — Client SDK Plan

**Authority:** ADR-0193, ADR-0157 API gateway, ADR-0150 cursor pagination, ADR-0151 X-Request-Id
**Owner:** council-analytics + axis-developer-experience
**Last reviewed:** 2026-05-18

## 1. Purpose

Define the client SDK strategy for tenant-facing access to the analytics µservice. SDKs are *generated* from the canonical OpenAPI / AsyncAPI / proto3 contracts in `microservices/analytics/contracts/`; no hand-rolled clients.

## 2. Languages

| Language | Surface | Generation tool | Versioning | Release cadence |
|---|---|---|---|---|
| TypeScript | REST + GraphQL + WebSocket | openapi-typescript + graphql-codegen + AsyncAPI react-component | semver from contract version | per-contract-version |
| Python | REST + gRPC | openapi-python-client + grpcio-tools | semver | per-contract-version |
| Go | REST + gRPC | oapi-codegen + protoc-gen-go-grpc | semver | per-contract-version |
| Rust | gRPC | tonic-build | semver | per-contract-version (internal-only) |
| Java | REST + gRPC | openapi-generator (kotlin) + grpc-java | semver | per-contract-version |

## 3. Contract authority

The canonical contract lives at:

- `microservices/analytics/contracts/openapi-v1.yaml` (REST 3.2.0)
- `microservices/analytics/contracts/asyncapi-v1.yaml` (AsyncAPI 3.1.0)
- `microservices/analytics/contracts/analytics.proto` (proto3)
- `microservices/analytics/contracts/graphql-v1.sdl` (GraphQL SDL)

Contract changes go through:
1. Author PR with contract change.
2. CI lane runs `oya-foundry-fitness-api-contract-stability` — denies breaking changes per ADR-0007 governance.
3. If breaking change is intentional: ADR amendment + major version bump.
4. SDK regeneration triggered by tag.

## 4. Authentication

- **OAuth 2.0 + PKCE** for human users — issued by the identity µservice; bearer token forwarded.
- **SPIFFE workload identity** for service-to-service — automatic via mesh sidecar (no SDK plumbing).
- **API keys** for tenant-bound automation — `Authorization: Bearer <key>`; key issued via tenant portal; scoped to a single capability.

## 5. Required headers (per ADR-0151)

| Header | Purpose | Set by |
|---|---|---|
| `X-Request-Id` | Distributed trace correlation | SDK auto-generates uuidv4 if absent |
| `X-Tenant-Id` | Tenant context (for tenant-scoped APIs) | SDK reads from auth token; over-ride forbidden |
| `Idempotency-Key` | For mutations (per ADR-0150) | SDK helper generates uuidv4 |
| `User-Agent` | `oyatie-sdk/<lang>/<sdk-version>` | SDK |

## 6. Idempotency contract (per ADR-0150)

For any POST/PUT/PATCH (currently only `POST /v1/regulator-export`):

```typescript
const idempotencyKey = crypto.randomUUID();
await client.regulatorExport.create(
  { tenant_id, from, to },
  { headers: { 'Idempotency-Key': idempotencyKey } }
);
// Re-sending the same key within 24h returns the same response (no duplicate work).
```

## 7. Pagination (per ADR-0150)

Cursor-based; opaque, HMAC-signed:

```typescript
let cursor: string | undefined;
do {
  const page = await client.auditLog.search({ axis: 'auth', from, to, cursor });
  for (const row of page.data) process(row);
  cursor = page.next_cursor;
} while (cursor);
```

The cursor encodes `(last_sort_key, signature)`; SDK never inspects it.

## 8. Retry policy

Default: exponential backoff with jitter; max 3 retries on 5xx and 429.

```typescript
// SDK default
{
  retries: 3,
  initialBackoffMs: 200,
  maxBackoffMs: 5000,
  retryableStatuses: [429, 500, 502, 503, 504],
  honorRetryAfterHeader: true,
}
```

On 429 + `Retry-After: <seconds>`, SDK sleeps the specified duration before retry.

## 9. Streaming

For `POST /v1/regulator-export` (potentially multi-GB), SDK exposes a streaming API:

```typescript
const stream = client.regulatorExport.streamNDJSON({ tenant_id, from, to });
for await (const row of stream) {
  process(row);
}
```

Backend uses chunked-transfer-encoding; SDK consumes line-by-line; backpressure via NodeJS Readable stream.

## 10. Error model

All errors are typed:

```typescript
class AnalyticsError extends Error {
  code: 'unauthorized' | 'forbidden' | 'not_found' | 'quota_exceeded' | 'invalid_argument' | 'internal' | 'unavailable';
  request_id: string;       // from X-Request-Id
  retry_after_ms?: number;  // present on quota_exceeded
}
```

GraphQL errors are surfaced via the `errors` array with `extensions.code` matching the same enum.

## 11. Observability

The SDK auto-emits OpenTelemetry spans for every request (per ADR-0151 invariant 2). The span has:
- `peer.service = "analytics"`
- `peer.name = "<endpoint>"`
- `oyatie.tenant_id = <from auth>`
- `oyatie.request_id = <X-Request-Id>`

OTLP exporter is configurable; default no-op if not configured.

## 12. Backward compatibility commitments

- **Patch version (1.0.x):** Bug fixes only; no observable behavior change.
- **Minor version (1.x.0):** Additive only; new endpoints, new optional fields, new error codes. Old clients work.
- **Major version (x.0.0):** Breaking change allowed. Requires ADR amendment. Old major version maintained for 12 months minimum.

## 13. SDK release pipeline

| Step | Trigger | Outcome |
|---|---|---|
| Contract change merged | Push to `dev` | Contracts published to artifact registry |
| SDK regeneration | Contract tag | PRs to per-language SDK repos |
| SDK CI | PR merge in SDK repo | Lint, test, type-check; publish dry-run |
| SDK release | Manual tag push | npm / pypi / crates.io / maven publish |
| Customer notification | SDK tag | Release notes to `docs/sdk-releases/` + customer email |

## 14. Per-language packaging

| Language | Package name | Registry |
|---|---|---|
| TypeScript | `@oyatie/analytics-sdk` | npm |
| Python | `oyatie-analytics` | PyPI |
| Go | `github.com/oyatie/analytics-sdk-go` | Go module |
| Rust | `oyatie-analytics-sdk` | crates.io (internal Cloudsmith for non-public) |
| Java | `co.oyatie:analytics-sdk` | Maven Central |

## 15. SDK testing surface

Each SDK ships:
- Contract conformance test — generates a request from the SDK, validates against the OpenAPI schema.
- Mock server — locally-runnable for tenant integration tests.
- Examples directory — one example per capability (dashboard query, audit log search, regulator export).

## 16. Out of scope (deferred)

- GraphQL subscriptions — analytics is read-mostly; subscription pattern not needed yet.
- Browser SDK CDN bundle — TypeScript SDK is npm-only initially.
- gRPC-Web — Java client uses gRPC directly; browser-side uses REST + GraphQL.

## 17. References

- ADR-0193, ADR-0157 API gateway, ADR-0150 cursor pagination, ADR-0151 X-Request-Id, ADR-0007 Cedar.
- OpenAPI 3.2.0 spec.
- AsyncAPI 3.1.0 spec.
- Stripe API SDK design (public reference for retry / idempotency patterns).
- GitHub Octokit (public reference for SDK release pipeline cadence).
