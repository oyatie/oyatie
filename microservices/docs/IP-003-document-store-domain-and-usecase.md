---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-003-document-store-domain-and-usecase
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: document-store domain + usecase

## Intent

Implement pure document-invariant math (block-tree ordering validation, ACL coverage check, hold coverage check) + usecase orchestrators (create-document, update-metadata, archive, apply-legal-hold, expire-retention, restore-from-version).

## ChangeSet boundary

2 crates: `oya-docs-document-store-domain` (pure logic; reads via ports) + `oya-docs-document-store-usecase` (orchestration).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-document-store-domain/Cargo.toml` | create |
| `microservices/docs/src/crates/oya-docs-document-store-domain/src/{lib,block_tree_validator,acl_coverage,hold_coverage,context_isolation}.rs` | create |
| `microservices/docs/src/crates/oya-docs-document-store-usecase/Cargo.toml` | create |
| `microservices/docs/src/crates/oya-docs-document-store-usecase/src/{lib,create_document,update_metadata,archive,apply_legal_hold,expire_retention,restore_from_version}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-document-store-domain -- context_isolation
cargo nextest run -p oya-docs-document-store-domain -- per_block_acl
cargo nextest run -p oya-docs-document-store-domain -- legal_hold
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice docs
```

## References

- ADR-0105, ADR-0106.
- PRD AC-01, AC-04, AC-07, AC-13.
