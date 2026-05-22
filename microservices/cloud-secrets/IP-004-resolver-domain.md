---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-004-resolver-domain
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [cargo-test, lean-a1]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: oya-cloud-secrets-secret-reference-resolver-domain

## Intent

Pure SecretReference URI parsing + cache-TTL clamp arithmetic + revocation invalidation logic. Zero I/O. Depends only on `-kernel`.

## ChangeSet boundary

One new crate; consumes test corpus from `microservices/cloud-secrets/contracts/secret-reference-uri-test-corpus.jsonl`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-secrets/src/crates/oya-cloud-secrets-secret-reference-resolver-domain/Cargo.toml` | create |
| `…/src/lib.rs` | create |
| `…/src/uri_parser.rs` | create — `parse(s: &str) -> Result<SecretReference, ParseError>` |
| `…/src/ttl_clamp.rs` | create — `clamp_ttl(d: Duration) -> Duration` (≤60s) |
| `…/src/invalidation.rs` | create — pure `should_invalidate(entry, event)` |
| `…/src/tests.rs` | create |
| `microservices/cloud-secrets/catalog/oya-cloud-secrets-secret-reference-resolver-domain.yaml` | create |

## Code Shape

```rust
const MAX_CACHE_TTL: Duration = Duration::from_secs(60);

pub fn clamp_ttl(d: Duration) -> Duration {
    std::cmp::min(d, MAX_CACHE_TTL)
}

pub fn parse(input: &str) -> Result<SecretReference, ParseError> {
    // ABNF per IP-002
    // openbao:secret/<tenant|shared>/<microservice>/<name>[@v<version>]
    // ...
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-domain
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-cloud-secrets-secret-reference-resolver-domain
```

## Test Plan

- Property tests: parser refuses any malformed input (proptest).
- Corpus tests: every entry in `secret-reference-uri-test-corpus.jsonl` parses or fails as expected.
- TTL clamp: arbitrary input clamps to ≤60s.

## Halt Conditions

- Parser accepts any URI not in ABNF — BLOCKER (security boundary).

## Next IP

`IP-005-resolver-usecase.md`

## References

- IP-002 + IP-003

## Wave 15-IP-substance A-G

### A. Problem
The resolver domain is the security boundary between a syntactically valid SecretReference and a safe runtime lookup plan. Without bespoke domain rules, cache TTLs, version pins, and revocation metadata could drift across SDKs and weaken the PRD's no-raw-secret invariant.

### B. Approach
Keep this crate pure: parse and normalize references, derive cache TTL ceilings, classify revocation behavior, and reject any URI outside the ABNF from IP-002. All OpenBao I/O remains in adapters; domain code only emits deterministic decisions that the usecase layer can audit.

### C. Deliverables
- `oya-cloud-secrets-secret-reference-resolver-domain` crate from `manifest.json`.
- `src/parser.rs`, `src/ttl.rs`, and `src/revocation.rs` targets already named in this IP.
- Shared fixtures from `specs/secret-reference-uri-test-corpus.jsonl`.
- Policy alignment with `policy/secret-isolation.md` and `policy/tenant-scope.cedar`.
- Contract alignment with `contracts/openapi/cloud-secrets.yaml`.

### D. Ordered Implementation Steps
1. Implement parser entry points over the IP-002 grammar.
2. Add normalization rules for pack/tenant path segments and optional versions.
3. Clamp requested cache TTLs to the PRD maximum of 60 seconds.
4. Reject path traversal, raw literal values, and unrecognized query keys.
5. Add property tests for malformed strings and TTL arithmetic.
6. Expose only typed domain errors for usecase/audit mapping.
7. Validate dependency direction with LEAN-A1 and layer correctness gates.

### E. Acceptance
- `cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-domain`.
- `cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-cloud-secrets-secret-reference-resolver-domain`.
- Parser accepts only corpus-approved references and clamps all TTLs to <=60s.
- Domain crate has no OpenBao, HTTP, Kubernetes, or audit-chain dependency.

### F. Evidence
Evidence anchors are `PRD.md` SecretReference functional requirements, `ARCHITECTURE.md` port-trait table, `manifest.json` crate registry, `catalog/oya-cloud-secrets-secret-reference-resolver-domain.yaml`, `policy/secret-isolation.md`, and `slos/secret-resolve-latency.openslo.yaml`.

### G. Counterpart Comparison
HashiCorp Vault, AWS Secrets Manager, and Google Secret Manager expose flexible naming and versioning, but the counterpart matrices mark SDK-enforced safety and TTL ceilings as Oyatie differentiators. This domain crate turns that differentiator into deterministic code instead of leaving it as SDK guidance.

Grep-recognized counterpart anchor: GitHub Actions Secrets is cited for CI-time secret reference validation, where workflow-distributed values must be converted into safe handles before domain parsing. It is not the primary runtime comparator for this domain crate.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`, `microservices/cloud-secrets/IP-004-resolver-domain.md`.
