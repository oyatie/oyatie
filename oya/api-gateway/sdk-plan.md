# api-gateway — SDK plan

**Authority:** ADR-0258 (versioning) + ADR-0246+amendment (library-first).

The api-gateway management plane (route config, rate-limit overrides, canary weighting, blue/green swap) is consumed via SDK by other µservices, operators, and CI lanes.

## A — Languages

| Language | SDK package | Use case |
|---|---|---|
| Rust | `oya-api-gateway-sdk` | Internal Foundry; CI lanes; operators |
| TypeScript | `@oyatie/api-gateway-sdk` | Tenant operator dashboards; CLI |
| Go | `github.com/oyatie/api-gateway-sdk-go` | Vendor integrations |
| Python | `oyatie-api-gateway-sdk` | Data-science / SRE notebooks |
| Java | `dev.oyatie:api-gateway-sdk` | Enterprise tenant integrations |

## B — Surfaces

- **Management plane (OpenAPI 3.2.0):** route CRUD, rate-limit-override CRUD, canary-weight CRUD, blue/green-swap.
- **AsyncAPI 3.1.0 events:** route-admitted, route-denied, canary-shifted, blue-green-swapped, tls-cert-rotated, ech-config-rotated, pqc-handshake-completed.
- **gRPC (proto3):** high-throughput management for the policy-engine µservice push channel.

## C — Auth model

- **Tenant operator:** OIDC token via identity µservice; passed as Bearer token.
- **Internal µservice:** SPIFFE SVID via mTLS.
- **CI lane:** SPIFFE SVID via mTLS + Cedar `ci-scope.cedar` permit.

## D — SDK versioning

- SemVer 2.0 per ADR-0258.
- Major bump on contract break (12mo notice + 6mo sunset).
- Minor bump on new surface.
- Patch on bugfix.

## E — Compatibility matrix

| SDK version | API version | h3 support | PQC support | ECH support |
|---|---|---|---|---|
| 0.1.0 | v1 | Yes | Yes | Yes |

## F — Code-quality bar (per documentation-rigor.md §1.2)

- Rust SDK: `deny(warnings)`; ≥85% line coverage; property tests for retry/backoff.
- TS SDK: `strict: true`; ≥85% coverage.
- Go SDK: `-race -vet` clean; ≥85% coverage.
- Python SDK: `mypy --strict`; ≥85% coverage.
- Java SDK: `--enable-preview` off; `-Werror`; ≥85% coverage.

## G — References

- `microservices/api-gateway/contracts/`
- ADR-0258, ADR-0246+amendment
