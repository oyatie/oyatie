---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-002-document-store-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-port-location, oya-governance-data-class-coverage]
---

# IP-002: document-store kernel — Document + BlockTree + RetentionPolicyRef + LegalHoldRef + port traits

## Intent

Author the document-store BC's kernel layer per ADR-0105 13-layer enum. Defines canonical types and port traits with zero I/O and zero business logic. Annotates every field with `#[data_class(...)]` per the LEAN data-class lane.

## ChangeSet boundary

1 crate (`oya-docs-document-store-kernel`); ~15 type definitions + 8 port traits + Cedar entity-shape declarations.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/docs/src/crates/oya-docs-document-store-kernel/Cargo.toml` | create | crate manifest; deps: chrono, serde, ed25519-dalek, uuid |
| `microservices/docs/src/crates/oya-docs-document-store-kernel/src/lib.rs` | create | crate root + re-exports |
| `microservices/docs/src/crates/oya-docs-document-store-kernel/src/entity.rs` | create | `Document`, `BlockTree`, `DocumentContext{Personal,Professional}`, `RetentionPolicyRef`, `LegalHoldRef` |
| `microservices/docs/src/crates/oya-docs-document-store-kernel/src/ports.rs` | create | `DocumentRepository`, `BlockBlobStore`, `AclRepository`, `LegalHoldStore`, `RetentionPolicyResolver`, `AttachmentScanner`, `AttachmentStorage`, `DocumentContextBoundaryGuard` |
| `microservices/docs/src/crates/oya-docs-document-store-kernel/src/error.rs` | create | `DocumentStoreError` variant enum (preserve order per Hyrum surfaces in `migration-from-connect.md`) |
| `microservices/docs/src/crates/oya-docs-document-store-kernel/src/data_class.rs` | create | `#[data_class]` macro re-export |

## Code Shape

```rust
// src/entity.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub document_id: DocumentId,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub tenant_id: TenantId,
    pub context: DocumentContext,
    #[data_class(PERSONAL_DOC_CONTENT, PROFESSIONAL_DOC_CONTENT)]
    pub title: String,
    pub block_tree_ref: BlockTreeRef,
    pub retention_policy_ref: RetentionPolicyRef,
    pub legal_hold_ref: Option<LegalHoldRef>,
    pub version: i64,
    pub version_sha: VersionSha,
}

// src/ports.rs
pub trait DocumentRepository: Send + Sync {
    fn create(&self, doc: Document) -> Result<DocumentId, DocumentStoreError>;
    fn read(&self, id: DocumentId) -> Result<Document, DocumentStoreError>;
    fn update_metadata(&self, id: DocumentId, patch: DocumentMetadataPatch) -> Result<Document, DocumentStoreError>;
    fn archive(&self, id: DocumentId) -> Result<(), DocumentStoreError>;
    fn list(&self, ctx: DocumentContext, cursor: Option<Cursor>) -> Result<DocumentPage, DocumentStoreError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-docs-document-store-kernel
cargo clippy -p oya-docs-document-store-kernel -- -D warnings
cargo nextest run -p oya-docs-document-store-kernel
cargo run -p oya-dev-cli -- gate validate port-location --microservice docs
cargo run -p oya-dev-cli -- gate validate data-class-coverage --microservice docs
```

## Test Plan

- Property tests on `DocumentContext` discriminated-union exhaustiveness.
- Trait object compile-checks: every port trait is `dyn`-safe.
- Data-class annotation coverage.

## Next IP

[`IP-003-document-store-domain-and-usecase.md`](IP-003-document-store-domain-and-usecase.md)

## References

- ADR-0105 (13-layer enum); ADR-0106 (usecase rename); ADR-0131.
- ADR-DOCS-0004 (per-block ACL; entity shape).
- PRD-docs §"Bounded Contexts" + §"Port traits declared in each kernel".
