---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-004-resolver-domain
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [buck2-test, lean-a1]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: oya-cloud-secrets-secret-reference-resolver-domain

## Intent

Pure SecretReference URI parsing + cache-TTL clamp arithmetic in the current `oya-secrets-domain` crate. Zero I/O. Revocation invalidation remains a follow-up usecase/adapter concern unless a later IP adds crate-backed domain rules.

## ChangeSet boundary

Current implementation lives in the existing `cloud/cloud-secrets/crates/oya-secrets-domain` crate and direct test target. Planned shared corpus artifacts must mirror this crate-backed behavior before becoming release authority.

## Concrete File Targets

| Path | Action |
|---|---|
| `cloud/cloud-secrets/crates/oya-secrets-domain/src/lib.rs` | update/verify current `SecretReferenceUri`, config-wrapper parser, normalized serializer, and `clamp_secret_reference_cache_ttl_seconds` |
| `cloud/cloud-secrets/crates/oya-secrets-domain/tests/secret_reference_uri.rs` | update/verify parser and TTL tests |
| `cloud/cloud-secrets/crates/oya-secrets-domain/BUCK` | update/verify Buck2 test/build targets |
| `cloud/cloud-secrets/catalog/oya-cloud-secrets-secret-reference-resolver-domain.yaml` | planned/verify catalog compatibility if a split resolver crate is reintroduced |

## Code Shape

```rust
pub const MAX_SECRET_REFERENCE_CACHE_TTL_SECONDS: u64 = 60;

pub fn clamp_secret_reference_cache_ttl_seconds(requested_seconds: u64) -> u64 {
    requested_seconds.min(MAX_SECRET_REFERENCE_CACHE_TTL_SECONDS)
}

pub fn parse(input: &str) -> Result<SecretReferenceUri, SecretReferenceUriError> {
    // ABNF per IP-002
    // openbao:secret/<path>[@v<positive-integer>]
    // segment characters: ASCII alphanumeric plus '_', '-', '.', ':'; '..' is refused.
    // `${openbao:secret/<path>[@vN]}` config wrappers parse through parse_config_reference.
    // ...
}
```

## Acceptance Gates

```bash
buck2 test //cloud/cloud-secrets/crates/oya-secrets-domain:secret-reference-uri-test
buck2 build //cloud/cloud-secrets/crates/oya-secrets-domain:oya-secrets-domain[check]
```

## Test Plan

- Direct tests: parser accepts canonical OpenBao references and `${...}` config wrappers.
- Rejection tests: parser refuses non-contract prefixes, empty segments, traversal, query strings, invalid/zero versions, and secret-shaped literals.
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
Keep this crate pure: parse and normalize references, derive cache TTL ceilings, and reject any URI outside the ABNF from IP-002. All OpenBao I/O remains in adapters; domain code only emits deterministic decisions that the usecase layer can audit. Revocation behavior remains future-facing unless backed by a later domain implementation.

### C. Deliverables
- `oya-secrets-domain` crate implementation from `cloud/cloud-secrets/crates/oya-secrets-domain/src/lib.rs`.
- Consolidated `src/lib.rs` parser/TTL surface rather than separate parser, TTL, and invalidation modules.
- Direct Buck2 test target `//cloud/cloud-secrets/crates/oya-secrets-domain:secret-reference-uri-test` until shared fixture artifacts are generated.
- Policy alignment with `policy/secret-isolation.md` and `policy/tenant-scope.cedar`.
- Contract alignment with `contracts/openapi/cloud-secrets.yaml`.

### D. Ordered Implementation Steps
1. Maintain parser entry points over the IP-002 grammar: `openbao:secret/<path>[@vN]` and `${...}` wrappers.
2. Normalize safe generic path segments and optional positive numeric versions without implying a fixed tenant/shared hierarchy.
3. Clamp requested cache TTLs to the PRD maximum of 60 seconds.
4. Reject path traversal, raw literal values, query strings, malformed wrappers, and invalid versions.
5. Keep malformed-string and TTL arithmetic tests in the Buck2 parser test target.
6. Expose only typed domain errors for usecase/audit mapping.
7. Validate dependency direction with Buck2 check targets and branch-protected `oya-ci-required` / owned `oya-ci` gates.

### E. Acceptance
- `buck2 test //cloud/cloud-secrets/crates/oya-secrets-domain:secret-reference-uri-test`.
- `buck2 build //cloud/cloud-secrets/crates/oya-secrets-domain:oya-secrets-domain[check]`.
- Branch-protected `oya-ci-required` / owned `oya-ci` evidence for LEAN-A1 before external release claims.
- Parser accepts only corpus-approved references and clamps all TTLs to <=60s.
- Domain crate has no OpenBao, HTTP, Kubernetes, or audit-chain dependency.

### F. Evidence
Evidence anchors are `PRD.md` SecretReference functional requirements, `ARCHITECTURE.md` port-trait table, `manifest.json` crate registry, `cloud/cloud-secrets/crates/oya-secrets-domain/src/lib.rs`, `cloud/cloud-secrets/crates/oya-secrets-domain/tests/secret_reference_uri.rs`, `policy/secret-isolation.md`, and `slos/secret-resolve-latency.openslo.yaml`.

### G. Counterpart Comparison
HashiCorp Vault, AWS Secrets Manager, and Google Secret Manager expose flexible naming and versioning, but the counterpart matrices mark SDK-enforced safety and TTL ceilings as Oyatie differentiators. This domain crate turns that differentiator into deterministic code instead of leaving it as SDK guidance.

Grep-recognized counterpart anchor: GitHub Actions Secrets is cited for CI-time secret reference validation, where workflow-distributed values must be converted into safe handles before domain parsing. It is not the primary runtime comparator for this domain crate.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `cloud/cloud-secrets/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `cloud/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `cloud/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `cloud/cloud-secrets/contracts/proto/cloud-secrets.proto`, `cloud/cloud-secrets/IP-004-resolver-domain.md`.
