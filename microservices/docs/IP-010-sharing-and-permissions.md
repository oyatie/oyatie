---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-010-sharing-and-permissions
status: pending
execution_unit: ChangeSet
owner: axis-docs + ops-security
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-per-block-acl, oya-governance-acl-enforcement-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: sharing-and-permissions BC (8 crates; per-block ACL per ADR-DOCS-0004)

## Intent

Implement per-doc + per-block ACL + share-link issuance (Ed25519-signed tokens) + share-grant lifecycle per ADR-DOCS-0004.

## ChangeSet boundary

8 crates per layer mapping.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-sharing-and-permissions-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,app}/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-sharing-and-permissions-domain/src/{acl_eval,share_link_token,grant_lifecycle}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-sharing-and-permissions-domain -- per_block_acl  # AC-04
cargo nextest run -p oya-docs-sharing-and-permissions-domain -- share_link_constant_time_verify
cargo run -p oya-dev-cli -- gate validate per-block-acl --microservice docs
cargo run -p oya-dev-cli -- gate validate acl-enforcement-correctness --microservice docs
```

## References

- ADR-DOCS-0004 (per-block ACL).
- `policy/tenant-scope.cedar` (per-block ACL Cedar rules).
