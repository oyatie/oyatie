---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-005-block-types-kernel-domain
status: pending
execution_unit: ChangeSet
owner: axis-docs + council-design-system
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-wcag-22-aa-conformance]
---

# IP-005: block-types kernel + domain (block schema + sanitisation per ADR-DOCS-0002)

## Intent

Implement the block-type system per ADR-DOCS-0002 (block-based per Notion). Defines block schema (paragraph, heading_1-3, ordered_list, unordered_list, checklist, table, image, embed, code, math, callout, divider, page_break) + InlineStyle + RenderedBlock. Sanitisation per `ammonia` for HTML; macros refused; XXE prevented.

## ChangeSet boundary

7 crates per layer mapping: kernel + domain + usecase + api + adapter + sdk + app.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-block-types-kernel/src/{lib,block,schema,inline_style,acl}.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-domain/src/{lib,sanitiser,heading_hierarchy,alt_text_validator}.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-usecase/src/{lib,validate_block_tree,apply_inline_style}.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-api/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-adapter/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-sdk/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-block-types-app/src/main.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-block-types-domain -- heading_hierarchy
cargo nextest run -p oya-docs-block-types-domain -- alt_text_required
cargo nextest run -p oya-docs-block-types-domain -- ammonia_sanitiser_fuzz
cargo run -p oya-dev-cli -- gate validate wcag-22-aa-conformance --microservice docs
```

## References

- ADR-DOCS-0002 (block-type system).
- WCAG 2.2 AA — `w3.org/TR/WCAG22/`.
- ammonia HTML sanitiser — `crates.io/crates/ammonia`.
