# consent-graph SDK plan

- Owner: axis-consent-graph + dx-axis
- Date: 2026-05-18
- Authority: ADR-0214 §F-AGR-* + §F-PRJ-* (typed SDK requirement).

## 1. SDK matrix

| Language | PHASE-01 (this PR) | PHASE-02 | PHASE-03 |
|----------|--------------------|----------|----------|
| Rust | ✓ full | — | — |
| TypeScript | — | ✓ full | — |
| Python | — | ✓ full | — |
| Go | — | — | ✓ |
| Java | — | — | ✓ |

PHASE-01 ships Rust only; TS/Python are partner-facing and slip to PHASE-02 after API contract is
exercised.

## 2. Rust SDK structure (PHASE-01)

### 2.1 Crates
- `oya-consent-graph-agreement-sdk` — agreement CRUD.
- `oya-consent-graph-enforcement-sdk` — enforcement evaluation hot-path.
- `oya-consent-graph-projection-gateway-sdk` — internal projection mint/destroy.
- `oya-consent-graph-revocation-sdk` — revocation originate/confirm.
- `oya-consent-graph-audit-bridge-sdk` — bilateral emission.
- `oya-consent-graph-partner-directory-sdk` — partner handshake.

### 2.2 Public surfaces

```rust
// agreement-sdk
let client = AgreementClient::builder()
    .mtls_identity(spiffe::WorkloadIdentity::detect()?)
    .endpoint("consent-graph.us-east-1.svc.cluster.local")
    .build()?;
let agreement_id = client.draft(DraftAgreementCommand {
    grantor: tenant_a, grantee: tenant_b,
    scope: scope, terms: terms, sovereignty: sovereignty,
    expiration: Some(Timestamp::now() + Duration::days(365)),
    template_id: Some(TemplateId::SupplyChainPoVisibility),
}).await?;

// enforcement-sdk (hot path)
let allowed = enforcement_client.check_project_read(
    grantor, grantee, resource_ref, principal_id, ctx
).await;
if !allowed { return Err(ConsentDenied); }
```

### 2.3 SDK error model
```rust
pub enum SdkError {
    Network(NetworkError),                   // mTLS, DNS, timeout
    Auth(AuthError),                         // JWT expired, mTLS rejected
    Validation(ValidationError),             // bad input
    EnforcementDenied(EnforcementDecision),  // semantic denial from policy
    ServerError(ServerError),                // 5xx
    ProtocolError(String),                   // wire decode failure
}
```

Critical pattern: `EnforcementDenied` is *not* an exception — it's a typed outcome callers handle.
Reserved for the hot-path enforcement-sdk where Deny is expected.

### 2.4 Retry + timeout policy
- Idempotent operations (read, list): 3× retry with exp-backoff 50ms/200ms/1s.
- State-changing (draft, offer, accept, revoke): 1 retry (idempotency via client-supplied
  `request_id`).
- Default timeout: 5s.
- Hot-path enforcement-sdk: 200ms timeout, no retry (fail-closed faster than retry-and-fail).

### 2.5 mTLS + JWT
- SPIFFE workload identity detection at client construction.
- JWT minted via identity µservice; auto-rotated at 80% TTL.
- mTLS cert from SPIFFE bundle.

## 3. TS SDK (PHASE-02)

Target: Node.js (server-side) primarily; browser support for B2C self-revoke flows.

```ts
const client = new AgreementClient({
  endpoint: 'https://consent-graph.us-east-1.oya.dev',
  authToken: process.env.OYA_TOKEN,
});
const id = await client.draft({
  grantor: 'tn-acme', grantee: 'tn-retail',
  scope: { entityType: 'FinishedGoodsInventory', fieldSet: { allow: ['sku', 'qty', 'eta'] } },
  terms: { purposeOfUse: 'inventory-visibility', mode: 'Projection' },
  sovereignty: { grantorRegion: 'us-east-1', permittedGranteeRegions: ['us-east-1'] },
});
```

Generated from OpenAPI 3.2.0 spec via `openapi-typescript-codegen`. Hand-written ergonomic wrappers
on top.

## 4. Python SDK (PHASE-02)

Target: Python 3.11+, async-first (httpx + asyncio).

```python
from oya_consent_graph import AgreementClient, DraftAgreementCommand, SharingMode
client = AgreementClient.from_env()
agreement_id = await client.draft(DraftAgreementCommand(
    grantor='tn-acme', grantee='tn-retail',
    scope=scope, terms=Terms(mode=SharingMode.Projection, ...),
))
```

Generated from OpenAPI via `openapi-python-client`. Typed with `pydantic` v2.

## 5. SDK documentation

Per-SDK quickstart in `docs/standards/sdk-quickstart-<lang>.md` (PHASE-02 deliverable).
PHASE-01 ships:
- Rust SDK rustdoc generated + hosted at `docs.oya.dev/rust/consent-graph/`.
- Quickstart in `microservices/consent-graph/sdk-plan.md` (this doc).

## 6. Versioning

- SDK version follows µservice version (`oya-consent-graph-agreement-sdk = 0.1.0` matches
  consent-graph 0.1.0).
- Wire-compatibility per ADR-0064; SDK supports current + N-1 wire schema.

## 7. SDK testing

- Unit tests in each SDK crate (mock server fixtures).
- Integration tests against local docker-compose substrate.
- Contract tests: SDK request bytes match OpenAPI spec exactly (`schemathesis` validates).

## 8. SDK observability

- SDK emits client-side OTEL spans with parent linkage from server-side.
- Per-call latency histogram exported to client's metrics endpoint.
- SDK-specific user-agent string `oya-consent-graph-rust-sdk/0.1.0`.

## 9. Migration discipline

When SDK signature changes:
- Add new method; deprecate old (don't remove for 6mo per ADR-0064).
- Generate migration guide in `docs/standards/sdk-migration-<from>-to-<to>-<lang>.md`.

## 10. Open questions

- Browser SDK security model (B2C self-revoke from web app) — PHASE-02 scope.
- gRPC-Web bridge for browser? Or REST-only at edge? — PHASE-02 decision.
- Java + Go demand prioritization — PHASE-03 contingent on adoption signal.
