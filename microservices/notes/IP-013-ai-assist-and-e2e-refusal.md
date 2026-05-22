---
doc_class: ImplementationPlan
impl_plan_id: IP-013-ai-assist-and-e2e-refusal
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes + axis-foundry-runtime + council-privacy
acceptance_lanes: [cargo-check, cargo-test, oya-governance-e2e-ai-refusal, oya-governance-dual-context-isolation]
---


# IP-013: ai-assist + E2E refusal CI lane

## Intent

Land `oya-notes-ai-assist-{kernel,domain,usecase,api,adapter,worker,sdk,app}` (T0 + T1 capabilities; T2 stub-but-disabled). Author the `oya-check-e2e-ai-refusal` CI lane and register it BLOCKER on `dev` per ADR-NOTES-0005.

## Type-System Invariant

```rust
pub trait AssistInvoker {
    fn invoke(&self, note: ProfessionalNoteRef, request: AssistRequest) -> Result<AssistResult, AssistError>;
    // No method accepting PersonalNoteRef. Period.
}
```

## Cedar Policy

`policy/tenant-scope.cedar` already carries:

```cedar
forbid (principal, action == Action::"invoke_ai_assist", resource in Note::?m)
when { resource has context_kind && resource.context_kind == "Personal" };
```

## CI Lane

`crates/oya-check-e2e-ai-refusal/`: AST + control-flow analysis verifying no path from `PersonalNoteRef` → `AssistInvoker::invoke`.

## Regression Suite

`tests/regression/e2e-ai-refusal/`:
- type-system: `compile_fail` test attempting to construct `PersonalNoteRef → AssistInvoker::invoke`.
- runtime: Cedar evaluation returns deny on Personal resource.
- CI lane: lane exit 0 with no findings.
- runtime metric: `oya_notes_ai_call_blocked_e2e_total` increments on attempted call.

## Acceptance Gates

```bash
cargo check -p oya-notes-ai-assist-kernel
cargo test --test e2e-ai-refusal
cargo run -p oya-dev-cli -- gate validate e2e-ai-refusal --microservice notes
cargo run -p oya-dev-cli -- gate validate dual-context-isolation --microservice notes
```

## Halt Conditions

- e2e-ai-refusal lane returns any finding — BLOCK PR.
- runtime metric increments in any non-test environment — Sev-1.

## Next IP

[`IP-014-e2e-key-management.md`](IP-014-e2e-key-management.md)


## A. Problem
`IP-013: ai-assist + E2E refusal CI lane` is not a generic implementation packet; it closes the `013 ai assist and e2e refusal` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

## B. Approach
Personal-tier plaintext exclusion is enforced before collaboration or AI: MLS/OpenMLS key material stays client-side, Loro CRDT is Professional-only unless an encrypted client flow exists. The implementation must keep the µservice boundary intact: contracts remain under `microservices/notes/contracts/openapi/notes.yaml` / `microservices/notes/contracts/proto/notes.proto`, policy decisions remain in `microservices/notes/policy/tenant-scope.cedar`, operational proof remains in `microservices/notes/slos/note-open-latency.openslo.yaml`, and the parity claim is checked against `microservices/notes/competitor-parity-matrix.md`.

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
- `microservices/notes/policy/e2e-personal-tier-default.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-e2e-key-management-adapter-mls.yaml` — verify/update as the authoritative artifact for this IP.
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
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `013 ai assist and e2e refusal` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
