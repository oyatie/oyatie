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
buck2 build //:quality-lane-registry-authority-check # lane=port-location --microservice notes
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice notes
buck2 build //:quality-lane-registry-authority-check # lane=dual-context-isolation --microservice notes
```

## Test Plan

- UI test: try to coerce `PersonalE2EKey` into `TenantDek` — MUST fail to compile.
- UI test: try to pass `PersonalNoteRef` where `ProfessionalNoteRef` expected — MUST fail to compile.
- Unit: `ContextKind` enum exhaustive match.
- Unit: `body_client_only` marker not extractable.

## Next IP

[`IP-004-tag-graph-kernel-domain.md`](IP-004-tag-graph-kernel-domain.md)


## A. Problem
`IP-003: note-store kernel + domain (port traits + entities + DCI invariants)` is not a generic implementation packet; it closes the `003 note store kernel domain` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

## B. Approach
Note-store domain rules preserve immutable Personal/Professional context, retention, hold, and audit boundaries before higher-level note features attach. The implementation must keep the µservice boundary intact: contracts remain under `microservices/notes/contracts/openapi/notes.yaml` / `microservices/notes/contracts/proto/notes.proto`, policy decisions remain in `microservices/notes/policy/tenant-scope.cedar`, operational proof remains in `microservices/notes/slos/note-open-latency.openslo.yaml`, and the parity claim is checked against `microservices/notes/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/notes/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/openapi/notes.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/proto/notes.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/asyncapi/notes-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/slos/note-open-latency.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/runbooks/sync-conflict-resolution.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/policy/dual-context-isolation.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/slos/note-create-latency.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/notes/PRD.md` and `microservices/notes/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `notes`.
2. Diff the declared contract in `microservices/notes/contracts/openapi/notes.yaml` and `microservices/notes/contracts/proto/notes.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/notes/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/notes/slos/note-open-latency.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/notes/PRD.md`, `microservices/notes/ARCHITECTURE.md`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/policy/tenant-scope.cedar`, `microservices/notes/slos/note-open-latency.openslo.yaml`, and `microservices/notes/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/notes/PRD.md`
- `microservices/notes/ARCHITECTURE.md`
- `microservices/notes/contracts/openapi/notes.yaml`
- `microservices/notes/contracts/proto/notes.proto`
- `microservices/notes/contracts/asyncapi/notes-events.yaml`
- `microservices/notes/policy/tenant-scope.cedar`
- `microservices/notes/slos/note-open-latency.openslo.yaml`
- `microservices/notes/runbooks/sync-conflict-resolution.md`
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml`
- `microservices/notes/competitor-parity-matrix.md`
- `microservices/notes/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `003 note store kernel domain` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
