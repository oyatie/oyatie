---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-notes-foundation
impl_plan_id: IP-003-note-store-kernel-domain
status: pending
execution_unit: ChangeSet
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location, oya-governance-layer-correctness, oya-governance-dual-context-isolation]
---

# IP-003: note-store kernel + domain (port traits + entities + DCI invariants)

## Intent

Land the foundational `oya-notes-note-store-kernel` + `-domain` crates. Define `NoteRepository`, `CedarNotePolicy`, `AuditChainClient` port traits. Implement domain entities: `Note`, `PersonalNoteRef`, `ProfessionalNoteRef`, `Notebook`, `RetentionPolicy`, `Hold`. Enforce DCI-01..09 invariants at the type system (sealed `ContextKind` enum; non-coercible `PersonalE2EKey` vs `TenantDek`; zero-byte `body_client_only` marker).

## Concrete File Targets

- `microservices/notes/src/oya-notes-note-store-kernel/src/{lib.rs,ports.rs,types.rs,context_kind.rs,e2e_marker.rs}` — all new.
- `microservices/notes/src/oya-notes-note-store-domain/src/{lib.rs,note.rs,notebook.rs,retention.rs,hold.rs}` — all new.

## Port Traits

```rust
pub trait NoteRepository {
    fn create_personal(&self, note: PersonalNote) -> Result<PersonalNoteRef, NoteRepoError>;
    fn create_professional(&self, note: ProfessionalNote) -> Result<ProfessionalNoteRef, NoteRepoError>;
    fn read(&self, scope: ReadScope) -> Result<Note, NoteRepoError>;
    fn edit(&self, patch: NotePatch) -> Result<Note, NoteRepoError>;
    fn delete(&self, scope: WriteScope) -> Result<(), NoteRepoError>;
}

pub trait CedarNotePolicy {
    fn evaluate(&self, request: CedarRequest) -> CedarVerdict;
}

pub trait AuditChainClient {
    fn seal(&self, event: AuditEvent) -> Result<SealReceipt, AuditError>;
}
```

`PersonalNote` and `ProfessionalNote` are distinct structs (DCI-01). No `From`/`Into` between them. `body_client_only: PhantomData<NeverServerPlaintext>` on Personal (Inv-E2E-02).

## Acceptance Gates

```bash
cargo check -p oya-notes-note-store-kernel
cargo check -p oya-notes-note-store-domain
cargo test  -p oya-notes-note-store-domain --lib
cargo run -p oya-dev-cli -- gate validate port-location --microservice notes
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice notes
cargo run -p oya-dev-cli -- gate validate dual-context-isolation --microservice notes
```

## Test Plan

- UI test: try to coerce `PersonalE2EKey` into `TenantDek` — MUST fail to compile.
- UI test: try to pass `PersonalNoteRef` where `ProfessionalNoteRef` expected — MUST fail to compile.
- Unit: `ContextKind` enum exhaustive match.
- Unit: `body_client_only` marker not extractable.

## Next IP

[`IP-004-tag-graph-kernel-domain.md`](IP-004-tag-graph-kernel-domain.md)
