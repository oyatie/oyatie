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
