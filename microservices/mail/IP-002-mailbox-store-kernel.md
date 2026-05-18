---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-002-mailbox-store-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, data-class, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: oya-mail-mailbox-store-kernel

## Intent

Scaffold the `kernel` layer crate for `mailbox-store` BC per ADR-0105: port traits (sealed) + entity types + value objects + error types. Zero I/O. Zero business logic. Foundation for all mailbox-store layers + cross-BC consumers.

## ChangeSet boundary

One new Rust crate at `microservices/mail/src/crates/oya-mail-mailbox-store-kernel/`. Workspace member added. Catalog row created. No downstream consumers in this IP.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/mail/src/crates/oya-mail-mailbox-store-kernel/Cargo.toml` | create | `[package]` + `async-trait` + `serde` + `chrono` |
| `microservices/mail/src/crates/oya-mail-mailbox-store-kernel/src/lib.rs` | create | module declarations |
| `microservices/mail/src/crates/oya-mail-mailbox-store-kernel/src/entities.rs` | create | `Mailbox`, `Thread`, `MailMessage`, `Folder`, `RetentionClass`, `MimeBlob`, `ContextKind`, `OwnershipPillar` |
| `microservices/mail/src/crates/oya-mail-mailbox-store-kernel/src/ports.rs` | create | `MailboxRepository`, `ThreadRepository`, `MimeBlobStore`, `RetentionLedgerWriter`, `ContextBoundaryGuard` traits |
| `microservices/mail/src/crates/oya-mail-mailbox-store-kernel/src/errors.rs` | create | error variants per port + entity |
| `Cargo.toml` (workspace) | update | add `microservices/mail/src/crates/oya-mail-mailbox-store-kernel` to `[workspace.members]` |
| `microservices/mail/catalog/oya-mail-mailbox-store-kernel.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-mail-mailbox-store-kernel
JUSTIFICATION:
- microservice = mail
- bc-tokens = mailbox-store (primary BC per PRD)
- layer = kernel (ADR-0105 13-value enum)
- exemptions claimed: none
```

## Code Shape

```rust
// src/lib.rs
pub mod entities;
pub mod errors;
pub mod ports;

pub use entities::*;
pub use errors::*;
pub use ports::*;

#[doc(hidden)]
mod sealed { pub trait Sealed {} }
```

```rust
// src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextKind { Personal, Professional }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OwnershipPillar { Person, Org }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mailbox {
    #[data_class(INTERNAL_ONLY)]
    pub mailbox_id: MailboxId,
    #[data_class(SENSITIVE_PIPA_ART23)]
    pub tenant_id: Option<TenantId>,         // None for Personal
    #[data_class(INTERNAL_ONLY)]
    pub context_kind: ContextKind,
    #[data_class(PII_IDENTIFYING)]
    pub owner_ref: UserId,
    #[data_class(PII_IDENTIFYING)]
    pub aliases: Vec<EmailAddress>,
    #[data_class(INTERNAL_ONLY)]
    pub quota_policy: QuotaPolicy,
    #[data_class(INTERNAL_ONLY)]
    pub region: PackRegion,
    #[data_class(INTERNAL_ONLY)]
    pub retention_policy_id: RetentionPolicyId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MailMessage {
    #[data_class(INTERNAL_ONLY)]
    pub message_id: MessageId,
    #[data_class(INTERNAL_ONLY)]
    pub mailbox_id: MailboxId,
    #[data_class(SENSITIVE_PIPA_ART23)]
    pub tenant_id: Option<TenantId>,
    #[data_class(INTERNAL_ONLY)]
    pub context_kind: ContextKind,
    #[data_class(PII_IDENTIFYING)]
    pub headers_ciphertext: Ciphertext,
    #[data_class(PII_IDENTIFYING)]
    pub body_ciphertext: Ciphertext,
    #[data_class(INTERNAL_ONLY)]
    pub retention_policy_id: RetentionPolicyId,
    #[data_class(AUDIT)]
    pub legal_hold_ids: Vec<LegalHoldId>,
    #[data_class(INTERNAL_ONLY)]
    pub data_class_annotation: DataClass,
    #[data_class(AUDIT)]
    pub received_at: DateTime<Utc>,
}
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait MailboxRepository: Send + Sync + Sealed {
    async fn read(&self, id: MailboxId) -> Result<Mailbox, RepositoryError>;
    async fn list_by_user(&self, user_id: UserId, ctx: ContextKind) -> Result<Vec<Mailbox>, RepositoryError>;
    async fn create(&self, mb: Mailbox) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait MimeBlobStore: Send + Sync + Sealed {
    async fn put(&self, blob: MimeBlob, tenant: Option<TenantId>) -> Result<BlobRef, BlobError>;
    async fn get(&self, r: BlobRef, tenant: Option<TenantId>) -> Result<MimeBlob, BlobError>;
    async fn delete(&self, r: BlobRef, tenant: Option<TenantId>) -> Result<(), BlobError>;
}

#[async_trait]
pub trait ContextBoundaryGuard: Send + Sync + Sealed {
    fn assert(&self, principal_ctx: ContextKind, resource_ctx: ContextKind) -> Result<(), ContextBoundaryError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-mail-mailbox-store-kernel --all-features
cargo build -p oya-mail-mailbox-store-kernel --all-features
cargo clippy -p oya-mail-mailbox-store-kernel --all-features -- -D warnings
cargo nextest run -p oya-mail-mailbox-store-kernel --all-features
cargo deny check
cargo doc -p oya-mail-mailbox-store-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-mail-mailbox-store-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-mail-mailbox-store-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-mail-mailbox-store-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-mail-mailbox-store-kernel
```

## Test Plan

Per PHASE-01 kernel class: 1 test per public type + 1 per port + 1 sealed-trait smoke. Coverage 90%/80%.

## Halt Conditions

- BNF violation — refer feedback_naming_justification.md.
- Port introduces business logic — refactor to domain/usecase.
- Any I/O reachable — refactor.

## Next IP

[`IP-003-mailbox-store-postgres-adapter.md`](IP-003-mailbox-store-postgres-adapter.md)

## References

- ADR-0056 BNF v4.1; ADR-0105 13-layer enum; ADR-0131 per-microservice flat layout
- PRD §"Bounded Contexts" port-trait table
- Bominal ADR-0028 (data-class taxonomy)
