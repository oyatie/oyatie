---
doc_class: ImplementationPlan
impl_plan_id: IP-004-tag-graph-kernel-domain
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location, oya-governance-layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: tag-graph kernel + domain

## Intent

Land `oya-notes-tag-graph-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk,app}`. Tags + tag-graph adjacency + tag rename + tag merge.

Personal-tier tags are client-side per ADR-NOTES-0001; this BC covers Professional-tier only.

## Port Traits

```rust
pub trait TagRepository {
    fn create(&self, tag: Tag) -> Result<TagId, TagError>;
    fn rename(&self, tag_id: TagId, new_name: String) -> Result<(), TagError>;
    fn merge(&self, source: TagId, target: TagId) -> Result<MergeReport, TagError>;
    fn delete(&self, tag_id: TagId) -> Result<(), TagError>;
    fn list(&self, tenant: TenantId, scope: TagListScope) -> Result<Vec<Tag>, TagError>;
    fn adjacency(&self, tenant: TenantId, tag_id: TagId) -> Result<Vec<TagEdge>, TagError>;
}
```

## Test Plan

- Tag rename idempotency.
- Tag merge: source tag deleted; note_tag re-pointed; tag_edge rebuilt.
- Per-tenant cardinality cap enforced.

## Acceptance Gates

```bash
cargo check -p oya-notes-tag-graph-kernel
cargo check -p oya-notes-tag-graph-domain
cargo run -p oya-dev-cli -- gate validate port-location --microservice notes
```

## Next IP

[`IP-005-backlink-graph-kernel-domain.md`](IP-005-backlink-graph-kernel-domain.md)
